use crate::prelude::*;

#[derive(Component, Clone, PartialEq)]
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

#[derive(Component)]
pub struct GpuDepthTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

#[derive(Component)]
pub struct GpuShadowMap {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
    pub light_buffer: wgpu::Buffer,
    pub light_dir_buffer: wgpu::Buffer,
    pub light_properties_buffer: wgpu::Buffer,
    pub shadow_uniform_bind_group: wgpu::BindGroup,
    pub light_dir: Vector3<f32>, // Store for comparison
    pub light_intensity: f32,
    pub light_color: [f32; 3],
}

// Helper constant for coordinate system conversion
#[rustfmt::skip]
const OPENGL_TO_WGPU: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

// GPU Component trait implementations
impl GpuComponent for Camera {
    type UserComponent = Camera;
    type GpuVariant = GpuCamera;
}

impl GpuInitialize for Camera {
    type Dependencies = (Transform,);

    fn initialize(
        user: &Self::UserComponent,
        dependencies: Option<&Self::Dependencies>,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        context: &GpuContext,
    ) -> Self::GpuVariant {
        use wgpu::util::DeviceExt;

        let transform = &dependencies.expect("Camera requires Transform component").0;

        // Compute view matrix from transform
        let up = transform.rotation * Vector3::y_axis();
        let view = Isometry3::look_at_rh(&transform.position, &user.target, &up).to_homogeneous();

        // Compute projection matrix (initial aspect ratio 1.0, will be updated)
        let proj = OPENGL_TO_WGPU
            * Perspective3::new(1.0, user.fovy, user.znear, user.zfar).to_homogeneous();

        let matrix = proj * view;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[matrix]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &context.camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        GpuCamera {
            buffer,
            bind_group,
            aspect: 1.0,
        }
    }
}

impl GpuUpdate for Camera {
    fn update(
        user: &Self::UserComponent,
        gpu: &mut Self::GpuVariant,
        dependencies: Option<&(Transform,)>,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let transform = &dependencies.expect("Camera requires Transform component").0;

        // Compute view matrix
        let up = transform.rotation * Vector3::y_axis();
        let view = Isometry3::look_at_rh(&transform.position, &user.target, &up).to_homogeneous();

        // Compute projection matrix using current aspect ratio
        let proj = OPENGL_TO_WGPU
            * Perspective3::new(gpu.aspect, user.fovy, user.znear, user.zfar).to_homogeneous();

        let matrix = proj * view;

        queue.write_buffer(&gpu.buffer, 0, bytemuck::cast_slice(&[matrix]));
    }
}
