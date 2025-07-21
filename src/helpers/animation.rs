use cgmath::num_traits::ToPrimitive;

pub fn ease_in_ease_out(dt: u64, delay: u64) -> f32 {
    if dt < delay {
        return 0.0;
    }
    let elapsed = (dt - delay) % 200;
    if elapsed >= 100 {
        let time = (199 - elapsed).to_f32().unwrap() / 100.0;
        let sqr = time * time;
        sqr / (2.0 * (sqr - time) + 1.0)
    } else {
        let time = elapsed.to_f32().unwrap() / 100.0;
        let sqr = time * time;
        sqr / (2.0 * (sqr - time) + 1.0)
    }
}
