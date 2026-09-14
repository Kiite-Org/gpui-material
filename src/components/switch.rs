//! Material 3 Expressive Switch component math and models.
//!
//! Specification:
//! - Track: 54x32dp pill (`rounded_full`)
//! - Thumb: 28x28dp idle, 34x28dp pressed
//! - Expansion: anchored left when unchecked (expands right),
//!   anchored right when checked (expands left)
//! - Icon: 16x16 canvas with morphing lines (checkmark <-> dash <-> cross)
//! - State layer: hover (0.08) and press (0.1) overlay inside thumb
//! - Damped spring animation for thumb translation, expansion, and icon morphing

use gpui::{Bounds, PathBuilder, Pixels, Rgba, Window, point, px};

use crate::motion::{SpringParams, SpringState, spring_step};
use crate::theme::{M3Palette, light_scheme};

/// Geometry matching Material 3 Expressive switch tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchMetrics {
    /// Track width: 54dp.
    pub track_width: f32,
    /// Track height: 32dp.
    pub track_height: f32,
    /// Thumb height: 28dp.
    pub thumb_height: f32,
    /// Idle thumb width: 28dp.
    pub thumb_width_idle: f32,
    /// Pressed thumb width: 34dp.
    pub thumb_width_pressed: f32,
    /// Icon canvas box: 16dp.
    pub icon_size: f32,
    /// Icon stroke thickness: 2.4dp.
    pub icon_stroke_width: f32,
}

impl Default for SwitchMetrics {
    fn default() -> Self {
        Self {
            track_width: 54.0,
            track_height: 32.0,
            thumb_height: 28.0,
            thumb_width_idle: 28.0,
            thumb_width_pressed: 34.0,
            icon_size: 16.0,
            icon_stroke_width: 2.4,
        }
    }
}

/// Colors resolved from M3 palette tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchColors {
    /// Pill track container background when checked (on).
    pub track_checked: Rgba,
    /// Pill track container background when unchecked (off).
    pub track_unchecked: Rgba,
    /// Floating circular thumb handle color when checked.
    pub thumb_checked: Rgba,
    /// Floating circular thumb handle color when unchecked.
    pub thumb_unchecked: Rgba,
    /// Hover and press state layer overlay tint when checked.
    pub state_layer_checked: Rgba,
    /// Hover and press state layer overlay tint when unchecked.
    pub state_layer_unchecked: Rgba,
    /// Stroke color for the checkmark icon when checked.
    pub icon_checked: Rgba,
    /// Stroke color for the cross icon when unchecked.
    pub icon_unchecked: Rgba,
    /// Track background color when checked and disabled.
    pub disabled_track_checked: Rgba,
    /// Track background color when unchecked and disabled.
    pub disabled_track_unchecked: Rgba,
    /// Thumb handle color when checked and disabled.
    pub disabled_thumb_checked: Rgba,
    /// Thumb handle color when unchecked and disabled.
    pub disabled_thumb_unchecked: Rgba,
    /// Icon stroke color when checked and disabled.
    pub disabled_icon_checked: Rgba,
    /// Icon stroke color when unchecked and disabled.
    pub disabled_icon_unchecked: Rgba,
}

impl SwitchColors {
    /// Light color mapping matching M3 defaults.
    #[must_use]
    pub fn light(p: &M3Palette) -> Self {
        Self {
            track_checked: p.primary,
            track_unchecked: p.surface_container_highest,
            thumb_checked: p.on_primary,
            thumb_unchecked: p.outline,
            state_layer_checked: p.primary,
            state_layer_unchecked: p.on_surface,
            icon_checked: p.primary,
            icon_unchecked: p.surface_container_highest,
            disabled_track_checked: p.on_surface,
            disabled_track_unchecked: p.surface_container_highest,
            disabled_thumb_checked: p.surface,
            disabled_thumb_unchecked: p.on_surface,
            disabled_icon_checked: p.outline_variant,
            disabled_icon_unchecked: p.surface_container_highest,
        }
    }

    /// Dark color mapping matching M3 defaults.
    #[must_use]
    pub fn dark(p: &M3Palette) -> Self {
        Self {
            track_checked: p.primary,
            track_unchecked: p.surface_container_highest,
            thumb_checked: p.on_primary,
            thumb_unchecked: p.outline,
            state_layer_checked: p.primary,
            state_layer_unchecked: p.on_surface,
            icon_checked: p.primary,
            icon_unchecked: p.surface_container_highest,
            disabled_track_checked: p.on_surface,
            disabled_track_unchecked: p.surface_container_highest,
            disabled_thumb_checked: p.surface,
            disabled_thumb_unchecked: p.on_surface,
            disabled_icon_checked: p.outline_variant,
            disabled_icon_unchecked: p.surface_container_highest,
        }
    }

    /// Resolve colors from palette and dark mode flag.
    #[must_use]
    pub fn from_palette(p: &M3Palette, is_dark: bool) -> Self {
        if is_dark {
            Self::dark(p)
        } else {
            Self::light(p)
        }
    }
}

impl Default for SwitchColors {
    fn default() -> Self {
        Self::light(&light_scheme())
    }
}

/// A single line segment in a 16x16 icon coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconSegment {
    /// Beginning X coordinate in the local 16x16 icon viewport.
    pub start_x: f32,
    /// Beginning Y coordinate in the local 16x16 icon viewport.
    pub start_y: f32,
    /// Ending X coordinate in the local 16x16 icon viewport.
    pub end_x: f32,
    /// Ending Y coordinate in the local 16x16 icon viewport.
    pub end_y: f32,
}

impl IconSegment {
    /// Create a new segment.
    #[must_use]
    pub const fn new(sx: f32, sy: f32, ex: f32, ey: f32) -> Self {
        Self {
            start_x: sx,
            start_y: sy,
            end_x: ex,
            end_y: ey,
        }
    }

    /// Linear interpolation between two segments.
    #[inline]
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            start_x: self.start_x + (other.start_x - self.start_x) * t,
            start_y: self.start_y + (other.start_y - self.start_y) * t,
            end_x: self.end_x + (other.end_x - self.end_x) * t,
            end_y: self.end_y + (other.end_y - self.end_y) * t,
        }
    }
}

/// 16x16 icon keyframe segments.
///
/// - Pressed checked / unchecked: horizontal dash along y=8.0
/// - Idle checked: checkmark
/// - Idle unchecked: cross X
#[inline]
#[must_use]
pub fn icon_segments(checked: bool, pressed: bool) -> (IconSegment, IconSegment) {
    if pressed {
        if checked {
            (
                IconSegment::new(3.2, 8.0, 6.4, 8.0),
                IconSegment::new(6.4, 8.0, 12.8, 8.0),
            )
        } else {
            (
                IconSegment::new(3.2, 8.0, 12.8, 8.0),
                IconSegment::new(3.2, 8.0, 12.8, 8.0),
            )
        }
    } else if checked {
        (
            IconSegment::new(2.4, 8.0, 6.4, 11.2),
            IconSegment::new(6.4, 11.2, 13.6, 3.2),
        )
    } else {
        (
            IconSegment::new(2.4, 2.4, 13.6, 13.6),
            IconSegment::new(2.4, 13.6, 13.6, 2.4),
        )
    }
}

/// Smoothly interpolated icon segments for 16x16 canvas.
#[inline]
#[must_use]
pub fn animated_icon_segments(pos: f32, press_pos: f32) -> (IconSegment, IconSegment) {
    let (un_idle1, un_idle2) = icon_segments(false, false);
    let (ch_idle1, ch_idle2) = icon_segments(true, false);
    let (un_press1, un_press2) = icon_segments(false, true);
    let (ch_press1, ch_press2) = icon_segments(true, true);

    let idle1 = un_idle1.lerp(ch_idle1, pos);
    let idle2 = un_idle2.lerp(ch_idle2, pos);
    let press1 = un_press1.lerp(ch_press1, pos);
    let press2 = un_press2.lerp(ch_press2, pos);

    (idle1.lerp(press1, press_pos), idle2.lerp(press2, press_pos))
}

/// Compute thumb left offset and width.
///
/// Unchecked: anchors left at 2dp, expands right when pressed (28dp -> 34dp).
/// Checked: anchors right at 52dp, expands left when pressed (24dp -> 18dp).
#[inline]
#[must_use]
pub fn thumb_geometry(metrics: &SwitchMetrics, pos: f32, press_pos: f32) -> (f32, f32) {
    let thumb_w = metrics.thumb_width_idle
        + (metrics.thumb_width_pressed - metrics.thumb_width_idle) * press_pos;
    let travel = metrics.track_width - 4.0 - thumb_w;
    let left = 2.0 + pos * travel;
    (left, thumb_w)
}

/// Draw the 16x16 morphing icon lines on a GPUI canvas.
pub fn draw_switch_icon(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    seg1: IconSegment,
    seg2: IconSegment,
    color: Rgba,
) {
    let mut builder = PathBuilder::stroke(px(2.4));
    let ox = bounds.left();
    let oy = bounds.top();
    builder.move_to(point(ox + px(seg1.start_x), oy + px(seg1.start_y)));
    builder.line_to(point(ox + px(seg1.end_x), oy + px(seg1.end_y)));
    builder.move_to(point(ox + px(seg2.start_x), oy + px(seg2.start_y)));
    builder.line_to(point(ox + px(seg2.end_x), oy + px(seg2.end_y)));
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

/// State model for an animated switch.
///
/// # Examples
///
/// ```rust,no_run
/// use gpui_material::switch::SwitchModel;
///
/// let mut sw = SwitchModel::new("Dark mode", false);
///
/// // Toggle state on click
/// sw.toggle();
/// ```
#[derive(Clone, Debug)]
pub struct SwitchModel {
    /// Descriptive label text displayed alongside the switch.
    pub label: String,
    /// Whether the switch is currently toggled on.
    pub checked: bool,
    /// Whether user interaction is enabled.
    pub enabled: bool,
    /// Whether to render the morphing checkmark and cross icon lines.
    pub show_icons: bool,
    /// Spring state tracking horizontal thumb travel progress from 0.0 to 1.0.
    pub display_pos: SpringState,
    /// Spring state tracking thumb expansion progress on press from 0.0 to 1.0.
    pub press_pos: SpringState,
    /// Whether the pointer is currently holding down on the switch.
    pub is_pressed: bool,
    /// Whether the pointer is currently hovering over the switch.
    pub is_hovered: bool,
}

impl SwitchModel {
    /// Create a new switch model.
    #[must_use]
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        let pos = if checked { 1.0 } else { 0.0 };
        Self {
            label: label.into(),
            checked,
            enabled: true,
            show_icons: true,
            display_pos: SpringState { pos, vel: 0.0 },
            press_pos: SpringState { pos: 0.0, vel: 0.0 },
            is_pressed: false,
            is_hovered: false,
        }
    }

    /// Set whether the switch is enabled.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set whether to render the morphing icons inside the thumb.
    #[must_use]
    pub fn show_icons(mut self, show: bool) -> Self {
        self.show_icons = show;
        self
    }

    /// Toggle checked state if enabled.
    pub fn toggle(&mut self) {
        if !self.enabled {
            return;
        }
        self.checked = !self.checked;
    }

    /// Advance spring animations by dt seconds, requesting an animation frame if unsettled.
    pub fn advance_springs(&mut self, window: &mut Window, dt: f32) {
        let target_pos = if self.checked { 1.0 } else { 0.0 };
        let target_press = if self.is_pressed && self.enabled { 1.0 } else { 0.0 };

        let pos_params = SpringParams::standard();
        let press_params = SpringParams {
            stiffness: 450.0,
            damping: 32.0,
            mass: 1.0,
        };

        let next_pos = spring_step(pos_params, self.display_pos, target_pos, dt);
        let settled_pos = (next_pos.pos - target_pos).abs() < 0.0005 && next_pos.vel.abs() < 0.0005;
        self.display_pos = if settled_pos {
            SpringState { pos: target_pos, vel: 0.0 }
        } else {
            window.request_animation_frame();
            next_pos
        };

        let next_press = spring_step(press_params, self.press_pos, target_press, dt);
        let settled_press = (next_press.pos - target_press).abs() < 0.0005 && next_press.vel.abs() < 0.0005;
        self.press_pos = if settled_press {
            SpringState { pos: target_press, vel: 0.0 }
        } else {
            window.request_animation_frame();
            next_press
        };
    }

    /// Resolve display colors for the current state.
    #[must_use]
    pub fn resolve_colors(&self, colors: &SwitchColors) -> (Rgba, Rgba, Rgba, Rgba, f32) {
        let (track_color, thumb_color, icon_color, state_layer_color) = if !self.enabled {
            if self.checked {
                (
                    colors.disabled_track_checked,
                    colors.disabled_thumb_checked,
                    colors.disabled_icon_checked,
                    colors.state_layer_checked,
                )
            } else {
                (
                    colors.disabled_track_unchecked,
                    colors.disabled_thumb_unchecked,
                    colors.disabled_icon_unchecked,
                    colors.state_layer_unchecked,
                )
            }
        } else if self.checked {
            (
                colors.track_checked,
                colors.thumb_checked,
                colors.icon_checked,
                colors.state_layer_checked,
            )
        } else {
            (
                colors.track_unchecked,
                colors.thumb_unchecked,
                colors.icon_unchecked,
                colors.state_layer_unchecked,
            )
        };

        let state_layer_opacity = if self.is_pressed && self.enabled {
            0.1f32
        } else if self.is_hovered && self.enabled {
            0.08f32
        } else {
            0.0f32
        };

        (track_color, thumb_color, icon_color, state_layer_color, state_layer_opacity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_match_m3_tokens() {
        let m = SwitchMetrics::default();
        assert_eq!(m.track_width, 54.0);
        assert_eq!(m.track_height, 32.0);
        assert_eq!(m.thumb_height, 28.0);
        assert_eq!(m.thumb_width_idle, 28.0);
        assert_eq!(m.thumb_width_pressed, 34.0);
        assert_eq!(m.icon_size, 16.0);
        assert_eq!(m.icon_stroke_width, 2.4);
    }

    #[test]
    fn thumb_geometry_unchecked_anchors_left() {
        let m = SwitchMetrics::default();
        // Idle unchecked: left is 2px, width is 28px
        let (left_idle, w_idle) = thumb_geometry(&m, 0.0, 0.0);
        assert_eq!(left_idle, 2.0);
        assert_eq!(w_idle, 28.0);

        // Pressed unchecked: left stays 2px, width expands to 34px (expands right)
        let (left_press, w_press) = thumb_geometry(&m, 0.0, 1.0);
        assert_eq!(left_press, 2.0);
        assert_eq!(w_press, 34.0);
    }

    #[test]
    fn thumb_geometry_checked_anchors_right() {
        let m = SwitchMetrics::default();
        // Idle checked: right is 52px (margin 2px), width is 28px, left is 24px
        let (left_idle, w_idle) = thumb_geometry(&m, 1.0, 0.0);
        assert_eq!(left_idle, 24.0);
        assert_eq!(w_idle, 28.0);
        assert_eq!(left_idle + w_idle, 52.0);

        // Pressed checked: right stays 52px, width expands to 34px, left moves to 18px (expands left)
        let (left_press, w_press) = thumb_geometry(&m, 1.0, 1.0);
        assert_eq!(left_press, 18.0);
        assert_eq!(w_press, 34.0);
        assert_eq!(left_press + w_press, 52.0);
    }

    #[test]
    fn icon_segments_match_coordinates() {
        // Idle checked checkmark
        let (s1, s2) = icon_segments(true, false);
        assert_eq!(s1, IconSegment::new(2.4, 8.0, 6.4, 11.2));
        assert_eq!(s2, IconSegment::new(6.4, 11.2, 13.6, 3.2));

        // Idle unchecked cross
        let (s1, s2) = icon_segments(false, false);
        assert_eq!(s1, IconSegment::new(2.4, 2.4, 13.6, 13.6));
        assert_eq!(s2, IconSegment::new(2.4, 13.6, 13.6, 2.4));

        // Pressed collapses to horizontal dash at y=8.0
        let (s1, s2) = icon_segments(true, true);
        assert_eq!(s1.start_y, 8.0);
        assert_eq!(s1.end_y, 8.0);
        assert_eq!(s2.start_y, 8.0);
        assert_eq!(s2.end_y, 8.0);
    }
}
