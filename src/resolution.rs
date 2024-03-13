pub type Resolution = (i32, i32);

pub fn parse_resolution(s: &String) -> Resolution {
    let mut parts = s.split("x");
    let width = parts.next().unwrap().parse().unwrap();
    let height = parts.next().unwrap().parse().unwrap();
    (width, height)
}

pub fn area(resolution: Resolution) -> usize {
    let (width, height) = resolution;
    (width * height) as usize
}