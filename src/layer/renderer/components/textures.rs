use crate::prelude::*;
use std::path::PathBuf;

#[derive(Component)]
pub struct Texture {
    pub bytes: Vec<u8>,
}

#[derive(Component)]
pub struct GpuTexture {
    pub texture: wgpu::Texture,
}
