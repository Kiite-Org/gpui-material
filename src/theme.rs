//! Material 3 Expressive color tokens and standard light and dark palettes.
//!
//! Values below define baseline Material Design color roles in sRGB.

use gpui::Rgba;

/// Material 3 Expressive color scheme in GPUI rgba.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct M3Palette {
    /// High-emphasis accent color for prominent interactive components and active tracks.
    pub primary: Rgba,
    /// High-contrast content color drawn on top of [`Self::primary`].
    pub on_primary: Rgba,
    /// Prominent container background with lower visual weight than primary.
    pub primary_container: Rgba,
    /// Medium-emphasis container background for tonal buttons and inactive tracks.
    pub secondary_container: Rgba,
    /// Contrast content color drawn on top of [`Self::secondary_container`].
    pub on_secondary_container: Rgba,
    /// Base background color for screens, dialogs, and window surfaces.
    pub surface: Rgba,
    /// High-emphasis text and icon foreground on [`Self::surface`].
    pub on_surface: Rgba,
    /// Medium-emphasis secondary text, helper labels, and subtle icons on surface.
    pub on_surface_variant: Rgba,
    /// Inverted background for floating popovers, tooltips, and snackbars.
    pub inverse_surface: Rgba,
    /// High-contrast text on top of [`Self::inverse_surface`].
    pub inverse_on_surface: Rgba,
    /// Distinct border and outline color for medium-low emphasis components.
    pub outline: Rgba,
    /// Subtle decorative divider and separator line color.
    pub outline_variant: Rgba,
    /// Highest-elevation container background for switches and elevated surfaces.
    pub surface_container_highest: Rgba,
}

#[inline]
fn srgb(r: u8, g: u8, b: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

/// Material 3 Expressive light scheme tailored for Rust.
///
/// Inspired by the official Rust geometric banner: vibrant flaming rust
/// primary (`#E85B2D`), warm golden apricot container (`#FECA91`), soft apricot
/// secondary (`#FFE2C7`), and warm sunlight cream surface (`#FFFAF5`).
#[must_use]
pub fn light_scheme() -> M3Palette {
    M3Palette {
        primary: srgb(232, 91, 45),
        on_primary: srgb(255, 255, 255),
        primary_container: srgb(254, 202, 145),
        secondary_container: srgb(255, 226, 199),
        on_secondary_container: srgb(46, 21, 10),
        surface: srgb(255, 250, 245),
        on_surface: srgb(35, 26, 22),
        on_surface_variant: srgb(87, 70, 63),
        inverse_surface: srgb(52, 42, 38),
        inverse_on_surface: srgb(254, 243, 237),
        outline: srgb(154, 126, 114),
        outline_variant: srgb(226, 203, 192),
        surface_container_highest: srgb(245, 225, 212),
    }
}

/// Material 3 Expressive dark scheme tailored for Rust.
///
/// Features radiant molten coral-rust primary (`#FA8559`), deep dark roast
/// mahogany on-primary (`#541702`), burning copper container (`#99310D`),
/// warm cedar secondary container (`#4E352C`), and warm obsidian surface (`#16110F`).
#[must_use]
pub fn dark_scheme() -> M3Palette {
    M3Palette {
        primary: srgb(250, 133, 89),
        on_primary: srgb(84, 23, 2),
        primary_container: srgb(153, 49, 13),
        secondary_container: srgb(78, 53, 44),
        on_secondary_container: srgb(254, 202, 145),
        surface: srgb(22, 17, 15),
        on_surface: srgb(245, 225, 212),
        on_surface_variant: srgb(226, 203, 192),
        inverse_surface: srgb(245, 225, 212),
        inverse_on_surface: srgb(52, 42, 38),
        outline: srgb(168, 136, 122),
        outline_variant: srgb(78, 53, 44),
        surface_container_highest: srgb(52, 40, 35),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_primary_matches_rust_m3_primary40() {
        let p = light_scheme();
        let expected = srgb(232, 91, 45);
        assert_eq!(p.primary, expected);
    }

    #[test]
    fn dark_primary_matches_rust_m3_primary80() {
        let p = dark_scheme();
        let expected = srgb(250, 133, 89);
        assert_eq!(p.primary, expected);
    }
}
