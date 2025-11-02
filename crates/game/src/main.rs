use trialogue_editor::EditorLayer;
use trialogue_engine::{
    ApplicationBuilder, Result,
    layers::{DeviceLayer, RenderLayer},
    prelude::*,
};
use winit::event_loop::EventLoop;

use crate::prelude::Planet;

mod components;
mod prelude;
mod sandbox_layer;
mod systems;

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_module("trialogue", log::LevelFilter::Debug)
        .filter_module("bevy_ecs", log::LevelFilter::Debug)
        .init();

    let event_loop = EventLoop::with_user_event().build()?;

    let mut app = ApplicationBuilder::new()
        .add_layer(|context| Box::new(DeviceLayer::new(context)))
        // .add_layer(|context| Box::new(RaytracerLayer::new(context)))
        .add_layer(|context| Box::new(RenderLayer::new(context)))
        .add_layer(|context| Box::new(sandbox_layer::SandboxLayer::new(context)))
        // Swap between WindowLayer and EditorLayer:
        // .add_layer(|context| Box::new(WindowLayer::new(context)))
        .add_layer(|context| Box::new(EditorLayer::new(context)))
        .build();

    // Register shaders
    app.register_shader(
        "crates/engine/src/layers/renderer/shader.wgsl",
        Shader::Standard,
        include_str!("../../engine/src/layers/renderer/shader.wgsl"),
    );

    app.register_shader(
        "crates/engine/src/layers/raytracer/raytracer.wgsl",
        Shader::Raytracer,
        include_str!("../../engine/src/layers/raytracer/raytracer.wgsl"),
    );

    app.spawn(
        "Cat",
        (
            Transform::default(),
            Planet {
                seed: "ExampleSeed".to_string(),
                subdivisions: 3,
            },
            Material::standard(),
            Texture {
                bytes: include_bytes!("cat.png").to_vec(),
            },
        ),
    );

    // Note: aspect ratio will be automatically set to match window dimensions
    app.spawn(
        "Camera",
        (
            Transform {
                position: Point3::new(0.0, 0.0, 10.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::identity(),
            },
            Camera {
                is_main: true,
                fovy: 1.0,
                target: Point3::new(0.0, 0.0, 0.0),
                zfar: 100.0,
                znear: 0.0001,
                aperture: 0.1,
                focus_distance: 10.0,
            },
            RenderTarget {},
        ),
    );

    // // Spawn lights for the raytracer
    app.spawn(
        "Main Light",
        (
            Light {
                intensity: 1.0,
                color: [1.0, 1.0, 1.0],
            },
            Transform {
                position: Point3::new(5.0, 10.0, 5.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
        ),
    );

    // Spawn environment map
    // app.spawn(
    //     "Environment Map",
    //     (EnvironmentMap {
    //         bytes: Vec::new(), // Empty for now - use the inspector to load an HDR file
    //     },),
    // );

    event_loop.run_app(&mut app)?;

    Ok(())
}
