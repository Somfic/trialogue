use nalgebra::{Point, Point3, Vector3};
use trialogue::{
    ApplicationBuilder, Result,
    layer::renderer::{Camera, Index, Mesh, Texture, Transform, Vertex},
};
use winit::event_loop::EventLoop;

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

    let mut app = ApplicationBuilder::new().build();
    app.spawn((
        Mesh {
            vertices: VERTICES.to_vec(),
            indices: INDICES.to_vec(),
        },
        Texture {
            bytes: include_bytes!("cat.png").to_vec(),
        },
    ));

    app.spawn((
        Transform {
            position: Point3::new(10.0, 0.0, 10.0),
            up: Vector3::new(0.0, 1.0, 0.0),
        },
        Camera {
            aspect: 1.0,
            fovy: 1.0,
            target: Point3::new(0.0, 0.0, 0.0),
            zfar: 100.0,
            znear: 0.0001,
        },
    ));

    event_loop.run_app(&mut app)?;

    Ok(())
}
