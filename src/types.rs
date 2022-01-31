use std::{ops, sync::Arc, thread::Thread};
use rand::{thread_rng, Rng, prelude::ThreadRng};

#[derive(Clone)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat_ptr: Option<Arc<dyn Material>>,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, r: Ray, outward_normal: Vec3) {
        self.front_face = r.direction.dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }

    pub fn blank() -> HitRecord {
        HitRecord {
            p: Point3::zero(),
            normal: Vec3::zero(),
            mat_ptr: None,
            t: 0.0,
            front_face: false,
        }
    }
}

pub trait Hittable {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        false
    }
}

pub trait Material {
    fn scatter(&self, r_in: Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray, rng: &mut ThreadRng) -> bool {
        false
    }
}

pub struct LambertianMaterial {
    albedo: Color
}

impl LambertianMaterial {
    pub fn new(albedo: Color) -> Self {
        LambertianMaterial {
            albedo
        }
    }
}

impl Material for LambertianMaterial {
    fn scatter(&self, r_in: Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray, rng: &mut ThreadRng) -> bool {
        let scatter_direction = rec.normal + Vec3::random_unit_vector(rng);

        let scatter_direction =
            if scatter_direction.near_zero() {
                rec.normal
            } else {
                rec.normal + Vec3::random_unit_vector(rng)
            };

        *scattered = Ray { origin: rec.p, direction: scatter_direction };
        *attenuation = self.albedo;
        true
    }
}

pub struct MetalMaterial {
    albedo: Color,
    fuzz: f64
}

impl MetalMaterial {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        MetalMaterial {
            albedo,
            fuzz
        }
    }
}

impl Material for MetalMaterial {
    fn scatter(&self, r_in: Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray, rng: &mut ThreadRng) -> bool {
        let reflected = Vec3::reflect(r_in.direction.unit_vector(), rec.normal);
        *scattered = Ray { origin: rec.p, direction: reflected + Vec3::random_in_unit_sphere(rng) * self.fuzz };
        *attenuation = self.albedo;
        scattered.direction.dot(rec.normal) > 0.0
    }
}

pub struct DielectricMaterial {
    ir: f64 // Index of refraction
}

impl DielectricMaterial {
    pub fn new(ir: f64) -> Self {
        DielectricMaterial {
           ir
        }
    }
}

impl Material for DielectricMaterial {
    fn scatter(&self, r_in: Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray, rng: &mut ThreadRng) -> bool {
        *attenuation = Color::one();
        let refraction_ratio = 
            if rec.front_face {
                1.0 / self.ir
            } else {
                self.ir
            };
        let unit_direction = r_in.direction.unit_vector();
        let refracted = Vec3::refract(unit_direction, rec.normal, refraction_ratio);

        *scattered = Ray { origin: rec.p, direction: refracted };
        true
    }
}

pub struct Sphere {
    center: Point3,
    radius: f64,
    mat_ptr: Arc<dyn Material>
}

impl Hittable for Sphere {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let oc = r.origin - self.center;
        let a = r.direction.length_squared();
        let half_b = oc.dot(r.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            false
        } else {
            let sqrtd = discriminant.sqrt();
            let mut root = (-half_b - sqrtd) / a;
            if root < t_min || t_max < root {
                root = (-half_b + sqrtd) / a;
                if root < t_min || t_max < root {
                    false
                } else {
                    rec.t = root;
                    rec.p = r.at(rec.t);
                    rec.normal = (rec.p - self.center) / self.radius;
                    let outward_normal = (rec.p - self.center) / self.radius;
                    rec.set_face_normal(r, outward_normal);
                    rec.mat_ptr = Some(self.mat_ptr.to_owned());
                    true
                }
            } else {
                rec.t = root;
                rec.p = r.at(rec.t);
                rec.normal = (rec.p - self.center) / self.radius;
                let outward_normal = (rec.p - self.center) / self.radius;
                rec.set_face_normal(r, outward_normal);
                rec.mat_ptr = Some(self.mat_ptr.to_owned());
                true
            }
        }
    }
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat_ptr: Arc<dyn Material>) -> Sphere {
        Sphere {
            center,
            radius,
            mat_ptr 
        }
    }
}

pub struct HittableList {
    pub objects: Vec<Arc<dyn Hittable>>,
}

impl Hittable for HittableList {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        for boxed_obj in &self.objects {
            let mut temp_rec: HitRecord = HitRecord {
                p: Point3::zero(),
                front_face: false,
                mat_ptr: None,
                normal: Vec3::zero(),
                t: 0.0,
            };
            if boxed_obj
                .as_ref()
                .hit(r, t_min, closest_so_far, &mut temp_rec)
            {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                *rec = temp_rec;
            }
        }
        hit_anything
    }
}

impl HittableList {
    pub fn new() -> HittableList {
        HittableList {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, obj: Arc<dyn Hittable>) {
        self.objects.push(obj)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::Sub<i32> for Vec3 {
    type Output = Vec3;

    fn sub(self, a: i32) -> Self::Output {
        Vec3 {
            x: self.x - a as f64,
            y: self.y - a as f64,
            z: self.z - a as f64,
        }
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Sub<f64> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Div<f64> for &Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl ops::Div<i64> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: i64) -> Self::Output {
        Vec3 {
            x: self.x / rhs as f64,
            y: self.y / rhs as f64,
            z: self.z / rhs as f64,
        }
    }
}

impl ops::Div<i64> for &Vec3 {
    type Output = Vec3;

    fn div(self, rhs: i64) -> Self::Output {
        Vec3 {
            x: self.x / rhs as f64,
            y: self.y / rhs as f64,
            z: self.z / rhs as f64,
        }
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl ops::DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        *self *= 1 as f64 / rhs
    }
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x: x, y: y, z: z }
    }

    pub fn zero() -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }

    pub fn one() -> Vec3 {
        Vec3::new(1.0, 1.0, 1.0)
    }

    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn dot(&self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn near_zero(&self) -> bool {
        let s = 1e-8;
        f64::abs(self.x) < s && f64::abs(self.y) < s && f64::abs(self.z) < s
    }

    pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
        v - n * (2 as f64) * v.dot(n)
    }

    pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: f64) -> Vec3 {
        let cos_theta = -uv.dot(n).min(1.0);
        let r_out_perp = (uv + n * cos_theta) * etai_over_etat;
        let r_out_parallel = n * (1.0 - r_out_perp.length_squared()).abs().sqrt();
        r_out_perp + r_out_parallel
    }

    pub fn unit_vector(&self) -> Vec3 {
        self / self.length()
    }

    pub fn random(min: f64, max: f64, rng: &mut ThreadRng) -> Vec3 {
        Self::new(
            rng.gen_range(min..=max),
            rng.gen_range(min..=max),
            rng.gen_range(min..=max)
        )
    }

    pub fn random_in_unit_sphere(rng: &mut ThreadRng) -> Vec3 {
        loop {
            let p = Vec3::random(-1.0, 1.0, rng);
            if p.length_squared() >= 1.0 {
                continue;
            } else {
                return p;
            }
        }
    }

    pub fn random_unit_vector(rng: &mut ThreadRng) -> Vec3 {
        Self::random_in_unit_sphere(rng).unit_vector()
    }

    // An alternate formula for diffuse
    pub fn random_in_hemisphere(normal: Vec3, rng: &mut ThreadRng) -> Vec3 {
        let in_unit_sphere = Self::random_in_unit_sphere(rng);
        if in_unit_sphere.dot(normal) > 0.0 {
            in_unit_sphere
        } else {
            -in_unit_sphere
        }
    }
}

// Using type-aliasing to create these 'child' types that can access Vec3 methods
pub type Point3 = Vec3;
pub type Color = Vec3;

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub fn at(&self, t: f64) -> Point3 {
        self.origin + (self.direction * t)
    }
}

pub struct Camera {
    pub origin: Point3,
    pub lower_left_corner: Point3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
}

impl Camera {
    pub fn default_camera() -> Self {
        let aspect_ratio = 16.0 / 9.0;
        let viewport_height = 2.0;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = 1.0;

        let origin = Point3::zero();
        let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        let vertical = Vec3::new(0.0, viewport_height, 0.0);
        let lower_left_corner = origin - horizontal / 2 - vertical / 2 - Vec3::new(0.0, 0.0, focal_length);

        Camera {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray {
            origin: self.origin,
            direction: self.lower_left_corner + self.horizontal * u + self.vertical * v
                - self.origin,
        }
    }
}
