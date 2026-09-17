//! Material 3 Divider component.
//!
//! Provides horizontal and vertical visual separators with customizable
//! thickness, insets, and color overrides adhering to Material 3 tokens.

use gpui::{Div, Rgba, Styled, div, px};

use crate::theme::M3Palette;

/// Orientation of the divider line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerOrientation {
    /// Horizontal separator spanning the container width.
    #[default]
    Horizontal,
    /// Vertical separator spanning the container height.
    Vertical,
}

/// Inset margins applied to the ends of the divider.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerInset {
    /// No inset, divider extends edge to edge.
    #[default]
    None,
    /// Inset from the start / leading edge.
    Start,
    /// Inset from the end / trailing edge.
    End,
    /// Inset equally from both edges.
    Both,
}

/// Material 3 Divider component model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DividerModel {
    /// Divider orientation (horizontal or vertical).
    pub orientation: DividerOrientation,
    /// Line thickness in pixels (default 1dp).
    pub thickness: f32,
    /// Edge insets.
    pub inset: DividerInset,
    /// Inset distance in pixels (default 16dp when inset is enabled).
    pub inset_amount: f32,
    /// Optional color override (defaults to `palette.outline_variant`).
    pub custom_color: Option<Rgba>,
}

impl Default for DividerModel {
    fn default() -> Self {
        Self {
            orientation: DividerOrientation::Horizontal,
            thickness: 1.0,
            inset: DividerInset::None,
            inset_amount: 16.0,
            custom_color: None,
        }
    }
}

impl DividerModel {
    /// Creates a default horizontal divider.
    #[must_use]
    pub fn horizontal() -> Self {
        Self::default()
    }

    /// Creates a vertical divider.
    #[must_use]
    pub fn vertical() -> Self {
        Self {
            orientation: DividerOrientation::Vertical,
            ..Self::default()
        }
    }

    /// Sets the divider orientation.
    #[must_use]
    pub fn orientation(mut self, orientation: DividerOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets the line thickness in pixels.
    #[must_use]
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Sets the divider inset type.
    #[must_use]
    pub fn inset(mut self, inset: DividerInset) -> Self {
        self.inset = inset;
        self
    }

    /// Sets the inset distance in pixels.
    #[must_use]
    pub fn inset_amount(mut self, amount: f32) -> Self {
        self.inset_amount = amount;
        self
    }

    /// Overrides the divider line color.
    #[must_use]
    pub fn color(mut self, color: Rgba) -> Self {
        self.custom_color = Some(color);
        self
    }

    /// Resolves the color of the divider.
    #[must_use]
    pub fn resolve_color(&self, palette: &M3Palette) -> Rgba {
        self.custom_color.unwrap_or(palette.outline_variant)
    }

    /// Renders the divider element.
    #[must_use]
    pub fn render(&self, palette: &M3Palette) -> Div {
        let color = self.resolve_color(palette);
        let t = px(self.thickness);
        let ins = px(self.inset_amount);

        let mut el = div().bg(color);

        match self.orientation {
            DividerOrientation::Horizontal => {
                el = el.w_full().h(t);
                match self.inset {
                    DividerInset::None => {}
                    DividerInset::Start => el = el.ml(ins),
                    DividerInset::End => el = el.mr(ins),
                    DividerInset::Both => el = el.mx(ins),
                }
            }
            DividerOrientation::Vertical => {
                el = el.h_full().w(t);
                match self.inset {
                    DividerInset::None => {}
                    DividerInset::Start => el = el.mt(ins),
                    DividerInset::End => el = el.mb(ins),
                    DividerInset::Both => el = el.my(ins),
                }
            }
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::light_scheme;

    #[test]
    fn horizontal_and_vertical_defaults() {
        let h = DividerModel::horizontal();
        assert_eq!(h.orientation, DividerOrientation::Horizontal);
        assert_eq!(h.thickness, 1.0);
        assert_eq!(h.inset, DividerInset::None);

        let v = DividerModel::vertical();
        assert_eq!(v.orientation, DividerOrientation::Vertical);
    }

    #[test]
    fn custom_color_resolution() {
        let palette = light_scheme();
        let d = DividerModel::horizontal();
        assert_eq!(d.resolve_color(&palette), palette.outline_variant);

        let d_custom = DividerModel::horizontal().color(palette.primary);
        assert_eq!(d_custom.resolve_color(&palette), palette.primary);
    }
}
