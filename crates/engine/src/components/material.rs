
use crate::prelude::*;

use bevy_ecs::component::Component;
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

/// Material component that references a shader by name
#[derive(Component, Clone, PartialEq)]
pub struct Material {
    /// Name of the shader to use (e.g., "standard", "pbr", "unlit")
    pub shader: Shader,
    // Future material properties can be added here:
    // pub albedo: Color,
    // pub roughness: f32,
    // pub metallic: f32,
    // etc.
}

impl Material {
    pub fn new(shader: Shader) -> Self {
        Self { shader }
    }

    /// Create a material using the standard shader
    pub fn standard() -> Self {
        Self::new(Shader::Standard)
    }
}
