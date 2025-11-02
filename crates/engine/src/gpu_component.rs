use crate::prelude::*;

use bevy_ecs::prelude::*;

/// Marker trait that associates a user-facing component with its GPU variant
pub trait GpuComponent {
    /// The user-facing component type (e.g., Mesh)
    type UserComponent: Component;

    /// The GPU-resident component type (e.g., GpuMesh)
    type GpuVariant: Component;
}

/// Trait for components that can be initialized to GPU variants
pub trait GpuInitialize: GpuComponent {
    /// Bundle of components this GPU component depends on (e.g., (Transform,) for Camera)
    /// Use () for no dependencies
    type Dependencies: Bundle;

    /// Initialize a GPU component from the user component
    ///
    /// # Arguments
    /// * `user` - The user-facing component data
    /// * `dependencies` - Optional bundle of sibling components this depends on
    /// * `device` - GPU device for creating buffers/textures
    /// * `queue` - GPU queue for writing data
    /// * `context` - Shared GPU context (bind group layouts, etc.)
    fn initialize(
        user: &Self::UserComponent,
        dependencies: Option<&Self::Dependencies>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        context: &GpuContext,
    ) -> Self::GpuVariant;
}

/// Trait for GPU components that support incremental updates
/// Only implement this if your component can be updated efficiently
pub trait GpuUpdate: GpuComponent {
    /// Update an existing GPU component when the user component changes
    ///
    /// # Arguments
    /// * `user` - The updated user-facing component data
    /// * `gpu` - Mutable reference to the existing GPU component
    /// * `dependencies` - Optional bundle of sibling components this depends on
    /// * `device` - GPU device for recreating resources if needed
    /// * `queue` - GPU queue for writing updated data
    fn update(
        user: &Self::UserComponent,
        gpu: &mut Self::GpuVariant,
        dependencies: Option<&<Self as GpuInitialize>::Dependencies>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) where
        Self: GpuInitialize;
}

/// Shared GPU context containing bind group layouts and other shared resources
#[derive(Resource)]
pub struct GpuContext {
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub transform_bind_group_layout: wgpu::BindGroupLayout,
}

impl GpuContext {
    pub fn new(
        texture_bind_group_layout: wgpu::BindGroupLayout,
        camera_bind_group_layout: wgpu::BindGroupLayout,
        transform_bind_group_layout: wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            texture_bind_group_layout,
            camera_bind_group_layout,
            transform_bind_group_layout,
        }
    }
}

/// System generator for components with no dependencies
/// Creates a system that initializes GPU components when user components are added
pub fn gpu_initialize_system<T>(
    mut commands: Commands,
    device: Res<GpuDevice>,
    queue: Res<GpuQueue>,
    context: Res<GpuContext>,
    query: Query<(Entity, &T::UserComponent), Without<T::GpuVariant>>,
) where
    T: GpuInitialize<Dependencies = ()>,
{
    for (entity, user_component) in query.iter() {
        let gpu_component = T::initialize(
            user_component,
            None, // No dependencies
            &device.0,
            &queue.0,
            &context,
        );

        commands.entity(entity).insert(gpu_component);
        log::debug!("Initialized GPU component for Entity {:?}", entity);
    }
}

/// System generator for components with no dependencies
/// Creates a system that updates existing GPU components when user components change
pub fn gpu_update_system<T>(
    device: Res<GpuDevice>,
    queue: Res<GpuQueue>,
    mut query: Query<(Entity, &T::UserComponent, &mut T::GpuVariant), Changed<T::UserComponent>>,
) where
    T: GpuUpdate + GpuInitialize<Dependencies = ()>,
    T::GpuVariant: Component<Mutability = bevy_ecs::component::Mutable>,
{
    for (entity, user_component, mut gpu_component) in query.iter_mut() {
        T::update(
            user_component,
            &mut gpu_component,
            None, // No dependencies
            &device.0,
            &queue.0,
        );

        log::debug!("Updated GPU component for Entity {:?}", entity);
    }
}

/// System generator for components with single Transform dependency
/// Creates a system that initializes GPU components when user components are added
pub fn gpu_initialize_with_transform_system<T>(
    mut commands: Commands,
    device: Res<GpuDevice>,
    queue: Res<GpuQueue>,
    context: Res<GpuContext>,
    query: Query<(Entity, &T::UserComponent, &Transform), Without<T::GpuVariant>>,
) where
    T: GpuInitialize<Dependencies = (Transform,)>,
{
    for (entity, user_component, transform) in query.iter() {
        let gpu_component = T::initialize(
            user_component,
            Some(&(transform.clone(),)),
            &device.0,
            &queue.0,
            &context,
        );

        commands.entity(entity).insert(gpu_component);
        log::debug!(
            "Initialized GPU component with Transform for Entity {:?}",
            entity
        );
    }
}

/// System generator for components with Transform dependency that updates on Transform OR Camera changes
pub fn gpu_update_with_transform_system<T>(
    device: Res<GpuDevice>,
    queue: Res<GpuQueue>,
    mut query: Query<
        (Entity, &T::UserComponent, &Transform, &mut T::GpuVariant),
        Or<(Changed<T::UserComponent>, Changed<Transform>)>,
    >,
) where
    T: GpuUpdate + GpuInitialize<Dependencies = (Transform,)>,
    T::GpuVariant: Component<Mutability = bevy_ecs::component::Mutable>,
{
    for (entity, user_component, transform, mut gpu_component) in query.iter_mut() {
        T::update(
            user_component,
            &mut gpu_component,
            Some(&(transform.clone(),)),
            &device.0,
            &queue.0,
        );

        log::debug!(
            "Updated GPU component with Transform for Entity {:?}",
            entity
        );
    }
}
