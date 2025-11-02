struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct TransformUniform {
    model: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>, 
}


@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(1) @binding(0) var<uniform> camera: CameraUniform;
@group(2) @binding(0) var<uniform> transform: TransformUniform;
 
@vertex
fn vertex(
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = uv;
    out.normal = normal;
    out.clip_position = camera.view_proj * transform.model * vec4<f32>(position, 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert normal from [-1, 1] range to [0, 1] color range
    let color = in.normal * 0.5 + 0.5;
    return vec4<f32>(color, 1.0);
}

