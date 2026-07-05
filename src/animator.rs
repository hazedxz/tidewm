use crate::layout::Rect;
use std::time::{Duration, Instant};

/// Represents one window's ongoing animation from `from` → `to`.
pub struct Animation {
    pub hwnd: isize,     // HWND as isize (Send-safe)
    pub from: Rect,
    pub to: Rect,
    started: Instant,
    duration: Duration,
}

impl Animation {
    pub fn new(hwnd: isize, from: Rect, to: Rect, duration_ms: u64) -> Self {
        Self {
            hwnd,
            from,
            to,
            started: Instant::now(),
            duration: Duration::from_millis(duration_ms),
        }
    }

    /// Returns the interpolated rect at the current moment.
    /// Returns None when the animation is complete.
    pub fn current(&self) -> Option<Rect> {
        let elapsed = self.started.elapsed();
        if elapsed >= self.duration {
            return None; // done
        }
        let t = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        let t = ease_out_cubic(t);
        Some(lerp_rect(self.from, self.to, t))
    }

    pub fn is_done(&self) -> bool {
        self.started.elapsed() >= self.duration
    }
}

/// Ease-out cubic: fast start, decelerates to a stop. Feels natural.
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn lerp_rect(a: Rect, b: Rect, t: f32) -> Rect {
    Rect {
        x: lerp(a.x, b.x, t),
        y: lerp(a.y, b.y, t),
        w: lerp(a.w, b.w, t),
        h: lerp(a.h, b.h, t),
    }
}

fn lerp(a: i32, b: i32, t: f32) -> i32 {
    (a as f32 + (b - a) as f32 * t).round() as i32
}

/// Manages all active animations and drives them from a single thread.
pub struct AnimationDriver {
    pub active: Vec<Animation>,
    duration_ms: u64,
}

impl AnimationDriver {
    pub fn new(duration_ms: u64) -> Self {
        Self {
            active: Vec::new(),
            duration_ms,
        }
    }

    /// Queue or replace an animation for a window.
    pub fn push(&mut self, hwnd: isize, from: Rect, to: Rect) {
        // If already animating this window, start from its current mid-point
        let actual_from = self.active
            .iter()
            .find(|a| a.hwnd == hwnd)
            .and_then(|a| a.current())
            .unwrap_or(from);

        self.active.retain(|a| a.hwnd != hwnd);

        if self.duration_ms == 0 {
            // Instant mode: store a completed animation (caller handles final set)
            self.active.push(Animation::new(hwnd, to, to, 0));
            return;
        }

        self.active.push(Animation::new(hwnd, actual_from, to, self.duration_ms));
    }

    /// Tick all animations. Returns (hwnd, current_rect) for each alive animation.
    /// Removes completed ones.
    pub fn tick(&mut self) -> Vec<(isize, Rect, bool)> {
        let mut updates = Vec::new();
        let mut done_indices = Vec::new();

        for (i, anim) in self.active.iter().enumerate() {
            if let Some(rect) = anim.current() {
                updates.push((anim.hwnd, rect, false));
            } else {
                updates.push((anim.hwnd, anim.to, true));
                done_indices.push(i);
            }
        }

        // Remove done animations (reverse order to preserve indices)
        for i in done_indices.into_iter().rev() {
            self.active.swap_remove(i);
        }

        updates
    }

    pub fn has_active(&self) -> bool {
        !self.active.is_empty()
    }
}
