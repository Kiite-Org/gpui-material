//! Material 3 Expressive Menus component.
//!
//! Implements Material 3 Expressive menus adhering to official AndroidX
//! specifications and visual reference designs. Supports grouped cards,
//! unified cards with dividers, two-line layout with supporting text,
//! keyboard shortcuts, submenu chevrons, badges, spring-animated open/close
//! transitions, spring state layers on items, and cascading popup submenus (Callout 6).
//!
//! ### Cascading Popup Submenus (Callout 6)
//!
//! Cascading submenus open from a parent menu item and display a secondary floating
//! menu card overlapping the triggering item with an elevated drop shadow (`shadow_xl`).
//!
//! - **Overlap**: The popup horizontally overlaps the parent menu (default 28px),
//!   preventing cursor gaps during diagonal movement.
//! - **Spring Physics**: Animated scale ($0.88 \to 1.0$), opacity ($0.0 \to 1.0$),
//!   and slide ($X$-offset) on open, with an unhurried, zero-snap spring fade-out on close.
//! - **Concentric Corner Fitting**: Items inside the submenu automatically fit
//!   concentrically within the container's rounded card.
//! - **Gliding Indicator**: Submenu items feature their own gliding hover and selection
//!   indicators with full spring physics.
//!
//! ```rust
//! use gpui_material::{MenuItem, MenuGroup, MenuModel};
//!
//! // 1. Build the nested submenu
//! let submenu = MenuModel::new([
//!     MenuGroup::new().items([
//!         MenuItem::new(201, "Sub Item 1"),
//!         MenuItem::new(202, "Sub Item 2"),
//!         MenuItem::new(203, "Sub Item 3"),
//!     ])
//! ]);
//!
//! // 2. Attach it to a parent item (automatically adds trailing chevron)
//! let parent_item = MenuItem::new(102, "Item 2").submenu(submenu);
//!
//! let menu = MenuModel::new([
//!     MenuGroup::new().items([
//!         MenuItem::new(101, "Item 1"),
//!         parent_item,
//!         MenuItem::new(103, "Item 3"),
//!     ])
//! ]);
//! ```

use gpui::{
    div, hsla, point, prelude::*, px, BoxShadow, IntoElement, ParentElement, Rgba, SharedString,
    Styled, Window,
};

use crate::components::icon::M3Icon;
use crate::motion::{spring_step, SpringParams, SpringState};
use crate::theme::M3Palette;

/// Trailing content type for a menu item.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuItemTrailing {
    /// No trailing accessory.
    None,
    /// Trailing vector icon (e.g. `Refresh`, `Settings`, `Help`, `MoreVert`).
    Icon(M3Icon),
    /// Keyboard shortcut string (e.g. `"⌘C"`).
    Shortcut(&'static str),
    /// Submenu navigation pointer chevron (`▸`).
    Chevron,
    /// Tag badge pill (e.g. `"New"`) with optional trailing arrow.
    Badge {
        /// Tag text.
        text: &'static str,
        /// Whether a small trailing chevron arrow is shown next to the badge.
        has_arrow: bool,
    },
}

/// A single interactive item inside a Material 3 Expressive Menu.
#[derive(Clone, Debug)]
pub struct MenuItem {
    /// Unique item identifier.
    pub id: usize,
    /// Primary label text.
    pub label: SharedString,
    /// Optional supporting text displayed below the label (two-line layout).
    pub supporting_text: Option<SharedString>,
    /// Optional leading vector icon.
    pub leading_icon: Option<M3Icon>,
    /// Optional trailing content.
    pub trailing: MenuItemTrailing,
    /// Whether the item is interactive.
    pub enabled: bool,
    /// Whether the item is actively selected (highlighted with pill background).
    pub selected: bool,
    /// Whether pointer is hovering over the item.
    pub is_hovered: bool,
    /// Whether pointer is pressing down on the item.
    pub is_pressed: bool,
    /// Animated spring state layer for hover and press overlay.
    pub state_layer: SpringState,
    /// Optional nested cascading submenu attached to this item.
    pub submenu: Option<Box<MenuModel>>,
}

impl MenuItem {
    /// Creates a new enabled menu item with the given id and label.
    #[must_use]
    pub fn new(id: usize, label: impl Into<SharedString>) -> Self {
        Self {
            id,
            label: label.into(),
            supporting_text: None,
            leading_icon: None,
            trailing: MenuItemTrailing::None,
            enabled: true,
            selected: false,
            is_hovered: false,
            is_pressed: false,
            state_layer: SpringState::default(),
            submenu: None,
        }
    }

    /// Sets supporting text for a two-line menu item.
    #[must_use]
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// Sets a leading M3 vector icon.
    #[must_use]
    pub fn leading_icon(mut self, icon: M3Icon) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Sets trailing content.
    #[must_use]
    pub fn trailing(mut self, trailing: MenuItemTrailing) -> Self {
        self.trailing = trailing;
        self
    }

    /// Sets a trailing vector icon (e.g. `Refresh`, `Settings`, `Help`, `MoreVert`).
    #[must_use]
    pub fn trailing_icon(mut self, icon: M3Icon) -> Self {
        self.trailing = MenuItemTrailing::Icon(icon);
        self
    }

    /// Sets a keyboard shortcut trailing label (e.g. `"⌘C"`).
    #[must_use]
    pub fn shortcut(mut self, shortcut: &'static str) -> Self {
        self.trailing = MenuItemTrailing::Shortcut(shortcut);
        self
    }

    /// Sets a trailing tag badge (e.g. `"New"`, with optional chevron arrow).
    #[must_use]
    pub fn badge(mut self, text: &'static str, has_arrow: bool) -> Self {
        self.trailing = MenuItemTrailing::Badge { text, has_arrow };
        self
    }

    /// Sets a trailing submenu chevron (`▸`).
    #[must_use]
    pub fn chevron(mut self) -> Self {
        self.trailing = MenuItemTrailing::Chevron;
        self
    }

    /// Attaches a nested cascading submenu to this menu item.
    ///
    /// If trailing content is currently [`MenuItemTrailing::None`], automatically assigns
    /// [`MenuItemTrailing::Chevron`] to render the standard submenu navigation pointer (`▸`).
    ///
    /// # Example
    /// ```
    /// use gpui_material::{MenuItem, MenuGroup, MenuModel};
    ///
    /// let sub = MenuModel::new([
    ///     MenuGroup::new().items([
    ///         MenuItem::new(201, "Sub Item 1"),
    ///         MenuItem::new(202, "Sub Item 2"),
    ///     ])
    /// ]);
    ///
    /// let item = MenuItem::new(102, "Item 2").submenu(sub);
    /// assert!(item.has_submenu());
    /// ```
    #[must_use]
    pub fn submenu(mut self, menu: MenuModel) -> Self {
        if self.trailing == MenuItemTrailing::None {
            self.trailing = MenuItemTrailing::Chevron;
        }
        self.submenu = Some(Box::new(menu));
        self
    }

    /// Returns `true` if this item has an attached cascading submenu.
    #[inline]
    #[must_use]
    pub fn has_submenu(&self) -> bool {
        self.submenu.is_some()
    }

    /// Sets selected state (rendered with rounded selection pill background).
    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets enabled state.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Target alpha for the state layer (press state only; hover is handled by the gliding indicator pill).
    #[must_use]
    pub fn target_state_layer_alpha(&self) -> f32 {
        if !self.enabled {
            0.0
        } else if self.is_pressed {
            0.12
        } else {
            0.0
        }
    }

    /// Step spring state layer animation by `dt` seconds. Returns true if unsettled.
    pub fn step(&mut self, dt: f32) -> bool {
        let target = self.target_state_layer_alpha();
        let spring = SpringParams::expressive();
        self.state_layer = spring_step(spring, self.state_layer, target, dt);
        !self.state_layer.is_settled(target, 0.002, 0.005)
    }
}

/// A logical group of menu items, optionally with a section header.
#[derive(Clone, Debug)]
pub struct MenuGroup {
    /// Optional header label at the top of the group (callout 9).
    pub header: Option<SharedString>,
    /// Items belonging to this group.
    pub items: Vec<MenuItem>,
    /// ID of the currently hovered item in this group, if any.
    pub hovered_item_id: Option<usize>,
    /// ID of the currently pressed item in this group, if any.
    pub pressed_item_id: Option<usize>,
    /// Spring-animated vertical position (in pixels) of the gliding hover indicator pill.
    pub indicator_y: SpringState,
    /// Spring-animated height (in pixels) of the gliding hover indicator pill.
    pub indicator_h: SpringState,
    /// Spring-animated opacity (0.0 to 1.0) of the gliding hover indicator pill.
    pub indicator_opacity: SpringState,
    /// Spring-animated scale for click / press tactile bounce (default 1.0).
    pub press_scale: SpringState,
    /// Target Y when hover indicator is exiting the group during continuous cross-group handoff.
    pub hover_exit_target_y: Option<f32>,
    /// Spring-animated vertical position (in pixels) of the gliding selected indicator pill.
    pub selected_indicator_y: SpringState,
    /// Spring-animated height (in pixels) of the gliding selected indicator pill.
    pub selected_indicator_h: SpringState,
    /// Spring-animated opacity (0.0 to 1.0) of the gliding selected indicator pill.
    pub selected_indicator_opacity: SpringState,
    /// Spring-animated scale for selected click / press tactile bounce (default 1.0).
    pub selected_press_scale: SpringState,
    /// Target Y when selected indicator is exiting the group during continuous cross-group handoff.
    pub selected_exit_target_y: Option<f32>,
}

impl MenuGroup {
    /// Creates a new empty menu group.
    #[must_use]
    pub fn new() -> Self {
        Self {
            header: None,
            items: Vec::new(),
            hovered_item_id: None,
            pressed_item_id: None,
            indicator_y: SpringState::default(),
            indicator_h: SpringState { pos: 44.0, vel: 0.0 },
            indicator_opacity: SpringState::default(),
            press_scale: SpringState { pos: 1.0, vel: 0.0 },
            hover_exit_target_y: None,
            selected_indicator_y: SpringState::default(),
            selected_indicator_h: SpringState { pos: 44.0, vel: 0.0 },
            selected_indicator_opacity: SpringState::default(),
            selected_press_scale: SpringState { pos: 1.0, vel: 0.0 },
            selected_exit_target_y: None,
        }
    }

    /// Sets a section header label for this group.
    #[must_use]
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Adds a single item to the group.
    #[must_use]
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Sets all items in this group.
    #[must_use]
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// Computes `(top_y, height)` for an item at `target_index` inside the items column.
    #[must_use]
    pub fn item_y_and_height(&self, target_index: usize, metrics: &MenuMetrics) -> (f32, f32) {
        let mut y = 0.0;
        let mut target_h = metrics.item_height_single;
        for (i, item) in self.items.iter().enumerate() {
            let h = if item.supporting_text.is_some() {
                metrics.item_height_double
            } else {
                metrics.item_height_single
            };
            if i == target_index {
                return (y, h);
            }
            y += h + metrics.item_gap;
            target_h = h;
        }
        (y, target_h)
    }

    /// Returns the target Y, height, opacity, and scale for this group's gliding hover indicator.
    #[must_use]
    pub fn indicator_targets(&self, metrics: &MenuMetrics) -> (f32, f32, f32, f32) {
        let target_opacity = if self.hovered_item_id.is_some() {
            1.0
        } else {
            0.0
        };
        let target_scale = if self.pressed_item_id.is_some() { 0.96 } else { 1.0 };

        if let Some(id) = self.hovered_item_id
            && let Some(idx) = self.items.iter().position(|it| it.id == id)
        {
            let (y, h) = self.item_y_and_height(idx, metrics);
            return (y, h, target_opacity, target_scale);
        }

        if let Some(exit_y) = self.hover_exit_target_y {
            return (exit_y, self.indicator_h.pos, 0.0, target_scale);
        }

        (self.indicator_y.pos, self.indicator_h.pos, target_opacity, target_scale)
    }

    /// Returns the target Y, height, opacity, and scale for this group's gliding selected indicator.
    #[must_use]
    pub fn selected_indicator_targets(&self, metrics: &MenuMetrics) -> (f32, f32, f32, f32) {
        let selected_idx = self.items.iter().position(|it| it.selected);
        let target_scale = if let Some(pressed_id) = self.pressed_item_id
            && let Some(sel_idx) = selected_idx
            && self.items[sel_idx].id == pressed_id
        {
            0.96
        } else {
            1.0
        };

        if let Some(idx) = selected_idx {
            let (y, h) = self.item_y_and_height(idx, metrics);
            (y, h, 1.0, target_scale)
        } else if let Some(exit_y) = self.selected_exit_target_y {
            (exit_y, self.selected_indicator_h.pos, 0.0, target_scale)
        } else {
            (self.selected_indicator_y.pos, self.selected_indicator_h.pos, 0.0, 1.0)
        }
    }

    /// Step spring animations for this group's gliding indicators (hover and selected) and its items.
    pub fn step(&mut self, metrics: &MenuMetrics, dt: f32) -> bool {
        let (hov_target_y, hov_target_h, hov_target_opacity, hov_target_scale) = self.indicator_targets(metrics);
        let (sel_target_y, sel_target_h, sel_target_opacity, sel_target_scale) = self.selected_indicator_targets(metrics);
        let glide_spring = SpringParams::smooth();
        let press_spring = SpringParams::expressive();

        // 1. Advance hover indicator
        self.indicator_y = spring_step(glide_spring, self.indicator_y, hov_target_y, dt);
        self.indicator_h = spring_step(glide_spring, self.indicator_h, hov_target_h, dt);
        self.indicator_opacity = spring_step(glide_spring, self.indicator_opacity, hov_target_opacity, dt);
        self.press_scale = spring_step(press_spring, self.press_scale, hov_target_scale, dt);

        // 2. Advance selected indicator
        self.selected_indicator_y = spring_step(glide_spring, self.selected_indicator_y, sel_target_y, dt);
        self.selected_indicator_h = spring_step(glide_spring, self.selected_indicator_h, sel_target_h, dt);
        self.selected_indicator_opacity = spring_step(glide_spring, self.selected_indicator_opacity, sel_target_opacity, dt);
        self.selected_press_scale = spring_step(press_spring, self.selected_press_scale, sel_target_scale, dt);

        if self.hovered_item_id.is_none() && self.indicator_opacity.pos < 0.01 {
            self.hover_exit_target_y = None;
        }
        if !self.items.iter().any(|it| it.selected) && self.selected_indicator_opacity.pos < 0.01 {
            self.selected_exit_target_y = None;
        }

        // Boundary clamping: eliminate overslide past group edges into thin air
        let max_y = self
            .items
            .len()
            .checked_sub(1)
            .map(|idx| self.item_y_and_height(idx, metrics).0)
            .unwrap_or(0.0);

        // Boundary clamping: eliminate overslide bounce past container edges into thin air.
        // We only clamp when moving towards the boundary to catch overshoot; we never clamp
        // an incoming slide that is entering smoothly from outside towards the target.
        if hov_target_y == 0.0 && self.indicator_y.vel < 0.0 && self.indicator_y.pos < 0.0 {
            self.indicator_y.pos = 0.0;
            self.indicator_y.vel = 0.0;
        } else if hov_target_y == max_y && self.indicator_y.vel > 0.0 && self.indicator_y.pos > max_y {
            self.indicator_y.pos = max_y;
            self.indicator_y.vel = 0.0;
        }

        if sel_target_y == 0.0 && self.selected_indicator_y.vel < 0.0 && self.selected_indicator_y.pos < 0.0 {
            self.selected_indicator_y.pos = 0.0;
            self.selected_indicator_y.vel = 0.0;
        } else if sel_target_y == max_y && self.selected_indicator_y.vel > 0.0 && self.selected_indicator_y.pos > max_y {
            self.selected_indicator_y.pos = max_y;
            self.selected_indicator_y.vel = 0.0;
        }

        let hover_animating = !self.indicator_y.is_settled(hov_target_y, 0.2, 0.5)
            || !self.indicator_h.is_settled(hov_target_h, 0.2, 0.5)
            || !self.indicator_opacity.is_settled(hov_target_opacity, 0.005, 0.01)
            || !self.press_scale.is_settled(hov_target_scale, 0.005, 0.01);

        let sel_animating = !self.selected_indicator_y.is_settled(sel_target_y, 0.2, 0.5)
            || !self.selected_indicator_h.is_settled(sel_target_h, 0.2, 0.5)
            || !self.selected_indicator_opacity.is_settled(sel_target_opacity, 0.005, 0.01)
            || !self.selected_press_scale.is_settled(sel_target_scale, 0.005, 0.01);

        let mut items_animating = false;
        for item in &mut self.items {
            if item.step(dt) {
                items_animating = true;
            }
        }

        hover_animating || sel_animating || items_animating
    }
}

impl Default for MenuGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Placement direction for cascading submenus relative to the parent menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SubmenuPlacement {
    /// Cascades to the right of the parent menu, overlapping its right edge (default).
    #[default]
    Right,
    /// Cascades to the left of the parent menu, overlapping its left edge.
    Left,
}

/// Container display style for Material 3 Expressive menus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuVariant {
    /// Separate stacked rounded cards with a vertical gap (callout 10).
    #[default]
    GroupedCards,
    /// Single unified rounded card with horizontal dividers between groups (callout 11).
    UnifiedCard,
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

/// Layout metrics matching Material 3 Expressive menu tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuMetrics {
    /// Outer container corner radius (24dp in M3 Expressive, callout 10/11).
    pub container_corner: f32,
    /// Inner container corner radius between grouped cards (8dp in M3 Expressive).
    pub inner_corner: f32,
    /// Selection highlight pill corner radius (12dp).
    pub item_corner: f32,
    /// Minimum height for single-line menu items (44dp).
    pub item_height_single: f32,
    /// Minimum height for two-line menu items with supporting text (60dp).
    pub item_height_double: f32,
    /// Container internal vertical padding.
    pub container_padding_y: f32,
    /// Container internal horizontal padding.
    pub container_padding_x: f32,
    /// Item horizontal padding.
    pub item_padding_x: f32,
    /// Item vertical padding.
    pub item_padding_y: f32,
    /// Gap between grouped cards in `GroupedCards` variant (callout 10).
    pub group_gap: f32,
    /// Icon size for leading and trailing icons (20dp).
    pub icon_size: f32,
    /// Space between leading icon and label.
    pub icon_label_space: f32,
    /// Typography font size for item label (14dp).
    pub label_font_size: f32,
    /// Typography font size for supporting text (12dp).
    pub supporting_font_size: f32,
    /// Typography font size for shortcut text (12dp).
    pub shortcut_font_size: f32,
    /// Typography font size for section header (12dp).
    pub header_font_size: f32,
    /// Minimum menu width.
    pub min_width: f32,
    /// Maximum menu width.
    pub max_width: f32,
    /// Vertical gap between items inside a group.
    pub item_gap: f32,
}

impl Default for MenuMetrics {
    fn default() -> Self {
        Self {
            container_corner: 24.0,
            inner_corner: 6.0,
            item_corner: 12.0,
            item_height_single: 44.0,
            item_height_double: 60.0,
            container_padding_y: 4.0,
            container_padding_x: 6.0,
            item_padding_x: 12.0,
            item_padding_y: 6.0,
            group_gap: 4.0,
            icon_size: 20.0,
            icon_label_space: 12.0,
            label_font_size: 14.0,
            supporting_font_size: 12.0,
            shortcut_font_size: 12.0,
            header_font_size: 12.0,
            min_width: 200.0,
            max_width: 280.0,
            item_gap: 2.0,
        }
    }
}

/// Color tokens for Material 3 Expressive menus.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuColors {
    /// Container background surface color.
    pub container: Rgba,
    /// Selected item pill background container color (`secondary_container`).
    pub selected_container: Rgba,
    /// Label text color.
    pub text: Rgba,
    /// Selected label text color.
    pub selected_text: Rgba,
    /// Supporting text color.
    pub supporting_text: Rgba,
    /// Leading icon color.
    pub leading_icon: Rgba,
    /// Selected leading icon color.
    pub selected_leading_icon: Rgba,
    /// Trailing text/icon color.
    pub trailing_content: Rgba,
    /// Selected trailing text/icon color.
    pub selected_trailing_content: Rgba,
    /// Section header text color.
    pub header_text: Rgba,
    /// Divider line color.
    pub divider: Rgba,
    /// Hover and press state layer tint color.
    pub state_layer: Rgba,
    /// Badge background container color.
    pub badge_container: Rgba,
    /// Badge text color.
    pub badge_text: Rgba,
}

impl MenuColors {
    /// Resolves colors for the current theme palette.
    #[must_use]
    pub fn for_palette(p: &M3Palette) -> Self {
        let is_dark = p.surface.r < 0.2;
        Self {
            container: if is_dark {
                srgb(30, 26, 36)
            } else {
                p.surface_container_highest
            },
            selected_container: if is_dark {
                srgb(78, 55, 68)
            } else {
                p.secondary_container
            },
            text: p.on_surface,
            selected_text: if is_dark {
                srgb(245, 227, 231)
            } else {
                p.on_secondary_container
            },
            supporting_text: p.on_surface_variant,
            leading_icon: p.on_surface_variant,
            selected_leading_icon: if is_dark {
                srgb(245, 227, 231)
            } else {
                p.on_secondary_container
            },
            trailing_content: p.on_surface_variant,
            selected_trailing_content: if is_dark {
                srgb(245, 227, 231)
            } else {
                p.on_secondary_container
            },
            header_text: p.on_surface_variant,
            divider: p.outline_variant,
            state_layer: p.on_surface,
            badge_container: p.primary_container,
            badge_text: p.on_primary,
        }
    }
}

/// Interactive model for a Material 3 Expressive Menu.
#[derive(Clone, Debug)]
pub struct MenuModel {
    /// Logical groups of menu items.
    pub groups: Vec<MenuGroup>,
    /// Container presentation style (GroupedCards or UnifiedCard).
    pub variant: MenuVariant,
    /// Whether the menu is currently visible/open.
    pub is_open: bool,
    /// ID of the currently selected menu item, if any.
    pub selected_id: Option<usize>,
    /// Spring animation state for open/close scaling and opacity.
    pub open_progress: SpringState,
    /// Optional custom layout metrics.
    pub custom_metrics: Option<MenuMetrics>,
    /// Optional custom color tokens.
    pub custom_colors: Option<MenuColors>,
    /// Optional custom spring physics.
    pub custom_spring: Option<SpringParams>,
    /// Whether gliding indicators smoothly link and hand off across group gaps (Item 3 -> Item 4).
    pub link_group_indicators: bool,
    /// Whether items adjacent to inner group card boundaries (e.g. at the gap in GroupedCards)
    /// should match the card's inner radius concentrically (`true`), or keep standard item corner rounding (`false`).
    ///
    /// Defaults to `true`.
    pub match_inner_group_card_corners: bool,
    /// Tracks which group was last hovered for continuous cross-group trajectory handoff.
    pub last_hovered_group_idx: Option<usize>,
    /// ID of the menu item whose cascading submenu is currently open / active.
    pub active_submenu_item_id: Option<usize>,
    /// ID of the submenu item during exit transition so smooth spring close animation completes without snapping.
    pub last_submenu_item_id: Option<usize>,
    /// Spring animation progress (0.0 to 1.0) for the cascading submenu popup.
    pub submenu_progress: SpringState,
    /// Horizontal overlap (in pixels) of the cascading submenu over the parent menu card.
    ///
    /// Matching reference design Callout 6, cascading submenus sit subtly overlapping the
    /// parent menu card right at its edge by 2 pixels. Defaults to `2.0` pixels.
    pub submenu_overlap: f32,
    /// Vertical offset (in pixels) of the cascading submenu relative to the triggering item.
    ///
    /// Defaults to `-4.0` pixels, matching reference design Callout 6 where Item 1 of the
    /// submenu aligns level with the triggering parent item.
    pub submenu_offset_y: f32,
    /// Cascading popup placement direction (defaults to `SubmenuPlacement::Right`).
    pub submenu_placement: SubmenuPlacement,
    /// Whether hovering over a parent item with an attached submenu automatically opens it.
    ///
    /// When `true` (default), hovering opens the submenu and hovering away closes it.
    /// When `false`, submenus are opened/toggled explicitly via click.
    pub open_submenu_on_hover: bool,
    /// Optional explicit menu container width for cascading position calculations.
    /// When `None`, defaults to `metrics.min_width`.
    pub menu_width: Option<f32>,
}

impl MenuModel {
    /// Creates a new menu model with the given groups.
    #[must_use]
    pub fn new(groups: impl IntoIterator<Item = MenuGroup>) -> Self {
        let mut groups_vec: Vec<MenuGroup> = groups.into_iter().collect();
        let metrics = MenuMetrics::default();
        let initial_selected = groups_vec
            .iter()
            .flat_map(|g| &g.items)
            .find(|it| it.selected)
            .map(|it| it.id);

        for group in &mut groups_vec {
            if let Some(idx) = group.items.iter().position(|it| it.selected) {
                let (y, h) = group.item_y_and_height(idx, &metrics);
                group.selected_indicator_y = SpringState { pos: y, vel: 0.0 };
                group.selected_indicator_h = SpringState { pos: h, vel: 0.0 };
                group.selected_indicator_opacity = SpringState { pos: 1.0, vel: 0.0 };
            }
        }

        Self {
            groups: groups_vec,
            variant: MenuVariant::GroupedCards,
            is_open: true,
            selected_id: initial_selected,
            open_progress: SpringState {
                pos: 1.0,
                vel: 0.0,
            },
            custom_metrics: None,
            custom_colors: None,
            custom_spring: None,
            link_group_indicators: true,
            match_inner_group_card_corners: true,
            last_hovered_group_idx: None,
            active_submenu_item_id: None,
            last_submenu_item_id: None,
            submenu_progress: SpringState::default(),
            submenu_overlap: 2.0,
            submenu_offset_y: -4.0,
            submenu_placement: SubmenuPlacement::Right,
            open_submenu_on_hover: true,
            menu_width: None,
        }
    }

    /// Sets the horizontal overlap (in pixels) of cascading submenus over the parent menu card.
    ///
    /// Defaults to `2.0` pixels, matching reference design Callout 6.
    #[must_use]
    pub fn submenu_overlap(mut self, overlap: f32) -> Self {
        self.submenu_overlap = overlap;
        self
    }

    /// Sets the vertical offset (in pixels) of cascading submenus relative to the triggering item's top.
    ///
    /// Defaults to `-4.0` pixels, matching reference design Callout 6 where Item 1 of the
    /// submenu aligns level with the triggering parent item.
    #[must_use]
    pub fn submenu_offset_y(mut self, offset: f32) -> Self {
        self.submenu_offset_y = offset;
        self
    }

    /// Returns the box shadow configuration for cascading submenus.
    ///
    /// Provides a subtle, directional elevation drop shadow that casts softly
    /// across the parent card boundary and around the floating container so the
    /// cascading popup is immediately recognizable against both light and dark backgrounds.
    #[must_use]
    pub fn submenu_box_shadow(placement: SubmenuPlacement) -> Vec<BoxShadow> {
        let offset_x = match placement {
            SubmenuPlacement::Right => px(-3.0),
            SubmenuPlacement::Left => px(3.0),
        };
        vec![
            // Directional elevation shadow casting across the overlapping boundary
            BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.38),
                offset: point(offset_x, px(4.0)),
                blur_radius: px(12.0),
                spread_radius: px(0.0),
                inset: false,
            },
            // Soft diffuse ambient shadow around the card perimeter
            BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.22),
                offset: point(px(0.0), px(2.0)),
                blur_radius: px(6.0),
                spread_radius: px(0.0),
                inset: false,
            },
        ]
    }

    /// Sets the cascading submenu placement direction (Right or Left).
    #[must_use]
    pub fn submenu_placement(mut self, placement: SubmenuPlacement) -> Self {
        self.submenu_placement = placement;
        self
    }

    /// Sets whether hovering an item with an attached submenu automatically opens it.
    ///
    /// When `true` (default), hovering opens the submenu and hovering away closes it.
    /// When `false`, submenus are opened/toggled explicitly via click.
    #[must_use]
    pub fn open_submenu_on_hover(mut self, open_on_hover: bool) -> Self {
        self.open_submenu_on_hover = open_on_hover;
        self
    }

    /// Sets an explicit menu container width used for cascading submenu offset calculations.
    ///
    /// When `None` (the default), `metrics.min_width` is used.
    #[must_use]
    pub fn menu_width(mut self, width: f32) -> Self {
        self.menu_width = Some(width);
        self
    }

    /// Returns the effective width of the menu for positioning cascading popups.
    #[inline]
    #[must_use]
    pub fn effective_width(&self, metrics: &MenuMetrics) -> f32 {
        self.menu_width.unwrap_or(metrics.min_width)
    }

    /// Opens the cascading submenu for `item_id` with smooth spring animation.
    pub fn open_submenu(&mut self, item_id: usize) {
        if self.has_submenu(item_id) {
            self.active_submenu_item_id = Some(item_id);
            self.last_submenu_item_id = Some(item_id);
        }
    }

    /// Closes any currently active cascading submenu with smooth spring animation.
    pub fn close_submenu(&mut self) {
        self.active_submenu_item_id = None;
    }

    /// Toggles the cascading submenu for `item_id`.
    pub fn toggle_submenu(&mut self, item_id: usize) {
        if self.active_submenu_item_id == Some(item_id) {
            self.close_submenu();
        } else {
            self.open_submenu(item_id);
        }
    }

    /// Returns `true` if the item with `item_id` has an attached cascading submenu.
    #[must_use]
    pub fn has_submenu(&self, item_id: usize) -> bool {
        self.groups
            .iter()
            .flat_map(|g| &g.items)
            .any(|it| it.id == item_id && it.has_submenu())
    }

    /// Returns a reference to the attached submenu for `item_id`, if any.
    #[must_use]
    pub fn submenu(&self, item_id: usize) -> Option<&MenuModel> {
        for group in &self.groups {
            for item in &group.items {
                if item.id == item_id {
                    return item.submenu.as_deref();
                }
            }
        }
        None
    }

    /// Returns a mutable reference to the attached submenu for `item_id`, if any.
    pub fn submenu_mut(&mut self, item_id: usize) -> Option<&mut MenuModel> {
        for group in &mut self.groups {
            for item in &mut group.items {
                if item.id == item_id {
                    return item.submenu.as_deref_mut();
                }
            }
        }
        None
    }

    /// Recursively finds a `MenuItem` by `item_id` in this menu or any attached submenus.
    #[must_use]
    pub fn find_item(&self, item_id: usize) -> Option<&MenuItem> {
        for group in &self.groups {
            for item in &group.items {
                if item.id == item_id {
                    return Some(item);
                }
                if let Some(ref sub) = item.submenu
                    && let Some(found) = sub.find_item(item_id)
                {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Returns `true` if this menu or any attached submenu contains the given `item_id`.
    #[must_use]
    pub fn contains_item(&self, item_id: usize) -> bool {
        self.find_item(item_id).is_some()
    }

    /// Computes the vertical offset `(top_y, height)` of the item with `target_id` relative to the menu root.
    #[must_use]
    pub fn item_offset_in_menu(&self, target_id: usize, metrics: &MenuMetrics) -> Option<(f32, f32)> {
        let mut current_y = 0.0;
        let total_groups = self.groups.len();

        for (g_idx, group) in self.groups.iter().enumerate() {
            let mut group_y = current_y;
            if self.variant == MenuVariant::GroupedCards || g_idx == 0 {
                group_y += metrics.container_padding_y;
            }

            if group.header.is_some() {
                group_y += metrics.header_font_size + 8.0;
            }

            for item in &group.items {
                let h = if item.supporting_text.is_some() {
                    metrics.item_height_double
                } else {
                    metrics.item_height_single
                };
                if item.id == target_id {
                    return Some((group_y, h));
                }
                group_y += h + metrics.item_gap;
            }

            let group_items_h: f32 = group
                .items
                .iter()
                .map(|it| {
                    if it.supporting_text.is_some() {
                        metrics.item_height_double
                    } else {
                        metrics.item_height_single
                    }
                })
                .sum();
            let gaps_h = (group.items.len().saturating_sub(1) as f32) * metrics.item_gap;
            let header_h = if group.header.is_some() {
                metrics.header_font_size + 8.0
            } else {
                0.0
            };

            match self.variant {
                MenuVariant::GroupedCards => {
                    let card_h = 2.0 * metrics.container_padding_y + header_h + group_items_h + gaps_h;
                    current_y += card_h + metrics.group_gap;
                }
                MenuVariant::UnifiedCard => {
                    current_y += header_h + group_items_h + gaps_h;
                    if g_idx < total_groups - 1 {
                        current_y += 9.0;
                    }
                }
            }
        }

        None
    }

    /// Sets whether gliding indicators link smoothly across group boundaries.
    ///
    /// When enabled (the default), moving between items across group gaps
    /// (for example, from the last item of Group 1 to the first item of Group 2)
    /// creates a continuous spatial handoff: the outgoing indicator glides smoothly
    /// into the gap while fading, and the incoming indicator glides in from the gap
    /// at matching velocity and opacity rather than a disconnected fade-in.
    #[must_use]
    pub fn link_group_indicators(mut self, link: bool) -> Self {
        self.link_group_indicators = link;
        self
    }

    /// Sets whether items adjacent to inner group card boundaries fit the card's inner radius concentrically.
    ///
    /// When `true` (the default), the bottom corners of the top group and top corners of the bottom
    /// group fit the card's inner radius (`(inner_corner - padding).max(2.0)`).
    /// When `false`, only the outermost top and bottom menu buttons match the card corners,
    /// while inner group boundary buttons keep standard `item_corner` (12px) rounding.
    #[must_use]
    pub fn match_inner_group_card_corners(mut self, enabled: bool) -> Self {
        self.match_inner_group_card_corners = enabled;
        self
    }

    /// Selects an item by its unique identifier, deselecting all other items.
    ///
    /// The selected indicator pill smoothly glides to the newly selected item.
    /// If the newly selected item is in a different group and `link_group_indicators`
    /// is enabled, the indicator performs a continuous spatial handoff across the group gap.
    pub fn select(&mut self, id: usize) {
        let metrics = self.custom_metrics.unwrap_or_default();
        let prev_selected_id = self.selected_id;
        self.selected_id = Some(id);

        let prev_group_idx = prev_selected_id.and_then(|prev_id| {
            self.groups
                .iter()
                .position(|g| g.items.iter().any(|it| it.id == prev_id))
        });
        let target_group_idx = self
            .groups
            .iter()
            .position(|g| g.items.iter().any(|it| it.id == id));

        if let (Some(prev_g_idx), Some(target_g_idx)) = (prev_group_idx, target_group_idx) {
            if self.link_group_indicators && prev_g_idx != target_g_idx {
                let target_item_idx = self.groups[target_g_idx]
                    .items
                    .iter()
                    .position(|it| it.id == id)
                    .unwrap_or(0);
                let (_target_y, target_h) = self.groups[target_g_idx].item_y_and_height(target_item_idx, &metrics);

                if target_g_idx > prev_g_idx {
                    // Moving downwards into target group:
                    // 1. Target group indicator enters smoothly from above the group (in the gap)
                    self.groups[target_g_idx].selected_indicator_y.pos = -44.0;
                    self.groups[target_g_idx].selected_indicator_y.vel = 180.0;
                    self.groups[target_g_idx].selected_indicator_h.pos = target_h;
                    self.groups[target_g_idx].selected_indicator_opacity.pos = 1.0;
                    self.groups[target_g_idx].selected_exit_target_y = None;

                    // 2. Previous group indicator glides down through its remaining items and exits off bottom
                    let prev_g = &mut self.groups[prev_g_idx];
                    let last_idx = prev_g.items.len().saturating_sub(1);
                    let (last_y, last_h) = prev_g.item_y_and_height(last_idx, &metrics);
                    let exit_y = last_y + last_h + 12.0;
                    prev_g.selected_exit_target_y = Some(exit_y);
                    prev_g.selected_indicator_y.vel = 180.0;
                } else {
                    // Moving upwards into target group:
                    // 1. Target group indicator enters smoothly from below the group (in the gap)
                    let target_last_idx = self.groups[target_g_idx].items.len().saturating_sub(1);
                    let (last_y, last_h) = self.groups[target_g_idx].item_y_and_height(target_last_idx, &metrics);
                    let entry_y = last_y + last_h + 12.0;
                    self.groups[target_g_idx].selected_indicator_y.pos = entry_y;
                    self.groups[target_g_idx].selected_indicator_y.vel = -180.0;
                    self.groups[target_g_idx].selected_indicator_h.pos = target_h;
                    self.groups[target_g_idx].selected_indicator_opacity.pos = 1.0;
                    self.groups[target_g_idx].selected_exit_target_y = None;

                    // 2. Previous group indicator glides up through its remaining items and exits off top
                    let prev_g = &mut self.groups[prev_g_idx];
                    prev_g.selected_exit_target_y = Some(-44.0);
                    prev_g.selected_indicator_y.vel = -180.0;
                }
            }
        } else if let Some(target_g_idx) = target_group_idx {
            let group = &mut self.groups[target_g_idx];
            if group.selected_indicator_opacity.pos <= 0.05 {
                let item_idx = group.items.iter().position(|it| it.id == id).unwrap_or(0);
                let (target_y, target_h) = group.item_y_and_height(item_idx, &metrics);
                group.selected_indicator_y.pos = target_y;
                group.selected_indicator_y.vel = 0.0;
                group.selected_indicator_h.pos = target_h;
                group.selected_indicator_opacity.pos = 0.0;
            }
        }

        for group in &mut self.groups {
            for item in &mut group.items {
                item.selected = item.id == id;
            }
        }
    }

    /// Sets or clears the selected item ID.
    pub fn select_optional(&mut self, id: Option<usize>) {
        if let Some(target_id) = id {
            self.select(target_id);
        } else {
            self.selected_id = None;
            for group in &mut self.groups {
                for item in &mut group.items {
                    item.selected = false;
                }
            }
        }
    }

    /// Returns whether the item with `id` is currently selected.
    #[must_use]
    pub fn is_selected(&self, id: usize) -> bool {
        self.selected_id == Some(id)
    }

    /// Returns a reference to the currently selected item, if any.
    #[must_use]
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.selected_id.and_then(|id| {
            for group in &self.groups {
                for item in &group.items {
                    if item.id == id {
                        return Some(item);
                    }
                }
            }
            None
        })
    }

    /// Sets the menu variant (GroupedCards or UnifiedCard).
    #[must_use]
    pub fn variant(mut self, variant: MenuVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets open or closed state.
    #[must_use]
    pub fn is_open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        if is_open {
            self.open_progress.pos = 1.0;
        } else {
            self.open_progress.pos = 0.0;
        }
        self
    }

    /// Sets custom layout metrics.
    #[must_use]
    pub fn metrics(mut self, metrics: MenuMetrics) -> Self {
        self.custom_metrics = Some(metrics);
        self
    }

    /// Sets custom color tokens.
    #[must_use]
    pub fn colors(mut self, colors: MenuColors) -> Self {
        self.custom_colors = Some(colors);
        self
    }

    /// Sets custom spring physics.
    #[must_use]
    pub fn spring(mut self, spring: SpringParams) -> Self {
        self.custom_spring = Some(spring);
        self
    }

    /// Toggles menu open/closed state.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Sets the hover state for the item with `item_id`.
    ///
    /// When entering an item from an unhovered state, snaps directly to the item and fades in.
    /// While hovering across items within a group, smoothly glides the indicator pill along the Y axis.
    /// When moving across group gaps with `link_group_indicators` enabled, executes continuous spatial handoff.
    pub fn hover_item(&mut self, item_id: usize, hovered: bool) {
        let metrics = self.custom_metrics.unwrap_or_default();
        let target_group_idx = self
            .groups
            .iter()
            .position(|g| g.items.iter().any(|it| it.id == item_id));

        if let Some(target_g_idx) = target_group_idx {
            if hovered {
                let prev_g_idx = self.last_hovered_group_idx;
                self.last_hovered_group_idx = Some(target_g_idx);

                let item_idx = self.groups[target_g_idx]
                    .items
                    .iter()
                    .position(|it| it.id == item_id)
                    .unwrap_or(0);
                let (target_y, target_h) = self.groups[target_g_idx].item_y_and_height(item_idx, &metrics);

                if self.link_group_indicators && let Some(prev_idx) = prev_g_idx && prev_idx != target_g_idx {
                    // Cross-group continuous transition across the gap (e.g. Item 3 <-> Item 4)
                    if target_g_idx > prev_idx {
                        // Moving downwards into target group: enter smoothly from above
                        self.groups[target_g_idx].indicator_y.pos = -44.0;
                        self.groups[target_g_idx].indicator_y.vel = 180.0;
                        self.groups[target_g_idx].indicator_h.pos = target_h;
                        self.groups[target_g_idx].indicator_opacity.pos = 1.0;
                        self.groups[target_g_idx].hover_exit_target_y = None;

                        // Previous group glides down towards the gap while fading out
                        let prev_g = &mut self.groups[prev_idx];
                        let last_idx = prev_g.items.len().saturating_sub(1);
                        let (last_y, last_h) = prev_g.item_y_and_height(last_idx, &metrics);
                        prev_g.hover_exit_target_y = Some(last_y + last_h + 12.0);
                        prev_g.indicator_y.vel = 180.0;
                    } else {
                        // Moving upwards into target group: enter smoothly from below
                        let target_last_idx = self.groups[target_g_idx].items.len().saturating_sub(1);
                        let (last_y, last_h) = self.groups[target_g_idx].item_y_and_height(target_last_idx, &metrics);
                        self.groups[target_g_idx].indicator_y.pos = last_y + last_h + 12.0;
                        self.groups[target_g_idx].indicator_y.vel = -180.0;
                        self.groups[target_g_idx].indicator_h.pos = target_h;
                        self.groups[target_g_idx].indicator_opacity.pos = 1.0;
                        self.groups[target_g_idx].hover_exit_target_y = None;

                        // Previous group glides up towards the gap while fading out
                        let prev_g = &mut self.groups[prev_idx];
                        prev_g.hover_exit_target_y = Some(-44.0);
                        prev_g.indicator_y.vel = -180.0;
                    }
                } else if self.groups[target_g_idx].indicator_opacity.pos <= 0.05 {
                    // Cold enter from outside the menu
                    self.groups[target_g_idx].indicator_y.pos = target_y;
                    self.groups[target_g_idx].indicator_y.vel = 0.0;
                    self.groups[target_g_idx].indicator_h.pos = target_h;
                    self.groups[target_g_idx].indicator_opacity.pos = 0.0;
                }

                // Set hovered_item_id on target group and clear from all other groups
                for (g_idx, group) in self.groups.iter_mut().enumerate() {
                    if g_idx == target_g_idx {
                        group.hovered_item_id = Some(item_id);
                        for it in &mut group.items {
                            it.is_hovered = it.id == item_id;
                        }
                    } else {
                        group.hovered_item_id = None;
                        for it in &mut group.items {
                            it.is_hovered = false;
                        }
                    }
                }
            } else if !self.link_group_indicators
                && let Some(group) = self.groups.get_mut(target_g_idx)
                && group.hovered_item_id == Some(item_id)
            {
                group.hovered_item_id = None;
                for it in &mut group.items {
                    if it.id == item_id {
                        it.is_hovered = false;
                    }
                }
            }
        }
    }

    /// Clears any active hover state across all menu groups.
    ///
    /// If an item currently has an active cascading submenu open, that item's hover highlight
    /// is preserved so the parent item remains visibly active while navigating inside the submenu
    /// (matching reference design Callout 6).
    pub fn clear_hover(&mut self) {
        self.last_hovered_group_idx = None;
        for group in &mut self.groups {
            if let Some(sub_id) = self.active_submenu_item_id
                && group.items.iter().any(|it| it.id == sub_id)
            {
                group.hovered_item_id = Some(sub_id);
                group.hover_exit_target_y = None;
                for it in &mut group.items {
                    it.is_hovered = it.id == sub_id;
                }
            } else {
                group.hovered_item_id = None;
                group.hover_exit_target_y = None;
                for it in &mut group.items {
                    it.is_hovered = false;
                }
            }
        }
    }

    /// Sets the press state for the item with `item_id`, triggering an expressive tactile click bounce.
    pub fn press_item(&mut self, item_id: usize, pressed: bool) {
        for group in &mut self.groups {
            let contains_item = group.items.iter().any(|it| it.id == item_id);
            if contains_item {
                if pressed {
                    group.pressed_item_id = Some(item_id);
                    for it in &mut group.items {
                        it.is_pressed = it.id == item_id;
                    }
                } else if group.pressed_item_id == Some(item_id) {
                    group.pressed_item_id = None;
                    for it in &mut group.items {
                        if it.id == item_id {
                            it.is_pressed = false;
                        }
                    }
                }
            }
        }
    }

    /// Advances spring physics by `dt` seconds for menu transition, gliding indicator pills,
    /// cascading popup submenus, and item state layers.
    /// Returns `true` if any spring is still animating and needs another frame.
    pub fn step(&mut self, dt: f32) -> bool {
        let target = if self.is_open { 1.0 } else { 0.0 };
        let spring = self.custom_spring.unwrap_or_else(SpringParams::expressive);
        self.open_progress = spring_step(spring, self.open_progress, target, dt);
        let menu_animating = !self.open_progress.is_settled(target, 0.002, 0.005);

        let metrics = self.custom_metrics.unwrap_or_default();
        let mut groups_animating = false;
        for group in &mut self.groups {
            if group.step(&metrics, dt) {
                groups_animating = true;
            }
        }

        // Advance cascading submenu popup spring
        let sub_target = if self.active_submenu_item_id.is_some() { 1.0 } else { 0.0 };
        self.submenu_progress = spring_step(spring, self.submenu_progress, sub_target, dt);
        let submenu_animating = !self.submenu_progress.is_settled(sub_target, 0.002, 0.005);

        // When closed and settled, clear last_submenu_item_id to complete unmounting
        if self.active_submenu_item_id.is_none()
            && self.submenu_progress.pos <= 0.002
            && self.submenu_progress.vel.abs() <= 0.005
        {
            self.last_submenu_item_id = None;
        }

        // Recursively step springs for all attached submenus
        let mut submenus_animating = false;
        for group in &mut self.groups {
            for item in &mut group.items {
                if let Some(ref mut sub) = item.submenu
                    && sub.step(dt)
                {
                    submenus_animating = true;
                }
            }
        }

        menu_animating || groups_animating || submenu_animating || submenus_animating
    }

    /// Advance spring animations, requesting an animation frame if unsettled.
    pub fn advance_springs(&mut self, window: &mut Window, dt: f32) {
        if self.step(dt) {
            window.request_animation_frame();
        }
    }

    /// Computes `(top_left, top_right, bottom_right, bottom_left)` corner radii for a group card.
    #[must_use]
    pub fn group_card_radii(ix: usize, total: usize, metrics: &MenuMetrics) -> (f32, f32, f32, f32) {
        if total <= 1 {
            (
                metrics.container_corner,
                metrics.container_corner,
                metrics.container_corner,
                metrics.container_corner,
            )
        } else if ix == 0 {
            (
                metrics.container_corner,
                metrics.container_corner,
                metrics.inner_corner,
                metrics.inner_corner,
            )
        } else if ix == total - 1 {
            (
                metrics.inner_corner,
                metrics.inner_corner,
                metrics.container_corner,
                metrics.container_corner,
            )
        } else {
            (
                metrics.inner_corner,
                metrics.inner_corner,
                metrics.inner_corner,
                metrics.inner_corner,
            )
        }
    }

    /// Computes `(top_left, top_right, bottom_right, bottom_left)` effective boundary radii for a group in a `UnifiedCard`.
    #[must_use]
    pub fn unified_group_card_radii(ix: usize, total: usize, metrics: &MenuMetrics) -> (f32, f32, f32, f32) {
        if total <= 1 {
            (
                metrics.container_corner,
                metrics.container_corner,
                metrics.container_corner,
                metrics.container_corner,
            )
        } else {
            let inner = metrics.item_corner + metrics.container_padding_y;
            let top = if ix == 0 {
                metrics.container_corner
            } else {
                inner
            };
            let bottom = if ix == total - 1 {
                metrics.container_corner
            } else {
                inner
            };
            (top, top, bottom, bottom)
        }
    }

    /// Computes individual corner radii `(tl, tr, br, bl)` for an item within a group
    /// so that outer items fit concentrically inside the enclosing card container.
    ///
    /// When `match_inner_corners` is `true`, items adjacent to inner group card boundaries
    /// (e.g. at the gap between cards in `GroupedCards`) also fit the card's inner radius.
    /// When `false`, only outer card boundaries are matched concentrically, while inner
    /// boundary items keep standard `item_corner` (12px) rounding.
    #[must_use]
    pub fn item_corner_radii(
        item_idx: usize,
        total_items: usize,
        card_radii: (f32, f32, f32, f32),
        metrics: &MenuMetrics,
        match_inner_corners: bool,
    ) -> (f32, f32, f32, f32) {
        if total_items == 0 {
            return (
                metrics.item_corner,
                metrics.item_corner,
                metrics.item_corner,
                metrics.item_corner,
            );
        }

        let p = metrics.container_padding_y;
        let is_top = item_idx == 0;
        let is_bottom = item_idx == total_items - 1;

        let concentric = |card_r: f32| -> f32 {
            if !match_inner_corners && card_r <= metrics.inner_corner + 0.1 {
                metrics.item_corner
            } else {
                (card_r - p).max(2.0)
            }
        };

        let tl = if is_top {
            concentric(card_radii.0)
        } else {
            metrics.item_corner
        };

        let tr = if is_top {
            concentric(card_radii.1)
        } else {
            metrics.item_corner
        };

        let br = if is_bottom {
            concentric(card_radii.2)
        } else {
            metrics.item_corner
        };

        let bl = if is_bottom {
            concentric(card_radii.3)
        } else {
            metrics.item_corner
        };

        (tl, tr, br, bl)
    }

    /// Interpolates corner radii `(tl, tr, br, bl)` for a gliding indicator at position `y`
    /// within a group, morphing seamlessly between top, middle, and bottom item shapes.
    #[must_use]
    pub fn interpolated_item_radii(
        y: f32,
        group: &MenuGroup,
        card_radii: (f32, f32, f32, f32),
        metrics: &MenuMetrics,
        match_inner_corners: bool,
    ) -> (f32, f32, f32, f32) {
        let total = group.items.len();
        if total == 0 {
            return (
                metrics.item_corner,
                metrics.item_corner,
                metrics.item_corner,
                metrics.item_corner,
            );
        }

        if total == 1 {
            return Self::item_corner_radii(0, 1, card_radii, metrics, match_inner_corners);
        }

        let first_radii = Self::item_corner_radii(0, total, card_radii, metrics, match_inner_corners);
        let last_radii = Self::item_corner_radii(total - 1, total, card_radii, metrics, match_inner_corners);

        let (y0, _) = group.item_y_and_height(0, metrics);
        let (y1, _) = group.item_y_and_height(1, metrics);
        let (y_last, _) = group.item_y_and_height(total - 1, metrics);
        let (y_prev, _) = group.item_y_and_height(total - 2, metrics);

        // Top morph factor: 0.0 at y0 (first item), 1.0 at y1 (second item)
        let top_dist = (y1 - y0).max(1.0);
        let t_top = ((y - y0) / top_dist).clamp(0.0, 1.0);

        // Bottom morph factor: 0.0 at y_prev (penultimate item), 1.0 at y_last (last item)
        let bottom_dist = (y_last - y_prev).max(1.0);
        let t_bottom = ((y - y_prev) / bottom_dist).clamp(0.0, 1.0);

        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

        let tl = lerp(first_radii.0, metrics.item_corner, t_top);
        let tr = lerp(first_radii.1, metrics.item_corner, t_top);
        let br = lerp(metrics.item_corner, last_radii.2, t_bottom);
        let bl = lerp(metrics.item_corner, last_radii.3, t_bottom);

        (tl, tr, br, bl)
    }

    /// Renders the menu as a GPUI element hierarchy according to variant, colors, and metrics.
    pub fn render(&self, pal: &M3Palette) -> impl IntoElement {
        let metrics = self.custom_metrics.unwrap_or_default();
        let colors = self.custom_colors.unwrap_or_else(|| MenuColors::for_palette(pal));

        let progress = self.open_progress.pos.clamp(0.0, 1.0);
        let opacity = progress;
        let menu_w = self.effective_width(&metrics);

        let root = div()
            .relative()
            .flex()
            .flex_col()
            .w(px(menu_w))
            .min_w(px(metrics.min_width))
            .max_w(px(metrics.max_width))
            .opacity(opacity)
            .when(!self.is_open && progress <= 0.001, |d| {
                d.invisible().size_0().overflow_hidden()
            });

        let mut container = match self.variant {
            MenuVariant::GroupedCards => {
                // Stacked separate cards with group_gap (callout 10)
                let mut container = root.gap(px(metrics.group_gap));
                let total = self.groups.len();
                for (ix, group) in self.groups.iter().enumerate() {
                    let card = self.render_group_card(group, ix, total, &metrics, &colors, pal);
                    container = container.child(card);
                }
                container
            }
            MenuVariant::UnifiedCard => {
                // Single unified card with dividers between groups (callout 11)
                let mut card = div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .bg(colors.container)
                    .rounded(px(metrics.container_corner))
                    .p(px(metrics.container_padding_y))
                    .shadow_md();

                let total = self.groups.len();
                for (ix, group) in self.groups.iter().enumerate() {
                    let card_radii = Self::unified_group_card_radii(ix, total, &metrics);
                    if ix > 0 {
                        card = card.child(
                            div()
                                .my(px(4.0))
                                .mx(px(6.0))
                                .h(px(1.0))
                                .bg(colors.divider),
                        );
                    }
                    card = card.child(self.render_group_contents(group, card_radii, &metrics, &colors, pal));
                }
                root.child(card)
            }
        };

        // Render cascading submenu popup if open or during spring exit transition
        let active_sub_id = self.active_submenu_item_id.or(self.last_submenu_item_id);
        let sub_p = self.submenu_progress.pos;
        if let Some(sub_item_id) = active_sub_id
            && sub_p > 0.001
            && let Some(sub_model) = self.submenu(sub_item_id)
        {
            let sub_metrics = sub_model.custom_metrics.unwrap_or(metrics);
            let sub_w = sub_model.effective_width(&sub_metrics);
            let (item_y, _) = self.item_offset_in_menu(sub_item_id, &metrics).unwrap_or((0.0, 44.0));
            let top_y = (item_y + self.submenu_offset_y).max(0.0);
            let sub_opacity = sub_p.clamp(0.0, 1.0);
            let slide_y = 8.0 * (1.0 - sub_p.clamp(0.0, 1.0));
            let popup_left = match self.submenu_placement {
                SubmenuPlacement::Right => menu_w - self.submenu_overlap,
                SubmenuPlacement::Left => -sub_w + self.submenu_overlap,
            };

            let sub_card = div()
                .id(("m3-cascading-submenu-static", sub_item_id))
                .absolute()
                .top(px(top_y + slide_y))
                .left(px(popup_left))
                .w(px(sub_w))
                .shadow(Self::submenu_box_shadow(self.submenu_placement))
                .rounded(px(sub_metrics.container_corner))
                .opacity(sub_opacity)
                .child(sub_model.render(pal));

            container = container.child(sub_card);
        }

        container
    }

    /// Renders the menu with built-in interactive spring animations:
    /// - Smooth gliding hover indicator pill sliding down the line between items
    /// - Tactile click micro-interaction with spring press compression and release bounce
    /// - Cascading popup submenu with overlapping positioning, elevation shadow, and spring physics
    /// - Automatic animation frame scheduling and view notification
    pub fn render_interactive<V: 'static>(
        &self,
        pal: &M3Palette,
        cx: &Context<V>,
        menu_accessor: impl Fn(&mut V) -> &mut MenuModel + 'static + Copy,
        on_click: impl Fn(&mut V, usize, &mut Context<V>) + 'static + Copy,
    ) -> impl IntoElement {
        let metrics = self.custom_metrics.unwrap_or_default();
        let colors = self.custom_colors.unwrap_or_else(|| MenuColors::for_palette(pal));

        let progress = self.open_progress.pos.clamp(0.0, 1.0);
        let opacity = progress;
        let menu_w = self.effective_width(&metrics);

        let root = div()
            .id("m3-menu-root-interactive")
            .relative()
            .flex()
            .flex_col()
            .w(px(menu_w))
            .min_w(px(metrics.min_width))
            .max_w(px(metrics.max_width))
            .opacity(opacity)
            .when(!self.is_open && progress <= 0.001, |d| {
                d.invisible().size_0().overflow_hidden()
            })
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                if !*hovered {
                    menu_accessor(this).clear_hover();
                    menu_accessor(this).close_submenu();
                    cx.notify();
                }
            }));

        let mut container = match self.variant {
            MenuVariant::GroupedCards => {
                let mut container = root.gap(px(metrics.group_gap));
                let total = self.groups.len();
                for (ix, group) in self.groups.iter().enumerate() {
                    let card_radii = Self::group_card_radii(ix, total, &metrics);
                    let card = div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .bg(colors.container)
                        .rounded_tl(px(card_radii.0))
                        .rounded_tr(px(card_radii.1))
                        .rounded_br(px(card_radii.2))
                        .rounded_bl(px(card_radii.3))
                        .p(px(metrics.container_padding_y))
                        .shadow_md()
                        .child(self.render_group_interactive(group, None, card_radii, &metrics, &colors, cx, menu_accessor, on_click));
                    container = container.child(card);
                }
                container
            }
            MenuVariant::UnifiedCard => {
                let mut card = div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .bg(colors.container)
                    .rounded(px(metrics.container_corner))
                    .p(px(metrics.container_padding_y))
                    .shadow_md();

                let total = self.groups.len();
                for (ix, group) in self.groups.iter().enumerate() {
                    let card_radii = Self::unified_group_card_radii(ix, total, &metrics);
                    if ix > 0 {
                        card = card.child(
                            div()
                                .w_full()
                                .my(px(4.0))
                                .mx(px(6.0))
                                .h(px(1.0))
                                .bg(colors.divider),
                        );
                    }
                    card = card.child(self.render_group_interactive(group, None, card_radii, &metrics, &colors, cx, menu_accessor, on_click));
                }
                root.child(card)
            }
        };

        // Render cascading submenu popup if open or during spring exit transition
        let active_sub_id = self.active_submenu_item_id.or(self.last_submenu_item_id);
        let sub_p = self.submenu_progress.pos;
        if let Some(sub_item_id) = active_sub_id
            && sub_p > 0.001
            && let Some(sub_model) = self.submenu(sub_item_id)
        {
            let sub_metrics = sub_model.custom_metrics.unwrap_or(metrics);
            let sub_colors = sub_model.custom_colors.unwrap_or(colors);
            let sub_w = sub_model.effective_width(&sub_metrics);

            let (item_y, _) = self.item_offset_in_menu(sub_item_id, &metrics).unwrap_or((0.0, 44.0));
            let top_y = (item_y + self.submenu_offset_y).max(0.0);
            let sub_opacity = sub_p.clamp(0.0, 1.0);
            let slide_y = 8.0 * (1.0 - sub_p.clamp(0.0, 1.0));
            let popup_left = match self.submenu_placement {
                SubmenuPlacement::Right => menu_w - self.submenu_overlap,
                SubmenuPlacement::Left => -sub_w + self.submenu_overlap,
            };

            let mut submenu_card = div()
                .id(("m3-cascading-submenu-popup", sub_item_id))
                .absolute()
                .top(px(top_y + slide_y))
                .left(px(popup_left))
                .w(px(sub_w))
                .min_w(px(sub_metrics.min_width))
                .max_w(px(sub_metrics.max_width))
                .bg(sub_colors.container)
                .rounded(px(sub_metrics.container_corner))
                .p(px(sub_metrics.container_padding_y))
                .shadow(Self::submenu_box_shadow(self.submenu_placement))
                .opacity(sub_opacity)
                .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                    if !*hovered
                        && let Some(sub) = menu_accessor(this).submenu_mut(sub_item_id)
                    {
                        sub.clear_hover();
                        cx.notify();
                    }
                }));

            let total_sub_groups = sub_model.groups.len();
            for (g_ix, sub_group) in sub_model.groups.iter().enumerate() {
                let card_radii = match sub_model.variant {
                    MenuVariant::GroupedCards => Self::group_card_radii(g_ix, total_sub_groups, &sub_metrics),
                    MenuVariant::UnifiedCard => Self::unified_group_card_radii(g_ix, total_sub_groups, &sub_metrics),
                };
                if sub_model.variant == MenuVariant::UnifiedCard && g_ix > 0 {
                    submenu_card = submenu_card.child(
                        div()
                            .w_full()
                            .my(px(4.0))
                            .mx(px(6.0))
                            .h(px(1.0))
                            .bg(sub_colors.divider),
                    );
                }
                let group_elem = self.render_group_interactive(
                    sub_group,
                    Some(sub_item_id),
                    card_radii,
                    &sub_metrics,
                    &sub_colors,
                    cx,
                    menu_accessor,
                    on_click,
                );
                submenu_card = submenu_card.child(group_elem);
            }

            container = container.child(submenu_card);
        }

        container
    }

    #[allow(clippy::too_many_arguments)]
    fn render_group_interactive<V: 'static>(
        &self,
        group: &MenuGroup,
        sub_item_id: Option<usize>,
        card_radii: (f32, f32, f32, f32),
        metrics: &MenuMetrics,
        colors: &MenuColors,
        cx: &Context<V>,
        menu_accessor: impl Fn(&mut V) -> &mut MenuModel + 'static + Copy,
        on_click: impl Fn(&mut V, usize, &mut Context<V>) + 'static + Copy,
    ) -> impl IntoElement {
        let mut col = div().w_full().flex().flex_col().gap_1();

        if let Some(ref header) = group.header {
            col = col.child(
                div()
                    .px(px(metrics.item_padding_x))
                    .py_1()
                    .text_size(px(metrics.header_font_size))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.header_text)
                    .child(header.clone()),
            );
        }

        let mut items_box = div().w_full().relative().flex().flex_col().gap(px(metrics.item_gap));

        // 1. Gliding selected indicator pill (accent/selected_container background)
        let sel_opacity = group.selected_indicator_opacity.pos.clamp(0.0, 1.0);
        if sel_opacity > 0.001 {
            let sel_y = group.selected_indicator_y.pos;
            let sel_h = group.selected_indicator_h.pos;
            let scale = group.selected_press_scale.pos.clamp(0.8, 1.2);
            let inset_y = (sel_h * (1.0 - scale)) * 0.5;
            let inset_x = 4.0 * (1.0 - scale);
            let (stl, str, sbr, sbl) = Self::interpolated_item_radii(sel_y, group, card_radii, metrics, self.match_inner_group_card_corners);

            let selected_indicator = div()
                .id(("m3-menu-selected-pill-interactive", group.items.first().map(|i| i.id).unwrap_or(0)))
                .absolute()
                .top(px(sel_y + inset_y))
                .left(px(inset_x))
                .right(px(inset_x))
                .h(px(sel_h * scale))
                .rounded_tl(px(stl))
                .rounded_tr(px(str))
                .rounded_br(px(sbr))
                .rounded_bl(px(sbl))
                .bg(colors.selected_container)
                .opacity(sel_opacity);

            items_box = items_box.child(selected_indicator);
        }

        // 2. Gliding hover indicator pill (state layer overlay)
        let pill_opacity = group.indicator_opacity.pos.clamp(0.0, 1.0);
        if pill_opacity > 0.001 {
            let pill_y = group.indicator_y.pos;
            let pill_h = group.indicator_h.pos;
            let scale = group.press_scale.pos.clamp(0.8, 1.2);
            let inset_y = (pill_h * (1.0 - scale)) * 0.5;
            let inset_x = 4.0 * (1.0 - scale);
            let (htl, htr, hbr, hbl) = Self::interpolated_item_radii(pill_y, group, card_radii, metrics, self.match_inner_group_card_corners);

            let indicator = div()
                .id(("m3-menu-gliding-pill-interactive", group.items.first().map(|i| i.id).unwrap_or(0)))
                .absolute()
                .top(px(pill_y + inset_y))
                .left(px(inset_x))
                .right(px(inset_x))
                .h(px(pill_h * scale))
                .rounded_tl(px(htl))
                .rounded_tr(px(htr))
                .rounded_br(px(hbr))
                .rounded_bl(px(hbl))
                .bg(colors.state_layer)
                .opacity(pill_opacity * 0.12);

            items_box = items_box.child(indicator);
        }

        let total_items = group.items.len();
        for (item_idx, item) in group.items.iter().enumerate() {
            let item_id = item.id;
            let item_radii = Self::item_corner_radii(item_idx, total_items, card_radii, metrics, self.match_inner_group_card_corners);
            let item_element = self.render_item_with_radii(item, metrics, colors, item_radii);
            let interactive_item = div()
                .id(("m3-menu-item-wrap", item_id))
                .w_full()
                .cursor_pointer()
                .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                    if *hovered {
                        if let Some(parent_id) = sub_item_id {
                            if let Some(sub) = menu_accessor(this).submenu_mut(parent_id) {
                                sub.hover_item(item_id, true);
                            }
                        } else {
                            menu_accessor(this).hover_item(item_id, true);
                            if menu_accessor(this).open_submenu_on_hover {
                                if menu_accessor(this).has_submenu(item_id) {
                                    menu_accessor(this).open_submenu(item_id);
                                } else {
                                    menu_accessor(this).close_submenu();
                                }
                            }
                        }
                        cx.notify();
                    } else if !menu_accessor(this).link_group_indicators {
                        if let Some(parent_id) = sub_item_id {
                            if let Some(sub) = menu_accessor(this).submenu_mut(parent_id) {
                                sub.hover_item(item_id, false);
                            }
                        } else {
                            menu_accessor(this).hover_item(item_id, false);
                        }
                        cx.notify();
                    }
                }))
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        if let Some(parent_id) = sub_item_id {
                            if let Some(sub) = menu_accessor(this).submenu_mut(parent_id) {
                                sub.press_item(item_id, true);
                            }
                        } else {
                            menu_accessor(this).press_item(item_id, true);
                        }
                        cx.notify();
                    }),
                )
                .on_mouse_up(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        if let Some(parent_id) = sub_item_id {
                            if let Some(sub) = menu_accessor(this).submenu_mut(parent_id) {
                                sub.press_item(item_id, false);
                            }
                        } else {
                            menu_accessor(this).press_item(item_id, false);
                            if menu_accessor(this).has_submenu(item_id) {
                                menu_accessor(this).toggle_submenu(item_id);
                            } else {
                                menu_accessor(this).close_submenu();
                            }
                        }
                        on_click(this, item_id, cx);
                        cx.notify();
                    }),
                )
                .on_mouse_up_out(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        if let Some(parent_id) = sub_item_id {
                            if let Some(sub) = menu_accessor(this).submenu_mut(parent_id) {
                                sub.press_item(item_id, false);
                            }
                        } else {
                            menu_accessor(this).press_item(item_id, false);
                        }
                        cx.notify();
                    }),
                )
                .child(item_element);

            items_box = items_box.child(interactive_item);
        }

        col.child(items_box)
    }

    /// Renders the menu with an interactive click callback attached to every item.
    pub fn render_with_handlers<V: 'static>(
        &self,
        pal: &M3Palette,
        cx: &Context<V>,
        on_click: impl Fn(&mut V, usize, &mut Context<V>) + 'static + Copy,
    ) -> impl IntoElement {
        let metrics = self.custom_metrics.unwrap_or_default();
        let colors = self.custom_colors.unwrap_or_else(|| MenuColors::for_palette(pal));

        let progress = self.open_progress.pos.clamp(0.0, 1.0);
        let opacity = progress;
        let menu_w = self.effective_width(&metrics);

        let root = div()
            .relative()
            .flex()
            .flex_col()
            .w(px(menu_w))
            .min_w(px(metrics.min_width))
            .max_w(px(metrics.max_width))
            .opacity(opacity)
            .when(!self.is_open && progress <= 0.001, |d| {
                d.invisible().size_0().overflow_hidden()
            });

        let mut container = match self.variant {
            MenuVariant::GroupedCards => {
                let mut container = root.gap(px(metrics.group_gap));
                let total = self.groups.len();
                for (ix, group) in self.groups.iter().enumerate() {
                    let card_radii = Self::group_card_radii(ix, total, &metrics);
                    let card = div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .bg(colors.container)
                        .rounded_tl(px(card_radii.0))
                        .rounded_tr(px(card_radii.1))
                        .rounded_br(px(card_radii.2))
                        .rounded_bl(px(card_radii.3))
                        .p(px(metrics.container_padding_y))
                        .shadow_md()
                        .child(self.render_group_with_handlers(group, card_radii, &metrics, &colors, cx, on_click));
                    container = container.child(card);
                }
                container
            }
            MenuVariant::UnifiedCard => {
                let mut card = div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .bg(colors.container)
                    .rounded(px(metrics.container_corner))
                    .p(px(metrics.container_padding_y))
                    .shadow_md();

                let total = self.groups.len();
                for (ix, group) in self.groups.iter().enumerate() {
                    let card_radii = Self::unified_group_card_radii(ix, total, &metrics);
                    if ix > 0 {
                        card = card.child(
                            div()
                                .w_full()
                                .my(px(4.0))
                                .mx(px(6.0))
                                .h(px(1.0))
                                .bg(colors.divider),
                        );
                    }
                    card = card.child(self.render_group_with_handlers(group, card_radii, &metrics, &colors, cx, on_click));
                }
                root.child(card)
            }
        };

        // Render cascading submenu popup if open or during spring exit transition
        let active_sub_id = self.active_submenu_item_id.or(self.last_submenu_item_id);
        let sub_p = self.submenu_progress.pos;
        if let Some(sub_item_id) = active_sub_id
            && sub_p > 0.001
            && let Some(sub_model) = self.submenu(sub_item_id)
        {
            let sub_metrics = sub_model.custom_metrics.unwrap_or(metrics);
            let sub_w = sub_model.effective_width(&sub_metrics);
            let (item_y, _) = self.item_offset_in_menu(sub_item_id, &metrics).unwrap_or((0.0, 44.0));
            let top_y = (item_y + self.submenu_offset_y).max(0.0);
            let sub_opacity = sub_p.clamp(0.0, 1.0);
            let slide_y = 8.0 * (1.0 - sub_p.clamp(0.0, 1.0));
            let popup_left = match self.submenu_placement {
                SubmenuPlacement::Right => menu_w - self.submenu_overlap,
                SubmenuPlacement::Left => -sub_w + self.submenu_overlap,
            };

            let sub_card = div()
                .id(("m3-cascading-submenu-handlers", sub_item_id))
                .absolute()
                .top(px(top_y + slide_y))
                .left(px(popup_left))
                .w(px(sub_w))
                .shadow(Self::submenu_box_shadow(self.submenu_placement))
                .rounded(px(sub_metrics.container_corner))
                .opacity(sub_opacity)
                .child(sub_model.render_with_handlers(pal, cx, on_click));

            container = container.child(sub_card);
        }

        container
    }

    fn render_group_with_handlers<V: 'static>(
        &self,
        group: &MenuGroup,
        card_radii: (f32, f32, f32, f32),
        metrics: &MenuMetrics,
        colors: &MenuColors,
        cx: &Context<V>,
        on_click: impl Fn(&mut V, usize, &mut Context<V>) + 'static + Copy,
    ) -> impl IntoElement {
        let mut col = div().w_full().flex().flex_col().gap_1();

        if let Some(ref header) = group.header {
            col = col.child(
                div()
                    .px(px(metrics.item_padding_x))
                    .py_1()
                    .text_size(px(metrics.header_font_size))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.header_text)
                    .child(header.clone()),
            );
        }

        let mut items_box = div().w_full().relative().flex().flex_col().gap(px(metrics.item_gap));

        // 1. Gliding selected indicator pill
        let sel_opacity = group.selected_indicator_opacity.pos.clamp(0.0, 1.0);
        if sel_opacity > 0.001 {
            let sel_y = group.selected_indicator_y.pos;
            let sel_h = group.selected_indicator_h.pos;
            let scale = group.selected_press_scale.pos.clamp(0.8, 1.2);
            let inset_y = (sel_h * (1.0 - scale)) * 0.5;
            let inset_x = 4.0 * (1.0 - scale);
            let (stl, str, sbr, sbl) = Self::interpolated_item_radii(sel_y, group, card_radii, metrics, self.match_inner_group_card_corners);

            let selected_indicator = div()
                .id(("m3-menu-selected-pill-handlers", group.items.first().map(|i| i.id).unwrap_or(0)))
                .absolute()
                .top(px(sel_y + inset_y))
                .left(px(inset_x))
                .right(px(inset_x))
                .h(px(sel_h * scale))
                .rounded_tl(px(stl))
                .rounded_tr(px(str))
                .rounded_br(px(sbr))
                .rounded_bl(px(sbl))
                .bg(colors.selected_container)
                .opacity(sel_opacity);

            items_box = items_box.child(selected_indicator);
        }

        // 2. Gliding hover indicator pill
        let pill_opacity = group.indicator_opacity.pos.clamp(0.0, 1.0);
        if pill_opacity > 0.001 {
            let pill_y = group.indicator_y.pos;
            let pill_h = group.indicator_h.pos;
            let scale = group.press_scale.pos.clamp(0.8, 1.2);
            let inset_y = (pill_h * (1.0 - scale)) * 0.5;
            let inset_x = 4.0 * (1.0 - scale);
            let (htl, htr, hbr, hbl) = Self::interpolated_item_radii(pill_y, group, card_radii, metrics, self.match_inner_group_card_corners);

            let indicator = div()
                .id(("m3-menu-gliding-pill-handlers", group.items.first().map(|i| i.id).unwrap_or(0)))
                .absolute()
                .top(px(pill_y + inset_y))
                .left(px(inset_x))
                .right(px(inset_x))
                .h(px(pill_h * scale))
                .rounded_tl(px(htl))
                .rounded_tr(px(htr))
                .rounded_br(px(hbr))
                .rounded_bl(px(hbl))
                .bg(colors.state_layer)
                .opacity(pill_opacity * 0.12);

            items_box = items_box.child(indicator);
        }

        let total_items = group.items.len();
        for (item_idx, item) in group.items.iter().enumerate() {
            let item_id = item.id;
            let item_radii = Self::item_corner_radii(item_idx, total_items, card_radii, metrics, self.match_inner_group_card_corners);
            let item_element = self.render_item_with_radii(item, metrics, colors, item_radii);
            let interactive_item = div()
                .id(("m3-menu-item-wrap", item_id))
                .w_full()
                .cursor_pointer()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        on_click(this, item_id, cx);
                    }),
                )
                .child(item_element);
            items_box = items_box.child(interactive_item);
        }

        col.child(items_box)
    }

    /// Renders a single group inside an isolated card (used for `GroupedCards` variant).
    fn render_group_card(
        &self,
        group: &MenuGroup,
        ix: usize,
        total: usize,
        metrics: &MenuMetrics,
        colors: &MenuColors,
        pal: &M3Palette,
    ) -> impl IntoElement {
        let (tl, tr, br, bl) = Self::group_card_radii(ix, total, metrics);
        div()
            .w_full()
            .flex()
            .flex_col()
            .bg(colors.container)
            .rounded_tl(px(tl))
            .rounded_tr(px(tr))
            .rounded_br(px(br))
            .rounded_bl(px(bl))
            .p(px(metrics.container_padding_y))
            .shadow_md()
            .child(self.render_group_contents(group, (tl, tr, br, bl), metrics, colors, pal))
    }

    /// Renders the header and item list for a group.
    fn render_group_contents(
        &self,
        group: &MenuGroup,
        card_radii: (f32, f32, f32, f32),
        metrics: &MenuMetrics,
        colors: &MenuColors,
        _pal: &M3Palette,
    ) -> impl IntoElement {
        let mut col = div().w_full().flex().flex_col().gap_1();

        // Optional section header (callout 9)
        if let Some(ref header) = group.header {
            col = col.child(
                div()
                    .px(px(metrics.item_padding_x))
                    .py_1()
                    .text_size(px(metrics.header_font_size))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.header_text)
                    .child(header.clone()),
            );
        }

        let mut items_box = div().w_full().relative().flex().flex_col().gap(px(metrics.item_gap));

        // 1. Gliding selected indicator pill
        let sel_opacity = group.selected_indicator_opacity.pos.clamp(0.0, 1.0);
        if sel_opacity > 0.001 {
            let sel_y = group.selected_indicator_y.pos;
            let sel_h = group.selected_indicator_h.pos;
            let scale = group.selected_press_scale.pos.clamp(0.8, 1.2);
            let inset_y = (sel_h * (1.0 - scale)) * 0.5;
            let inset_x = 4.0 * (1.0 - scale);
            let (stl, str, sbr, sbl) = Self::interpolated_item_radii(sel_y, group, card_radii, metrics, self.match_inner_group_card_corners);

            let selected_indicator = div()
                .id(("m3-menu-selected-pill", group.items.first().map(|i| i.id).unwrap_or(0)))
                .absolute()
                .top(px(sel_y + inset_y))
                .left(px(inset_x))
                .right(px(inset_x))
                .h(px(sel_h * scale))
                .rounded_tl(px(stl))
                .rounded_tr(px(str))
                .rounded_br(px(sbr))
                .rounded_bl(px(sbl))
                .bg(colors.selected_container)
                .opacity(sel_opacity);

            items_box = items_box.child(selected_indicator);
        }

        // 2. Gliding hover indicator pill (smoothly glides along the Y-axis between hovered items)
        let pill_opacity = group.indicator_opacity.pos.clamp(0.0, 1.0);
        if pill_opacity > 0.001 {
            let pill_y = group.indicator_y.pos;
            let pill_h = group.indicator_h.pos;
            let scale = group.press_scale.pos.clamp(0.8, 1.2);
            let inset_y = (pill_h * (1.0 - scale)) * 0.5;
            let inset_x = 4.0 * (1.0 - scale);
            let (htl, htr, hbr, hbl) = Self::interpolated_item_radii(pill_y, group, card_radii, metrics, self.match_inner_group_card_corners);

            let indicator = div()
                .id(("m3-menu-gliding-pill", group.items.first().map(|i| i.id).unwrap_or(0)))
                .absolute()
                .top(px(pill_y + inset_y))
                .left(px(inset_x))
                .right(px(inset_x))
                .h(px(pill_h * scale))
                .rounded_tl(px(htl))
                .rounded_tr(px(htr))
                .rounded_br(px(hbr))
                .rounded_bl(px(hbl))
                .bg(colors.state_layer)
                .opacity(pill_opacity * 0.12);

            items_box = items_box.child(indicator);
        }

        // Render each item
        let total_items = group.items.len();
        for (item_idx, item) in group.items.iter().enumerate() {
            let item_radii = Self::item_corner_radii(item_idx, total_items, card_radii, metrics, self.match_inner_group_card_corners);
            items_box = items_box.child(self.render_item_with_radii(item, metrics, colors, item_radii));
        }

        col.child(items_box)
    }

    /// Renders a single menu item with custom corner radii (top_left, top_right, bottom_right, bottom_left).
    pub fn render_item_with_radii(
        &self,
        item: &MenuItem,
        metrics: &MenuMetrics,
        colors: &MenuColors,
        radii: (f32, f32, f32, f32),
    ) -> impl IntoElement {
        let is_two_line = item.supporting_text.is_some();
        let min_h = if is_two_line {
            metrics.item_height_double
        } else {
            metrics.item_height_single
        };

        let (text_color, icon_color, trailing_color) = if item.selected {
            (
                colors.selected_text,
                colors.selected_leading_icon,
                colors.selected_trailing_content,
            )
        } else {
            (
                colors.text,
                colors.leading_icon,
                colors.trailing_content,
            )
        };

        let bg_color = Rgba { a: 0.0, ..colors.container };

        let mut row = div()
            .id(("m3-menu-item", item.id))
            .relative()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .min_h(px(min_h))
            .px(px(metrics.item_padding_x))
            .py(px(metrics.item_padding_y))
            .rounded_tl(px(radii.0))
            .rounded_tr(px(radii.1))
            .rounded_br(px(radii.2))
            .rounded_bl(px(radii.3))
            .bg(bg_color)
            .cursor_pointer();

        // State layer overlay (spring-animated hover / press)
        let layer_alpha = item.state_layer.pos.clamp(0.0, 1.0);
        if layer_alpha > 0.001 {
            row = row.child(
                div()
                    .id(("m3-menu-item-layer", item.id))
                    .absolute()
                    .inset_0()
                    .rounded_tl(px(radii.0))
                    .rounded_tr(px(radii.1))
                    .rounded_br(px(radii.2))
                    .rounded_bl(px(radii.3))
                    .bg(colors.state_layer)
                    .opacity(layer_alpha),
            );
        }

        // Left side: leading icon + label + optional supporting text
        let mut left = div()
            .relative()
            .flex()
            .items_center()
            .gap(px(metrics.icon_label_space));

        if let Some(icon) = item.leading_icon {
            left = left.child(icon.element(metrics.icon_size, icon_color));
        }

        if let Some(ref sup) = item.supporting_text {
            // Two-line layout (callout 8)
            left = left.child(
                div()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(metrics.label_font_size))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(text_color)
                            .child(item.label.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(metrics.supporting_font_size))
                            .text_color(colors.supporting_text)
                            .child(sup.clone()),
                    ),
            );
        } else {
            left = left.child(
                div()
                    .text_size(px(metrics.label_font_size))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(text_color)
                    .child(item.label.clone()),
            );
        }

        // Right side: trailing content
        let right = match &item.trailing {
            MenuItemTrailing::None => div(),
            MenuItemTrailing::Icon(icon) => {
                div().child(icon.element(metrics.icon_size, trailing_color))
            }
            MenuItemTrailing::Shortcut(shortcut) => div()
                .text_size(px(metrics.shortcut_font_size))
                .font_weight(gpui::FontWeight::NORMAL)
                .text_color(trailing_color)
                .child(*shortcut),
            MenuItemTrailing::Chevron => div().child(
                M3Icon::ChevronRight.element(16.0, trailing_color),
            ),
            MenuItemTrailing::Badge { text, has_arrow } => {
                let mut badge_box = div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_full()
                            .bg(colors.badge_container)
                            .text_size(px(10.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors.badge_text)
                            .child(*text),
                    );

                if *has_arrow {
                    badge_box = badge_box.child(
                        M3Icon::ChevronRight.element(14.0, trailing_color),
                    );
                }
                badge_box
            }
        };

        row.child(left).child(right)
    }

    /// Renders a single menu item with leading icon, label, supporting text, trailing content, and state layer.
    pub fn render_item(
        &self,
        item: &MenuItem,
        metrics: &MenuMetrics,
        colors: &MenuColors,
    ) -> impl IntoElement {
        let r = metrics.item_corner;
        self.render_item_with_radii(item, metrics, colors, (r, r, r, r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_item_creation_and_defaults() {
        let item = MenuItem::new(1, "Copy")
            .shortcut("⌘C")
            .leading_icon(M3Icon::ContentCopy);

        assert_eq!(item.id, 1);
        assert_eq!(item.label.as_ref(), "Copy");
        assert_eq!(item.trailing, MenuItemTrailing::Shortcut("⌘C"));
        assert_eq!(item.leading_icon, Some(M3Icon::ContentCopy));
        assert!(item.enabled);
        assert!(!item.selected);
    }

    #[test]
    fn menu_group_and_variant() {
        let group = MenuGroup::new()
            .header("Actions")
            .item(MenuItem::new(1, "Edit").selected(true))
            .item(MenuItem::new(2, "Delete"));

        assert_eq!(group.items.len(), 2);
        assert!(group.items[0].selected);
        assert_eq!(group.header.as_deref(), Some("Actions"));

        let mut menu = MenuModel::new(vec![group]).variant(MenuVariant::UnifiedCard);
        assert_eq!(menu.variant, MenuVariant::UnifiedCard);
        assert!(menu.is_open);

        menu.toggle();
        assert!(!menu.is_open);
    }

    #[test]
    fn menu_spring_step() {
        let mut item = MenuItem::new(1, "Test");
        item.is_pressed = true;
        let running = item.step(1.0 / 60.0);
        assert!(running);
        assert!(item.state_layer.pos > 0.0);
    }

    #[test]
    fn menu_metrics_connected_corners() {
        let metrics = MenuMetrics::default();
        assert_eq!(metrics.container_corner, 24.0);
        assert_eq!(metrics.inner_corner, 6.0);
        assert_eq!(metrics.group_gap, 4.0);

        // 2 groups: top card has 24 top, 6 bottom
        let (tl, tr, br, bl) = MenuModel::group_card_radii(0, 2, &metrics);
        assert_eq!((tl, tr, br, bl), (24.0, 24.0, 6.0, 6.0));

        // 2 groups: bottom card has 6 top, 24 bottom
        let (tl, tr, br, bl) = MenuModel::group_card_radii(1, 2, &metrics);
        assert_eq!((tl, tr, br, bl), (6.0, 6.0, 24.0, 24.0));

        // 1 group: all corners are container_corner (24.0)
        let (tl, tr, br, bl) = MenuModel::group_card_radii(0, 1, &metrics);
        assert_eq!((tl, tr, br, bl), (24.0, 24.0, 24.0, 24.0));
    }

    #[test]
    fn check_flex_col() {
        use gpui::Styled;
        let mut d = gpui::div().flex().flex_col();
        let style = d.style();
        assert!(style.display.is_some());
    }

    #[test]
    fn menu_gliding_indicator_advances_towards_target_item() {
        let group = MenuGroup::new()
            .item(MenuItem::new(1, "First"))
            .item(MenuItem::new(2, "Second"))
            .item(MenuItem::new(3, "Third"));

        let mut menu = MenuModel::new(vec![group]);
        menu.open_progress.pos = 1.0;

        // Hover second item (index 1)
        menu.hover_item(2, true);
        assert_eq!(menu.groups[0].hovered_item_id, Some(2));

        // Advance 30 frames of spring physics
        for _ in 0..30 {
            menu.step(1.0 / 60.0);
        }

        // Expected Y for index 1: 44.0 (item 1 height) + 2.0 (gap) = 46.0
        let target_y = 46.0;
        let diff = (menu.groups[0].indicator_y.pos - target_y).abs();
        assert!(diff < 2.0, "Expected indicator_y ~46.0, got {}", menu.groups[0].indicator_y.pos);
        assert!(menu.groups[0].indicator_opacity.pos > 0.8, "Expected opacity near 1.0");

        // Hover third item (index 2: 44 + 2 + 44 + 2 = 92.0)
        menu.hover_item(3, true);
        for _ in 0..30 {
            menu.step(1.0 / 60.0);
        }
        let diff3 = (menu.groups[0].indicator_y.pos - 92.0).abs();
        assert!(diff3 < 2.0, "Expected indicator_y ~92.0, got {}", menu.groups[0].indicator_y.pos);

        // Clear hover -> opacity fades out
        menu.clear_hover();
        for _ in 0..60 {
            menu.step(1.0 / 60.0);
        }
        assert!(menu.groups[0].indicator_opacity.pos < 0.05, "Expected opacity to fade out near 0");
    }

    #[test]
    fn menu_press_scale_bounce() {
        let group = MenuGroup::new()
            .item(MenuItem::new(1, "Item 1"));

        let mut menu = MenuModel::new(vec![group]);
        menu.open_progress.pos = 1.0;

        // Press item down -> scale targets 0.96
        menu.press_item(1, true);
        for _ in 0..15 {
            menu.step(1.0 / 60.0);
        }
        assert!(menu.groups[0].press_scale.pos < 0.99, "Expected press scale compression");

        // Release item -> scale bounces back to 1.0
        menu.press_item(1, false);
        for _ in 0..60 {
            menu.step(1.0 / 60.0);
        }
        assert!((menu.groups[0].press_scale.pos - 1.0).abs() < 0.01, "Expected scale to settle back at 1.0");
    }

    #[test]
    fn menu_settles_to_idle() {
        let group = MenuGroup::new().item(MenuItem::new(1, "Item 1"));
        let mut menu = MenuModel::new(vec![group]);
        menu.open_progress.pos = 1.0;

        // Step 120 frames without interaction
        let mut animating = true;
        for _ in 0..120 {
            animating = menu.step(1.0 / 60.0);
        }
        assert!(!animating, "Expected menu physics to settle and halt idle animation loop");
    }

    #[test]
    fn menu_gliding_selected_indicator_advances_on_selection() {
        let group = MenuGroup::new()
            .item(MenuItem::new(1, "First").selected(true))
            .item(MenuItem::new(2, "Second"))
            .item(MenuItem::new(3, "Third"));

        let mut menu = MenuModel::new(vec![group]);
        menu.open_progress.pos = 1.0;

        // Group 0 should have initialized selected indicator at y=0.0 with opacity 1.0
        assert_eq!(menu.groups[0].selected_indicator_y.pos, 0.0);
        assert_eq!(menu.groups[0].selected_indicator_opacity.pos, 1.0);

        // Select item 3 (expected target_y: 44 + 2 + 44 + 2 = 92.0)
        menu.select(3);
        assert!(menu.is_selected(3));
        assert!(!menu.is_selected(1));

        // Advance 30 frames
        for _ in 0..30 {
            menu.step(1.0 / 60.0);
        }

        let diff = (menu.groups[0].selected_indicator_y.pos - 92.0).abs();
        assert!(diff < 2.0, "Expected selected_indicator_y ~92.0, got {}", menu.groups[0].selected_indicator_y.pos);
        assert!(menu.groups[0].selected_indicator_opacity.pos > 0.9, "Expected opacity near 1.0");

        // Hover item 3 (the selected item): hover opacity remains active so indicator glides across it
        menu.hover_item(3, true);
        for _ in 0..30 {
            menu.step(1.0 / 60.0);
        }
        assert!(menu.groups[0].indicator_opacity.pos > 0.8, "Expected hover opacity to remain active over selected item");
    }

    #[test]
    fn menu_gliding_indicator_zero_overshoot_at_boundaries() {
        let group = MenuGroup::new()
            .item(MenuItem::new(1, "First"))
            .item(MenuItem::new(2, "Second"))
            .item(MenuItem::new(3, "Third"));

        let mut menu = MenuModel::new(vec![group]);
        menu.open_progress.pos = 1.0;

        // Move to item 3 (bottom, y = 92)
        menu.hover_item(3, true);
        for _ in 0..60 {
            menu.step(1.0 / 60.0);
        }

        // Now move up to item 1 (top, y = 0).
        // Check that indicator_y never goes below 0.0 (no overslide into thin air above)
        menu.hover_item(1, true);
        let mut min_y = 100.0;
        for _ in 0..60 {
            menu.step(1.0 / 60.0);
            if menu.groups[0].indicator_y.pos < min_y {
                min_y = menu.groups[0].indicator_y.pos;
            }
        }
        assert!(
            min_y >= 0.0,
            "Indicator Y must not overshoot below 0.0 into thin air (got {})",
            min_y
        );

        // Now move back down to item 3 (y = 92).
        // Check that indicator_y never exceeds 92.0 (no overslide into thin air below)
        menu.hover_item(3, true);
        let mut max_y: f32 = 0.0;
        for _ in 0..60 {
            menu.step(1.0 / 60.0);
            if menu.groups[0].indicator_y.pos > max_y {
                max_y = menu.groups[0].indicator_y.pos;
            }
        }
        assert!(
            max_y <= 92.05,
            "Indicator Y must not overshoot above 92.0 into thin air (got {})",
            max_y
        );
    }

    #[test]
    fn menu_cross_group_linking_continuous_spatial_handoff() {
        let g1 = MenuGroup::new()
            .item(MenuItem::new(101, "Item 1"))
            .item(MenuItem::new(102, "Item 2"))
            .item(MenuItem::new(103, "Item 3"));
        let g2 = MenuGroup::new()
            .item(MenuItem::new(104, "Item 4"))
            .item(MenuItem::new(105, "Item 5"));

        let mut menu = MenuModel::new(vec![g1, g2]).link_group_indicators(true);
        menu.open_progress.pos = 1.0;

        // Hover Item 3 in Group 0
        menu.hover_item(103, true);
        for _ in 0..30 {
            menu.step(1.0 / 60.0);
        }
        assert_eq!(menu.last_hovered_group_idx, Some(0));

        // Move down across gap to Item 4 in Group 1
        menu.hover_item(104, true);
        assert_eq!(menu.last_hovered_group_idx, Some(1));

        // Group 1's indicator should have started from above 0.0 (in the gap) with opacity 1.0
        assert!(
            menu.groups[1].indicator_y.pos < 0.0,
            "Incoming indicator should enter from above the group (got {})",
            menu.groups[1].indicator_y.pos
        );
        assert!(
            menu.groups[1].indicator_opacity.pos > 0.9,
            "Incoming indicator should have full continuous opacity"
        );

        // Advance 30 frames: Group 1 settles at Item 4 (y=0.0)
        for _ in 0..30 {
            menu.step(1.0 / 60.0);
        }
        assert!(
            menu.groups[1].indicator_y.pos.abs() < 2.0,
            "Expected indicator to settle at Item 4 y=0.0, got {}",
            menu.groups[1].indicator_y.pos
        );

        // Move back up across gap to Item 3 in Group 0
        menu.hover_item(103, true);
        assert_eq!(menu.last_hovered_group_idx, Some(0));
        // Group 0's indicator should have entered from below Item 3 (y > 92.0)
        assert!(
            menu.groups[0].indicator_y.pos > 92.0,
            "Incoming indicator should enter from below the group (got {})",
            menu.groups[0].indicator_y.pos
        );

        for _ in 0..30 {
            menu.step(1.0 / 60.0);
        }
        assert!(
            (menu.groups[0].indicator_y.pos - 92.0).abs() < 2.0,
            "Expected indicator to settle back at Item 3 y=92.0"
        );
    }

    #[test]
    fn menu_cross_group_selection_continuous_glide_between_distant_items() {
        let g1 = MenuGroup::new()
            .item(MenuItem::new(101, "Item 1"))
            .item(MenuItem::new(102, "Item 2"))
            .item(MenuItem::new(103, "Item 3"));
        let g2 = MenuGroup::new()
            .item(MenuItem::new(104, "Item 4"))
            .item(MenuItem::new(105, "Item 5").selected(true));

        let mut menu = MenuModel::new(vec![g1, g2]).link_group_indicators(true);
        menu.open_progress.pos = 1.0;
        menu.select(105);

        // Step 60 frames to settle at Item 5 (y = 46.0 in Group 1)
        for _ in 0..60 {
            menu.step(1.0 / 60.0);
        }
        assert!((menu.groups[1].selected_indicator_y.pos - 46.0).abs() < 1.0);

        // Click from Item 5 (in Group 1) to Item 1 (in Group 0) across the gap!
        menu.select(101);

        // Group 0 should enter smoothly from below Group 0 (entry_y = 148.0) with full opacity
        assert!(
            menu.groups[0].selected_indicator_y.pos > 92.0,
            "Group 0 selected indicator must enter from the gap below Item 3 (got {})",
            menu.groups[0].selected_indicator_y.pos
        );
        assert!(
            menu.groups[0].selected_indicator_opacity.pos > 0.9,
            "Group 0 selected indicator must have full continuous opacity"
        );

        // Group 1 should glide up towards the exit (-44.0) while fading out
        assert_eq!(menu.groups[1].selected_exit_target_y, Some(-44.0));
        assert!(menu.groups[1].selected_indicator_y.vel < -100.0);

        // Advance 30 frames: Group 0 glides all the way up to Item 1 (y = 0.0)
        for _ in 0..30 {
            menu.step(1.0 / 60.0);
        }
        assert!(
            menu.groups[0].selected_indicator_y.pos.abs() < 2.0,
            "Expected selected indicator to settle at Item 1 (y=0.0), got {}",
            menu.groups[0].selected_indicator_y.pos
        );
        assert!(menu.groups[1].selected_indicator_opacity.pos < 0.1);

        // Now click back from Item 1 down to Item 5
        menu.select(105);

        // Group 1 should enter from the top (gap, -44.0) with full opacity
        assert!(
            menu.groups[1].selected_indicator_y.pos < 0.0,
            "Group 1 selected indicator must enter from the gap above Item 4 (got {})",
            menu.groups[1].selected_indicator_y.pos
        );
        // Group 0 should exit downwards towards the gap (148.0)
        assert_eq!(menu.groups[0].selected_exit_target_y, Some(148.0));
        assert!(menu.groups[0].selected_indicator_y.vel > 100.0);

        for _ in 0..30 {
            menu.step(1.0 / 60.0);
        }
        assert!(
            (menu.groups[1].selected_indicator_y.pos - 46.0).abs() < 2.0,
            "Expected selected indicator to settle at Item 5 (y=46.0), got {}",
            menu.groups[1].selected_indicator_y.pos
        );
    }

    #[test]
    fn item_corner_radii_concentric_fitting() {
        let metrics = MenuMetrics::default();
        // card_radii: top=24, bottom=6
        let card_radii = (24.0, 24.0, 6.0, 6.0);

        // Group with 3 items (match_inner_corners = true)
        // Item 0 (top): top corners are (24 - 4) = 20.0, bottom are standard 12.0
        let r0 = MenuModel::item_corner_radii(0, 3, card_radii, &metrics, true);
        assert_eq!(r0, (20.0, 20.0, 12.0, 12.0));

        // Item 1 (middle): all standard 12.0
        let r1 = MenuModel::item_corner_radii(1, 3, card_radii, &metrics, true);
        assert_eq!(r1, (12.0, 12.0, 12.0, 12.0));

        // Item 2 (bottom): top are standard 12.0, bottom are (6 - 4) = 2.0
        let r2 = MenuModel::item_corner_radii(2, 3, card_radii, &metrics, true);
        assert_eq!(r2, (12.0, 12.0, 2.0, 2.0));

        // When match_inner_corners = false: Item 2 bottom corners stay standard 12.0!
        let r2_unmatched = MenuModel::item_corner_radii(2, 3, card_radii, &metrics, false);
        assert_eq!(r2_unmatched, (12.0, 12.0, 12.0, 12.0));

        // Unified card with 1 item: all corners concentric 20.0
        let unified_card = (24.0, 24.0, 24.0, 24.0);
        let single = MenuModel::item_corner_radii(0, 1, unified_card, &metrics, true);
        assert_eq!(single, (20.0, 20.0, 20.0, 20.0));
    }

    #[test]
    fn interpolated_item_radii_smooth_morphing() {
        let metrics = MenuMetrics::default();
        let group = MenuGroup::new()
            .item(MenuItem::new(1, "Item 1"))
            .item(MenuItem::new(2, "Item 2"))
            .item(MenuItem::new(3, "Item 3"));
        let card_radii = (24.0, 24.0, 6.0, 6.0);

        // At y = 0.0 (Item 1), top corners are 20.0, bottom corners are 12.0
        let r_top = MenuModel::interpolated_item_radii(0.0, &group, card_radii, &metrics, true);
        assert_eq!(r_top, (20.0, 20.0, 12.0, 12.0));

        // Halfway between Item 1 and Item 2 (y = 23.0), top corners morph towards 12.0
        let r_mid1 = MenuModel::interpolated_item_radii(23.0, &group, card_radii, &metrics, true);
        assert!((r_mid1.0 - 16.0).abs() < 0.1);
        assert!((r_mid1.1 - 16.0).abs() < 0.1);
        assert_eq!(r_mid1.2, 12.0);
        assert_eq!(r_mid1.3, 12.0);

        // At y = 46.0 (Item 2, middle), all corners are 12.0
        let r_mid = MenuModel::interpolated_item_radii(46.0, &group, card_radii, &metrics, true);
        assert_eq!(r_mid, (12.0, 12.0, 12.0, 12.0));

        // At y = 92.0 (Item 3, bottom), top corners are 12.0, bottom corners are 2.0
        let r_bot = MenuModel::interpolated_item_radii(92.0, &group, card_radii, &metrics, true);
        assert_eq!(r_bot, (12.0, 12.0, 2.0, 2.0));

        // When match_inner_corners = false, bottom corners stay 12.0
        let r_bot_unmatched = MenuModel::interpolated_item_radii(92.0, &group, card_radii, &metrics, false);
        assert_eq!(r_bot_unmatched, (12.0, 12.0, 12.0, 12.0));
    }

    #[test]
    fn submenu_attachment_and_chevron_default() {
        let sub = MenuModel::new([
            MenuGroup::new().items([
                MenuItem::new(201, "Sub 1"),
                MenuItem::new(202, "Sub 2"),
            ])
        ]);

        // Attaching submenu should automatically set Chevron when trailing was None
        let item = MenuItem::new(102, "Item 2").submenu(sub.clone());
        assert!(item.has_submenu());
        assert_eq!(item.trailing, MenuItemTrailing::Chevron);

        // Attaching submenu should preserve custom trailing if already set
        let item_with_shortcut = MenuItem::new(103, "Item 3")
            .shortcut("⌘S")
            .submenu(sub);
        assert!(item_with_shortcut.has_submenu());
        assert_eq!(item_with_shortcut.trailing, MenuItemTrailing::Shortcut("⌘S"));
    }

    #[test]
    fn submenu_open_close_toggle_state() {
        let sub = MenuModel::new([
            MenuGroup::new().items([
                MenuItem::new(201, "Sub 1"),
                MenuItem::new(202, "Sub 2"),
            ])
        ]);
        let mut menu = MenuModel::new([
            MenuGroup::new().items([
                MenuItem::new(101, "Item 1"),
                MenuItem::new(102, "Item 2").submenu(sub),
                MenuItem::new(103, "Item 3"),
            ])
        ]);

        assert!(menu.has_submenu(102));
        assert!(!menu.has_submenu(101));
        assert!(menu.active_submenu_item_id.is_none());

        // Open submenu
        menu.open_submenu(102);
        assert_eq!(menu.active_submenu_item_id, Some(102));
        assert_eq!(menu.last_submenu_item_id, Some(102));

        // Toggle closes it
        menu.toggle_submenu(102);
        assert!(menu.active_submenu_item_id.is_none());
        assert_eq!(menu.last_submenu_item_id, Some(102)); // preserved for exit animation!

        // Toggle opens it again
        menu.toggle_submenu(102);
        assert_eq!(menu.active_submenu_item_id, Some(102));

        // Close submenu explicitly
        menu.close_submenu();
        assert!(menu.active_submenu_item_id.is_none());
        assert_eq!(menu.last_submenu_item_id, Some(102));
    }

    #[test]
    fn submenu_spring_advances_and_settles() {
        let sub = MenuModel::new([
            MenuGroup::new().items([
                MenuItem::new(201, "Sub 1"),
            ])
        ]);
        let mut menu = MenuModel::new([
            MenuGroup::new().items([
                MenuItem::new(101, "Item 1").submenu(sub),
            ])
        ]);

        // Initially closed
        assert_eq!(menu.submenu_progress.pos, 0.0);

        // Open
        menu.open_submenu(101);
        for _ in 0..40 {
            menu.step(1.0 / 60.0);
        }
        assert!((menu.submenu_progress.pos - 1.0).abs() < 0.05);

        // Close
        menu.close_submenu();
        for _ in 0..60 {
            menu.step(1.0 / 60.0);
        }
        assert!(menu.submenu_progress.pos < 0.01);
        assert!(menu.last_submenu_item_id.is_none()); // unmounted after settling!
    }

    #[test]
    fn submenu_recursive_stepping() {
        let mut sub = MenuModel::new([
            MenuGroup::new().items([
                MenuItem::new(201, "Sub 1"),
                MenuItem::new(202, "Sub 2"),
            ])
        ]);
        sub.hover_item(202, true);

        let mut menu = MenuModel::new([
            MenuGroup::new().items([
                MenuItem::new(101, "Item 1").submenu(sub),
            ])
        ]);

        // Stepping parent menu should recursively step child submenu springs
        let animating = menu.step(1.0 / 60.0);
        assert!(animating);
        let child_sub = menu.submenu(101).unwrap();
        assert!(child_sub.groups[0].indicator_opacity.pos > 0.0);
    }

    #[test]
    fn item_offset_in_menu_grouped_and_unified() {
        let metrics = MenuMetrics::default();
        let group1 = MenuGroup::new().items([
            MenuItem::new(1, "Item 1"),
            MenuItem::new(2, "Item 2"),
        ]);
        let group2 = MenuGroup::new().items([
            MenuItem::new(3, "Item 3"),
        ]);

        // GroupedCards
        let menu_grouped = MenuModel::new([group1.clone(), group2.clone()]);
        let (y1, h1) = menu_grouped.item_offset_in_menu(1, &metrics).unwrap();
        assert_eq!(y1, metrics.container_padding_y);
        assert_eq!(h1, 44.0);

        let (y2, h2) = menu_grouped.item_offset_in_menu(2, &metrics).unwrap();
        assert_eq!(y2, metrics.container_padding_y + 44.0 + metrics.item_gap);
        assert_eq!(h2, 44.0);

        // UnifiedCard
        let menu_unified = MenuModel::new([group1, group2]).variant(MenuVariant::UnifiedCard);
        let (uy1, uh1) = menu_unified.item_offset_in_menu(1, &metrics).unwrap();
        assert_eq!(uy1, metrics.container_padding_y);
        assert_eq!(uh1, 44.0);
    }

    #[test]
    fn submenu_find_item_recursive() {
        let sub = MenuModel::new([
            MenuGroup::new().items([
                MenuItem::new(201, "Sub 1"),
                MenuItem::new(202, "Sub 2"),
            ])
        ]);
        let menu = MenuModel::new([
            MenuGroup::new().items([
                MenuItem::new(101, "Item 1"),
                MenuItem::new(102, "Item 2").submenu(sub),
            ])
        ]);

        assert!(menu.contains_item(101));
        assert!(menu.contains_item(202));
        assert!(!menu.contains_item(999));

        let found = menu.find_item(202).unwrap();
        assert_eq!(found.label.as_ref(), "Sub 2");
    }

    #[test]
    fn submenu_overlap_and_placement_builders() {
        let default_menu = MenuModel::new([]);
        assert_eq!(default_menu.submenu_overlap, 2.0);
        assert_eq!(default_menu.submenu_offset_y, -4.0);

        let menu = MenuModel::new([])
            .submenu_overlap(32.0)
            .submenu_offset_y(-8.0)
            .submenu_placement(SubmenuPlacement::Left)
            .open_submenu_on_hover(false)
            .menu_width(240.0);

        let metrics = MenuMetrics::default();
        assert_eq!(menu.submenu_overlap, 32.0);
        assert_eq!(menu.submenu_offset_y, -8.0);
        assert_eq!(menu.submenu_placement, SubmenuPlacement::Left);
        assert!(!menu.open_submenu_on_hover);
        assert_eq!(menu.menu_width, Some(240.0));
        assert_eq!(menu.effective_width(&metrics), 240.0);

        let shadows_r = MenuModel::submenu_box_shadow(SubmenuPlacement::Right);
        assert_eq!(shadows_r.len(), 2);
        assert_eq!(shadows_r[0].offset.x, px(-3.0));

        let shadows_l = MenuModel::submenu_box_shadow(SubmenuPlacement::Left);
        assert_eq!(shadows_l.len(), 2);
        assert_eq!(shadows_l[0].offset.x, px(3.0));
    }
}
