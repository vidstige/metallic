use crate::color::{mix, Color};

pub struct Gradient {
    stops: Vec<(Color, f32)>,
}

impl Gradient {
    pub fn new() -> Gradient{
        Gradient { stops: Vec::new() }
    }
    pub fn add_stop(&mut self, color: u32, t: f32) {
        self.stops.push((color.to_le_bytes(), t));
    }
    pub fn sample(&self, t: f32) -> Color {
        let tc = t.clamp(0.0, 1.0);
        for ((c0, t0), (c1, t1)) in self.stops.windows(2).map(|pair| (pair[0], pair[1])) {
            if tc >= t0 && tc < t1 {
                return mix(c0, c1, (tc - t0) / (t1 - t0));
            }
        }
        self.stops[0].0
    }
}
