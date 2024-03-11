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

fn main() -> io::Result<()>{
    let resolution = parse_resolution(&env::var("RESOLUTION").unwrap_or("506x253".to_string()));
    let buffer = vec![0; area(resolution) * 4];
    std::io::stdout().write(&buffer)?;
    Ok(())
}
