pub use crate::prelude::*;
mod prelude;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Resource, Default)]
struct Time {
    seconds: f32,
}

fn main() {
    let mut world = World::default();
    world.insert_resource(Time::default());

    world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));

    let mut schedule = Schedule::default();
    schedule.add_systems(print_position);

    schedule.run(&mut world);
}

fn print_position(query: Query<(Entity, &Position)>) {
    for (entity, position) in &query {
        println!(
            "Entity {} is at position: x {}, y {}",
            entity, position.x, position.y
        );
    }
}
