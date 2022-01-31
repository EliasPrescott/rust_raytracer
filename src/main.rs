mod types;
use std::sync::Arc;

use rand::{thread_rng, Rng, prelude::ThreadRng};
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

fn ray_color(r: Ray, world: &dyn Hittable, depth: i64, rng: &mut ThreadRng) -> Color {
    let mut rec = HitRecord::blank();

    if depth <= 0 {
        return Color::zero();
    }

    if world.hit(r, 0.0001, INFINITY, &mut rec) {

        let mut scattered = Ray { origin: Vec3::zero(), direction: Vec3::zero() };
        let mut attenuation = Color::zero();
        if let Some(ref mat) = rec.mat_ptr {
            if mat.scatter(r, &rec, &mut attenuation, &mut scattered, rng) {
                return attenuation * ray_color(scattered, world, depth - 1, rng)
            }
        }
        Color::zero()
    } else {
        let unit_direction = r.direction.unit_vector();
        let t = 0.5 * (unit_direction.y + 1.0);
        Color::one() * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
    }
}

fn write_color(color: Color, samples_per_pixel: i64) {
    let scale = 1.0 / samples_per_pixel as f64;
    
    let r = (color.x * scale).sqrt();
    let g = (color.y * scale).sqrt();
    let b = (color.z * scale).sqrt();

    print!(
        "{} {} {}\n",
        (256.0 * r.clamp(0.0, 0.999)) as i64,
        (256.0 * g.clamp(0.0, 0.999)) as i64,
        (256.0 * b.clamp(0.0, 0.999)) as i64
    );
}

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: u16 = 400;
const IMAGE_HEIGHT: u16 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u16;
const SAMPLES_PER_PIXEL: i64 = 100;
const MAX_DEPTH: i64 = 50;

fn render_test_image() {
    // World
    let mut world = HittableList::new();

    let material_ground = Arc::new(LambertianMaterial::new(Color::new(0.8, 0.8, 0.0)));
    let material_center = Arc::new(LambertianMaterial::new(Color::new(0.7, 0.3, 0.3)));
    let material_left = Arc::new(MetalMaterial::new(Color::new(0.8, 0.8, 0.8), 0.3));
    let material_right = Arc::new(MetalMaterial::new(Color::new(0.8, 0.6, 0.2), 1.0));

    world.add(Arc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, material_ground)));
    world.add(Arc::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, material_center)));
    world.add(Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, material_left)));
    world.add(Arc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, material_right)));
    
    // Camera
    let camera = Camera::default_camera();

    print!("P3\n{IMAGE_WIDTH} {IMAGE_HEIGHT}\n255\n");

    let mut rng = thread_rng();

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaining: {j}      ");
        for i in 0..IMAGE_WIDTH {
            let mut pixel_color = Color::zero();
            for _s in 0..SAMPLES_PER_PIXEL {
                let u = (i as f64 + rng.gen_range(0.0..=1.0)) / (IMAGE_WIDTH - 1) as f64;
                let v = (j as f64 + rng.gen_range(0.0..=1.0)) / (IMAGE_HEIGHT - 1) as f64;
                let r = camera.get_ray(u, v);
                pixel_color += ray_color(r, &world, MAX_DEPTH, &mut rng);
            }
            write_color(pixel_color, SAMPLES_PER_PIXEL);
        }
    }
    eprintln!("\rOperation complete.      ")
}

fn main() {
    render_test_image();
}
