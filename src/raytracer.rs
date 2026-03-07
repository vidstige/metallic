use std::f32::consts::TAU;

use na::{Point3, Scalar, Vector3};

use crate::color::Color;
use crate::color::mix_colors;
use crate::gradient::Gradient;
use crate::meatballs::{field_value, normal_at, Metaball};
use crate::sphere::{spherical, Sphere};

pub struct Ray<T: Scalar> {
    pub origin: Point3<T>,
    pub direction: Vector3<T>,
}

impl Ray<f32> {
    pub fn at(&self, t: f32) -> Point3<f32> {
        self.origin + self.direction * t
    }
}

pub trait Traceable<T: Scalar> {
    fn trace(&self, ray: &Ray<T>) -> Option<Ray<T>>;
}

impl Traceable<f32> for Vec<Metaball> {
    fn trace(&self, ray: &Ray<f32>) -> Option<Ray<f32>> {
        // find all intersections with sphere of influence
        // also keep track of the ray enters (true) or leavs the sphere
        let mut intersections: Vec<_> = Vec::new();
        for metaball in self {
            if let Some((t0, t1)) = sphere_ray_intersections(&ray, &metaball.sphere) {
                intersections.push((t0, metaball, true));
                intersections.push((t1, metaball, false));
            }
        }

        // sort intersections by ray parameter t
        intersections.sort_unstable_by(|(a, _, _), (b, _, _)| a.partial_cmp(b).unwrap());

        // keep track of "active" spheres
        let mut active = Vec::new();
        for slice in intersections.windows(2) {
            let (t0, metaball, enter) = slice[0];
            let (t1, _, _) = slice[1];
            if enter {
                active.push(metaball);
            } else {
                active.retain_mut(|mb| mb != &metaball);
            }
            // trace between t0 and t1
            let n = 5;
            let level = 0.3;
            for i in 0..n {
                let ti = crate::lerp::lerp(t0, t1, i as f32 / n as f32);
                let qi = field_value(&active, &ray.at(ti));
                if qi > level {
                    // i-1 was positive
                    let tj = crate::lerp::lerp(t0, t1, (i - 1) as f32 / n as f32);
                    // TODO: avoid recomputing qj
                    let qj = field_value(&active, &ray.at(tj));
                    // lerp ray parameter t
                    let t = crate::lerp::lerp(tj, ti, (level - qj) / (qi - qj));
                    // compute normal
                    let position = ray.at(t);
                    let normal = normal_at(&active, &position);
                    return Some(Ray {
                        origin: position,
                        direction: normal,
                    });
                }
            }
        }
        None
    }
}

pub struct Light {
    pub direction: Vector3<f32>,
}

impl Light {
    pub fn intensity(&self, direction: &Vector3<f32>) -> f32 {
        self.direction.dot(direction)
    }
}

pub struct EnvironmentMap {
    pub gradient: Gradient,
}

impl EnvironmentMap {
    pub fn color(&self, direction: &Vector3<f32>) -> Color {
        let s = spherical(direction);
        let (theta, phi) = (s.y, s.z);
        let mut colors = Vec::new();
        colors.push((self.gradient.sample(theta / TAU), 1.0));
        //colors.push((checker(phi / (0.5 * TAU), theta / (0.5 * TAU), (16, 16)), 0.2));
        mix_colors(&colors)
        //if theta / TAU > 0.5 { WHITE } else { BLACK }
    }
}

pub struct Scene {
    pub metaballs: Vec<Metaball>,
    pub lights: Vec<Light>,
    pub environment: EnvironmentMap,
}

pub fn trace(scene: &Scene, ray: &Ray<f32>) -> Color {
    if let Some(out) = scene.metaballs.trace(ray) {
        // reflect ray
        let reflected = reflect(&ray.direction, &out.direction);
        let mut colors: Vec<_> = scene
            .lights
            .iter()
            .map(|light| (0xffffffff_u32.to_le_bytes(), 0.0 * light.intensity(&ray.direction)))
            .collect();
        colors.push((0xff842996_u32.to_le_bytes(), 1.0)); // add own color
        colors.push((scene.environment.color(&reflected), 1.0));
        mix_colors(&colors)
    } else {
        // background
        scene.environment.color(&ray.direction)
    }
}

fn reflect(v: &Vector3<f32>, normal: &Vector3<f32>) -> Vector3<f32> {
    v - 2.0 * (v.dot(normal)) * normal
}

fn sphere_ray_intersections(ray: &Ray<f32>, sphere: &Sphere<f32>) -> Option<(f32, f32)> {
    let v = sphere.center - ray.origin;
    let tca = v.dot(&ray.direction);
    //if tca < 0.0 { return None; }
    let d2 = v.dot(&v) - tca * tca;
    let r2 = sphere.radius * sphere.radius;
    if d2 > r2 {
        return None;
    }
    let thc = (r2 - d2).sqrt();
    Some((tca - thc, tca + thc))
}
