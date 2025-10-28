use bevy_ecs::prelude::*;

use crate::layer::renderer::{
    GpuQueue,
    components::{GpuDevice, GpuTexture, Texture, TextureBindGroupLayout},
};

pub fn initialize_texture_buffers(
    mut commands: Commands,
    device: Res<GpuDevice>,
    queue: Res<GpuQueue>,
    texture_bind_group_layout: Res<TextureBindGroupLayout>,
    texture_query: Query<(Entity, &Texture), Without<GpuTexture>>,
) {
    let device = &device.0;
    let queue = &queue.0;
    let texture_bind_group_layout = &texture_bind_group_layout.0;

    for (entity, texture) in texture_query.iter() {
        let image = image::load_from_memory(&texture.bytes).unwrap();
        let image = image.to_rgba8();

        let dimensions = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        // create new texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("texture"), // todo: label
            view_formats: &[],
        });

        // write to new texture
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"), // todo: label
        });

        commands.entity(entity).insert(GpuTexture {
            texture,
            view,
            sampler,
            bind_group,
        });

        log::debug!("Created texture buffer")
    }
}
