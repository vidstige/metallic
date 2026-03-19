use std::{
    env,
    f32::consts::TAU,
    io::{self, Write},
};
extern crate nalgebra as na;
use na::{Isometry3, Point2, Point3, Vector2, Vector3};
mod color;
mod eq;
mod gradient;
mod lerp;
mod meatballs;
mod raytracer;
mod resolution;
mod sdf;
mod simplex;
mod sphere;
use crate::color::Color;
use crate::simplex::SimplexNoise;
use gradient::Gradient;
use meatballs::{Meatballs, Metaball};
use raytracer::{trace, EnvironmentMap, Light, Ray, Scene, Tracer};
use resolution::{area, parse_resolution, Resolution};

struct Buffer {
    resolution: Resolution,
    pixels: Vec<u8>,
}

impl Buffer {
    fn new(resolution: Resolution) -> Buffer {
        Buffer {
            resolution,
            pixels: vec![0; area(resolution) * 4],
        }
    }
}

fn pixel(target: &mut Buffer, x: i32, y: i32, color: &Color) {
    let (stride, _) = target.resolution;
    let index = ((x + y * stride) * 4) as usize;
    target.pixels[index..index + color.len()].copy_from_slice(color);
}

const WHITE: Color = 0xffffffff_u32.to_le_bytes();
const BLACK: Color = 0xff000000_u32.to_le_bytes();

fn checker(x: f32, y: f32, resolution: Resolution) -> Color {
    let (w, h) = resolution;
    if ((x * w as f32) as i32 + (y * h as f32) as i32) % 2 == 0 {
        WHITE
    } else {
        BLACK
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
        ((screen - center) / center.min() * (0.5 * self.fov).tan())
            .to_homogeneous()
            .normalize()
    }
}

fn render(
    scene: &Scene,
    tracer: &Tracer,
    surface: &Meatballs,
    camera: &Camera,
    target: &mut Buffer,
) {
    let (width, height) = target.resolution;
    for y in 0..height {
        for x in 0..width {
            let screen = Point2::new(x as f32, y as f32);
            let ray = Ray {
                origin: camera.pose.inverse_transform_point(&Point3::origin()),
                direction: camera
                    .pose
                    .rotation
                    .inverse_transform_vector(&camera.ray_direction(&screen)),
            };

            let color = trace(scene, tracer, surface, &ray);
            pixel(target, x, y, &color);
        }
    }
}

fn metallic() -> Gradient {
    let mut gradient = Gradient::new();
    gradient.add_stop(0xffE2E1DE, 0.0);
    gradient.add_stop(0xffE2E1DE, 0.1);
    gradient.add_stop(0xff404240, 0.2);
    gradient.add_stop(0xff575955, 0.3);
    gradient.add_stop(0xff989691, 0.4);
    gradient.add_stop(0xff989691, 1.0);

    gradient
}

fn two_point_rig() -> Vec<Light> {
    vec![
        Light {
            direction: Vector3::new(1.0, 1.0, 1.0),
        },
        Light {
            direction: Vector3::new(-1.0, 1.0, 1.0),
        },
    ]
}

fn fill(buffer: &mut Buffer, gradient: &Gradient) {
    let (w, h) = buffer.resolution;
    for y in 0..h {
        for x in 0..w {
            let t = (x as f32) / (w as f32);
            pixel(buffer, x, y, &gradient.sample(t));
        }
    }
}

fn main() -> io::Result<()> {
    let resolution = parse_resolution(&env::var("RESOLUTION").unwrap_or("506x253".to_string()));
    let mut buffer = Buffer::new(resolution);
    /*fill(&mut buffer, &metallic());
    std::io::stdout().write_all(&buffer.pixels)?;*/
    let mut metaballs = Vec::new();
    for _ in 0..5 {
        metaballs.push(Metaball::new(Point3::origin(), 3.0, 0.50));
    }
    let scene = Scene {
        lights: two_point_rig(),
        environment: EnvironmentMap {
            gradient: metallic(),
            simplex: SimplexNoise {
                scale: 4.0,
                strength: 0.5,
            },
        },
    };
    let tracer = Tracer {
        near: 2.0,
        far: 8.0,
        steps: 16,
    };
    let mut surface = Meatballs::new(metaballs, 0.3);
    let camera = Camera {
        resolution,
        pose: Isometry3::look_at_lh(
            &Point3::new(1.0, 1.0, -5.0),
            &Point3::origin(),
            &Vector3::new(0.0, -1.0, 0.0),
        ),
        fov: 90.0_f32.to_radians(),
    };
    let fps = 30.0;
    let omega = fps * TAU / 256.0;
    let n = 1024;
    for i in 0..n {
        let time = i as f32 / fps;
        let alpha = omega * time;
        let count = surface.metaballs.len() as f32;
        for (j, metaball) in &mut surface.metaballs.iter_mut().enumerate() {
            let phase = (j as f32) / count;
            let beta = alpha + phase.sin() * TAU;
            metaball.sphere.center.x = (13.0 * beta).cos() * 1.3;
            metaball.sphere.center.y = (5.0 * beta).sin() * 1.3;
            metaball.sphere.center.z = (2.0 * beta).sin() * 1.3;
        }
        render(&scene, &tracer, &surface, &camera, &mut buffer);
        std::io::stdout().write_all(&buffer.pixels)?;
    }
    Ok(())
}
