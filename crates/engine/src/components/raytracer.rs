
use crate::prelude::*;

/// User-facing component for spawning spheres in the raytracer scene
/// Position is taken from the Transform component
/// The Transform's scale.x is used as the radius (uniform scaling)
#[derive(Component, Clone, Copy, PartialEq)]
pub struct Sphere {
    pub color: [f32; 3],
    pub material_type: u32, // 0 = lambertian, 1 = metal, 2 = dielectric
}

/// User-facing component for spawning lights in the raytracer scene
/// Position is taken from the Transform component
#[derive(Component, Clone, Copy, PartialEq)]
pub struct Light {
    pub intensity: f32,
    pub color: [f32; 3],
}

/// GPU-side component that holds the buffer data for the entire raytracer scene
/// This is attached to a single entity that manages the scene
#[derive(Component)]
pub struct GpuRaytracerScene {
    pub spheres_buffer: wgpu::Buffer,
    pub lights_buffer: wgpu::Buffer,
    pub sphere_count: u32,
    pub light_count: u32,
}

/// User-facing component for environment map
/// Provide either a path or raw bytes to an HDR image
#[derive(Component, Clone, PartialEq)]
pub struct EnvironmentMap {
    pub bytes: Vec<u8>,
}

/// GPU-side component for environment map texture
#[derive(Component)]
pub struct GpuEnvironmentMap {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bytes_hash: u64, // Hash of the source bytes to detect actual changes
}
