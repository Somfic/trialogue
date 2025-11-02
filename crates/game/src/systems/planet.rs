use crate::prelude::*;

pub fn generate_planet_mesh(
    mut commands: Commands,
    query: Query<(Entity, &Planet), Changed<Planet>>,
) {
    for (entity, planet) in query.iter() {
        let mesh = generate_cube_sphere_mesh(planet.subdivisions);
        commands.entity(entity).insert(mesh);
    }
}

fn generate_cube_sphere_mesh(subdivisions: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for face in CubeFace::to_vec() {
        let (face_vertices, face_indices) = generate_face_vertices(face, subdivisions);

        let index_offset = vertices.len() as u32;
        vertices.extend(face_vertices);
        indices.extend(face_indices.iter().map(|i| i + index_offset as Index));
    }

    Mesh { vertices, indices }
}

enum CubeFace {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl CubeFace {
    fn to_vector(&self) -> Vector3<f32> {
        match self {
            CubeFace::PositiveX => Vector3::new(1.0, 0.0, 0.0),
            CubeFace::NegativeX => Vector3::new(-1.0, 0.0, 0.0),
            CubeFace::PositiveY => Vector3::new(0.0, 1.0, 0.0),
            CubeFace::NegativeY => Vector3::new(0.0, -1.0, 0.0),
            CubeFace::PositiveZ => Vector3::new(0.0, 0.0, 1.0),
            CubeFace::NegativeZ => Vector3::new(0.0, 0.0, -1.0),
        }
    }

    fn to_vec() -> Vec<CubeFace> {
        vec![
            CubeFace::PositiveX,
            CubeFace::NegativeX,
            CubeFace::PositiveY,
            CubeFace::NegativeY,
            CubeFace::PositiveZ,
            CubeFace::NegativeZ,
        ]
    }
}

fn generate_face_vertices(face: CubeFace, subdivisions: u32) -> (Vec<Vertex>, Vec<Index>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let step = 1.0 / subdivisions as f32;

    for y in 0..=subdivisions {
        for x in 0..=subdivisions {
            let u = x as f32 * step;
            let v = y as f32 * step;

            let position_on_cube = cube_face_uv_to_xyz(&face, u, v);
            let position = position_on_cube.normalize();
            let normal = position;
            let uv = [u, v];

            vertices.push(Vertex {
                position: [position.x, position.y, position.z],
                uv,
                normal: [normal.x, normal.y, normal.z],
            });
        }
    }

    for y in 0..subdivisions {
        for x in 0..subdivisions {
            let i0 = y * (subdivisions + 1) + x;
            let i1 = i0 + 1;
            let i2 = i0 + (subdivisions + 1);
            let i3 = i2 + 1;

            indices.push(i0 as Index);
            indices.push(i2 as Index);
            indices.push(i1 as Index);

            indices.push(i1 as Index);
            indices.push(i2 as Index);
            indices.push(i3 as Index);
        }
    }

    (vertices, indices)
}

fn cube_face_uv_to_xyz(face: &CubeFace, u: f32, v: f32) -> Vector3<f32> {
    let a = 2.0 * u - 1.0;
    let b = 2.0 * v - 1.0;

    match face {
        CubeFace::PositiveX => Vector3::new(1.0, b, -a),
        CubeFace::NegativeX => Vector3::new(-1.0, b, a),
        CubeFace::PositiveY => Vector3::new(a, 1.0, -b),
        CubeFace::NegativeY => Vector3::new(a, -1.0, b),
        CubeFace::PositiveZ => Vector3::new(a, b, 1.0),
        CubeFace::NegativeZ => Vector3::new(-a, b, -1.0),
    }
}
