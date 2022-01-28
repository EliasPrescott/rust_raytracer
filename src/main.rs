mod types;
use std::sync::Arc;

use types::*;

const INFINITY: f64 = f64::INFINITY;
const PI: f64 = 3.1415926535897932385;

fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

fn hit_sphere(center: Point3, radius: f64, ray: Ray) -> f64 {
    let oc = ray.origin - center;
    let a = ray.direction.length_squared();
    let half_b = oc.dot(ray.direction);
    let c = oc.length_squared() - radius * radius;
    let discriminant = half_b * half_b - a * c;

    if discriminant < 0.0 {
        -1.0
    } else {
        (-half_b - discriminant.sqrt()) / a
    }
}

fn ray_color(r: Ray, world: &dyn Hittable) -> Color {
    let mut rec = HitRecord::blank();
    if world.hit(r, 0.0, INFINITY, &mut rec) {
        (rec.normal + Color::one()) * 0.5
    } else {
        let unit_direction = r.direction.unit_vector();
        let t = 0.5 * (unit_direction.y + 1.0);
        Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
    }
}

fn write_color(color: Color) {
    print!(
        "{} {} {}\n", 
        (255.999 * color.x) as i64,
        (255.999 * color.y) as i64,
        (255.999 * color.z) as i64
    );
}

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: u16 = 400;
const IMAGE_HEIGHT: u16 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u16;

fn render_test_image() {

    // Image
    let viewport_height = 2.0;
    let viewport_width = ASPECT_RATIO * viewport_height;
    let focal_length = 1.0;

    // World
    let mut world = HittableList::new();
    world.add(
        Arc::new(
            Sphere::new(
                Point3::new(0.0, 0.0, -1.0),
                0.5
            )
        )
    );
    world.add(
        Arc::new(
            Sphere::new(
                Point3::new(0.0, -100.5, -1.0),
                100.0
            )
        )
    );

    // Camera
    let origin = Point3 { x:0.0, y:0.0, z:0.0 };
    let horizontal = Vec3 { x:viewport_width, y:0.0, z:0.0 };
    let vertical = Vec3 { x:0.0, y:viewport_height, z:0.0 };
    let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - Vec3 { x:0.0, y: 0.0, z:focal_length };

    print!("P3\n{IMAGE_WIDTH} {IMAGE_HEIGHT}\n255\n");
    
    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaining: {j}      ");
        for i in 0..IMAGE_WIDTH {
            let u = i as f64 / (IMAGE_WIDTH-1) as f64;
            let v = j as f64 / (IMAGE_HEIGHT-1) as f64;
            let ray = Ray {
                origin: origin,
                direction: lower_left_corner + horizontal * u + vertical * v - origin,
            };
            let color = ray_color(ray, &world);
            write_color(color);
        }
    }
    eprint!("\rOperation complete.      ")
}

fn main() {
    render_test_image();
}