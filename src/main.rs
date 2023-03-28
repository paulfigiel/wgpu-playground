mod syntax_highlighting;

use futures::executor::block_on;
use std::{borrow::Cow, future::Future};
use wgpu::{Adapter, Error};

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        renderer::RenderDevice,
    },
};
use bevy_egui::{
    egui::{self},
    EguiContexts, EguiPlugin,
};
use syntax_highlighting::CodeTheme;

#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

#[derive(Default, Resource)]
struct CurrentCode {
    code: String,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .init_resource::<OccupiedScreenSpace>()
        .init_resource::<CurrentCode>()
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_startup_system(setup)
        .add_system(ui_example_system)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // cube
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        material: materials.add(CustomMaterial {
            color: Color::BLUE,
            color_texture: Some(asset_server.load("icon.png")),
            alpha_mode: AlphaMode::Blend,
        }),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "custom_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode,
}

async fn log_shader_error(error: impl Future<Output = Option<Error>> + Send) {
    let e = error.await;
    println!("{:?} hello ?", e);
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut code: ResMut<CurrentCode>,
    mut render_device: ResMut<RenderDevice>,
) {
    let ctx = contexts.ctx_mut();

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("WGSL Code");
            let theme = CodeTheme::from_memory(ui.ctx());
            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job =
                    syntax_highlighting::highlight(ui.ctx(), &theme, string, "wgsl");
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut code.code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
            if ui.button("Compile").clicked() {
                render_device
                    .wgpu_device()
                    .push_error_scope(wgpu::ErrorFilter::Validation);
                render_device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&code.code)),
                });
                let error = render_device.wgpu_device().pop_error_scope();
                //Adapter::request_device
                //let shader_module = render_device.create_shader_module(module_descriptor);
                if let Some(wgpu::Error::Validation { description, .. }) = block_on(error) {
                    print!("{}", description);
                }
            }

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
}
