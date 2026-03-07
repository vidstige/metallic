use std::f32::consts::TAU;

use na::{Point3, Scalar, Vector3};

use crate::color::Color;
use crate::color::mix_colors;
use crate::eq::linesearch;
use crate::gradient::Gradient;
use crate::sdf::SDF;
use crate::sphere::spherical;

pub struct Ray<T: Scalar> {
    pub origin: Point3<T>,
    pub direction: Vector3<T>,
}

impl Ray<f32> {
    pub fn at(&self, t: f32) -> Point3<f32> {
        self.origin + self.direction * t
    }
}

pub struct Tracer {
    pub near: f32,
    pub far: f32,
    pub steps: usize,
}

impl Tracer {
    pub fn trace<S: SDF>(&self, ray: &Ray<f32>, surface: &S) -> Option<Ray<f32>> {
        if let Some((t0, t1)) = linesearch(|t| surface.sdf(&ray.at(t)), self.near, self.far, self.steps) {
            let q0 = surface.sdf(&ray.at(t0));
            let q1 = surface.sdf(&ray.at(t1));
            let t = crate::lerp::lerp(t0, t1, -q0 / (q1 - q0));
            let position = ray.at(t);
            let normal = surface.normal_at(&position);
            return Some(Ray {
                origin: position,
                direction: normal,
            });
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
    pub tracer: Tracer,
    pub lights: Vec<Light>,
    pub environment: EnvironmentMap,
}

pub fn trace<S: SDF>(scene: &Scene, surface: &S, ray: &Ray<f32>) -> Color {
    if let Some(out) = scene.tracer.trace(ray, surface) {
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
