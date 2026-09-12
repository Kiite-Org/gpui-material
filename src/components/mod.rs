//! Material 3 Expressive UI components.

pub mod slider;
pub mod split_button;
pub mod switch;

pub use slider::{SliderColors, SliderMetrics, SliderSpec};
pub use split_button::{
    SplitButtonColors, SplitButtonMetrics, SplitButtonModel, SplitButtonSize, SplitButtonVariant,
    SplitIconKind, draw_chevron_icon, draw_split_icon,
};
pub use switch::{
    IconSegment, SwitchColors, SwitchMetrics, SwitchModel, animated_icon_segments,
    draw_switch_icon, icon_segments, thumb_geometry,
};
