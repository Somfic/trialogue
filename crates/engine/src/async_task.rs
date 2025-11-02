use crate::prelude::*;

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// Generic tracker for async task generations.
///
/// Prevents stale async results from being applied by tracking generation numbers.
/// Each time a new task is started for a key, the generation is incremented.
/// Async tasks can check if their generation is still current before applying results.
///
/// # Example
/// ```
/// use trialogue_engine::AsyncTaskTracker;
///
/// let mut tracker = AsyncTaskTracker::new();
///
/// // Start a task
/// let generation = tracker.start_task("mesh_1");
///
/// // In async thread:
/// if tracker.is_current(&"mesh_1", generation) {
///     // Apply result
/// }
/// ```
#[derive(Resource)]
pub struct AsyncTaskTracker<K: Hash + Eq + Clone + Send + Sync + 'static> {
    generations: HashMap<K, u64>,
}

impl<K: Hash + Eq + Clone + Send + Sync + 'static> AsyncTaskTracker<K> {
    /// Create a new async task tracker
    pub fn new() -> Self {
        Self {
            generations: HashMap::new(),
        }
    }

    /// Start a new task for the given key, returning the generation ID.
    /// This increments the generation counter, invalidating any previous tasks.
    pub fn start_task(&mut self, key: K) -> u64 {
        let generation = self.generations.entry(key).or_insert(0);
        *generation += 1;
        *generation
    }

    /// Check if a task generation is still current for the given key.
    /// Returns false if a newer task has been started or if the key doesn't exist.
    pub fn is_current(&self, key: &K, generation: u64) -> bool {
        self.generations
            .get(key)
            .map_or(false, |&current| current == generation)
    }

    /// Clean up tracking for a key (e.g., when entity is deleted)
    pub fn remove(&mut self, key: &K) {
        self.generations.remove(key);
    }

    /// Spawn an async task with automatic generation tracking.
    ///
    /// This is a helper method that encapsulates the entire async task pattern:
    /// 1. Starts a new generation for the key
    /// 2. Spawns a background thread to execute the work
    /// 3. Checks if the generation is still current before applying the result
    /// 4. Applies the result only if no newer task was started
    ///
    /// # Arguments
    /// * `world` - The world handle to access ECS
    /// * `key` - The key to track (e.g., Entity)
    /// * `work` - Closure that produces the result (runs on background thread)
    /// * `apply` - Closure that applies the result to the world (runs on background thread if generation is current)
    ///
    /// # Example
    /// ```ignore
    /// tracker.spawn_async_task(
    ///     world_handle.0.clone(),
    ///     entity,
    ///     || generate_mesh(&planet),  // work
    ///     |world, entity, mesh| {     // apply
    ///         if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
    ///             entity_mut.insert(mesh);
    ///         }
    ///     }
    /// );
    /// ```
    pub fn spawn_async_task<T, W, A>(&mut self, world: Arc<Mutex<World>>, key: K, work: W, apply: A)
    where
        T: Send + 'static,
        W: FnOnce() -> T + Send + 'static,
        A: FnOnce(&mut World, K, T) + Send + 'static,
    {
        let generation = self.start_task(key.clone());

        rayon::spawn(move || {
            // Execute work on background thread
            let result = work();

            // Lock world and apply result if generation is still current
            if let Ok(mut world) = world.lock() {
                // Check if this generation is still current (no newer task started)
                let is_current = world
                    .get_resource::<AsyncTaskTracker<K>>()
                    .map_or(false, |tracker| tracker.is_current(&key, generation));

                if is_current {
                    // Safe to apply - this is the latest generation
                    apply(&mut world, key, result);
                }
                // Otherwise, a newer task was started - discard this result
            }
        });
    }
}

impl<K: Hash + Eq + Clone + Send + Sync + 'static> Default for AsyncTaskTracker<K> {
    fn default() -> Self {
        Self::new()
    }
}

// Specialized implementation for Entity to add entity existence checks
impl AsyncTaskTracker<Entity> {
    /// Spawn an async task for an Entity with automatic existence checking.
    ///
    /// This is a specialized version that checks if the entity still exists
    /// before calling the apply closure. The apply closure receives an
    /// `EntityWorldMut` for convenient entity manipulation.
    pub fn spawn_async_task_for_entity<T, W, A>(
        &mut self,
        world: Arc<Mutex<World>>,
        entity: Entity,
        work: W,
        apply: A,
    ) where
        T: Send + 'static,
        W: FnOnce() -> T + Send + 'static,
        A: FnOnce(EntityWorldMut, T) + Send + 'static,
    {
        let generation = self.start_task(entity);

        rayon::spawn(move || {
            // Execute work on background thread
            let result = work();

            // Lock world and apply result if generation is still current and entity exists
            if let Ok(mut world) = world.lock() {
                // Check if entity still exists
                if world.get_entity(entity).is_err() {
                    return; // Entity was deleted, discard result
                }

                // Check if this generation is still current (no newer task started)
                let is_current = world
                    .get_resource::<AsyncTaskTracker<Entity>>()
                    .map_or(false, |tracker| tracker.is_current(&entity, generation));

                if is_current {
                    // Safe to apply - entity exists and this is the latest generation
                    if let Ok(entity_mut) = world.get_entity_mut(entity) {
                        apply(entity_mut, result);
                    }
                }
                // Otherwise, a newer task was started - discard this result
            }
        });
    }
}
