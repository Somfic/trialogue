use noise::{NoiseFn, Perlin};
use rayon::prelude::*;

use crate::prelude::*;

pub fn planet_mesh(
    mut tracker: ResMut<AsyncTaskTracker<Entity>>,
    query: Query<(Entity, &Planet), Changed<Planet>>,
) {
    for (entity, planet) in query.iter() {
        let planet = planet.clone();

        tracker.spawn_for_entity(
            entity,
            move || generate_planet_mesh(&planet),
            |mut entity_mut, mesh| {
                entity_mut.insert(mesh);
            },
        );
    }
}

fn generate_planet_mesh(planet: &Planet) -> Mesh {
    let noise = Perlin::new(planet.seed());

    // Generate all 6 faces in parallel
    let faces_data: Vec<_> = CubeFace::to_vec()
        .into_par_iter()
        .map(|face| generate_face_vertices(face, planet, &noise))
        .collect();

    // Combine results sequentially
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (face_vertices, face_indices) in faces_data {
        let index_offset = vertices.len() as u32;
        vertices.extend(face_vertices);
        indices.extend(face_indices.iter().map(|i| i + index_offset as Index));
    }

    Mesh { vertices, indices }
}

#[derive(Clone, Copy)]
enum CubeFace {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl CubeFace {
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

fn generate_face_vertices(
    face: CubeFace,
    planet: &Planet,
    noise: &Perlin,
) -> (Vec<Vertex>, Vec<Index>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let step = 1.0 / planet.subdivisions as f32;
    let epsilon = step * 0.01; // Small offset for normal calculation

    for y in 0..=planet.subdivisions {
        for x in 0..=planet.subdivisions {
            let u = x as f32 * step;
            let v = y as f32 * step;

            let position_on_cube = cube_face_uv_to_xyz(&face, u, v);
            let position_sphere = position_on_cube.normalize();

            let terrain_height = generate_terrain_height(noise, &position_sphere, &planet.terrain_config);
            let terrain_height = 1.0 + terrain_height * planet.terrain_config.noise_strength;
            let position = position_sphere * terrain_height;

            // Calculate normal using central differences for better accuracy
            // Clamp offsets to stay within valid UV range
            let u_plus = (u + epsilon).min(1.0);
            let u_minus = (u - epsilon).max(0.0);
            let v_plus = (v + epsilon).min(1.0);
            let v_minus = (v - epsilon).max(0.0);

            // Sample points in both directions
            let pos_u_plus = cube_face_uv_to_xyz(&face, u_plus, v).normalize();
            let pos_u_minus = cube_face_uv_to_xyz(&face, u_minus, v).normalize();
            let pos_v_plus = cube_face_uv_to_xyz(&face, u, v_plus).normalize();
            let pos_v_minus = cube_face_uv_to_xyz(&face, u, v_minus).normalize();

            // Get heights
            let h_u_plus = generate_terrain_height(noise, &pos_u_plus, &planet.terrain_config);
            let h_u_minus = generate_terrain_height(noise, &pos_u_minus, &planet.terrain_config);
            let h_v_plus = generate_terrain_height(noise, &pos_v_plus, &planet.terrain_config);
            let h_v_minus = generate_terrain_height(noise, &pos_v_minus, &planet.terrain_config);

            let p_u_plus = pos_u_plus * (1.0 + h_u_plus * planet.terrain_config.noise_strength);
            let p_u_minus = pos_u_minus * (1.0 + h_u_minus * planet.terrain_config.noise_strength);
            let p_v_plus = pos_v_plus * (1.0 + h_v_plus * planet.terrain_config.noise_strength);
            let p_v_minus = pos_v_minus * (1.0 + h_v_minus * planet.terrain_config.noise_strength);

            // Central difference tangent vectors
            let tangent_u = p_u_plus - p_u_minus;
            let tangent_v = p_v_plus - p_v_minus;

            // Normal is perpendicular to both tangents
            // Ensure it points outward by checking dot product with sphere normal
            let mut normal = tangent_u.cross(&tangent_v).normalize();
            if normal.dot(&position_sphere) < 0.0 {
                normal = -normal;
            }

            let uv = [u, v];

            vertices.push(Vertex {
                position: [position.x, position.y, position.z],
                uv,
                normal: [normal.x, normal.y, normal.z],
            });
        }
    }

    for y in 0..planet.subdivisions {
        for x in 0..planet.subdivisions {
            let i0 = y * (planet.subdivisions + 1) + x;
            let i1 = i0 + 1;
            let i2 = i0 + (planet.subdivisions + 1);
            let i3 = i2 + 1;

            // CCW winding for front faces
            indices.push(i0 as Index);
            indices.push(i1 as Index);
            indices.push(i2 as Index);

            indices.push(i2 as Index);
            indices.push(i1 as Index);
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

fn generate_terrain_height(noise: &Perlin, position: &Vector3<f32>, config: &TerrainConfig) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = config.noise_scale;

    for _ in 0..config.octaves {
        // Sample 3D noise at this position
        let sample = noise.get([
            (position.x * frequency) as f64,
            (position.y * frequency) as f64,
            (position.z * frequency) as f64,
        ]) as f32;

        value += sample * amplitude;

        // Each octave: higher frequency, lower amplitude
        frequency *= config.lacunarity;
        amplitude *= config.persistence;
    }

    value
}
