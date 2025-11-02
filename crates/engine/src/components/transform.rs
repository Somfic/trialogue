
use crate::prelude::*;

#[derive(Component, Clone, PartialEq)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Component)]
pub struct GpuTransform {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

// Helper function for computing model matrix
fn compute_model_matrix(transform: &Transform) -> Matrix4<f32> {
    let translation = Matrix4::new_translation(&transform.position.coords);
    let rotation = transform.rotation.to_homogeneous();
    let scale = Matrix4::new_nonuniform_scaling(&transform.scale);
    translation * rotation * scale
}

// GPU Component trait implementations
impl GpuComponent for Transform {
    type UserComponent = Transform;
    type GpuVariant = GpuTransform;
}

impl GpuInitialize for Transform {
    type Dependencies = ();

    fn initialize(
        user: &Self::UserComponent,
        _dependencies: Option<&Self::Dependencies>,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        context: &GpuContext,
    ) -> Self::GpuVariant {
        use wgpu::util::DeviceExt;

        let model_matrix = compute_model_matrix(user);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[model_matrix]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &context.transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("transform_bind_group"),
        });

        GpuTransform { buffer, bind_group }
    }
}

impl GpuUpdate for Transform {
    fn update(
        user: &Self::UserComponent,
        gpu: &mut Self::GpuVariant,
        _dependencies: Option<&()>,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let model_matrix = compute_model_matrix(user);
        queue.write_buffer(&gpu.buffer, 0, bytemuck::cast_slice(&[model_matrix]));
    }
}
