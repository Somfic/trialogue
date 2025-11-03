use crate::prelude::*;
use bevy_ecs::system::ParamSet;
use noise::{NoiseFn, Perlin};

/// Spawn 6 root chunks (one per cube face) when a PlanetLod is added
pub fn initialize_planet_lod_chunks(
    mut commands: Commands,
    planet_query: Query<(Entity, Option<&Material>, Option<&Texture>), (With<PlanetLod>, Without<PlanetChunk>)>,
    chunk_query: Query<&PlanetChunk>,
) {
    for (planet_entity, material, texture) in planet_query.iter() {
        // Check if this planet already has chunks (avoid respawning)
        let has_chunks = chunk_query
            .iter()
            .any(|chunk| chunk.parent_planet == planet_entity);

        if has_chunks {
            continue; // Already initialized
        }

        log::info!("Initializing LOD chunks for planet entity {:?}", planet_entity);

        // Get material from parent, or use default
        let material = material.cloned().unwrap_or_else(|| Material::standard());

        // Clone texture bytes if present
        let texture_bytes = texture.map(|tex| tex.bytes.clone());

        // Spawn 6 root chunks, one for each cube face
        for face in CubeFace::all() {
            let chunk = PlanetChunk::new_root(planet_entity, face);

            let mut entity_commands = commands.spawn((
                chunk,
                ChunkParent { entity: planet_entity },
                Transform::default(),
                material.clone(),
            ));

            // Add texture if parent has one
            if let Some(bytes) = texture_bytes.clone() {
                entity_commands.insert(Texture { bytes });
            }
        }
    }
}

/// Update raycast hit point for all LOD planets based on main camera view
pub fn update_planet_lod_raycast(
    camera_query: Query<(&Camera, &Transform), With<Camera>>,
    mut planet_query: Query<(&mut PlanetLod, &Transform)>,
) {
    // Find the main camera
    let main_camera = camera_query.iter().find(|(cam, _)| cam.is_main);

    let Some((camera, camera_transform)) = main_camera else {
        return; // No main camera found
    };

    // Generate ray from camera center
    let ray = camera_center_ray(camera, camera_transform);

    // Check intersection with each LOD planet
    for (mut planet_lod, planet_transform) in planet_query.iter_mut() {
        // Planet center is at its transform position
        let planet_center = planet_transform.position;

        // For now, assume planet radius is 1.0 (we'll apply scale later if needed)
        // Since the planet is scaled via transform, we need to account for that
        let planet_radius = planet_transform.scale.x; // Assume uniform scale

        // Test intersection
        if let Some(intersection) = ray_sphere_intersection(&ray, planet_center, planet_radius) {
            planet_lod.raycast_hit = Some(intersection.point);
        } else {
            planet_lod.raycast_hit = None;
        }
    }
}

/// Generate meshes for chunks that don't have them yet
pub fn generate_chunk_meshes(
    mut tracker: ResMut<AsyncTaskTracker<Entity>>,
    chunk_query: Query<(Entity, &PlanetChunk), Without<Mesh>>,
    planet_query: Query<&PlanetLod>,
) {
    let chunk_count = chunk_query.iter().count();
    if chunk_count > 0 {
        log::info!("Generating meshes for {} chunks", chunk_count);
    }

    for (chunk_entity, chunk) in chunk_query.iter() {
        // Skip if a task is already in progress for this entity
        // (we check generation > 0 because new entities start at generation 0)
        if tracker.has_pending_task(&chunk_entity) {
            continue;
        }

        // Get the parent planet's configuration
        let Ok(planet_lod) = planet_query.get(chunk.parent_planet) else {
            continue; // Parent planet doesn't exist or doesn't have PlanetLod
        };

        // Clone data needed for async task
        let seed = planet_lod.seed_u32();
        let terrain_config = planet_lod.terrain_config.clone();
        let base_subdivisions = planet_lod.config.base_subdivisions;
        let face = chunk.face;
        let uv_bounds = chunk.uv_bounds;

        // Spawn async task to generate the chunk mesh
        tracker.spawn_for_entity(
            chunk_entity,
            move || generate_chunk_mesh(seed, &terrain_config, base_subdivisions, face, uv_bounds),
            |mut entity_mut, mesh| {
                log::info!("Inserting mesh for chunk entity {:?}", entity_mut.id());
                entity_mut.insert(mesh);
            },
        );
    }
}

/// Generate a mesh for a single chunk
fn generate_chunk_mesh(
    seed: u32,
    terrain_config: &TerrainConfig,
    subdivisions: u32,
    face: CubeFace,
    uv_bounds: (f32, f32, f32, f32),
) -> Mesh {
    let noise = Perlin::new(seed);
    let (u_min, u_max, v_min, v_max) = uv_bounds;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let step = 1.0 / subdivisions as f32;
    let epsilon = step * 0.01;

    // Generate vertices within the UV bounds
    for y in 0..=subdivisions {
        for x in 0..=subdivisions {
            // Map grid coordinates to the chunk's UV range
            let u_local = x as f32 * step;
            let v_local = y as f32 * step;
            let u = u_min + u_local * (u_max - u_min);
            let v = v_min + v_local * (v_max - v_min);

            let position_on_cube = cube_face_uv_to_xyz(&face, u, v);
            let position_sphere = position_on_cube.normalize();

            let terrain_height = generate_terrain_height(&noise, &position_sphere, terrain_config);
            let terrain_height = 1.0 + terrain_height * terrain_config.noise_strength;
            let position = position_sphere * terrain_height;

            // Calculate normal using central differences
            let u_plus = (u + epsilon).min(u_max);
            let u_minus = (u - epsilon).max(u_min);
            let v_plus = (v + epsilon).min(v_max);
            let v_minus = (v - epsilon).max(v_min);

            let pos_u_plus = cube_face_uv_to_xyz(&face, u_plus, v).normalize();
            let pos_u_minus = cube_face_uv_to_xyz(&face, u_minus, v).normalize();
            let pos_v_plus = cube_face_uv_to_xyz(&face, u, v_plus).normalize();
            let pos_v_minus = cube_face_uv_to_xyz(&face, u, v_minus).normalize();

            let h_u_plus = generate_terrain_height(&noise, &pos_u_plus, terrain_config);
            let h_u_minus = generate_terrain_height(&noise, &pos_u_minus, terrain_config);
            let h_v_plus = generate_terrain_height(&noise, &pos_v_plus, terrain_config);
            let h_v_minus = generate_terrain_height(&noise, &pos_v_minus, terrain_config);

            let p_u_plus = pos_u_plus * (1.0 + h_u_plus * terrain_config.noise_strength);
            let p_u_minus = pos_u_minus * (1.0 + h_u_minus * terrain_config.noise_strength);
            let p_v_plus = pos_v_plus * (1.0 + h_v_plus * terrain_config.noise_strength);
            let p_v_minus = pos_v_minus * (1.0 + h_v_minus * terrain_config.noise_strength);

            let tangent_u = p_u_plus - p_u_minus;
            let tangent_v = p_v_plus - p_v_minus;

            let mut normal = tangent_u.cross(&tangent_v).normalize();
            if normal.dot(&position_sphere) < 0.0 {
                normal = -normal;
            }

            vertices.push(Vertex {
                position: [position.x, position.y, position.z],
                uv: [u_local, v_local], // Use local UV for texturing
                normal: [normal.x, normal.y, normal.z],
            });
        }
    }

    // Generate indices
    for y in 0..subdivisions {
        for x in 0..subdivisions {
            let i0 = y * (subdivisions + 1) + x;
            let i1 = i0 + 1;
            let i2 = i0 + (subdivisions + 1);
            let i3 = i2 + 1;

            indices.push(i0 as Index);
            indices.push(i1 as Index);
            indices.push(i2 as Index);

            indices.push(i2 as Index);
            indices.push(i1 as Index);
            indices.push(i3 as Index);
        }
    }

    Mesh { vertices, indices }
}

/// Helper: Convert UV on cube face to 3D position
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

/// Helper: Generate terrain height at a position using noise
fn generate_terrain_height(noise: &Perlin, position: &Vector3<f32>, config: &TerrainConfig) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = config.noise_scale;

    for _ in 0..config.octaves {
        let sample = noise.get([
            (position.x * frequency) as f64,
            (position.y * frequency) as f64,
            (position.z * frequency) as f64,
        ]) as f32;

        value += sample * amplitude;

        frequency *= config.lacunarity;
        amplitude *= config.persistence;
    }

    value
}

/// Copy Material from parent to children when it changes
pub fn copy_material_to_children(
    parent_query: Query<(Entity, &Material), (With<CopyToChildren>, Changed<Material>, Without<ChunkParent>)>,
    mut children_query: Query<(&ChunkParent, &mut Material), With<ChunkParent>>,
) {
    for (parent_entity, parent_material) in parent_query.iter() {
        // Find all children of this parent and update their material
        for (chunk_parent, mut child_material) in children_query.iter_mut() {
            if chunk_parent.entity == parent_entity {
                *child_material = parent_material.clone();
            }
        }
    }
}

/// Copy Texture from parent to children when it changes
pub fn copy_texture_to_children(
    parent_query: Query<(Entity, &Texture), (With<CopyToChildren>, Changed<Texture>, Without<ChunkParent>)>,
    mut child_queries: ParamSet<(
        Query<(&ChunkParent, &mut Texture), With<ChunkParent>>,
        Query<(Entity, &ChunkParent), (With<ChunkParent>, Without<Texture>)>,
    )>,
    mut commands: Commands,
) {
    // Collect parent texture changes first
    let mut updates: Vec<(Entity, Vec<u8>)> = Vec::new();
    for (parent_entity, parent_texture) in parent_query.iter() {
        updates.push((parent_entity, parent_texture.bytes.clone()));
    }

    // Apply updates in two passes using ParamSet
    for (parent_entity, texture_bytes) in &updates {
        // First pass: Update children that already have textures
        for (chunk_parent, mut child_texture) in child_queries.p0().iter_mut() {
            if &chunk_parent.entity == parent_entity {
                child_texture.bytes = texture_bytes.clone();
            }
        }
    }

    for (parent_entity, texture_bytes) in &updates {
        // Second pass: Add texture to children that don't have one
        for (child_entity, chunk_parent) in child_queries.p1().iter() {
            if &chunk_parent.entity == parent_entity {
                commands.entity(child_entity).insert(Texture {
                    bytes: texture_bytes.clone(),
                });
            }
        }
    }
}

/// Update children transforms based on parent transform changes
/// Children maintain their local position/rotation/scale relative to parent
pub fn update_children_transforms(
    parent_query: Query<(Entity, &Transform), (With<CopyToChildren>, Changed<Transform>, Without<ChunkParent>)>,
    mut children_query: Query<(&ChunkParent, &mut Transform), With<ChunkParent>>,
) {
    for (parent_entity, parent_transform) in parent_query.iter() {
        // For now, children just inherit the parent's scale
        // Position and rotation stay at their local values (usually default)
        // This makes the chunks render at the planet's position/rotation/scale
        for (chunk_parent, mut child_transform) in children_query.iter_mut() {
            if chunk_parent.entity == parent_entity {
                // Inherit parent's position, rotation, and scale
                // This makes chunks render in world space at the same transform as parent
                child_transform.position = parent_transform.position;
                child_transform.rotation = parent_transform.rotation;
                child_transform.scale = parent_transform.scale;
            }
        }
    }
}
