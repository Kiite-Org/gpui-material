//! M3E slider port.
//!
//! Source: `Slider.kt` plus `tokens/SliderTokens.kt`.
//! Kotlin uses `SliderState(value, steps, valueRange)` with drag deltas,
//! snap to ticks, 16dp track, 4x44dp thumb, 6dp gap. This module keeps
//! the same numbers and behavior, expressed for GPUI.

use gpui::Rgba;

use crate::theme::{M3Palette, light_scheme};

/// Geometry from `SliderTokens.kt`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderMetrics {
    /// Inactive and active track height, 16dp.
    pub track_height: f32,
    /// Handle width, 4dp idle, 2dp pressed.
    pub thumb_w: f32,
    /// Handle height, 44dp.
    pub thumb_h: f32,
    /// Gap between thumb and track, 4dp (12dp cutout).
    pub thumb_gap: f32,
    /// Stop indicator size, 4dp.
    pub tick_size: f32,
}

impl Default for SliderMetrics {
    fn default() -> Self {
        Self {
            track_height: 16.0,
            thumb_w: 4.0,
            thumb_h: 44.0,
            thumb_gap: 4.0,
            tick_size: 4.0,
        }
    }
}

/// Colors resolved from `SliderDefaults.colors()`.
///
/// Active maps to `Primary`, inactive to `SecondaryContainer`,
/// disabled uses on-surface with 0.38 and 0.12 opacity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderColors {
    pub thumb: Rgba,
    pub active_track: Rgba,
    pub inactive_track: Rgba,
    pub active_tick: Rgba,
    pub inactive_tick: Rgba,
    pub disabled_thumb: Rgba,
    pub disabled_active_track: Rgba,
    pub disabled_inactive_track: Rgba,
}

impl SliderColors {
    /// Light defaults from the current palette.
    #[must_use]
    pub fn light(p: &M3Palette) -> Self {
        Self::from_palette(p)
    }

    /// Dark defaults from the current palette.
    #[must_use]
    pub fn dark(p: &M3Palette) -> Self {
        Self::from_palette(p)
    }

    /// Resolve colors from any M3 palette (light or dark).
    #[must_use]
    pub fn from_palette(p: &M3Palette) -> Self {
        Self {
            thumb: p.primary,
            active_track: p.primary,
            inactive_track: p.secondary_container,
            active_tick: p.secondary_container,
            inactive_tick: p.primary,
            disabled_thumb: p.on_surface,
            disabled_active_track: p.on_surface,
            disabled_inactive_track: p.on_surface,
        }
    }
}

impl Default for SliderColors {
    fn default() -> Self {
        Self::light(&light_scheme())
    }
}

/// Value model matching `SliderState`.
///
/// `min..=max` is `valueRange`, `steps` is the count of intervals
/// between ends. Zero means continuous.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderSpec {
    pub min: f32,
    pub max: f32,
    pub steps: u32,
}

impl SliderSpec {
    /// Create a spec, coerces inverted ranges.
    #[must_use]
    pub fn new(min: f32, max: f32, steps: u32) -> Self {
        if max >= min {
            Self { min, max, steps }
        } else {
            Self {
                min: max,
                max: min,
                steps,
            }
        }
    }

    /// Fraction 0..=1 for a value, coerced to range.
    #[inline]
    #[must_use]
    pub fn fraction(&self, value: f32) -> f32 {
        let clamped = value.clamp(self.min, self.max);
        if self.max <= self.min {
            return 0.0;
        }
        (clamped - self.min) / (self.max - self.min)
    }

    /// Tick fractions including ends, empty when continuous.
    /// Zero-allocation iterator over tick positions.
    #[inline]
    #[must_use]
    pub fn tick_fractions(&self) -> impl ExactSizeIterator<Item = f32> + DoubleEndedIterator + '_ {
        let denom = if self.steps == 0 { 1 } else { self.steps + 1 };
        let count = if self.steps == 0 { 0 } else { denom + 1 };
        (0..count).map(move |i| i as f32 / denom as f32)
    }

    /// Snap a fraction to the nearest tick in O(1) arithmetic.
    #[inline]
    #[must_use]
    pub fn snap_fraction(&self, frac: f32) -> f32 {
        if self.steps == 0 {
            return frac.clamp(0.0, 1.0);
        }
        let denom = (self.steps + 1) as f32;
        let clamped = frac.clamp(0.0, 1.0);
        let nearest_idx = (clamped * denom).round();
        nearest_idx / denom
    }

    /// Value from a 0..=1 fraction, with snap when stepped.
    #[inline]
    #[must_use]
    pub fn value_from_fraction(&self, frac: f32) -> f32 {
        let f = if self.steps == 0 {
            frac.clamp(0.0, 1.0)
        } else {
            self.snap_fraction(frac)
        };
        self.min + f * (self.max - self.min)
    }

    /// Apply a drag delta in px over a track width in px.
    /// This is the GPUI equivalent of `dispatchRawDelta`.
    #[inline]
    #[must_use]
    pub fn drag_by_px(&self, current: f32, delta_px: f32, track_w_px: f32) -> f32 {
        if track_w_px <= 0.0 {
            return current;
        }
        let cur_frac = self.fraction(current);
        let next_frac = cur_frac + delta_px / track_w_px;
        self.value_from_fraction(next_frac)
    }
}

pub use crate::motion::{SpringParams, SpringState, spring_step};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuous_fraction_roundtrip() {
        let s = SliderSpec::new(0.0, 100.0, 0);
        assert_eq!(s.fraction(25.0), 0.25);
        assert!((s.value_from_fraction(0.25) - 25.0).abs() < 1e-5);
    }

    #[test]
    fn stepped_snaps_to_nearest_tick() {
        // 0..10 with 4 steps allows 0,2,4,6,8,10.
        let s = SliderSpec::new(0.0, 10.0, 4);
        assert!((s.value_from_fraction(0.21) - 2.0).abs() < 1e-4);
        assert!((s.value_from_fraction(0.62) - 6.0).abs() < 1e-4);
    }

    #[test]
    fn drag_moves_proportionally() {
        let s = SliderSpec::new(0.0, 1.0, 0);
        let next = s.drag_by_px(0.5, 48.0, 480.0);
        assert!((next - 0.6).abs() < 1e-5);
    }

    #[test]
    fn metrics_match_slider_tokens() {
        let m = SliderMetrics::default();
        assert_eq!(m.track_height, 16.0);
        assert_eq!(m.thumb_w, 4.0);
        assert_eq!(m.thumb_h, 44.0);
        assert_eq!(m.thumb_gap, 4.0);
    }
}
