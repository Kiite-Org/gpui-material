//! Material 3 Badge component.
//!
//! Provides small dot indicators and large numeric badges adhering to
//! Material 3 specifications for notification counters and status indicators.

use gpui::{Div, ParentElement, Rgba, SharedString, Styled, div, px};

use crate::theme::M3Palette;

/// Size variant of the badge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeSize {
    /// 6dp dot badge indicating an unread state without counts.
    #[default]
    Small,
    /// 16dp pill badge displaying a numerical or textual count.
    Large,
}

/// Material 3 Badge component model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BadgeModel {
    /// Optional label displayed in the badge (e.g. "3", "99+").
    pub label: Option<SharedString>,
    /// Optional container background color override (defaults to `error`).
    pub container_color: Option<Rgba>,
    /// Optional label text color override (defaults to `on_error`).
    pub content_color: Option<Rgba>,
    /// Explicit size override (if None, derived from presence of label).
    pub size: Option<BadgeSize>,
}

impl BadgeModel {
    /// Creates a new small dot badge.
    #[must_use]
    pub fn dot() -> Self {
        Self {
            label: None,
            size: Some(BadgeSize::Small),
            ..Self::default()
        }
    }

    /// Creates a new numeric/text badge.
    #[must_use]
    pub fn count(count: usize) -> Self {
        let label = if count > 999 {
            SharedString::from("999+")
        } else {
            SharedString::from(count.to_string())
        };
        Self {
            label: Some(label),
            size: Some(BadgeSize::Large),
            ..Self::default()
        }
    }

    /// Creates a badge with arbitrary text label.
    #[must_use]
    pub fn text(text: impl Into<SharedString>) -> Self {
        Self {
            label: Some(text.into()),
            size: Some(BadgeSize::Large),
            ..Self::default()
        }
    }

    /// Overrides container background color.
    #[must_use]
    pub fn container_color(mut self, color: Rgba) -> Self {
        self.container_color = Some(color);
        self
    }

    /// Overrides label text color.
    #[must_use]
    pub fn content_color(mut self, color: Rgba) -> Self {
        self.content_color = Some(color);
        self
    }

    /// Resolves the effective size of the badge.
    #[must_use]
    pub fn resolve_size(&self) -> BadgeSize {
        self.size.unwrap_or_else(|| {
            if self.label.is_some() {
                BadgeSize::Large
            } else {
                BadgeSize::Small
            }
        })
    }

    /// Resolves container color against current theme palette.
    #[must_use]
    pub fn resolve_container_color(&self, palette: &M3Palette) -> Rgba {
        self.container_color.unwrap_or(palette.error)
    }

    /// Resolves content color against current theme palette.
    #[must_use]
    pub fn resolve_content_color(&self, palette: &M3Palette) -> Rgba {
        self.content_color.unwrap_or(palette.on_error)
    }

    /// Renders the badge element.
    #[must_use]
    pub fn render(&self, palette: &M3Palette) -> Div {
        let bg = self.resolve_container_color(palette);
        let fg = self.resolve_content_color(palette);
        let size = self.resolve_size();

        match size {
            BadgeSize::Small => div()
                .w(px(6.0))
                .h(px(6.0))
                .rounded(px(3.0))
                .bg(bg),
            BadgeSize::Large => {
                let mut el = div()
                    .h(px(16.0))
                    .min_w(px(16.0))
                    .px(px(4.0))
                    .rounded(px(8.0))
                    .bg(bg)
                    .text_color(fg)
                    .text_size(px(10.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .flex()
                    .items_center()
                    .justify_center();

                if let Some(ref text) = self.label {
                    el = el.child(text.clone());
                }
                el
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::light_scheme;

    #[test]
    fn dot_badge_resolves_small() {
        let b = BadgeModel::dot();
        assert_eq!(b.resolve_size(), BadgeSize::Small);
        assert!(b.label.is_none());
    }

    #[test]
    fn count_badge_resolves_large_and_caps() {
        let b = BadgeModel::count(5);
        assert_eq!(b.resolve_size(), BadgeSize::Large);
        assert_eq!(b.label.as_deref(), Some("5"));

        let b_high = BadgeModel::count(1500);
        assert_eq!(b_high.label.as_deref(), Some("999+"));
    }

    #[test]
    fn color_resolution() {
        let palette = light_scheme();
        let b = BadgeModel::dot();
        assert_eq!(b.resolve_container_color(&palette), palette.error);
        assert_eq!(b.resolve_content_color(&palette), palette.on_error);
    }
}
