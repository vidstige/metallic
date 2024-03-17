use std::{env, f32::consts::{PI, TAU}, io::{self, Write}};
extern crate nalgebra as na;
use color::mix_colors;
use na::{Isometry3, Point2, Point3, Scalar, Vector2, Vector3};
mod gradient;
mod sphere;
mod color;
mod resolution;
use sphere::{Sphere, spherical};
use gradient::Gradient;
use resolution::{Resolution, parse_resolution, area};
use crate::color::Color;

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
struct Metaball {
    sphere: Sphere<f32>,
    strength: f32,
}

impl Metaball {
    fn new(position: Point3<f32>, radius: f32, strength: f32) -> Metaball {
        Metaball {
            sphere: Sphere::new(position, radius),
            strength,
        }
    }
    fn field_value(&self, p: &Point3<f32>) -> f32 {
        let d2 = (self.sphere.center - p).magnitude_squared();
        let r2 = self.sphere.radius_squared();
        if d2 > r2 {
            return 0.0;
        }
        let t = 1.0 - (d2 / r2).sqrt();
        self.strength * g(t)
    }
    fn normal(&self, p: &Point3<f32>) -> Vector3<f32> {
        // The normal is simply the normalized vector from center to the point o
        (p - self.sphere.center).normalize()
    }
}

struct Ray<T: Scalar> {
    origin: Point3<T>,
    direction: Vector3<T>,
}

impl Ray<f32> {
    fn at(&self, t: f32) -> Point3<f32> {
        self.origin + self.direction * t
    }
}

trait Traceable<T: Scalar> {
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
                    let position = ray.at(t);
                    let normal = normal_at(&active, &position);
                    return Some(Ray{origin: position, direction: normal});
                }
            }
        }
        None
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

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

fn field_value(metaballs: &[&Metaball], p: &Point3<f32>) -> f32 {
    metaballs.iter().map(|mb| mb.field_value(p)).sum()
}

fn normal_at(metaballs: &[&Metaball], p: &Point3<f32>) -> Vector3<f32> {
    let qs: Vec<_> = metaballs.iter().map(|mb| mb.field_value(p)).collect();
    let q: f32 = qs.iter().sum();
    let normal: Vector3<f32> = metaballs.iter().zip(qs).map(|(mb, qi)| qi * mb.normal(p)).sum();
    normal / q
}

fn checker(x: f32, y: f32, resolution: Resolution) -> Color {
    let (w, h) = resolution;
    let black = 0xff000000_u32.to_le_bytes();
    let white = 0xffffffff_u32.to_le_bytes();
    if ((x * w as f32) as i32 + (y * h as f32) as i32) % 2 == 0{
        white
    } else {
        black
    }
}

struct Light {
    direction: Vector3<f32>,
}

impl Light {
    fn intensity(&self, direction: &Vector3<f32>) -> f32 {
        self.direction.dot(direction)
    }
}

struct EnvironmentMap {
    gradient: Gradient,
}

impl EnvironmentMap {
    fn color(&self, direction: &Vector3<f32>) -> Color {
        let s = spherical(direction);
        let (theta, phi) = (s.y, s.z);
        let mut colors = Vec::new();
        colors.push((self.gradient.sample(theta / TAU), 1.0));
        colors.push((checker(phi / PI, theta / PI, (16, 16)), 0.2));
        /*for light in self.lights.iter() {
            colors.push((0xffffffff_u32.to_le_bytes(), light.color(direction)));
        }*/
        //let checker = ;
        mix_colors(&colors)
    }
}

struct Scene {
    metaballs: Vec<Metaball>,
    lights: Vec<Light>,
    environment: EnvironmentMap,
}

fn trace(scene: &Scene, ray: &Ray<f32>) -> Color {
    if let Some(out) = scene.metaballs.trace(ray) {
        // reflect ray
        let reflected = reflect(&ray.direction, &out.direction);
        let white = 0xffffffff_u32.to_le_bytes();
        let mut colors: Vec<_> = scene.lights.iter().map(|light| (white, 0.1  *light.intensity(&ray.direction))).collect();
        colors.push((scene.environment.color(&reflected), 1.0));
        mix_colors(&colors)
    } else {
        // background
        scene.environment.color(&ray.direction)
    }
}

struct Camera {
    resolution: Resolution,
    pose: Isometry3<f32>,
    fov: f32,
}

impl Camera {
    fn ray_direction(&self, screen: &Point2<f32>) -> Vector3<f32> {
        let (width, height) = self.resolution;
        let center = 0.5 * Vector2::new(width as f32, height as f32);
        ((screen - center) / center.min() * (0.5 * self.fov).tan()).to_homogeneous().normalize()
    }
}

fn render(scene: &Scene, camera: &Camera, target: &mut Buffer) {
    let (width, height) = target.resolution;
    for y in 0..height {
        for x in 0..width {
            let screen = Point2::new(x as f32, y as f32);
            let ray = Ray{
                origin: camera.pose.inverse_transform_point(&Point3::origin()),
                direction: camera.pose.rotation.inverse_transform_vector(&camera.ray_direction(&screen)),
            };

            let color = trace(scene, &ray);
            pixel(target, x, y, &color);
        }
    }
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

fn two_point_rig() -> Vec<Light> {
    vec![
        Light{direction: Vector3::new(1.0, 1.0, 1.0)},
        Light{direction: Vector3::new(-1.0, 1.0, 1.0)},
    ]
}

fn main() -> io::Result<()>{
    let resolution = parse_resolution(&env::var("RESOLUTION").unwrap_or("506x253".to_string()));
    let mut buffer = Buffer::new(resolution);
    let mut metaballs = Vec::new();
    for _ in 0..5 {
        metaballs.push(Metaball::new(Point3::origin(), 3.0, 0.50));
    }
    let mut scene = Scene {
        metaballs: metaballs,
        lights: two_point_rig(),
        environment: EnvironmentMap {
            gradient: metallic(),
        },
    };
    let camera = Camera {
        resolution,
        pose: Isometry3::look_at_lh(
            &Point3::new(1.0, 1.0, -5.0),
            &Point3::origin(),
            &Vector3::new(0.0, -1.0, 0.0),
        ),
        fov: 90.0_f32.to_radians(),
    };
    let n = 260;
    for i in 0..n {
        let alpha = TAU * (i as f32) / (n as f32);
        let count = scene.metaballs.len() as f32;
        for (j, metaball) in &mut scene.metaballs.iter_mut().enumerate() {
            let phase = (j as f32) / count;
            let beta = alpha + phase.sin() * TAU;
            metaball.sphere.center.x = (13.0 * beta).cos() * 1.3;
            metaball.sphere.center.y = (5.0 * beta).sin() * 1.3;
            metaball.sphere.center.z = (2.0 * beta).sin() * 1.3;
        }
        render(&scene, &camera, &mut buffer);
        std::io::stdout().write_all(&buffer.pixels)?;
    }
    
    Ok(())
}

