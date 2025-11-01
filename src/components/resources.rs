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
