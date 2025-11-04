// Instanced rendering shader - uses per-instance transforms instead of uniform
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) light_space_pos: vec4<f32>,
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(1) @binding(0) var<uniform> camera: CameraUniform;
@group(2) @binding(0) var t_shadow: texture_depth_2d;
@group(2) @binding(1) var sampler_shadow: sampler_comparison;
@group(2) @binding(2) var<uniform> light_space_matrix: mat4x4<f32>;
@group(2) @binding(3) var<uniform> light_direction: vec4<f32>;
@group(2) @binding(4) var<uniform> light_properties: vec4<f32>; // x=intensity, yzw=color
 
@vertex
fn vertex(
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    // Instance data (per-instance transform matrix)
    @location(3) model_matrix_0: vec4<f32>,
    @location(4) model_matrix_1: vec4<f32>,
    @location(5) model_matrix_2: vec4<f32>,
    @location(6) model_matrix_3: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Reconstruct model matrix from instance data
    let model_matrix = mat4x4<f32>(
        model_matrix_0,
        model_matrix_1,
        model_matrix_2,
        model_matrix_3,
    );
    
    let world_position = model_matrix * vec4<f32>(position, 1.0);

    out.uv = uv;
    out.normal = normalize((model_matrix * vec4<f32>(normal, 0.0)).xyz);
    out.world_pos = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.light_space_pos = light_space_matrix * world_position;
    return out;
}

fn calculate_shadow(light_space_pos: vec4<f32>, normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    // Perspective divide
    let proj_coords = light_space_pos.xyz / light_space_pos.w;

    // Transform to [0,1] range (from NDC [-1,1])
    let uv = proj_coords.xy * 0.5 + 0.5;
    let depth = proj_coords.z;

    // Outside shadow map = fully lit
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 || depth < 0.0 || depth > 1.0) {
        return 1.0;
    }

    // Facing away from light = in shadow
    let n_dot_l = dot(normal, light_dir);
    if (n_dot_l <= 0.0) {
        return 0.0;
    }

    // Smaller adaptive bias for higher resolution shadow map
    let bias = max(0.002 * (1.0 - n_dot_l), 0.0005);

    // PCF with Poisson disk samples for smoother, less grid-like shadows
    let texel_size = 1.0 / 4096.0;
    let filter_radius = 2.0 * texel_size;

    // 16-sample Poisson disk
    let poisson = array<vec2<f32>, 16>(
        vec2<f32>(-0.94201624, -0.39906216),
        vec2<f32>(0.94558609, -0.76890725),
        vec2<f32>(-0.094184101, -0.92938870),
        vec2<f32>(0.34495938, 0.29387760),
        vec2<f32>(-0.91588581, 0.45771432),
        vec2<f32>(-0.81544232, -0.87912464),
        vec2<f32>(-0.38277543, 0.27676845),
        vec2<f32>(0.97484398, 0.75648379),
        vec2<f32>(0.44323325, -0.97511554),
        vec2<f32>(0.53742981, -0.47373420),
        vec2<f32>(-0.26496911, -0.41893023),
        vec2<f32>(0.79197514, 0.19090188),
        vec2<f32>(-0.24188840, 0.99706507),
        vec2<f32>(-0.81409955, 0.91437590),
        vec2<f32>(0.19984126, 0.78641367),
        vec2<f32>(0.14383161, -0.14100790)
    );

    var shadow = 0.0;
    for (var i = 0; i < 16; i++) {
        let offset = poisson[i] * filter_radius;
        shadow += textureSampleCompare(t_shadow, sampler_shadow, uv + offset, depth - bias);
    }

    return shadow / 16.0;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get light properties
    let light_dir = normalize(light_direction.xyz);
    let light_intensity = light_properties.x;
    let light_color = light_properties.yzw * light_intensity;

    // Material
    let albedo = vec3<f32>(0.7, 0.6, 0.5);

    let normal = normalize(in.normal);
    let view_dir = normalize(-in.world_pos);

    // Basic diffuse
    let n_dot_l = max(dot(normal, light_dir), 0.0);

    // Shadow
    let shadow = calculate_shadow(in.light_space_pos, normal, light_dir);

    // Ambient light so we can always see something
    let ambient = albedo * 0.1;

    // Simple Lambertian shading
    let diffuse = albedo * light_color * n_dot_l * shadow;

    // Tiny bit of specular for highlights
    let half_dir = normalize(light_dir + view_dir);
    let n_dot_h = max(dot(normal, half_dir), 0.0);
    let spec = pow(n_dot_h, 32.0) * 0.2;
    let specular = light_color * spec * shadow;

    let final_color = ambient + diffuse + specular;

    return vec4<f32>(final_color, 1.0);
}
