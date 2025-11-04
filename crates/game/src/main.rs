use trialogue_editor::EditorLayer;
use trialogue_engine::{
    ApplicationBuilder, Result,
    layers::{DeviceLayer, RenderLayer},
    prelude::*,
};
use winit::event_loop::EventLoop;

use crate::prelude::{PlanetLod, CopyToChildren, QuadLodTest, CameraController};

mod components;
mod prelude;
mod sandbox_layer;
mod systems;
mod utils;

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
        "crates/engine/src/layers/renderer/shader_instanced.wgsl",
        Shader::Instanced,
        include_str!("../../engine/src/layers/renderer/shader_instanced.wgsl"),
    );

    app.register_shader(
        "crates/engine/src/layers/raytracer/raytracer.wgsl",
        Shader::Raytracer,
        include_str!("../../engine/src/layers/raytracer/raytracer.wgsl"),
    );

    // ===== QUAD LOD TEST (INSTANCED) =====
    // Create base unit cube mesh (-1 to 1 in XZ, -0.5 to 0.5 in Y)
    // Each instance will transform this base mesh
    let base_cube_mesh = {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Simple unit cube centered at origin
        let positions = [
            [-1.0, -0.5, -1.0], // 0: bottom-left-front
            [ 1.0, -0.5, -1.0], // 1: bottom-right-front
            [ 1.0, -0.5,  1.0], // 2: bottom-right-back
            [-1.0, -0.5,  1.0], // 3: bottom-left-back
            [-1.0,  0.5, -1.0], // 4: top-left-front
            [ 1.0,  0.5, -1.0], // 5: top-right-front
            [ 1.0,  0.5,  1.0], // 6: top-right-back
            [-1.0,  0.5,  1.0], // 7: top-left-back
        ];

        // Bottom face (y = -0.5) - viewed from below, CCW
        for &i in &[0, 1, 2, 3] {
            vertices.push(Vertex {
                position: positions[i],
                uv: [0.0, 0.0],
                normal: [0.0, -1.0, 0.0],
            });
        }
        indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
        
        // Top face (y = 0.5) - viewed from above, CCW
        for &i in &[4, 5, 6, 7] {
            vertices.push(Vertex {
                position: positions[i],
                uv: [0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
            });
        }
        let offset = 4;
        indices.extend_from_slice(&[offset+0, offset+2, offset+1, offset+0, offset+3, offset+2]);
        
        // Front face (z = -1.0) - viewed from front, CCW
        for &i in &[0, 1, 5, 4] {
            vertices.push(Vertex {
                position: positions[i],
                uv: [0.0, 0.0],
                normal: [0.0, 0.0, -1.0],
            });
        }
        let offset = 8;
        indices.extend_from_slice(&[offset+0, offset+2, offset+1, offset+0, offset+3, offset+2]);
        
        // Back face (z = 1.0) - viewed from back, CCW
        for &i in &[2, 3, 7, 6] {
            vertices.push(Vertex {
                position: positions[i],
                uv: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            });
        }
        let offset = 12;
        indices.extend_from_slice(&[offset+0, offset+1, offset+2, offset+0, offset+2, offset+3]);
        
        // Left face (x = -1.0) - viewed from left, CCW
        for &i in &[3, 0, 4, 7] {
            vertices.push(Vertex {
                position: positions[i],
                uv: [0.0, 0.0],
                normal: [-1.0, 0.0, 0.0],
            });
        }
        let offset = 16;
        indices.extend_from_slice(&[offset+0, offset+1, offset+2, offset+0, offset+2, offset+3]);
        
        // Right face (x = 1.0) - viewed from right, CCW
        for &i in &[1, 2, 6, 5] {
            vertices.push(Vertex {
                position: positions[i],
                uv: [0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            });
        }
        let offset = 20;
        indices.extend_from_slice(&[offset+0, offset+2, offset+1, offset+0, offset+3, offset+2]);
        
        Mesh { vertices, indices }
    };
    
    app.spawn(
        "Quad LOD Test (Instanced)",
        (
            QuadLodTest::new(),
            InstancedLodMesh::new(base_cube_mesh),
            Transform::default(),
            Material::instanced(),
            Texture {
                bytes: include_bytes!("cat.png").to_vec(),
            },
        ),
    );

    // ===== PLANET LOD (disabled for quad test) =====
    // app.spawn(
    //     "LOD Planet",
    //     (
    //         Transform {
    //             scale: Vector3::new(500.0, 500.0, 500.0),
    //             position: Point3::new(0.0, 0.0, -150.0),
    //             ..Default::default()
    //         },
    //         PlanetLod::new("ExampleSeed".to_string()),
    //         CopyToChildren, // Components will be copied to chunk children when changed
    //         Material::standard(),
    //         Texture {
    //             bytes: include_bytes!("cat.png").to_vec(),
    //         },
    //     ),
    // );

    // Old Planet (comment out when testing LOD)
    // app.spawn(
    //     "Planet",
    //     (
    //         Transform {
    //             scale: Vector3::new(500.0, 500.0, 500.0),
    //             position: Point3::new(0.0, 0.0, -150.0),
    //             ..Default::default()
    //         },
    //         Planet {
    //             seed: "ExampleSeed".to_string(),
    //             subdivisions: 3,
    //             terrain_config: Default::default(),
    //         },
    //         Material::standard(),
    //         Texture {
    //             bytes: include_bytes!("cat.png").to_vec(),
    //         },
    //     ),
    // );

    // Camera positioned directly above looking down at the quad
    app.spawn(
        "Camera",
        (
            Transform {
                position: Point3::new(200.0, 200.0, 200.0), // Directly above the quad
                rotation: UnitQuaternion::identity(),
                scale: Vector3::identity(),
            },
            Camera {
                is_main: true,
                fovy: 1.0,
                target: Point3::new(0.0, 0.0, 0.0), // Looking straight down at origin
                zfar: 100000.0,
                znear: 0.1,
                aperture: 0.1,
                focus_distance: 2000.0,
            },
            CameraController::default(),
            RenderTarget {},
        ),
    );

    // ===== PLANET CAMERA (commented out for quad test) =====
    // app.spawn(
    //     "Camera",
    //     (
    //         Transform {
    //             position: Point3::new(0.0, 0.0, -1000.0),
    //             rotation: UnitQuaternion::identity(),
    //             scale: Vector3::identity(),
    //         },
    //         Camera {
    //             is_main: true,
    //             fovy: 1.0,
    //             target: Point3::new(0.0, 0.0, 0.0),
    //             zfar: 100.0,
    //             znear: 0.01,
    //             aperture: 0.1,
    //             focus_distance: 10.0,
    //         },
    //         RenderTarget {},
    //     ),
    // );

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
