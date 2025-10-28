use bevy_ecs::prelude::*;

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
