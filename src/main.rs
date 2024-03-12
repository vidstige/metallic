use std::{env, io::{self, Write}};
extern crate nalgebra as na;
use na::{Scalar, Vector2, Vector3};

type Resolution = (i32, i32);
type Color = [u8; 4];

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
        if t < 0.0 || t > 1.0 { 0.0 } else { self.strength * g(t) }
    }
}

fn gray(g: f32) -> Color {
    let g = (255.0 * g.clamp(0.0, 1.0)) as u8;
    [g, g, g, 0xff]
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

fn field_value(metaballs: &Vec<&Metaball>, p: &Vector3<f32>) -> f32 {
    metaballs.iter().map(|mb| mb.field_value(p)).sum()
}

fn trace(metaballs: &Vec<Metaball>, ray: &Ray<f32>) -> Option<f32> {
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
        let level = 0.5;
        for i in 0..n {
            let ti = lerp(t0, t1, i as f32 / n as f32);
            let qi = field_value(&active, &ray.at(ti));
            if qi > level {
                // i-1 was positive
                let tj = lerp(t0, t1, (i - 1) as f32 / n as f32);
                // TODO: avoid recomputing qj
                let qj = field_value(&active, &ray.at(tj));
                // lerp ray parameter t
                return Some((level - qj) / (qi - qj));
            }
        }   
    }
    None
}

fn render(target: &mut Buffer, fov: f32, position: Vector3<f32>, metaballs: &Vec<Metaball>) {
    let (width, height) = target.resolution;
    for y in 0..height {
        for x in 0..width {
            let ray = Ray{
                origin: position,
                direction: direction(x, y, target.resolution, fov),
            };

            if let Some(t) = trace(metaballs, &ray) {
                pixel(target, x, y, &gray(t));
            } else {
                pixel(target, x, y, &gray(0.0));
            }
        }
    }
}

fn main() -> io::Result<()>{
    let resolution = parse_resolution(&env::var("RESOLUTION").unwrap_or("506x253".to_string()));
    let mut buffer = Buffer::new(resolution);
    let mut metaballs = Vec::new();
    metaballs.push(Metaball::new(Vector3::new(-0.6, 0.0, 0.0), 1.0, 1.0));
    metaballs.push(Metaball::new(Vector3::new(0.6, 0.0, 0.0), 1.0, 1.0));
    render(&mut buffer, 30.0_f32.to_radians(), Vector3::new(0.0, 0.0, -3.0), &metaballs);
    std::io::stdout().write_all(&buffer.pixels)?;
    Ok(())
}
