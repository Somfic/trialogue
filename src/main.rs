use trialogue::{
    ApplicationBuilder, Result,
    layer::renderer::{Index, Mesh, Vertex},
};
use winit::event_loop::EventLoop;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // E
];

const INDICES: &[Index] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build()?;

    let mut app = ApplicationBuilder::new().build();
    app.spawn((Mesh {
        vertices: VERTICES.to_vec(),
        indices: INDICES.to_vec(),
    },));

    event_loop.run_app(&mut app)?;

    Ok(())
}
