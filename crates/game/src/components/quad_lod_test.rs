use crate::prelude::*;

/// Marker component for the root quad LOD test entity
#[derive(Component)]
pub struct QuadLodTest {
    pub config: QuadLodConfig,
}

impl QuadLodTest {
    pub fn new() -> Self {
        Self {
            config: QuadLodConfig::default(),
        }
    }
}

/// Configuration for quad LOD behavior
#[derive(Clone, Copy, Debug)]
pub struct QuadLodConfig {
    /// Number of subdivisions per quad chunk
    pub subdivisions: u32,
    /// Maximum quadtree depth (0-indexed, so 9 = 10 total levels)
    pub max_depth: u32,
    /// Distance thresholds to trigger split at each depth level
    /// If chunk distance < split_distances[depth], it should split
    pub split_distances: [f32; 10],
    /// Distance thresholds to trigger collapse at each depth level
    /// These should be higher than split_distances to provide hysteresis
    pub collapse_distances: [f32; 10],
}

impl Default for QuadLodConfig {
    fn default() -> Self {
        Self {
            subdivisions: 10,
            max_depth: 9,
            // Distance thresholds - starts at 1000m and halves each level
            split_distances: [1000.0, 500.0, 250.0, 125.0, 62.5, 31.25, 15.6, 7.8, 3.9, 2.0],
            // Collapse at 1.5x split distance for hysteresis
            collapse_distances: [1500.0, 750.0, 375.0, 187.5, 93.75, 46.875, 23.4, 11.7, 5.85, 3.0],
        }
    }
}

/// Marker component to track parent-child relationship for quad chunks
#[derive(Component)]
pub struct QuadChunkParent {
    pub entity: Entity,
}

/// Represents one node in the quad LOD tree
#[derive(Component)]
pub struct QuadChunk {
    /// Parent QuadLodTest entity
    pub parent_test: Entity,
    /// XZ bounds: (x_min, x_max, z_min, z_max)
    pub bounds: (f32, f32, f32, f32),
    /// Current depth in the quadtree (0 = root chunk)
    pub depth: u32,
    /// Cached center point for distance calculations
    pub center: Point3<f32>,
    /// If this chunk is subdivided, references to its 4 child chunks
    /// Ordering: [bottom-left, bottom-right, top-left, top-right]
    pub children: Option<[Entity; 4]>,
}

impl QuadChunk {
    /// Create a new root chunk
    pub fn new_root(parent_test: Entity, bounds: (f32, f32, f32, f32)) -> Self {
        let (x_min, x_max, z_min, z_max) = bounds;
        let center = Point3::new(
            (x_min + x_max) / 2.0,
            0.0, // Flat quad at y=0
            (z_min + z_max) / 2.0,
        );

        Self {
            parent_test,
            bounds,
            depth: 0,
            center,
            children: None,
        }
    }

    /// Create a child chunk from parent's bounds
    /// child_index: 0=bottom-left, 1=bottom-right, 2=top-left, 3=top-right
    pub fn new_child(
        parent_test: Entity,
        parent_bounds: (f32, f32, f32, f32),
        child_index: u32,
        parent_depth: u32,
    ) -> Self {
        let (x_min, x_max, z_min, z_max) = parent_bounds;
        let x_mid = (x_min + x_max) / 2.0;
        let z_mid = (z_min + z_max) / 2.0;

        let bounds = match child_index {
            0 => (x_min, x_mid, z_min, z_mid), // Bottom-left
            1 => (x_mid, x_max, z_min, z_mid), // Bottom-right
            2 => (x_min, x_mid, z_mid, z_max), // Top-left
            3 => (x_mid, x_max, z_mid, z_max), // Top-right
            _ => panic!("Invalid child index: {}", child_index),
        };

        let (x_min, x_max, z_min, z_max) = bounds;
        let center = Point3::new((x_min + x_max) / 2.0, 0.0, (z_min + z_max) / 2.0);

        Self {
            parent_test,
            bounds,
            depth: parent_depth + 1,
            center,
            children: None,
        }
    }

    /// Get the size of this chunk (assumes square)
    pub fn size(&self) -> f32 {
        let (x_min, x_max, _, _) = self.bounds;
        x_max - x_min
    }
}

/// Marker component to hide chunks that have been subdivided
/// Chunks with this component should not be rendered
#[derive(Component)]
pub struct ChunkHidden;
