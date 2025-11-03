// Shadow pass shader - depth-only rendering from light's perspective

struct TransformUniform {
    model: mat4x4<f32>,
}

struct ShadowUniform {
    light_space_matrix: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> transform: TransformUniform;
@group(1) @binding(0) var<uniform> shadow: ShadowUniform;

@vertex
fn vertex(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    let world_position = transform.model * vec4<f32>(position, 1.0);
    return shadow.light_space_matrix * world_position;
}
