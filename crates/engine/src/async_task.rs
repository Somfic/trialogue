use crate::prelude::*;

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// Type alias for closures that apply async results to the world.
/// These are executed on the main thread by the `apply_async_entity_results` system.
type ApplyClosure = Box<dyn FnOnce(&mut World) + Send>;

/// Generic tracker for async task generations.
///
/// Prevents stale async results from being applied by tracking generation numbers.
/// Each time a new task is started for a key, the generation is incremented.
/// Results are queued and applied by the `apply_async_entity_results` system.
///
/// # Architecture
///
/// This implementation uses a queue-based approach:
/// 1. Tasks are spawned with generation tracking
/// 2. Background threads execute work and queue results
/// 3. The `apply_async_entity_results` system processes queued results each frame
/// 4. Results are only applied if their generation is still current
///
/// This eliminates the need to lock the World from background threads.
///
/// # Example
/// ```ignore
/// fn my_system(mut tracker: ResMut<AsyncTaskTracker<Entity>>, query: Query<(Entity, &Data), Changed<Data>>) {
///     for (entity, data) in query.iter() {
///         let data = data.clone();
///
///         tracker.spawn_for_entity(
///             entity,
///             move || expensive_computation(&data),
///             |mut entity_mut, result| {
///                 entity_mut.insert(result);
///             },
///         );
///     }
/// }
/// ```
#[derive(Resource)]
pub struct AsyncTaskTracker<K: Hash + Eq + Clone + Send + Sync + 'static> {
    /// Generation counter for each key. Incremented on each new task.
    generations: Arc<Mutex<HashMap<K, u64>>>,
    /// Queue of pending results to be applied on the main thread.
    pending_results: Arc<Mutex<Vec<ApplyClosure>>>,
}

impl<K: Hash + Eq + Clone + Send + Sync + 'static> AsyncTaskTracker<K> {
    /// Create a new async task tracker.
    pub fn new() -> Self {
        Self {
            generations: Arc::new(Mutex::new(HashMap::new())),
            pending_results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start a new task for the given key, returning the generation ID.
    /// This increments the generation counter, invalidating any previous tasks.
    fn start_task(&mut self, key: K) -> u64 {
        let mut generations = self.generations.lock().unwrap();
        let generation = generations.entry(key).or_insert(0);
        *generation += 1;
        log::debug!("Started async task generation {}", *generation);
        *generation
    }

    /// Check if a task generation is still current for the given key.
    /// Returns false if a newer task has been started or if the key doesn't exist.
    pub fn is_current(&self, key: &K, generation: u64) -> bool {
        self.generations
            .lock()
            .unwrap()
            .get(key)
            .map_or(false, |&current| current == generation)
    }

    /// Check if there's a pending task for this key.
    /// Returns true if a task has been started (generation > 0).
    pub fn has_pending_task(&self, key: &K) -> bool {
        self.generations
            .lock()
            .unwrap()
            .get(key)
            .map_or(false, |&generation| generation > 0)
    }

    /// Clean up tracking for a key (e.g., when an entity is deleted).
    pub fn remove(&mut self, key: &K) {
        self.generations.lock().unwrap().remove(key);
    }

    /// Spawn an async task with automatic generation tracking.
    ///
    /// This method:
    /// 1. Starts a new generation for the key
    /// 2. Spawns a background thread to execute the work
    /// 3. Queues the result to be applied on the main thread
    /// 4. The result is only applied if no newer task was started (checked by the apply system)
    ///
    /// # Arguments
    /// * `key` - The key to track (e.g., Entity, String, etc.)
    /// * `work` - Closure that produces the result (runs on background thread)
    /// * `apply` - Closure that applies the result to the world (runs on main thread if generation is current)
    ///
    /// # Example
    /// ```ignore
    /// tracker.spawn(
    ///     "my_key".to_string(),
    ///     || expensive_work(),
    ///     |world, key, result| {
    ///         // Apply result to world
    ///     }
    /// );
    /// ```
    pub fn spawn<T, W, A>(&mut self, key: K, work: W, apply: A)
    where
        T: Send + 'static,
        W: FnOnce() -> T + Send + 'static,
        A: FnOnce(&mut World, K, T) + Send + 'static,
    {
        let generation = self.start_task(key.clone());
        let generations = self.generations.clone();
        let pending_results = self.pending_results.clone();

        rayon::spawn(move || {
            // Execute work on background thread
            let result = work();

            // Create closure that will check generation and apply result
            let apply_closure: ApplyClosure = Box::new(move |world: &mut World| {
                // Check if generation is still current
                let is_current = generations
                    .lock()
                    .unwrap()
                    .get(&key)
                    .map_or(false, |&current| current == generation);

                if is_current {
                    log::debug!("Applying async result for generation {}", generation);
                    apply(world, key, result);
                } else {
                    log::debug!(
                        "Discarding stale async result for generation {}",
                        generation
                    );
                }
            });

            // Queue the result to be applied on the main thread
            pending_results.lock().unwrap().push(apply_closure);
        });
    }
}

// Specialized implementation for Entity to add entity existence checks
impl AsyncTaskTracker<Entity> {
    /// Spawn an async task for an Entity with automatic existence checking.
    ///
    /// This is a specialized version that checks if the entity still exists
    /// before calling the apply closure. The apply closure receives an
    /// `EntityWorldMut` for convenient entity manipulation.
    ///
    /// # Example
    /// ```ignore
    /// tracker.spawn_for_entity(
    ///     entity,
    ///     move || generate_mesh(&planet),
    ///     |mut entity_mut, mesh| {
    ///         entity_mut.insert(mesh);
    ///     },
    /// );
    /// ```
    pub fn spawn_for_entity<T, W, A>(&mut self, entity: Entity, work: W, apply: A)
    where
        T: Send + 'static,
        W: FnOnce() -> T + Send + 'static,
        A: FnOnce(EntityWorldMut, T) + Send + 'static,
    {
        let generation = self.start_task(entity);
        let generations = self.generations.clone();
        let pending_results = self.pending_results.clone();

        rayon::spawn(move || {
            // Execute work on background thread
            let result = work();

            // Create closure that will check entity existence, generation, and apply result
            let apply_closure: ApplyClosure = Box::new(move |world: &mut World| {
                // Check if entity still exists
                if world.get_entity(entity).is_err() {
                    log::debug!(
                        "Discarding async result for generation {} - entity {:?} no longer exists",
                        generation,
                        entity
                    );
                    return; // Entity was deleted, discard result
                }

                // Check if generation is still current
                let is_current = generations
                    .lock()
                    .unwrap()
                    .get(&entity)
                    .map_or(false, |&current| current == generation);

                if is_current {
                    log::debug!(
                        "Applying async result for entity {:?} generation {}",
                        entity,
                        generation
                    );
                    // Safe to apply - entity exists and this is the latest generation
                    if let Ok(entity_mut) = world.get_entity_mut(entity) {
                        apply(entity_mut, result);
                    }
                } else {
                    log::debug!(
                        "Discarding stale async result for entity {:?} generation {}",
                        entity,
                        generation
                    );
                }
            });

            // Queue the result to be applied on the main thread
            pending_results.lock().unwrap().push(apply_closure);
        });
    }
}

impl<K: Hash + Eq + Clone + Send + Sync + 'static> Default for AsyncTaskTracker<K> {
    fn default() -> Self {
        Self::new()
    }
}

/// System that processes pending async results for Entity-keyed tasks.
///
/// This should be added to your Bevy schedule to process results from
/// `AsyncTaskTracker<Entity>`. It runs all pending result closures on
/// the main thread, which apply results to entities if they're still valid.
///
/// Add this system to your app like:
/// ```ignore
/// app.add_systems(Update, apply_async_entity_results);
/// ```
pub fn apply_async_entity_results(world: &mut World) {
    // Get the pending results queue (cloning the Arc)
    let pending_results = world
        .get_resource::<AsyncTaskTracker<Entity>>()
        .map(|tracker| tracker.pending_results.clone());

    if let Some(pending_results) = pending_results {
        // Lock and drain all pending results
        let mut results = pending_results.lock().unwrap();

        let count = results.len();
        if count > 0 {
            log::debug!("Processing {} pending async results", count);
        }

        // Apply each result closure to the world
        for apply in results.drain(..) {
            apply(world);
        }
    }
}
