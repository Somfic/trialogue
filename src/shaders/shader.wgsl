// Basic Raytracer Shader
@group(0) @binding(3)
var output_texture: texture_storage_2d<rgba8unorm, write>;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct Sphere {
    center: vec3<f32>,
    radius: f32
}

fn hit_sphere(sphere: Sphere, ray: Ray) -> f32 {
    let oc = ray.origin - sphere.center;
    let a = dot(ray.direction, ray.direction); // note: do we need this if our ray direction is always normalized?
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - (sphere.radius * sphere.radius);

    // discriminant = b² - 4ac
    let discriminant = (b * b) - 4.0 * a * c;

    // this ray doesn't hit, return early
    if discriminant < 0 {
        return 0.0;
    }

    let distance = (-b - sqrt(discriminant)) / (2.0 * a);

    // this sphere is behind us, return early
    if distance < 0.0 {
        return 0.0;
    }

    return distance;
}

fn build_ray(uv: vec2<f32>, aspect_ratio: f32) -> Ray {
    let point_on_plane = vec3<f32>(
        (uv.x * 2.0 - 1.0) * aspect_ratio,
        uv.y * 2.0 - 1.0,
        -1.0
    );

    let origin = vec3<f32>(0.0, 0.0, 0.0);
    let direction = normalize(point_on_plane - origin);

    return Ray(origin, direction);
}

// get the color for a specific pixel
fn get_pixel_color(size: vec2<u32>, pixel: vec2<i32>) -> vec3<f32> {
    let aspect_ratio = f32(size.x) / f32(size.y);
    let uv = vec2<f32>((f32(pixel.x) + 0.5) / f32(size.x), (f32(pixel.y) + 0.5) / f32(size.y));
    let ray = build_ray(uv, aspect_ratio);

    // testing
    let sphere = Sphere(vec3(0.0, 0.0, -3.0), 1.0);
    let distance = hit_sphere(sphere, ray);

    if distance > 0.0 {
        return vec3<f32>(1.0, 0.0, 0.0);
    }

    return vec3<f32>(0.5, 0.7, 1.0);
}

// raytracer entry point
@compute @workgroup_size(8, 8)
fn raytracer(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let size = textureDimensions(output_texture);
    
    // return early if out of bounds
    if global_id.x >= size.x || global_id.y >= size.y {
        return;
    }

    let pixel = vec2<i32>(global_id.xy);
    let color = get_pixel_color(size, pixel);

    textureStore(output_texture, pixel, vec4<f32>(color, 1.0));
}

// basic vertex and fragment shaders to display the raytraced texture
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let uv = vec2<f32>(
        f32((vertex_index << 1u) & 2u),
        f32(vertex_index & 2u)
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@group(0) @binding(0)
var result_texture: texture_2d<f32>;
@group(0) @binding(1)
var result_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(result_texture, result_sampler, in.uv);
}
 