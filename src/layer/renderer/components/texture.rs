use crate::prelude::*;

#[derive(Component)]
pub struct Texture {
    pub bytes: Vec<u8>,
}

#[derive(Component)]
pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}
