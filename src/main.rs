use std::{env, f32::consts::TAU, io::{self, Write}};
extern crate nalgebra as na;
use na::{Scalar, Vector2, Vector3};
mod gradient;
use gradient::Gradient;
mod color;
use crate::color::Color;

type Resolution = (i32, i32);

fn parse_resolution(s: &String) -> Resolution {
    let mut parts = s.split("x");
    let width = parts.next().unwrap().parse().unwrap();
    let height = parts.next().unwrap().parse().unwrap();
    (width, height)
}

fn area(resolution: Resolution) -> usize {
    let (width, height) = resolution;
    (width * height) as usize
}

struct Buffer {
    resolution: Resolution,
    pixels: Vec<u8>,
}

impl Buffer {
    fn new(resolution: Resolution) -> Buffer {
        Buffer { resolution, pixels: vec![0; area(resolution) * 4]}
    }
}

fn pixel(target: &mut Buffer, x: i32, y: i32, color: &Color) {
    let (stride, _) = target.resolution;
    let index = ((x + y * stride) * 4) as usize;
    target.pixels[index..index + color.len()].copy_from_slice(color);
}

// the special tween function
// g(0) = 0 and g(1) = 1 as well as
// g'(0) = 0 and g'(1)
fn g(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[derive(PartialEq, PartialOrd)]
struct Sphere<T: Scalar> {
    center: Vector3<T>,
    radius: T,
}

impl Sphere<f32> {
    fn new(center: Vector3<f32>, radius: f32) -> Sphere<f32> {
        Sphere {center, radius}
    }
    fn radius_squared(&self) -> f32 {
        self.radius * self.radius
    }
}

// transforms cartesian cordinates x,yz to spherical cordinates r, theta, phi
fn spherical(cartesian: &Vector3<f32>) -> Vector3<f32> {
    let r = cartesian.magnitude();
    let theta = (cartesian.y / r).acos();
    let phi = cartesian.z.signum() * (cartesian.x / cartesian.xz().magnitude()).acos();
    Vector3::new(r, theta, phi)
}

#[derive(PartialEq, PartialOrd)]
struct Metaball {
    sphere: Sphere<f32>,
    strength: f32,
}

impl Metaball {
    fn new(position: Vector3<f32>, radius: f32, strength: f32) -> Metaball {
        Metaball {
            sphere: Sphere::new(position, radius),
            strength,
        }
    }
    fn field_value(&self, p: &Vector3<f32>) -> f32 {
        let d2 = (self.sphere.center - p).magnitude_squared();
        let r2 = self.sphere.radius_squared();
        let t = 1.0 - (d2 / r2).sqrt();
        // TODO: Is this range check needed?
        if t < 0.0 || t > 1.0 { 0.0 } else { self.strength * g(t) }
    }
    fn normal(&self, p: &Vector3<f32>) -> Vector3<f32> {
        // The normal is simply the normalized vector from center to the point o
        (p - self.sphere.center).normalize()
    }
}

fn direction(x: i32, y: i32, resolution: Resolution, fov: f32) -> Vector3<f32> {
    let (width, height) = resolution;
    let screen = Vector2::new(x as f32, y as f32);
    let center = 0.5 * Vector2::new(width as f32, height as f32);
    ((screen - center) / center.min() * (0.5 * fov).tan()).push(1.0).normalize()
}

struct Ray<T> {
    origin: Vector3<T>,
    direction: Vector3<T>,
}

impl Ray<f32> {
    fn at(&self, t: f32) -> Vector3<f32> {
        self.origin + self.direction * t
    }
}

fn reflect(v: &Vector3<f32>, normal: &Vector3<f32>) -> Vector3<f32> {
    v - 2.0 * (v.dot(normal)) * normal
}

fn intersection(ray: &Ray<f32>, sphere: &Sphere<f32>) -> Option<(f32, f32)> {
    let v = sphere.center - ray.origin;
    let tca = v.dot(&ray.direction);
    // if (tca < 0) return false;
    let d2 = v.dot(&v) - tca * tca;
    let r2 = sphere.radius * sphere.radius;
    if d2 > r2 {
        return None;
    }
    let thc = (r2 - d2).sqrt();
    Some((tca - thc, tca + thc))
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

fn field_value(metaballs: &[&Metaball], p: &Vector3<f32>) -> f32 {
    metaballs.iter().map(|mb| mb.field_value(p)).sum()
}

fn normal_at(metaballs: &[&Metaball], p: &Vector3<f32>) -> Vector3<f32> {
    let qs: Vec<_> = metaballs.iter().map(|mb| mb.field_value(p)).collect();
    let q: f32 = qs.iter().sum();
    let normal: Vector3<f32> = metaballs.iter().zip(qs).map(|(mb, qi)| qi * mb.normal(p)).sum();
    normal / q
}

trait EnvironmentMap {
    fn color(&self, direction: &Vector3<f32>) -> Color;
}

impl EnvironmentMap for Gradient {
    fn color(&self, direction: &Vector3<f32>) -> Color {
        let theta = spherical(direction).y;
        self.sample(theta / TAU)
    }
}

fn trace(metaballs: &Vec<Metaball>, environment: &dyn EnvironmentMap, ray: &Ray<f32>) -> Option<Color> {
    // find all intersections with sphere of influence
    // also keep track of the ray enters (true) or leavs the sphere
    let mut intersections: Vec<_> = Vec::new();
    for metaball in metaballs {
        if let Some((t0, t1)) = intersection(&ray, &metaball.sphere) {
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
        let n = 10;
        let level = 0.3;
        for i in 0..n {
            let ti = lerp(t0, t1, i as f32 / n as f32);
            let qi = field_value(&active, &ray.at(ti));
            if qi > level {
                // i-1 was positive
                let tj = lerp(t0, t1, (i - 1) as f32 / n as f32);
                // TODO: avoid recomputing qj
                let qj = field_value(&active, &ray.at(tj));
                // lerp ray parameter t
                let t = lerp(tj, ti, (level - qj) / (qi - qj));
                // compute normal
                let normal = normal_at(&active, &ray.at(t));
                //let light = Vector3::new(0.0, -1.0, -1.0).normalize();
                //let g = normal.dot(&light);
                let reflected = reflect(&ray.direction, &normal);
                return Some(environment.color(&reflected));
            }
        }
    }
    None
}

fn render(target: &mut Buffer, fov: f32, position: Vector3<f32>, metaballs: &Vec<Metaball>, environment: &dyn EnvironmentMap) {
    let (width, height) = target.resolution;
    for y in 0..height {
        for x in 0..width {
            let ray = Ray{
                origin: position,
                direction: direction(x, y, target.resolution, fov),
            };

            if let Some(color) = trace(metaballs, environment, &ray) {
                pixel(target, x, y, &color);
            } else {
                // background
                //let BLACK = [0xff, 0, 0, 0];
                let color = environment.color(&ray.direction);
                pixel(target, x, y, &color);
            }
        }
    }
}

fn gray() -> Gradient {
    let mut gradient = Gradient::new();
    gradient.add_stop(0xff000000);
    gradient.add_stop(0xffdddddd);
    gradient
}

fn metallic() -> Gradient {
    let mut gradient = Gradient::new();
    gradient.add_stop(0xff772884);
    gradient.add_stop(0xffDFC0FB);
    gradient.add_stop(0xff842996);
    gradient.add_stop(0xff671D77);
    gradient.add_stop(0xff42104D);
    gradient.add_stop(0xffD3B4E9);
    gradient
}

fn main() -> io::Result<()>{
    let resolution = parse_resolution(&env::var("RESOLUTION").unwrap_or("506x253".to_string()));
    let mut buffer = Buffer::new(resolution);
    let mut metaballs = Vec::new();
    metaballs.push(Metaball::new(Vector3::new(-0.6, 0.0, 0.0), 1.0, 1.0));
    metaballs.push(Metaball::new(Vector3::new(0.6, 0.0, 0.0), 1.0, 1.0));
    let environment = metallic();
    render(&mut buffer, 30.0_f32.to_radians(), Vector3::new(0.0, 0.0, -3.0), &metaballs, &environment);
    std::io::stdout().write_all(&buffer.pixels)?;
    Ok(())
}

