use crate::prelude::*;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Shader {
    Standard,
    Raytracer,
}

impl Display for Shader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shader::Standard => write!(f, "standard"),
            Shader::Raytracer => write!(f, "raytracer"),
        }
    }
}

/// Rendering mode configuration for materials
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderMode {
    pub polygon_mode: wgpu::PolygonMode,
    // Future render mode properties can be added here:
    // pub cull_mode: Option<wgpu::Face>,
    // pub depth_test: bool,
    // etc.
}

impl RenderMode {
    pub fn filled() -> Self {
        Self {
            polygon_mode: wgpu::PolygonMode::Fill,
        }
    }

    pub fn wireframe() -> Self {
        Self {
            polygon_mode: wgpu::PolygonMode::Line,
        }
    }
}

impl Default for RenderMode {
    fn default() -> Self {
        Self::filled()
    }
}

#[derive(Component, Clone, PartialEq)]
pub struct Material {
    pub shader: Shader,
    pub render_mode: RenderMode,
}

impl Material {
    pub fn new(shader: Shader) -> Self {
        Self {
            shader,
            render_mode: RenderMode::default(),
        }
    }

    pub fn standard() -> Self {
        Self::new(Shader::Standard)
    }

    /// Set the render mode for this material
    pub fn with_render_mode(mut self, render_mode: RenderMode) -> Self {
        self.render_mode = render_mode;
        self
    }

    /// Enable wireframe mode
    pub fn wireframe(mut self) -> Self {
        self.render_mode = RenderMode::wireframe();
        self
    }
}
