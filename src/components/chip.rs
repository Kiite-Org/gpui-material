//! Material 3 Expressive Chip component.
//!
//! Provides Assist, Filter, Input, and Suggestion chips adhering to
//! Material 3 design specifications with state layers and leading/trailing icons.

use gpui::{
    Div, ElementId, InteractiveElement, ParentElement, Rgba, SharedString, Stateful, Styled,
    div, px,
};

use crate::components::icon::M3Icon;
use crate::theme::M3Palette;

/// Visual type of the chip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChipVariant {
    /// Assist chips guide users or trigger contextual actions.
    #[default]
    Assist,
    /// Filter chips allow multi-select filtering with an active/selected state.
    Filter,
    /// Input chips represent information entered by the user, usually with a trailing remove icon.
    Input,
    /// Suggestion chips provide dynamically offered user prompts.
    Suggestion,
}

/// Elevation style for chips.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChipElevation {
    /// Flat style with a 1dp outline border (elevation 0).
    #[default]
    Flat,
    /// Elevated style with surface elevation and shadow (elevation 1).
    Elevated,
}

/// Material 3 Expressive chip model.
#[derive(Clone, Debug, PartialEq)]
pub struct ChipModel {
    /// Unique element identifier for interactivity.
    pub id: ElementId,
    /// Text label displayed in the chip.
    pub label: SharedString,
    /// Visual chip variant (Assist, Filter, Input, Suggestion).
    pub variant: ChipVariant,
    /// Elevation styling (Flat or Elevated).
    pub elevation: ChipElevation,
    /// Whether the chip is selected (primarily for Filter and Input chips).
    pub selected: bool,
    /// Whether the chip is interactive.
    pub enabled: bool,
    /// Whether the chip has pill corners (16dp radius) vs standard M3 (8dp radius).
    pub pill_shape: bool,
    /// Optional leading icon.
    pub leading_icon: Option<M3Icon>,
    /// Optional trailing icon.
    pub trailing_icon: Option<M3Icon>,
    /// Optional custom container background color override.
    pub custom_container: Option<Rgba>,
    /// Optional custom text/icon color override.
    pub custom_content: Option<Rgba>,
}

impl ChipModel {
    /// Creates a new chip with the given element ID and label.
    #[must_use]
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ChipVariant::default(),
            elevation: ChipElevation::default(),
            selected: false,
            enabled: true,
            pill_shape: false,
            leading_icon: None,
            trailing_icon: None,
            custom_container: None,
            custom_content: None,
        }
    }

    /// Sets the chip variant.
    #[must_use]
    pub fn variant(mut self, variant: ChipVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets whether the chip is elevated.
    #[must_use]
    pub fn elevation(mut self, elevation: ChipElevation) -> Self {
        self.elevation = elevation;
        self
    }

    /// Sets whether the chip is selected.
    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets whether the chip is enabled.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets whether the chip uses a rounded pill shape (16dp) rather than standard 8dp corners.
    #[must_use]
    pub fn pill(mut self, pill: bool) -> Self {
        self.pill_shape = pill;
        self
    }

    /// Sets an optional leading icon.
    #[must_use]
    pub fn leading_icon(mut self, icon: M3Icon) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Sets an optional trailing icon.
    #[must_use]
    pub fn trailing_icon(mut self, icon: M3Icon) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// Overrides container background color.
    #[must_use]
    pub fn container_color(mut self, color: Rgba) -> Self {
        self.custom_container = Some(color);
        self
    }

    /// Overrides text and icon color.
    #[must_use]
    pub fn content_color(mut self, color: Rgba) -> Self {
        self.custom_content = Some(color);
        self
    }

    /// Resolves container color against current theme palette.
    #[must_use]
    pub fn resolve_container_color(&self, palette: &M3Palette) -> Rgba {
        if let Some(c) = self.custom_container {
            return c;
        }
        if self.selected {
            palette.secondary_container
        } else {
            match self.elevation {
                ChipElevation::Flat => palette.surface,
                ChipElevation::Elevated => palette.surface_container_low,
            }
        }
    }

    /// Resolves content (text/icon) color against current theme palette.
    #[must_use]
    pub fn resolve_content_color(&self, palette: &M3Palette) -> Rgba {
        if let Some(c) = self.custom_content {
            return c;
        }
        if self.selected {
            palette.on_secondary_container
        } else {
            palette.on_surface_variant
        }
    }

    /// Renders the chip element.
    #[must_use]
    pub fn render(&self, palette: &M3Palette) -> Stateful<Div> {
        let bg = self.resolve_container_color(palette);
        let fg = self.resolve_content_color(palette);
        let radius = if self.pill_shape { px(16.0) } else { px(8.0) };

        let mut el = div()
            .id(self.id.clone())
            .h(px(32.0))
            .rounded(radius)
            .bg(bg)
            .text_color(fg)
            .text_size(px(14.0))
            .font_weight(gpui::FontWeight::MEDIUM)
            .flex()
            .items_center()
            .gap(px(8.0));

        // Padding adjustments depending on presence of leading/trailing icons
        let pad_start = if self.leading_icon.is_some() || (self.variant == ChipVariant::Filter && self.selected) {
            px(8.0)
        } else {
            px(16.0)
        };
        let pad_end = if self.trailing_icon.is_some() {
            px(8.0)
        } else {
            px(16.0)
        };
        el = el.pl(pad_start).pr(pad_end);

        // Elevation or border stroke
        if !self.selected && self.elevation == ChipElevation::Flat {
            el = el.border_1().border_color(palette.outline_variant);
        } else if self.elevation == ChipElevation::Elevated {
            el = el.shadow_sm();
        }

        // Leading icon (or checkmark if selected filter chip without custom leading icon)
        let effective_leading = if self.variant == ChipVariant::Filter && self.selected && self.leading_icon.is_none() {
            Some(M3Icon::Check)
        } else {
            self.leading_icon
        };

        if let Some(icon) = effective_leading {
            el = el.child(icon.element(18.0, fg));
        }

        // Label
        el = el.child(self.label.clone());

        // Trailing icon (or close 'X' if input chip without custom trailing icon)
        let effective_trailing = if self.variant == ChipVariant::Input && self.trailing_icon.is_none() {
            Some(M3Icon::Close)
        } else {
            self.trailing_icon
        };

        if let Some(icon) = effective_trailing {
            el = el.child(icon.element(18.0, fg));
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::light_scheme;

    #[test]
    fn default_chip_creation() {
        let c = ChipModel::new("test_chip", "Filter Tag");
        assert_eq!(c.label.as_ref(), "Filter Tag");
        assert_eq!(c.variant, ChipVariant::Assist);
        assert!(!c.selected);
        assert_eq!(c.elevation, ChipElevation::Flat);
    }

    #[test]
    fn filter_chip_selection_colors() {
        let palette = light_scheme();
        let unselected = ChipModel::new("filter", "Active").variant(ChipVariant::Filter);
        assert_eq!(unselected.resolve_container_color(&palette), palette.surface);
        assert_eq!(unselected.resolve_content_color(&palette), palette.on_surface_variant);

        let selected = unselected.clone().selected(true);
        assert_eq!(selected.resolve_container_color(&palette), palette.secondary_container);
        assert_eq!(selected.resolve_content_color(&palette), palette.on_secondary_container);
    }
}
