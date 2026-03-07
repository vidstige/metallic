use na::{Point3, Vector3};

use crate::sdf::SDF;
use crate::sphere::Sphere;

#[derive(PartialEq, PartialOrd)]
pub struct Metaball {
    pub sphere: Sphere<f32>,
    strength: f32,
}

pub struct Meatballs {
    pub metaballs: Vec<Metaball>,
    pub level: f32,
}

impl Meatballs {
    pub fn new(metaballs: Vec<Metaball>, level: f32) -> Meatballs {
        Meatballs { metaballs, level }
    }
    pub fn field_value(&self, p: &Point3<f32>) -> f32 {
        self.metaballs.iter().map(|mb| mb.field_value(p)).sum()
    }
    pub fn normal_at(&self, p: &Point3<f32>) -> Vector3<f32> {
        let active: Vec<_> = self.metaballs.iter().collect();
        normal_at(&active, p)
    }
}

impl Metaball {
    pub fn new(position: Point3<f32>, radius: f32, strength: f32) -> Metaball {
        Metaball {
            sphere: Sphere::new(position, radius),
            strength,
        }
    }
    pub fn field_value(&self, p: &Point3<f32>) -> f32 {
        let d2 = (self.sphere.center - p).magnitude_squared();
        let r2 = self.sphere.radius_squared();
        if d2 > r2 {
            return 0.0;
        }
        let t = 1.0 - (d2 / r2).sqrt();
        self.strength * g(t)
    }
    pub fn normal(&self, p: &Point3<f32>) -> Vector3<f32> {
        // The normal is simply the normalized vector from center to the point o
        (p - self.sphere.center).normalize()
    }
}

pub(crate) fn normal_at(metaballs: &[&Metaball], p: &Point3<f32>) -> Vector3<f32> {
    let qs: Vec<_> = metaballs.iter().map(|mb| mb.field_value(p)).collect();
    let q: f32 = qs.iter().sum();
    let normal: Vector3<f32> = metaballs
        .iter()
        .zip(qs)
        .map(|(mb, qi)| qi * mb.normal(p))
        .sum();
    normal / q
}

// the special tween function
// g(0) = 0 and g(1) = 1 as well as
// g'(0) = 0 and g'(1)
fn g(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

impl SDF for Meatballs {
    fn sdf(&self, p: &Point3<f32>) -> f32 {
        self.field_value(p) - self.level
    }

    fn normal_at(&self, p: &Point3<f32>) -> Vector3<f32> {
        Meatballs::normal_at(self, p)
    }
}
