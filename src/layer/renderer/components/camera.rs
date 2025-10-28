use crate::prelude::*;

#[derive(Component)]
pub struct Transform {
    pub position: Point3<f32>,
    pub up: Vector3<f32>,
}

#[derive(Component)]
pub struct Camera {
    pub target: Point3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[derive(Component)]
pub struct GpuCamera {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_projection: Matrix4<f32>,
}
