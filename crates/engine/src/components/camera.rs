
use crate::prelude::*;

#[derive(Component)]
pub struct Camera {
    pub is_main: bool,
    pub target: Point3<f32>,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub aperture: f32,
    pub focus_distance: f32,
}

#[derive(Component)]
pub struct GpuCamera {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub aspect: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_projection: Matrix4<f32>,
}

#[derive(Component)]
pub struct RenderTarget {}

#[derive(Component)]
pub struct GpuRenderTarget {
    pub texture: wgpu::Texture,
}
