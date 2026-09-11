//! Material 3 Expressive UI components.

pub mod slider;
pub mod switch;

pub use slider::{SliderColors, SliderMetrics, SliderSpec};
pub use switch::{
    IconSegment, SwitchColors, SwitchMetrics, SwitchModel, animated_icon_segments,
    draw_switch_icon, icon_segments, thumb_geometry,
};
