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
    out.normal = normalize((transform.model * vec4<f32>(normal, 0.0)).xyz);
    out.clip_position = camera.view_proj * transform.model * vec4<f32>(position, 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Hardcoded directional light (like sunlight)
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3)); // From top-right
    let light_color = vec3<f32>(1.0, 1.0, 0.9); // Slightly warm white

    // Ambient light (so dark side isn't pure black)
    let ambient = vec3<f32>(0.3, 0.3, 0.4); // Slight blue tint

    // Base color for the planet
    let base_color = vec3<f32>(0.7, 0.6, 0.5); // Sandy/rocky color

    // Diffuse lighting calculation
    let normal = normalize(in.normal);
    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse = light_color * diffuse_strength;
    
    let up = vec3<f32>(0.0, 1.0, 0.0);  // Up direction
    let ao = max(dot(normal, up), 0.0);  // 1.0 = facing up, 0.0 = facing down
    let ao_strength = 0.3 + ao * 0.7;    // Remap to 0.3-1.0 range

    // Combine ambient + diffuse, modulated by AO
    let lighting = (ambient + diffuse) * ao_strength;
    let final_color = base_color * lighting;

    return vec4<f32>(final_color, 1.0);
}
