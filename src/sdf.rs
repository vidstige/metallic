use na::{Point3, Vector3};

pub trait SDF {
    fn sdf(&self, p: &Point3<f32>) -> f32;
    fn normal_at(&self, p: &Point3<f32>) -> Vector3<f32>;
}
