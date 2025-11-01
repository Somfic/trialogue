use trialogue::{ApplicationBuilder, Result};
use trialogue::{layers, prelude::*};
use winit::event_loop::EventLoop;

mod sandbox_layer;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        uv: [0.4131759, 0.00759614],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        uv: [0.0048659444, 0.43041354],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        uv: [0.28081453, 0.949397],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        uv: [0.85967, 0.84732914],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        uv: [0.9414737, 0.2652641],
    }, // E
];

const INDICES: &[Index] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_module("trialogue", log::LevelFilter::Debug)
        .filter_module("bevy_ecs", log::LevelFilter::Debug)
        .init();

    let event_loop = EventLoop::with_user_event().build()?;

    let mut app = ApplicationBuilder::new()
        .add_layer(|context| Box::new(layers::DeviceLayer::new(context)))
        .add_layer(|context| Box::new(layers::RaytracerLayer::new(context)))
        // .add_layer(|context| Box::new(sandbox_layer::SandboxLayer::new(context)))
        // Swap between WindowLayer and EditorLayer:
        // .add_layer(|context| Box::new(layers::WindowLayer::new(context)))
        .add_layer(|context| Box::new(layers::EditorLayer::new(context)))
        .build();

    // app.spawn(
    //     "Cat",
    //     (
    //         Transform::default(),
    //         Mesh {
    //             vertices: VERTICES.to_vec(),
    //             indices: INDICES.to_vec(),
    //         },
    //         Texture {
    //             bytes: include_bytes!("cat.png").to_vec(),
    //         },
    //     ),
    // );

    // Note: aspect ratio will be automatically set to match window dimensions
    app.spawn(
        "Camera",
        (
            Transform {
                position: Point3::new(10.0, 0.0, 10.0),
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

    // Generate random spheres
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let num_spheres = 20;

    for i in 0..num_spheres {
        let x = rng.gen_range(-10.0..10.0);
        let y = rng.gen_range(0.5..3.0);
        let z = rng.gen_range(-10.0..10.0);
        let radius = rng.gen_range(0.3..1.5);

        let color = [
            rng.gen_range(0.1..1.0),
            rng.gen_range(0.1..1.0),
            rng.gen_range(0.1..1.0),
        ];

        let material_type = if rng.gen_bool(0.3) { 1 } else { 0 }; // 30% metal, 70% diffuse

        app.spawn(
            format!("Sphere {}", i),
            (
                Sphere {
                    color,
                    material_type,
                },
                Transform {
                    position: Point3::new(x, y, z),
                    rotation: UnitQuaternion::identity(),
                    scale: Vector3::new(radius, radius, radius),
                },
            ),
        );
    }

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
    app.spawn(
        "Environment Map",
        (EnvironmentMap {
            bytes: Vec::new(), // Empty for now - use the inspector to load an HDR file
        },),
    );

    // app.spawn(
    //     "Secondary Light",
    //     (
    //         Light {
    //             intensity: 0.5,
    //             color: [0.8, 0.9, 1.0],
    //         },
    //         Transform {
    //             position: Point3::new(-5.0, 5.0, 5.0),
    //             rotation: UnitQuaternion::identity(),
    //             scale: Vector3::new(1.0, 1.0, 1.0),
    //         },
    //     ),
    // );

    event_loop.run_app(&mut app)?;

    Ok(())
}
