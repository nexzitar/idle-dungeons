//! Debounced inspect content swaps with a short alpha fade-in.

use bevy::prelude::Color;
use bevy::prelude::Alpha;

const DEBOUNCE_SECS: f32 = 0.05;
const FADE_SECS: f32 = 0.15;
const FADE_START_ALPHA: f32 = 0.55;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectFadeTick {
    Idle,
    ApplyContent,
    Animating,
}

/// Tracks pending inspect target changes and fade progress.
#[derive(Clone, Debug, Default)]
pub struct InspectContentFade {
    displayed_key: String,
    pending_key: String,
    debounce_remaining: f32,
    pub fade_remaining: f32,
    pub alpha: f32,
    apply_pending: bool,
}

impl InspectContentFade {
    pub fn is_uninitialized(&self) -> bool {
        self.displayed_key.is_empty()
    }

    pub fn is_debouncing(&self) -> bool {
        self.debounce_remaining > 0.0
    }

    pub fn mark_initialized(&mut self, key: String) {
        self.displayed_key = key;
        self.pending_key.clear();
        self.debounce_remaining = 0.0;
        self.fade_remaining = 0.0;
        self.alpha = 1.0;
        self.apply_pending = true;
    }

    pub fn notify_target(&mut self, key: String) {
        if key == self.displayed_key {
            self.pending_key.clear();
            self.debounce_remaining = 0.0;
            self.apply_pending = false;
            return;
        }
        if key == self.pending_key && self.debounce_remaining > 0.0 {
            return;
        }
        self.pending_key = key;
        self.debounce_remaining = DEBOUNCE_SECS;
        self.apply_pending = false;
    }

    pub fn tick(&mut self, dt: f32) -> InspectFadeTick {
        if self.debounce_remaining > 0.0 {
            self.debounce_remaining = (self.debounce_remaining - dt).max(0.0);
            if self.debounce_remaining == 0.0
                && !self.pending_key.is_empty()
                && self.pending_key != self.displayed_key
            {
                self.displayed_key.clone_from(&self.pending_key);
                self.fade_remaining = FADE_SECS;
                self.alpha = FADE_START_ALPHA;
                self.apply_pending = true;
                return InspectFadeTick::ApplyContent;
            }
        }

        if self.fade_remaining > 0.0 {
            self.fade_remaining = (self.fade_remaining - dt).max(0.0);
            let t = 1.0 - (self.fade_remaining / FADE_SECS).clamp(0.0, 1.0);
            self.alpha = FADE_START_ALPHA + t * (1.0 - FADE_START_ALPHA);
            if self.fade_remaining == 0.0 {
                self.alpha = 1.0;
            }
            return InspectFadeTick::Animating;
        }

        self.alpha = 1.0;
        InspectFadeTick::Idle
    }

    pub fn take_apply_pending(&mut self) -> bool {
        let apply = self.apply_pending;
        self.apply_pending = false;
        apply
    }
}

#[must_use]
pub fn fade_tint(base: Color, alpha: f32) -> Color {
    let a = base.to_srgba().alpha * alpha.clamp(0.0, 1.0);
    base.with_alpha(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debounce_then_apply_on_target_change() {
        let mut fade = InspectContentFade::default();
        fade.notify_target("a".to_string());
        assert_eq!(fade.tick(0.02), InspectFadeTick::Idle);
        assert_eq!(fade.tick(0.04), InspectFadeTick::ApplyContent);
        assert!(fade.take_apply_pending());
    }

    #[test]
    fn fade_alpha_eases_up_after_apply() {
        let mut fade = InspectContentFade::default();
        fade.notify_target("a".to_string());
        let _ = fade.tick(DEBOUNCE_SECS);
        assert!((fade.alpha - FADE_START_ALPHA).abs() < f32::EPSILON);
        let _ = fade.tick(FADE_SECS * 0.5);
        assert!(fade.alpha > FADE_START_ALPHA);
        let _ = fade.tick(FADE_SECS);
        assert!((fade.alpha - 1.0).abs() < f32::EPSILON);
    }
}
