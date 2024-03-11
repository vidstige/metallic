use std::{env, io::{self, Write}};

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
        Buffer { resolution: resolution, pixels: vec![0; area(resolution) * 4]}
    }
}

fn pixel(target: &mut Buffer, x: i32, y: i32, color: u32) {
    let (stride, _) = target.resolution;
    let index = ((x + y * stride) * 4) as usize;
    let bytes = color.to_le_bytes();
    target.pixels[index..index + bytes.len()].copy_from_slice(&bytes);
}

fn render(target: &mut Buffer) {
    let (width, height) = target.resolution;
    for y in 0..height {
        for x in 0..width {
            pixel(target, x, y, 0xffcecece);
        }
    }
}

fn main() -> io::Result<()>{
    let resolution = parse_resolution(&env::var("RESOLUTION").unwrap_or("506x253".to_string()));
    let mut buffer = Buffer::new(resolution);
    render(&mut buffer);
    std::io::stdout().write(&buffer.pixels)?;
    Ok(())
}
