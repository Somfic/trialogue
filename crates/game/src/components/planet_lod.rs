use crate::prelude::*;

/// Marker component indicating that certain components should be copied to children when changed
#[derive(Component)]
pub struct CopyToChildren;

/// Marker component for chunk entities to track their parent planet
/// This allows us to find all children of a parent efficiently
#[derive(Component)]
pub struct ChunkParent {
    pub entity: Entity,
}

/// Configuration for LOD behavior
#[derive(Clone, Copy, Debug)]
pub struct LodConfig {
    /// Number of subdivisions per chunk at each LOD level
    pub base_subdivisions: u32,
    /// Maximum quadtree depth (0-indexed, so 5 = 6 total levels)
    pub max_depth: u32,
    /// Distance thresholds to trigger split at each depth level
    /// If chunk distance < split_distances[depth], it should split
    pub split_distances: [f32; 6],
    /// Distance thresholds to trigger collapse at each depth level
    /// If chunk distance > collapse_distances[depth], it should collapse
    /// These should be higher than split_distances to provide hysteresis
    pub collapse_distances: [f32; 6],
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            base_subdivisions: 20,
            max_depth: 5,
            // Tuned for planet radius ~1.0
            // Index 0 = root level, index 5 = deepest level
            split_distances: [5.0, 3.0, 1.5, 0.8, 0.4, 0.0],
            collapse_distances: [6.0, 4.0, 2.0, 1.0, 0.5, 0.0],
        }
    }
}

/// Parent component for LOD-managed planet
#[derive(Component)]
pub struct PlanetLod {
    /// LOD configuration
    pub config: LodConfig,
    /// Current raycast hit point on the planet surface (if any)
    pub raycast_hit: Option<Point3<f32>>,
    /// Terrain noise configuration
    pub terrain_config: TerrainConfig,
    /// Seed for deterministic noise generation
    pub seed: String,
}

impl PlanetLod {
    pub fn new(seed: String) -> Self {
        Self {
            config: LodConfig::default(),
            raycast_hit: None,
            terrain_config: TerrainConfig::default(),
            seed,
        }
    }

    /// Convert seed string to deterministic u32 for noise generation
    pub fn seed_u32(&self) -> u32 {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.seed.hash(&mut hasher);
        hasher.finish() as u32
    }
}

/// Represents one node in the planet quadtree
#[derive(Component)]
pub struct PlanetChunk {
    /// Parent planet entity (the one with PlanetLod)
    pub parent_planet: Entity,
    /// Which cube face this chunk belongs to
    pub face: CubeFace,
    /// UV bounds within the cube face: (u_min, u_max, v_min, v_max)
    /// Root chunks have (0.0, 1.0, 0.0, 1.0)
    pub uv_bounds: (f32, f32, f32, f32),
    /// Current depth in the quadtree (0 = root chunk)
    pub depth: u32,
    /// If this chunk is subdivided, references to its 4 child chunks
    /// Ordering: [bottom-left, bottom-right, top-left, top-right]
    pub children: Option<[Entity; 4]>,
}

impl PlanetChunk {
    /// Create a new root chunk for a cube face
    pub fn new_root(parent_planet: Entity, face: CubeFace) -> Self {
        Self {
            parent_planet,
            face,
            uv_bounds: (0.0, 1.0, 0.0, 1.0),
            depth: 0,
            children: None,
        }
    }

    /// Create a child chunk from a parent's UV bounds
    /// child_index: 0=bottom-left, 1=bottom-right, 2=top-left, 3=top-right
    pub fn new_child(parent_planet: Entity, face: CubeFace, parent_bounds: (f32, f32, f32, f32), child_index: u32, parent_depth: u32) -> Self {
        let (u_min, u_max, v_min, v_max) = parent_bounds;
        let u_mid = (u_min + u_max) / 2.0;
        let v_mid = (v_min + v_max) / 2.0;

        let uv_bounds = match child_index {
            0 => (u_min, u_mid, v_min, v_mid), // Bottom-left
            1 => (u_mid, u_max, v_min, v_mid), // Bottom-right
            2 => (u_min, u_mid, v_mid, v_max), // Top-left
            3 => (u_mid, u_max, v_mid, v_max), // Top-right
            _ => panic!("Invalid child index: {}", child_index),
        };

        Self {
            parent_planet,
            face,
            uv_bounds,
            depth: parent_depth + 1,
            children: None,
        }
    }

    /// Calculate the world-space center of this chunk
    pub fn center(&self) -> Point3<f32> {
        let (u_min, u_max, v_min, v_max) = self.uv_bounds;
        let u_center = (u_min + u_max) / 2.0;
        let v_center = (v_min + v_max) / 2.0;

        // Convert UV to cube space, then normalize to get sphere position
        let cube_pos = cube_face_uv_to_xyz(&self.face, u_center, v_center);
        let sphere_pos = cube_pos.normalize();

        Point3::from(sphere_pos)
    }
}

/// Convert UV coordinates on a cube face to 3D xyz position on unit cube
fn cube_face_uv_to_xyz(face: &CubeFace, u: f32, v: f32) -> Vector3<f32> {
    // Map UV [0,1] to cube coordinates [-1,1]
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
