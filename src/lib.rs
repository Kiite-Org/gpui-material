//! gpui-m3e: Material 3 Expressive for GPUI.
//!
//! Transcribed from `references/androidx/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Slider.kt`
//! and `tokens/SliderTokens.kt`, Apache-2.0, (c) Google. This crate reimplements
//! the behavior in idiomatic Rust, it does not copy Kotlin bodies.

pub mod components;
pub mod motion;
pub mod theme;

pub use components::slider::{SliderColors, SliderMetrics, SliderSpec};
pub use components::split_button::{
    SplitButtonColors, SplitButtonMetrics, SplitButtonModel, SplitButtonSize, SplitButtonVariant,
    SplitIconKind, draw_chevron_icon, draw_split_icon,
};
pub use components::switch::{
    IconSegment, SwitchColors, SwitchMetrics, SwitchModel, animated_icon_segments,
    draw_switch_icon, icon_segments, thumb_geometry,
};
pub use motion::{SpringParams, SpringState, spring_step};
pub use theme::{M3Palette, dark_scheme, light_scheme};

pub mod slider {
    pub use crate::components::slider::*;
    pub use crate::motion::*;
}
pub mod split_button {
    pub use crate::components::split_button::*;
}
pub mod switch {
    pub use crate::components::switch::*;
}
