use crate::prelude::*;

/// Update instanced LOD chunks based on camera distance
/// This replaces the old split/collapse entity spawning with in-memory Vec updates
pub fn update_instanced_quad_lod(
    camera_query: Query<(&Camera, &Transform), With<Camera>>,
    mut lod_query: Query<(&mut InstancedLodMesh, &Transform), With<QuadLodTest>>,
    test_query: Query<&QuadLodTest>,
) {
    // Find main camera
    let Some((_, camera_transform)) = camera_query.iter().find(|(cam, _)| cam.is_main) else {
        return;
    };

    let camera_pos = camera_transform.position;

    for (mut instanced_mesh, lod_transform) in lod_query.iter_mut() {
        // Get parent entity to find config
        // For now, just use default config
        let config = QuadLodConfig::default();
        
        // Debug: Check distance to root chunk (only log visible leaf chunks)
        let visible_leaf_count = instanced_mesh.chunks.iter()
            .filter(|c| c.visible && c.children.is_none())
            .count();
        
        // Compute entity's world transform matrix
        let entity_matrix = {
            let translation = Matrix4::new_translation(&lod_transform.position.coords);
            let rotation = lod_transform.rotation.to_homogeneous();
            let scale = Matrix4::new_nonuniform_scaling(&lod_transform.scale);
            translation * rotation * scale
        };
        
        let mut needs_update = false;

        // Check all chunks for split/collapse
        let mut chunks_to_process: Vec<usize> = (0..instanced_mesh.chunks.len()).collect();
        
        while let Some(chunk_idx) = chunks_to_process.pop() {
            if chunk_idx >= instanced_mesh.chunks.len() {
                continue;
            }

            let chunk = &instanced_mesh.chunks[chunk_idx];
            
            // Skip if has children (only process leaf nodes)
            if chunk.children.is_some() {
                continue;
            }

            let distance = (camera_pos - chunk.center).magnitude();

            // Check if should split
            if chunk.depth < config.max_depth as u32 {
                let split_threshold = config.split_distances[chunk.depth as usize];
                
                if distance < split_threshold {
                    // SPLIT: Create 4 child chunks
                    log::info!("Splitting instanced chunk at depth {} (distance: {:.1} < threshold: {})", 
                        chunk.depth, distance, split_threshold);
                    
                    let (x_min, x_max, z_min, z_max) = chunk.bounds;
                    let x_mid = (x_min + x_max) / 2.0;
                    let z_mid = (z_min + z_max) / 2.0;
                    let child_depth = chunk.depth + 1;

                    let child_bounds = [
                        (x_min, x_mid, z_min, z_mid), // Bottom-left
                        (x_mid, x_max, z_min, z_mid), // Bottom-right
                        (x_min, x_mid, z_mid, z_max), // Top-left
                        (x_mid, x_max, z_mid, z_max), // Top-right
                    ];

                    let mut child_indices = [0; 4];
                    for (i, bounds) in child_bounds.iter().enumerate() {
                        let (cx_min, cx_max, cz_min, cz_max) = bounds;
                        let center_local = Point3::new(
                            (cx_min + cx_max) / 2.0,
                            0.0,
                            (cz_min + cz_max) / 2.0,
                        );

                        // Transform for this chunk (scale and position) in local space
                        let size = cx_max - cx_min;
                        let local_transform = Matrix4::new_translation(&Vector3::new(*cx_min + size / 2.0, 0.0, *cz_min + size / 2.0))
                            * Matrix4::new_nonuniform_scaling(&Vector3::new(size / 2.0, 50.0, size / 2.0));
                        
                        // Apply entity transform to get world space transform
                        let world_transform = entity_matrix * local_transform;
                        
                        // Transform center to world space for distance calculations
                        let center_world = entity_matrix.transform_point(&center_local);

                        let child = LodChunk::new(*bounds, child_depth, center_world, world_transform);
                        
                        child_indices[i] = instanced_mesh.chunks.len();
                        instanced_mesh.chunks.push(child);
                        chunks_to_process.push(child_indices[i]);
                    }

                    // Update parent to reference children and hide it
                    instanced_mesh.chunks[chunk_idx].children = Some(child_indices);
                    instanced_mesh.chunks[chunk_idx].visible = false;
                    
                    needs_update = true;
                }
            }

            // Check if parent should collapse
            // (This requires checking parent chunks, which is more complex - skip for now)
        }

        if needs_update {
            instanced_mesh.mark_dirty();
            log::info!("LOD updated: {} total chunks, {} visible leaves", 
                instanced_mesh.chunks.len(), visible_leaf_count);
        }
    }
}

/// Initialize instanced LOD mesh with root chunk
pub fn initialize_instanced_quad_lod(
    mut lod_query: Query<(&mut InstancedLodMesh, &Transform), (With<QuadLodTest>, Added<InstancedLodMesh>)>,
) {
    for (mut instanced_mesh, lod_transform) in lod_query.iter_mut() {
        if !instanced_mesh.chunks.is_empty() {
            continue; // Already initialized
        }

        log::info!("Initializing instanced quad LOD");

        // Compute entity's world transform matrix
        let entity_matrix = {
            let translation = Matrix4::new_translation(&lod_transform.position.coords);
            let rotation = lod_transform.rotation.to_homogeneous();
            let scale = Matrix4::new_nonuniform_scaling(&lod_transform.scale);
            translation * rotation * scale
        };

        // Create root chunk covering entire area
        let bounds = (-1000.0, 1000.0, -1000.0, 1000.0);
        let center_local = Point3::new(0.0, 0.0, 0.0);
        
        // Root chunk transform in local space (covers -1000 to 1000)
        let local_transform = Matrix4::new_nonuniform_scaling(&Vector3::new(1000.0, 50.0, 1000.0));
        
        // Apply entity transform to get world space transform
        let world_transform = entity_matrix * local_transform;
        
        // Transform center to world space
        let center_world = entity_matrix.transform_point(&center_local);

        let root_chunk = LodChunk::new(bounds, 0, center_world, world_transform);
        
        instanced_mesh.chunks.push(root_chunk);
        instanced_mesh.mark_dirty();
        
        log::info!("Created root chunk with {} vertices, {} indices", 
            instanced_mesh.base_mesh.vertices.len(), 
            instanced_mesh.base_mesh.indices.len());
    }
}

/// Update chunk transforms when entity Transform changes
pub fn update_instanced_lod_transforms(
    mut lod_query: Query<(&mut InstancedLodMesh, &Transform), (With<QuadLodTest>, Changed<Transform>)>,
) {
    for (mut instanced_mesh, lod_transform) in lod_query.iter_mut() {
        // Compute entity's world transform matrix
        let entity_matrix = {
            let translation = Matrix4::new_translation(&lod_transform.position.coords);
            let rotation = lod_transform.rotation.to_homogeneous();
            let scale = Matrix4::new_nonuniform_scaling(&lod_transform.scale);
            translation * rotation * scale
        };
        
        log::info!("Transform changed - updating {} chunk transforms", instanced_mesh.chunks.len());
        
        // Rebuild transforms for all chunks
        // Store original local transforms if not already stored
        // For now, just rebuild from scratch based on bounds
        for chunk in instanced_mesh.chunks.iter_mut() {
            let (x_min, x_max, z_min, z_max) = chunk.bounds;
            let size = x_max - x_min;
            
            // Reconstruct local transform from bounds
            let local_transform = Matrix4::new_translation(&Vector3::new(x_min + size / 2.0, 0.0, z_min + size / 2.0))
                * Matrix4::new_nonuniform_scaling(&Vector3::new(size / 2.0, 50.0, size / 2.0));
            
            // Apply entity transform
            chunk.transform = entity_matrix * local_transform;
            
            // Update center position in world space
            let center_local = Vector3::new((x_min + x_max) / 2.0, 0.0, (z_min + z_max) / 2.0);
            let center_world = entity_matrix.transform_point(&Point3::from(center_local));
            chunk.center = center_world;
        }
        
        instanced_mesh.mark_dirty();
    }
}

/// Clear dirty flags after GPU update has processed them
pub fn clear_instanced_lod_dirty_flags(
    mut lod_query: Query<&mut InstancedLodMesh>,
) {
    for mut instanced_mesh in lod_query.iter_mut() {
        if instanced_mesh.dirty {
            instanced_mesh.dirty = false;
        }
    }
}
