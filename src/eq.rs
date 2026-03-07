pub fn linesearch<F>(f: F, lo: f32, hi: f32, steps: usize) -> Option<(f32, f32)>
where
    F: Fn(f32) -> f32,
{
    if steps == 0 {
        return None;
    }

    let mut previous_t = lo;
    let mut previous_q = f(lo);

    for i in 1..=steps {
        let t = crate::lerp::lerp(lo, hi, i as f32 / steps as f32);
        let q = f(t);
        if previous_q.signum() != q.signum() || q == 0.0 {
            return Some((previous_t, t));
        }
        previous_t = t;
        previous_q = q;
    }

    None
}
