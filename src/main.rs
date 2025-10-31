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
    env_logger::init();

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
            },
            RenderTarget {},
        ),
    );

    // Spawn spheres for the raytracer
    app.spawn(
        "Red Sphere",
        (
            Sphere {
                color: [0.8, 0.3, 0.3],
                material_type: 0, // Lambertian
            },
            Transform {
                position: Point3::new(0.0, 0.0, 0.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0), // radius = scale.x
            },
        ),
    );

    // app.spawn(
    //     "Ground Plane",
    //     (
    //         Sphere {
    //             color: [0.5, 0.5, 0.5],
    //             material_type: 0, // Lambertian
    //         },
    //         Transform {
    //             position: Point3::new(0.0, -1001.0, 0.0),
    //             rotation: UnitQuaternion::identity(),
    //             scale: Vector3::new(1000.0, 1000.0, 1000.0), // radius = scale.x
    //         },
    //     ),
    // );

    app.spawn(
        "Green Sphere",
        (
            Sphere {
                color: [0.3, 0.8, 0.3],
                material_type: 0,
            },
            Transform {
                position: Point3::new(2.5, 0.5, -1.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(0.5, 0.5, 0.5), // radius = scale.x
            },
        ),
    );

    app.spawn(
        "Blue Sphere",
        (
            Sphere {
                color: [0.3, 0.3, 0.8],
                material_type: 0,
            },
            Transform {
                position: Point3::new(-2.5, 0.5, -1.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(0.5, 0.5, 0.5), // radius = scale.x
            },
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
