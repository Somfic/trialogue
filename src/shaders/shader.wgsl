// Basic Raytracer Shader
@group(0) @binding(3)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<storage, read> spheres: array<Sphere>;

@group(0) @binding(0)
var<uniform> camera: Camera;

fn pcg_hash(seed: u32) -> u32 {
    var state = seed * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn random_normal(seed: ptr<function, u32>) -> f32 {
    let u1 = random_float(seed);
    let u2 = random_float(seed);
    
    // Box-Muller transform
    return sqrt(-2.0 * log(u1)) * cos(6.28318530718 * u2);
}

fn random_float(seed: ptr<function, u32>) -> f32 {
    *seed = pcg_hash(*seed);
    return f32(*seed) / f32(0xffffffffu);
}

fn random_unit_vector(seed: ptr<function, u32>) -> vec3<f32> {
    return vec3<f32>(
        random_normal(seed),
        random_normal(seed),
        random_normal(seed)
    );
}

fn random_in_hemisphere(normal: vec3<f32>, seed: ptr<function, u32>) -> vec3<f32> {
    let v = random_unit_vector(seed);
    if dot(v, normal) > 0.0 {
        return v;
    } else {
        return -v;
    }
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct Sphere {
    center: vec3<f32>,
    radius: f32,
    color: vec3<f32>,
    material_type: u32,
}

struct Camera {
    position: vec3<f32>,
    look_at: vec3<f32>,
    up: vec3<f32>,
    fov: f32,
    aspect_ratio: f32,
}

fn hit_sphere(sphere: Sphere, ray: Ray) -> f32 {
    let oc = ray.origin - sphere.center;
    let a = dot(ray.direction, ray.direction); // note: do we need this if our ray direction is always normalized?
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - (sphere.radius * sphere.radius);

    // discriminant = b² - 4ac
    let discriminant = (b * b) - 4.0 * a * c;

    // this ray doesn't hit, return early
    if discriminant < 0.0 {
        return 0.0;
    }

    let distance = (-b - sqrt(discriminant)) / (2.0 * a);

    // this sphere is behind us, return early
    if distance < 0.0 {
        return 0.0;
    }

    return distance;
}

fn build_ray(u: f32, v: f32) -> Ray {
    let w = normalize(camera.position - camera.look_at);
    let u_vec = normalize(cross(camera.up, w));
    let v_vec = cross(w, u_vec);

    let theta = camera.fov * 3.14159265359 / 180.0;
    let half_height = tan(theta / 2.0);
    let half_width = camera.aspect_ratio * half_height;

    let lower_left_corner = camera.position - half_width * u_vec - half_height * v_vec - w;
    let horizontal = 2.0 * half_width * u_vec;
    let vertical = 2.0 * half_height * v_vec;

    let direction = normalize(lower_left_corner + u * horizontal + v * vertical - camera.position);

    return Ray(camera.position, direction);
}

// get the color for a specific pixel
fn get_pixel_color(size: vec2<u32>, pixel: vec2<i32>) -> vec3<f32> {
    let aspect_ratio = f32(size.x) / f32(size.y);
    let bounces = 5;
    let samples = 4;

    var seed = u32(pixel.x) + u32(pixel.y) * size.x;
    var accumulated_color = vec3(0.0, 0.0, 0.0);

    for (var sample = 0; sample < samples; sample++) {
        let u = (f32(pixel.x) + random_float(&seed)) / f32(size.x);
        let v = (f32(i32(size.y) - pixel.y) + random_float(&seed)) / f32(size.y);
        var ray = build_ray(u, v);
        var color = vec3(1.0, 1.0, 1.0);

        for (var bounce = 0; bounce < bounces; bounce++) {
            var closest_distance = -1.0;
            var hit_sphere_index = -1;

            for (var i = 0u; i < arrayLength(&spheres); i++) {
                let distance = hit_sphere(spheres[i], ray);
                if distance > 0.0 && (closest_distance < 0.0 || distance < closest_distance) {
                    closest_distance = distance;
                    hit_sphere_index = i32(i);
                }
            }
            // if nothing was hit, return sky color
            if closest_distance < 0.0 {
                color *= vec3<f32>(0.5, 0.7, 1.0);
                break;
            }

            let hit_point = ray.origin + closest_distance * ray.direction;
            let sphere = spheres[hit_sphere_index];
            let normal = normalize(hit_point - sphere.center);

            color *= sphere.color;

            let bounce_direction = normalize(normal + random_unit_vector(&seed));
            ray = Ray(hit_point + normal * 0.001, bounce_direction);
        }

        accumulated_color += color;
    }

    let linear_color = accumulated_color / f32(samples);
    let gamma_corrected_color = pow(linear_color, vec3(1.0 / 2.2));

    return gamma_corrected_color;
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
 