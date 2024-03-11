use std::{env, io::{self, Write}};
extern crate nalgebra as na;
use na::{Vector2, Vector3};

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
        Buffer { resolution: resolution, pixels: vec![0; area(resolution) * 4]}
    }
}

fn pixel(target: &mut Buffer, x: i32, y: i32, color: &Color) {
    let (stride, _) = target.resolution;
    let index = ((x + y * stride) * 4) as usize;
    target.pixels[index..index + color.len()].copy_from_slice(color);
}

struct Metaball {
    position: Vector3<f32>,
    radius: f32,
}

impl Metaball {
    fn new(position: Vector3<f32>, radius: f32) -> Metaball {
        Metaball {position, radius}
    }
    fn sdf(&self, p: &Vector3<f32>) -> f32 {
        // TODO: Can we avoid sqrt here?
        (self.position - p).magnitude() - self.radius
    }
}

struct Metaballs {
    metaballs: Vec<Metaball>,
}

impl Metaballs {
    fn new() -> Metaballs { Metaballs { metaballs: Vec::new() }}
    fn sdf(&self, p: &Vector3<f32>) -> f32 {
        self.metaballs.iter().map(|mb| mb.sdf(p)).sum()
    }
}

fn gray(g: f32) -> Color {
    let g = (255.0 * g) as u8;
    [g, g, g, 0xff]
}

fn direction(x: i32, y: i32, resolution: Resolution, fov: f32) -> Vector3<f32> {
    let (width, height) = resolution;
    let screen = Vector2::new(x as f32, y as f32);
    let center = 0.5 * Vector2::new(width as f32, height as f32);
    // TODO use fov instead of 0.1
    ((screen - center) * 0.1).push(1.0).normalize()
}

fn render(target: &mut Buffer, fov: f32, position: Vector3<f32>, metaballs: &Metaballs) {
    let (width, height) = target.resolution;
    for y in 0..height {
        for x in 0..width {
            // TODO: precalc this
            let direction = direction(x, y, target.resolution, fov);

            // trace ray
            let mut position = position;
            let mut i = 0;
            let n = 10;
            while metaballs.sdf(&position) > 0.0 && i < n {
                //eprintln!("position {}", metaballs.sdf(&position));
                position += direction * (3.0 / n as f32);
                i += 1;
            }
            pixel(target, x, y, &gray(i as f32 / n as f32));
        }
    }
}

fn main() -> io::Result<()>{
    let resolution = parse_resolution(&env::var("RESOLUTION").unwrap_or("506x253".to_string()));
    let mut buffer = Buffer::new(resolution);
    let mut scene = Metaballs::new();
    let a = Metaball::new(Vector3::zeros(), 2.5);
    scene.metaballs.push(a);
    render(&mut buffer, 90.0, Vector3::new(0.0, 0.0, -3.0), &scene);
    std::io::stdout().write(&buffer.pixels)?;
    Ok(())
}
