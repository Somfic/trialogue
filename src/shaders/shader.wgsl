// Basic Raytracer Shader

struct Camera {
    position: vec3<f32>,
    look_at: vec3<f32>,
    up: vec3<f32>,
    fov: f32,
    aspect_ratio: f32,
}

struct Sphere {
    center: vec3<f32>,
    radius: f32,
    color: vec3<f32>,
    material_type: u32, // 0 = lambertian, 1 = metal, 2 = dielectric
}

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct HitRecord {
    point: vec3<f32>,
    normal: vec3<f32>,
    distance: f32,
    front_face: bool,
    material_type: u32,
    color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(0) @binding(1)
var<storage, read> spheres: array<Sphere>;

@group(0) @binding(2)
var<storage, read> lights: array<Light>;

@group(0) @binding(3)
var output_texture: texture_storage_2d<rgba8unorm, write>;

// Generate a ray from camera through pixel (u, v)
fn get_camera_ray(u: f32, v: f32) -> Ray {
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

// Check if ray hits sphere
fn hit_sphere(sphere: Sphere, ray: Ray, min_distance: f32, max_distance: f32) -> HitRecord {
    let oc = ray.origin - sphere.center;
    let a = dot(ray.direction, ray.direction);
    let half_b = dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = half_b * half_b - a * c;

    var hit: HitRecord;
    hit.distance = -1.0; // No hit by default

    if discriminant >= 0.0 {
        let sqrt_disc = sqrt(discriminant);
        var distance = (-half_b - sqrt_disc) / a;

        if distance < min_distance || distance > max_distance {
            distance = (-half_b + sqrt_disc) / a;
        }

        if distance >= min_distance && distance <= max_distance {
            hit.distance = distance;
            hit.point = ray.origin + distance * ray.direction;
            let outward_normal = (hit.point - sphere.center) / sphere.radius;
            hit.front_face = dot(ray.direction, outward_normal) < 0.0;
            hit.normal = select(-outward_normal, outward_normal, hit.front_face);
            hit.color = sphere.color;
            hit.material_type = sphere.material_type;
        }
    }

    return hit;
}

// Check ray collision with all spheres
fn hit_world(ray: Ray, min_distance: f32, max_distance: f32) -> HitRecord {
    var closest_hit: HitRecord;
    closest_hit.distance = -1.0;
    var closest_t = max_distance;

    for (var i = 0u; i < arrayLength(&spheres); i++) {
        let hit = hit_sphere(spheres[i], ray, min_distance, closest_t);
        if hit.distance > 0.0 && hit.distance < closest_t {
            closest_t = hit.distance;
            closest_hit = hit;
        }
    }

    return closest_hit;
}

// Simple Lambertian shading
fn lambertian_shading(hit: HitRecord) -> vec3<f32> {
    return vec3(hit.color);
}

// Ray color calculation
fn ray_color(ray: Ray, max_depth: i32) -> vec3<f32> {
    var current_ray = ray;
    var color = vec3<f32>(1.0);

    for (var depth = 0; depth < max_depth; depth++) {
        let hit = hit_world(current_ray, 0.001, 1000.0);

        if hit.distance > 0.0 {
            // Calculate lighting based on material type
            if hit.material_type == 0u { // Lambertian
                let shading = lambertian_shading(hit);
                color *= shading;
                
                // Generate scattered ray for next bounce (simplified)
                let scatter_target = hit.point + hit.normal + normalize(vec3<f32>(
                    sin(hit.point.x * 12.9898 + hit.point.y * 78.233) * 43758.5453,
                    sin(hit.point.y * 12.9898 + hit.point.z * 78.233) * 43758.5453,
                    sin(hit.point.z * 12.9898 + hit.point.x * 78.233) * 43758.5453
                ));
                current_ray = Ray(hit.point, normalize(scatter_target - hit.point));
                color *= 0.5; // Energy loss per bounce
            } else {
                // Simple reflection for metals/other materials
                color *= hit.color;
                break;
            }
        } else {
            // Sky color gradient
            let unit_direction = normalize(current_ray.direction);
            let t = 0.5 * (unit_direction.y + 1.0);
            let sky_color = (1.0 - t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);
            color *= sky_color;
            break;
        }
    }

    return color;
}

@compute @workgroup_size(8, 8)
fn raytracer(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(output_texture);
    let coords = vec2<i32>(global_id.xy);

    if global_id.x >= dims.x || global_id.y >= dims.y {
        return;
    }
    
    // Convert pixel coordinates to UV coordinates
    let u = (f32(global_id.x) + 0.5) / f32(dims.x);
    let v = (f32(dims.y - global_id.y) + 0.5) / f32(dims.y); // Flip Y
    
    // Generate ray and trace
    let ray = get_camera_ray(u, v);
    let color = ray_color(ray, 1); // Max 5 bounces
    
    // Gamma correction and tone mapping
    let gamma_corrected = pow(color, vec3<f32>(1.0 / 2.2));
    let final_color = clamp(gamma_corrected, vec3<f32>(0.0), vec3<f32>(1.0));

    textureStore(output_texture, coords, vec4<f32>(final_color, 1.0));
}

// Vertex shader for displaying the raytraced result
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
 