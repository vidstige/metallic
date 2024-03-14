use na::{Scalar, Vector3, Point3};

#[derive(PartialEq, PartialOrd)]
pub struct Sphere<T: Scalar> {
    pub center: Point3<T>,
    pub radius: T,
}

impl Sphere<f32> {
    pub fn new(center: Point3<f32>, radius: f32) -> Sphere<f32> {
        Sphere {center, radius}
    }
    pub fn radius_squared(&self) -> f32 {
        self.radius * self.radius
    }
}

// transforms cartesian cordinates x,yz to spherical cordinates r, theta, phi
pub fn spherical(cartesian: &Vector3<f32>) -> Vector3<f32> {
    let r = cartesian.magnitude();
    let theta = (cartesian.y / r).acos();
    let phi = cartesian.z.signum() * (cartesian.x / cartesian.xz().magnitude()).acos();
    Vector3::new(r, theta, phi)
}