
use crate::prelude::*;

use std::time::Duration;

#[derive(Resource)]
pub struct GpuDevice(pub wgpu::Device);

#[derive(Resource)]
pub struct GpuQueue(pub wgpu::Queue);

#[derive(Resource)]
pub struct TextureBindGroupLayout(pub wgpu::BindGroupLayout);

#[derive(Resource)]
pub struct CameraBindGroupLayout(pub wgpu::BindGroupLayout);

#[derive(Resource)]
pub struct TransformBindGroupLayout(pub wgpu::BindGroupLayout);

#[derive(Resource)]
pub struct Time(pub Duration);

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Resource)]
pub struct GpuSurface(pub Option<wgpu::Surface<'static>>);

#[derive(Resource)]
pub struct GpuAdapter(pub Option<wgpu::Adapter>);

// Raytracer Resources
#[derive(Resource)]
pub struct RaytracerComputePipeline(pub wgpu::ComputePipeline);

#[derive(Resource)]
pub struct RaytracerDisplayPipeline(pub wgpu::RenderPipeline);

#[derive(Resource)]
pub struct RaytracerBindGroupLayout(pub wgpu::BindGroupLayout);

#[derive(Resource)]
pub struct RaytracerDisplayBindGroupLayout(pub wgpu::BindGroupLayout);

#[derive(Resource)]
pub struct RaytracerOutputTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

#[derive(Resource)]
pub struct RaytracerBindGroup(pub wgpu::BindGroup);

#[derive(Resource)]
pub struct RaytracerDisplayBindGroup(pub wgpu::BindGroup);

// Raytracer scene data
#[derive(ShaderType)]
pub struct RaytracerCamera {
    pub position: Vector3<f32>,
    pub look_at: Vector3<f32>,
    pub up: Vector3<f32>,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub aperture: f32,
    pub focus_distance: f32,
    // Precomputed basis vectors
    pub u: Vector3<f32>,
    pub v: Vector3<f32>,
    pub w: Vector3<f32>,
    pub lower_left_corner: Vector3<f32>,
    pub horizontal: Vector3<f32>,
    pub vertical: Vector3<f32>,
}

impl RaytracerCamera {
    pub fn new(
        position: Vector3<f32>,
        look_at: Vector3<f32>,
        up: Vector3<f32>,
        fov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_distance: f32,
    ) -> Self {
        // Compute camera basis vectors
        let w = (position - look_at).normalize();
        let u = up.cross(&w).normalize();
        let v = w.cross(&u);

        // Compute viewport dimensions
        let theta = fov.to_radians();
        let half_height = (theta / 2.0).tan();
        let half_width = aspect_ratio * half_height;

        // Compute viewport corners and spans
        let lower_left_corner = position - half_width * u - half_height * v - w;
        let horizontal = 2.0 * half_width * u;
        let vertical = 2.0 * half_height * v;

        Self {
            position,
            look_at,
            up,
            fov,
            aspect_ratio,
            aperture,
            focus_distance,
            u,
            v,
            w,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    /// Update the camera and recompute basis vectors
    pub fn update(
        &mut self,
        position: Vector3<f32>,
        look_at: Vector3<f32>,
        up: Vector3<f32>,
        fov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_distance: f32,
    ) {
        self.position = position;
        self.look_at = look_at;
        self.up = up;
        self.fov = fov;
        self.aspect_ratio = aspect_ratio;
        self.aperture = aperture;
        self.focus_distance = focus_distance;

        // Recompute basis vectors
        self.w = (position - look_at).normalize();
        self.u = up.cross(&self.w).normalize();
        self.v = self.w.cross(&self.u);

        // Recompute viewport
        let theta = fov.to_radians();
        let half_height = (theta / 2.0).tan();
        let half_width = aspect_ratio * half_height;

        self.lower_left_corner = position - half_width * self.u - half_height * self.v - self.w;
        self.horizontal = 2.0 * half_width * self.u;
        self.vertical = 2.0 * half_height * self.v;
    }
}

#[derive(ShaderType)]
pub struct RaytracerSphere {
    pub center: Vector3<f32>,
    pub radius: f32,
    pub color: Vector3<f32>,
    pub material_type: u32,
}

#[derive(ShaderType)]
pub struct RaytracerLight {
    pub position: Vector3<f32>,
    pub intensity: f32,
    pub color: Vector3<f32>,
}

#[derive(Resource)]
pub struct RaytracerCameraBuffer(pub wgpu::Buffer);

#[derive(Resource)]
pub struct RaytracerSpheresBuffer(pub wgpu::Buffer);

#[derive(Resource)]
pub struct RaytracerLightsBuffer(pub wgpu::Buffer);

#[derive(Resource)]
pub struct RaytracerEnvironmentMap {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

#[derive(Resource, Default)]
pub struct SupportedFeatures {
    pub polygon_mode_line: bool,
    pub polygon_mode_point: bool,
}
