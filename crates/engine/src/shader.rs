use bevy_ecs::prelude::Resource;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, channel},
};

use crate::prelude::Shader;

/// Validates WGSL shader source using naga
pub fn validate_wgsl(source: &str, shader_name: &str) -> Result<(), String> {
    let module = naga::front::wgsl::parse_str(source)
        .map_err(|e| format!("Shader parsing failed for {}: {:?}", shader_name, e))?;

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );

    validator
        .validate(&module)
        .map_err(|e| format!("Shader validation failed for {}: {:?}", shader_name, e))?;

    Ok(())
}

/// Trait for loading and managing shader modules
pub trait ShaderLoader: Send + Sync {
    /// Get the current shader module
    fn get_shader(&self, device: &wgpu::Device) -> wgpu::ShaderModule;

    /// Get the current shader source code
    fn get_source(&self) -> String;

    /// Check if the shader needs to be reloaded and return the new module and source if so
    fn check_reload(
        &mut self,
        device: &wgpu::Device,
    ) -> Option<Result<(wgpu::ShaderModule, String), String>>;

    /// Get the shader name/identifier
    fn name(&self) -> &str;
}

/// Static shader loader that embeds shader source at compile time
pub struct StaticShaderLoader {
    source: Cow<'static, str>,
    label: String,
}

impl StaticShaderLoader {
    pub fn new(source: &'static str, label: impl Into<String>) -> Self {
        Self {
            source: Cow::Borrowed(source),
            label: label.into(),
        }
    }
}

impl ShaderLoader for StaticShaderLoader {
    fn get_shader(&self, device: &wgpu::Device) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&self.label),
            source: wgpu::ShaderSource::Wgsl(self.source.clone()),
        })
    }

    fn get_source(&self) -> String {
        self.source.to_string()
    }

    fn check_reload(
        &mut self,
        _device: &wgpu::Device,
    ) -> Option<Result<(wgpu::ShaderModule, String), String>> {
        // Static shaders never reload
        None
    }

    fn name(&self) -> &str {
        &self.label
    }
}

/// Hot-reloading shader loader that watches the filesystem for changes
pub struct HotReloadShaderLoader {
    path: PathBuf,
    label: String,
    source: Mutex<String>,
    _watcher: RecommendedWatcher,
    receiver: Mutex<Receiver<notify::Result<Event>>>,
    needs_reload: Mutex<bool>,
}

impl HotReloadShaderLoader {
    pub fn new(
        path: impl AsRef<Path>,
        label: impl Into<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref().to_path_buf();
        let label = label.into();

        // Read initial shader source
        let source = std::fs::read_to_string(&path)?;

        // Validate the initial shader
        if let Err(e) = validate_wgsl(&source, &label) {
            return Err(e.into());
        }

        // Set up file watcher
        let (tx, receiver) = channel();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        log::info!(
            "Hot-reload enabled for shader: {} ({})",
            label,
            path.display()
        );

        Ok(Self {
            path,
            label,
            source: Mutex::new(source),
            _watcher: watcher,
            receiver: Mutex::new(receiver),
            needs_reload: Mutex::new(false),
        })
    }
}

impl ShaderLoader for HotReloadShaderLoader {
    fn get_shader(&self, device: &wgpu::Device) -> wgpu::ShaderModule {
        let source = self.source.lock().unwrap();
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&self.label),
            source: wgpu::ShaderSource::Wgsl(source.as_str().into()),
        })
    }

    fn get_source(&self) -> String {
        self.source.lock().unwrap().clone()
    }

    fn check_reload(
        &mut self,
        device: &wgpu::Device,
    ) -> Option<Result<(wgpu::ShaderModule, String), String>> {
        // Check for file system events
        let receiver = self.receiver.lock().unwrap();
        let mut needs_reload = self.needs_reload.lock().unwrap();

        while let Ok(event) = receiver.try_recv() {
            match event {
                Ok(event) if event.kind.is_modify() => {
                    *needs_reload = true;
                }
                Err(e) => {
                    log::error!("File watcher error for {}: {:?}", self.label, e);
                }
                _ => {}
            }
        }

        if !*needs_reload {
            return None;
        }

        *needs_reload = false;
        drop(needs_reload); // Release the lock before potentially long operations
        drop(receiver); // Release receiver lock

        // Try to read and validate the new shader source
        let new_source = match std::fs::read_to_string(&self.path) {
            Ok(source) => source,
            Err(e) => {
                let error = format!("Failed to read shader file {}: {}", self.path.display(), e);
                log::error!("{}", error);
                return Some(Err(error));
            }
        };

        // Validate the new shader
        if let Err(e) = validate_wgsl(&new_source, &self.label) {
            log::error!("{}", e);
            return Some(Err(e));
        }

        // Try to create the shader module
        let shader_descriptor = wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{} (Hot Reloaded)", self.label)),
            source: wgpu::ShaderSource::Wgsl(new_source.as_str().into()),
        };

        // Use catch_unwind to gracefully handle WGPU validation panics
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            device.create_shader_module(shader_descriptor)
        }));

        match result {
            Ok(shader) => {
                // Success! Update our stored source
                *self.source.lock().unwrap() = new_source.clone();
                log::info!("Successfully reloaded shader: {}", self.label);
                Some(Ok((shader, new_source)))
            }
            Err(_) => {
                let error = format!(
                    "Shader compilation failed for {} (WGPU validation error)",
                    self.label
                );
                log::error!("{}", error);
                Some(Err(error))
            }
        }
    }

    fn name(&self) -> &str {
        &self.label
    }
}

/// Factory function that creates the appropriate shader loader based on build configuration
#[cfg(debug_assertions)]
pub fn create_shader_loader(
    path: impl AsRef<Path>,
    label: impl Into<String>,
) -> Result<Box<dyn ShaderLoader>, Box<dyn std::error::Error>> {
    Ok(Box::new(HotReloadShaderLoader::new(path, label)?))
}

/// Factory function that creates the appropriate shader loader based on build configuration
#[cfg(not(debug_assertions))]
pub fn create_shader_loader(
    _path: impl AsRef<Path>,
    label: impl Into<String>,
) -> Result<Box<dyn ShaderLoader>, Box<dyn std::error::Error>> {
    // In release builds, we need the static source
    // This should be passed in by the caller using include_str!
    panic!("create_shader_loader should not be used in release builds without static source");
}

/// Creates a static shader loader with embedded source (for release builds)
pub fn create_static_shader_loader(
    source: &'static str,
    label: impl Into<String>,
) -> Box<dyn ShaderLoader> {
    Box::new(StaticShaderLoader::new(source, label))
}

/// Describes what kind of data a bind group expects based on shader variable names
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindGroupRequirement {
    Texture,         // Detected from: texture_2d, sampler variables (t_diffuse, s_diffuse, etc.)
    Camera,          // Detected from: camera variable
    Transform,       // Detected from: transform variable
    Unknown(String), // For bind groups we don't recognize yet
}

impl BindGroupRequirement {
    /// Parse shader source to detect what each bind group needs
    pub fn parse_from_shader(source: &str) -> Vec<Option<Self>> {
        let mut bind_groups: Vec<Option<Self>> = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line = line.trim();

            // Look for @group(N) @binding(M) patterns
            if line.starts_with("@group(") {
                if let Some(group_idx) = Self::extract_group_index(line) {
                    // Ensure we have enough slots
                    while bind_groups.len() <= group_idx {
                        bind_groups.push(None);
                    }

                    // Parse the variable declaration on this line or next line
                    let requirement = if line.contains("var") {
                        // Declaration is on the same line
                        Self::detect_requirement(line)
                    } else if i + 1 < lines.len() {
                        // Declaration is on the next line
                        Self::detect_requirement(lines[i + 1])
                    } else {
                        Self::Unknown("unknown".to_string())
                    };

                    // If we already have a requirement for this group, keep it (first declaration wins)
                    if bind_groups[group_idx].is_none() {
                        bind_groups[group_idx] = Some(requirement);
                    }
                }
            }
        }

        bind_groups
    }

    fn extract_group_index(line: &str) -> Option<usize> {
        // Extract N from "@group(N)"
        if let Some(start) = line.find("@group(") {
            let rest = &line[start + 7..];
            if let Some(end) = rest.find(')') {
                return rest[..end].parse().ok();
            }
        }
        None
    }

    fn detect_requirement(line: &str) -> Self {
        let lower = line.to_lowercase();

        // Check for common patterns
        if lower.contains("texture")
            || lower.contains("sampler")
            || lower.contains("t_diffuse")
            || lower.contains("s_diffuse")
        {
            Self::Texture
        } else if lower.contains("camera") {
            Self::Camera
        } else if lower.contains("transform") {
            Self::Transform
        } else {
            // Extract variable name for unknown types
            let var_name = Self::extract_variable_name(line);
            Self::Unknown(var_name)
        }
    }

    fn extract_variable_name(line: &str) -> String {
        // Try to extract variable name from patterns like "var<uniform> camera:" or "var t_diffuse:"
        if let Some(var_pos) = line.find("var") {
            let after_var = &line[var_pos + 3..].trim_start();
            // Skip over <uniform> or other qualifiers
            let after_qualifier = if after_var.starts_with('<') {
                if let Some(end) = after_var.find('>') {
                    &after_var[end + 1..].trim_start()
                } else {
                    after_var
                }
            } else {
                after_var
            };

            // Extract the identifier
            let name: String = after_qualifier
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();

            if !name.is_empty() {
                return name;
            }
        }
        "unknown".to_string()
    }
}

/// Instance of a loaded shader with its pipeline and bind group requirements
pub struct ShaderInstance {
    pub module: wgpu::ShaderModule,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_requirements: Vec<Option<BindGroupRequirement>>,
}

/// Central cache for managing shaders and their hot-reload state
#[derive(Resource)]
pub struct ShaderCache {
    shaders: HashMap<Shader, Arc<ShaderInstance>>,
    loaders: HashMap<Shader, Box<dyn ShaderLoader>>,
    sources: HashMap<Shader, String>,
}

impl ShaderCache {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
            loaders: HashMap::new(),
            sources: HashMap::new(),
        }
    }

    /// Register a shader with the cache
    pub fn register_shader(
        &mut self,
        shader: Shader,
        loader: Box<dyn ShaderLoader>,
        instance: ShaderInstance,
    ) {
        let source = loader.get_source();
        self.shaders.insert(shader.clone(), Arc::new(instance));
        self.sources.insert(shader.clone(), source);
        self.loaders.insert(shader.clone(), loader);
    }

    /// Get a shader instance by name
    pub fn get_shader(&self, name: &Shader) -> Option<Arc<ShaderInstance>> {
        self.shaders.get(name).cloned()
    }

    /// Get the shader source by name
    pub fn get_source(&self, name: &Shader) -> Option<&str> {
        self.sources.get(name).map(|s| s.as_str())
    }

    /// Check all shaders for hot-reload and return updated shaders
    pub fn check_hot_reload(
        &mut self,
        device: &wgpu::Device,
    ) -> Vec<(Shader, Result<(wgpu::ShaderModule, String), String>)> {
        let mut reloaded = Vec::new();

        for (name, loader) in &mut self.loaders {
            if let Some(reload_result) = loader.check_reload(device) {
                // If reload was successful, update stored source
                if let Ok((_, ref new_source)) = reload_result {
                    self.sources.insert(name.clone(), new_source.clone());
                }
                reloaded.push((name.clone(), reload_result));
            }
        }

        reloaded
    }

    /// Update a shader instance after successful reload
    pub fn update_shader(&mut self, shader: &Shader, instance: ShaderInstance) {
        self.shaders.insert(shader.clone(), Arc::new(instance));
    }

    /// Get all shader names
    pub fn shader_names(&self) -> impl Iterator<Item = &Shader> {
        self.shaders.keys()
    }
}

/// Dedicated resource for the raytracer shader (not part of material system)
#[derive(Resource)]
pub struct RaytracerShader {
    pub loader: Box<dyn ShaderLoader>,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub display_pipeline: wgpu::RenderPipeline,
}

impl RaytracerShader {
    pub fn new(
        loader: Box<dyn ShaderLoader>,
        compute_pipeline: wgpu::ComputePipeline,
        display_pipeline: wgpu::RenderPipeline,
    ) -> Self {
        Self {
            loader,
            compute_pipeline,
            display_pipeline,
        }
    }

    /// Check for hot-reload and return the new shader module and source if reloaded
    pub fn check_reload(
        &mut self,
        device: &wgpu::Device,
    ) -> Option<Result<(wgpu::ShaderModule, String), String>> {
        self.loader.check_reload(device)
    }

    /// Get the shader name
    pub fn name(&self) -> &str {
        self.loader.name()
    }
}
