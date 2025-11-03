use crate::prelude::*;

/// Represents a ray in 3D space
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>, // Should be normalized
}

impl Ray {
    /// Create a new ray with a normalized direction
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get a point along the ray at distance t
    pub fn point_at(&self, t: f32) -> Point3<f32> {
        self.origin + self.direction * t
    }
}

/// Result of a ray-sphere intersection test
pub struct SphereIntersection {
    /// Distance along the ray to the hit point
    pub distance: f32,
    /// The hit point in world space
    pub point: Point3<f32>,
    /// Surface normal at the hit point (points outward from sphere)
    pub normal: Vector3<f32>,
}

/// Test if a ray intersects a sphere
/// Returns the nearest intersection point (if any)
pub fn ray_sphere_intersection(
    ray: &Ray,
    sphere_center: Point3<f32>,
    sphere_radius: f32,
) -> Option<SphereIntersection> {
    // Vector from ray origin to sphere center
    let oc = ray.origin - sphere_center;

    // Quadratic equation coefficients: at^2 + bt + c = 0
    let a = ray.direction.dot(&ray.direction);
    let b = 2.0 * oc.dot(&ray.direction);
    let c = oc.dot(&oc) - sphere_radius * sphere_radius;

    let discriminant = b * b - 4.0 * a * c;

    // No intersection if discriminant is negative
    if discriminant < 0.0 {
        return None;
    }

    // Calculate both solutions
    let sqrt_discriminant = discriminant.sqrt();
    let t1 = (-b - sqrt_discriminant) / (2.0 * a);
    let t2 = (-b + sqrt_discriminant) / (2.0 * a);

    // We want the nearest positive t (closest intersection in front of the ray)
    let t = if t1 > 0.0 {
        t1 // Nearest intersection
    } else if t2 > 0.0 {
        t2 // Ray origin is inside sphere, use far intersection
    } else {
        return None; // Both intersections are behind the ray
    };

    let point = ray.point_at(t);
    let normal = (point - sphere_center).normalize();

    Some(SphereIntersection {
        distance: t,
        point,
        normal,
    })
}

/// Generate a ray from a camera through the viewport center
/// This assumes we want to cast through the center of the screen
pub fn camera_center_ray(camera: &Camera, transform: &Transform) -> Ray {
    // Camera looks from its position toward the target
    let direction = (camera.target - transform.position).normalize();

    Ray::new(transform.position, direction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_sphere_hit() {
        let ray = Ray::new(
            Point3::new(0.0, 0.0, -5.0),
            Vector3::new(0.0, 0.0, 1.0),
        );
        let sphere_center = Point3::origin();
        let sphere_radius = 1.0;

        let hit = ray_sphere_intersection(&ray, sphere_center, sphere_radius);
        assert!(hit.is_some());

        let hit = hit.unwrap();
        assert!((hit.distance - 4.0).abs() < 0.001); // Should hit at t=4 (5 - 1)
        assert!((hit.point.z - (-1.0)).abs() < 0.001); // Hit at z = -1
    }

    #[test]
    fn test_ray_sphere_miss() {
        let ray = Ray::new(
            Point3::new(0.0, 5.0, -5.0),
            Vector3::new(0.0, 0.0, 1.0),
        );
        let sphere_center = Point3::origin();
        let sphere_radius = 1.0;

        let hit = ray_sphere_intersection(&ray, sphere_center, sphere_radius);
        assert!(hit.is_none());
    }
}
