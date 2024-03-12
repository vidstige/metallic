type Color = [u8; 4];

pub struct Gradient {
    stops: Vec<Color>,
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    let f = (1.0 - t) * (a as f32) + t * (b as f32);
    f.clamp(0.0, 255.0) as u8
}

// Mixes colors x and y according to f. t = 0 means only x, t = 1 means only
// The names x and y are chosen to avoid "b" which may stand for "blue"
fn mix(x: Color, y: Color, t: f32) -> Color {
    [
        lerp(x[0], y[0], t),
        lerp(x[1], y[1], t),
        lerp(x[2], y[2], t),
        lerp(x[3], y[3], t),
    ]
}

impl Gradient {
    pub fn new() -> Gradient{
        Gradient { stops: Vec::new() }
    }
    pub fn add_stop(&mut self, color: u32) {
        self.stops.push(color.to_le_bytes());
    }
    pub fn sample(&self, t: f32) -> Color {
        let n = self.stops.len() - 1;
        let i = (t * n as f32) as usize;
        mix(self.stops[i], self.stops[i + 1], t - i as f32 / n as f32)
    }
}
