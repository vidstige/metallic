use crate::color::{mix, Color};

pub struct Gradient {
    stops: Vec<Color>,
}

impl Gradient {
    pub fn new() -> Gradient{
        Gradient { stops: Vec::new() }
    }
    pub fn add_stop(&mut self, color: u32) {
        self.stops.push(color.to_le_bytes());
    }
    pub fn sample(&self, t: f32) -> Color {
        let tc = t.clamp(0.0, 1.0);
        let n = (self.stops.len() - 1) as f32;
        let i = (tc * n) as usize;
        mix(self.stops[i], self.stops[i + 1], (tc - (i as f32) / n) * n)
    }
}
