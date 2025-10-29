use std::time::Duration;

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
