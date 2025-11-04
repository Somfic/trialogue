use crate::prelude::*;

/// Initialize root quad chunk when QuadLodTest is added
pub fn initialize_quad_lod(
    mut commands: Commands,
    test_query: Query<Entity, (With<QuadLodTest>, Without<QuadChunk>)>,
    chunk_query: Query<&QuadChunk>,
) {
    for test_entity in test_query.iter() {
        // Check if this test already has chunks
        let has_chunks = chunk_query
            .iter()
            .any(|chunk| chunk.parent_test == test_entity);

        if has_chunks {
            continue;
        }

        log::info!("Initializing quad LOD test for entity {:?}", test_entity);

        // Spawn single root chunk covering entire test area
        let root_bounds = (-1000.0, 1000.0, -1000.0, 1000.0);
        let chunk = QuadChunk::new_root(test_entity, root_bounds);

        log::info!("Spawning root chunk with bounds {:?}", root_bounds);

        commands.spawn((
            Tag {
                label: format!("Quad Root Chunk"),
            },
            chunk,
            QuadChunkParent {
                entity: test_entity,
            },
            Transform::default(),
            Material::standard(),
            Texture {
                bytes: include_bytes!("../cat.png").to_vec(),
            },
        ));
    }
}

/// Generate meshes for quad chunks that don't have them yet
pub fn generate_quad_chunk_meshes(
    mut tracker: ResMut<AsyncTaskTracker<Entity>>,
    chunk_query: Query<(Entity, &QuadChunk), Without<Mesh>>,
    test_query: Query<&QuadLodTest>,
) {
    let chunks_without_mesh = chunk_query.iter().count();
    if chunks_without_mesh > 0 {
        log::info!("Generating meshes for {} quad chunks", chunks_without_mesh);
    }

    for (chunk_entity, chunk) in chunk_query.iter() {
        // Skip if already generating
        if tracker.has_pending_task(&chunk_entity) {
            continue;
        }

        // Get the parent test's configuration
        let Ok(test) = test_query.get(chunk.parent_test) else {
            log::warn!("Chunk {:?} has invalid parent_test reference", chunk_entity);
            continue;
        };

        let subdivisions = test.config.subdivisions;
        let bounds = chunk.bounds;

        log::debug!("Spawning mesh generation for chunk {:?} with bounds {:?}", chunk_entity, bounds);

        // Spawn async task to generate the mesh
        tracker.spawn_for_entity(
            chunk_entity,
            move || generate_flat_quad_mesh(subdivisions, bounds),
            |mut entity_mut, mesh| {
                log::info!("Inserting mesh for chunk entity {:?}", entity_mut.id());
                entity_mut.insert(mesh);
            },
        );
    }
}

/// Generate a cube mesh for a quad chunk (so it's visible from any angle)
fn generate_flat_quad_mesh(subdivisions: u32, bounds: (f32, f32, f32, f32)) -> Mesh {
    let (x_min, x_max, z_min, z_max) = bounds;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let height = 50.0; // Cube height

    log::info!("Generating cube mesh: bounds=({}, {}, {}, {}), subdivisions={}", 
        x_min, x_max, z_min, z_max, subdivisions);

    // Generate a simple cube (8 vertices, 12 triangles)
    let positions = [
        [x_min, -height/2.0, z_min], // 0: bottom-left-front
        [x_max, -height/2.0, z_min], // 1: bottom-right-front
        [x_max, -height/2.0, z_max], // 2: bottom-right-back
        [x_min, -height/2.0, z_max], // 3: bottom-left-back
        [x_min,  height/2.0, z_min], // 4: top-left-front
        [x_max,  height/2.0, z_min], // 5: top-right-front
        [x_max,  height/2.0, z_max], // 6: top-right-back
        [x_min,  height/2.0, z_max], // 7: top-left-back
    ];

    // Add vertices for each face (need unique normals per face)
    // Bottom face (y = -height/2)
    for &i in &[0, 1, 2, 3] {
        vertices.push(Vertex {
            position: positions[i],
            uv: [0.0, 0.0],
            normal: [0.0, -1.0, 0.0],
        });
    }
    
    // Top face (y = height/2)
    for &i in &[4, 5, 6, 7] {
        vertices.push(Vertex {
            position: positions[i],
            uv: [0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
        });
    }

    // Front face (z = z_min)
    for &i in &[0, 1, 5, 4] {
        vertices.push(Vertex {
            position: positions[i],
            uv: [0.0, 0.0],
            normal: [0.0, 0.0, -1.0],
        });
    }

    // Back face (z = z_max)
    for &i in &[2, 3, 7, 6] {
        vertices.push(Vertex {
            position: positions[i],
            uv: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        });
    }

    // Left face (x = x_min)
    for &i in &[3, 0, 4, 7] {
        vertices.push(Vertex {
            position: positions[i],
            uv: [0.0, 0.0],
            normal: [-1.0, 0.0, 0.0],
        });
    }

    // Right face (x = x_max)
    for &i in &[1, 2, 6, 5] {
        vertices.push(Vertex {
            position: positions[i],
            uv: [0.0, 0.0],
            normal: [1.0, 0.0, 0.0],
        });
    }

    // Generate indices for all 6 faces (2 triangles per face, CCW from outside)
    for face in 0..6 {
        let base = face * 4;
        // Triangle 1 (CCW from outside)
        indices.push((base + 0) as Index);
        indices.push((base + 2) as Index);
        indices.push((base + 1) as Index);
        // Triangle 2 (CCW from outside)
        indices.push((base + 0) as Index);
        indices.push((base + 3) as Index);
        indices.push((base + 2) as Index);
    }

    log::info!("Generated cube mesh with {} vertices, {} indices", vertices.len(), indices.len());

    Mesh { vertices, indices }
}

/// Split quad chunks based on camera distance
pub fn split_quad_chunks(
    mut commands: Commands,
    camera_query: Query<(&Camera, &Transform), With<Camera>>,
    test_query: Query<&QuadLodTest>,
    mut chunk_query: Query<(Entity, &mut QuadChunk)>,
) {
    // Find main camera
    let Some((_, camera_transform)) = camera_query.iter().find(|(cam, _)| cam.is_main) else {
        return;
    };

    let camera_pos = camera_transform.position;

    // Collect chunks to split (to avoid borrow conflicts)
    let mut chunks_to_split = Vec::new();

    for (chunk_entity, chunk) in chunk_query.iter() {
        // Skip if already has children
        if chunk.children.is_some() {
            continue;
        }

        // Get configuration
        let Ok(test) = test_query.get(chunk.parent_test) else {
            continue;
        };

        // Check if at max depth
        if chunk.depth >= test.config.max_depth {
            continue;
        }

        // Calculate distance from camera to chunk center
        let distance = (camera_pos - chunk.center).magnitude();

        // Check if should split
        let split_threshold = test.config.split_distances[chunk.depth as usize];
        if distance < split_threshold {
            chunks_to_split.push((chunk_entity, chunk.parent_test, chunk.bounds, chunk.depth));
        }
    }

    // Perform splits
    for (parent_entity, parent_test, parent_bounds, parent_depth) in chunks_to_split {
        log::debug!(
            "Splitting chunk {:?} at depth {}",
            parent_entity,
            parent_depth
        );

        let mut child_entities = [Entity::PLACEHOLDER; 4];

        // Spawn 4 children
        for child_index in 0..4 {
            let child_chunk = QuadChunk::new_child(parent_test, parent_bounds, child_index, parent_depth);

            let child_entity = commands
                .spawn((
                    Tag {
                        label: format!("Quad Chunk D{} I{}", parent_depth + 1, child_index),
                    },
                    child_chunk,
                    QuadChunkParent {
                        entity: parent_test,
                    },
                    Transform::default(),
                    Material::standard(),
                    Texture {
                        bytes: include_bytes!("../cat.png").to_vec(),
                    },
                ))
                .id();

            child_entities[child_index as usize] = child_entity;
        }

        // Update parent to reference children and hide it
        if let Ok((_, mut parent_chunk)) = chunk_query.get_mut(parent_entity) {
            parent_chunk.children = Some(child_entities);
        }

        // Hide parent by removing its mesh and GPU mesh
        commands.entity(parent_entity).remove::<(Mesh, GpuMesh)>();
    }
}

/// Collapse quad chunks based on camera distance
pub fn collapse_quad_chunks(
    mut commands: Commands,
    camera_query: Query<(&Camera, &Transform), With<Camera>>,
    test_query: Query<&QuadLodTest>,
    mut chunk_query: Query<(Entity, &mut QuadChunk)>,
) {
    // Find main camera
    let Some((_, camera_transform)) = camera_query.iter().find(|(cam, _)| cam.is_main) else {
        return;
    };

    let camera_pos = camera_transform.position;

    // Collect chunks to collapse
    let mut chunks_to_collapse = Vec::new();

    for (chunk_entity, chunk) in chunk_query.iter() {
        // Only check chunks that have children
        let Some(children) = chunk.children else {
            continue;
        };

        // Get configuration
        let Ok(test) = test_query.get(chunk.parent_test) else {
            continue;
        };

        // Check if ALL children are far enough to collapse
        let collapse_threshold = test.config.collapse_distances[chunk.depth as usize];
        let all_far = children.iter().all(|&child_entity| {
            if let Ok((_, child_chunk)) = chunk_query.get(child_entity) {
                let distance = (camera_pos - child_chunk.center).magnitude();
                distance > collapse_threshold
            } else {
                true // Child doesn't exist, allow collapse
            }
        });

        if all_far {
            chunks_to_collapse.push((chunk_entity, children));
        }
    }

    // Perform collapses
    for (parent_entity, children) in chunks_to_collapse {
        log::debug!("Collapsing chunk {:?}", parent_entity);

        // Recursively despawn all children
        for child_entity in children {
            despawn_chunk_recursive(&mut commands, &chunk_query, child_entity);
        }

        // Update parent to remove children reference
        // Note: Mesh will be regenerated by generate_quad_chunk_meshes system
        if let Ok((_, mut parent_chunk)) = chunk_query.get_mut(parent_entity) {
            parent_chunk.children = None;
        }
    }
}

/// Recursively despawn a chunk and all its descendants
fn despawn_chunk_recursive(
    commands: &mut Commands,
    chunk_query: &Query<(Entity, &mut QuadChunk)>,
    entity: Entity,
) {
    // Get children before despawning
    if let Ok((_, chunk)) = chunk_query.get(entity) {
        if let Some(children) = chunk.children {
            for child_entity in children {
                despawn_chunk_recursive(commands, chunk_query, child_entity);
            }
        }
    }

    commands.entity(entity).despawn();
}

/// Copy Material from QuadLodTest parent to children when it changes
pub fn copy_quad_material_to_children(
    parent_query: Query<(Entity, &Material), (With<QuadLodTest>, Changed<Material>)>,
    mut children_query: Query<(&QuadChunkParent, &mut Material), With<QuadChunkParent>>,
) {
    for (parent_entity, parent_material) in parent_query.iter() {
        for (chunk_parent, mut child_material) in children_query.iter_mut() {
            if chunk_parent.entity == parent_entity {
                *child_material = parent_material.clone();
            }
        }
    }
}

/// Copy Texture from QuadLodTest parent to children when it changes
pub fn copy_quad_texture_to_children(
    parent_query: Query<(Entity, &Texture), (With<QuadLodTest>, Changed<Texture>)>,
    mut commands: Commands,
    mut child_queries: ParamSet<(
        Query<(&QuadChunkParent, &mut Texture), With<QuadChunkParent>>,
        Query<(Entity, &QuadChunkParent), (With<QuadChunkParent>, Without<Texture>)>,
    )>,
) {
    // Collect updates first to avoid borrow conflicts
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

/// Copy Transform from QuadLodTest parent to children when it changes
pub fn copy_quad_transform_to_children(
    parent_query: Query<(Entity, &Transform), (With<QuadLodTest>, Changed<Transform>)>,
    mut children_query: Query<(&QuadChunkParent, &mut Transform), With<QuadChunkParent>>,
) {
    for (parent_entity, parent_transform) in parent_query.iter() {
        for (chunk_parent, mut child_transform) in children_query.iter_mut() {
            if chunk_parent.entity == parent_entity {
                // Copy parent's transform to children
                child_transform.position = parent_transform.position;
                child_transform.rotation = parent_transform.rotation;
                child_transform.scale = parent_transform.scale;
            }
        }
    }
}
