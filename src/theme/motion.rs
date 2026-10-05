use egui::Context;

/// Cubic ease-out. Used for panel entrances and theme cross-fades.
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
}

/// Exponential smoothing toward a target — frame-rate independent.
///
/// `speed` ≈ 1/seconds-to-settle. speed=6 → ~99% in 0.5 s.
pub fn smooth_to(current: f32, target: f32, dt: f32, speed: f32) -> f32 {
    let a = 1.0 - (-speed * dt).exp();
    current + (target - current) * a
}

/// Tracks an animation that runs from 0 to 1 over `duration`.
#[derive(Clone, Debug)]
pub struct Anim {
    pub started: f64,
    pub duration: f32,
    pub active: bool,
}

impl Anim {
    pub fn new(now: f64, duration: f32) -> Self {
        Self { started: now, duration: duration.max(0.0001), active: true }
    }

    pub fn progress(&self, ctx: &Context) -> f32 {
        if !self.active { return 1.0; }
        let now = ctx.input(|i| i.time);
        ((now - self.started) as f32 / self.duration).clamp(0.0, 1.0)
    }

    pub fn is_done(&self, ctx: &Context) -> bool {
        self.progress(ctx) >= 1.0
    }
}
