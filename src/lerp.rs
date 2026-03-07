use std::ops::{Add, Mul};

pub fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: Add<Output = T> + Mul<f32, Output = T>,
{
    a * (1.0 - t) + b * t
}
