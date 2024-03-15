pub type Color = [u8; 4];

fn make_color(r: f32, g: f32, b: f32, alpha: u8) -> Color {
    [
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
        alpha,
    ]
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    let f = (1.0 - t) * (a as f32) + t * (b as f32);
    f.clamp(0.0, 255.0) as u8
}

pub fn mix_colors(colors: &[(Color, f32)]) -> Color {
    let total_weight: f32 = colors.iter().map(|(_, w)| w).sum();
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    for (color, weight) in colors {
        r += (color[0] as f32) * weight;
        g += (color[1] as f32) * weight;
        b += (color[2] as f32) * weight;
    }
    make_color(r / total_weight, g / total_weight, b / total_weight, 0xff)
}

// Mixes colors x and y according to f. t = 0 means only x, t = 1 means only
// The names x and y are chosen to avoid "b" which may stand for "blue"
pub fn mix(x: Color, y: Color, t: f32) -> Color {
    [
        lerp(x[0], y[0], t),
        lerp(x[1], y[1], t),
        lerp(x[2], y[2], t),
        lerp(x[3], y[3], t),
    ]
}