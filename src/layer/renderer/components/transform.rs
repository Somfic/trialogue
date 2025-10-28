use crate::prelude::*;

#[derive(Component)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            rotation: Quaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Component)]
pub struct GpuTransform {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}
