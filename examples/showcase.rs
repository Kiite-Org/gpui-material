use gpui::{
    App, Bounds, Context, Pixels, Render, SharedString, Window, WindowBounds, WindowOptions,
    deferred, div, prelude::*, px, size,
};
use gpui_platform::application;
use gpui_m3e::{
    M3Palette, SliderSpec, SplitButtonModel, SplitButtonSize, SplitButtonVariant, SplitIconKind,
    SpringParams, SpringState, SwitchColors, SwitchMetrics, SwitchModel, animated_icon_segments,
    dark_scheme, draw_chevron_icon, draw_split_icon, draw_switch_icon, light_scheme, spring_step,
    thumb_geometry,
};

const TRACK_W: f32 = 480.0;
const DT: f32 = 1.0 / 60.0;

#[derive(Clone)]
struct SliderDrag {
    ix: usize,
}

struct EmptyDrag;

impl Render for EmptyDrag {
    fn render(&mut self, _w: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_0()
    }
}

#[derive(Clone)]
struct SliderModel {
    label: SharedString,
    spec: SliderSpec,
    value: f32,
    display: SpringState,
    press_state: SpringState,
    hover_state: SpringState,
    is_pressed: bool,
    is_hovered: bool,
    drag: Option<(Pixels, f32)>,
    track_left: Option<f32>,
    track_w: f32,
    enabled: bool,
    expressive: bool,
}

impl SliderModel {
    fn new(label: impl Into<SharedString>, spec: SliderSpec, value: f32) -> Self {
        let frac = spec.fraction(value);
        Self {
            label: label.into(),
            spec,
            value,
            display: SpringState {
                pos: frac,
                vel: 0.0,
            },
            press_state: SpringState::default(),
            hover_state: SpringState::default(),
            is_pressed: false,
            is_hovered: false,
            drag: None,
            track_left: Some(48.0),
            track_w: TRACK_W,
            enabled: true,
            expressive: true,
        }
    }

    fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn expressive(mut self, expressive: bool) -> Self {
        self.expressive = expressive;
        self
    }

    fn target_frac(&self) -> f32 {
        self.spec.fraction(self.value)
    }
}

struct Showcase {
    sliders: Vec<SliderModel>,
    switches: Vec<SwitchModel>,
    split_buttons: Vec<SplitButtonModel>,
    dark_theme: bool,
}

impl Showcase {
    fn seek_absolute(&mut self, ix: usize, mouse_x: f32, left: f32, w: f32, cx: &mut Context<Self>) {
        if !self.sliders[ix].enabled {
            return;
        }
        self.sliders[ix].track_left = Some(left);
        self.sliders[ix].track_w = w.max(1.0);
        let avail_w = (w - 12.0).max(1.0);
        let frac = ((mouse_x - left - 6.0) / avail_w).clamp(0.0, 1.0);
        let spec = self.sliders[ix].spec;
        self.sliders[ix].value = spec.value_from_fraction(frac);
        cx.notify();
    }

    fn on_down(&mut self, ix: usize, x: Pixels, cx: &mut Context<Self>) {
        if !self.sliders[ix].enabled {
            return;
        }
        self.sliders[ix].is_pressed = true;
        // Click-to-seek: jump straight to click point.
        // Spring animation smooths the thumb to the click point.
        let left = self.sliders[ix].track_left.unwrap_or(48.0);
        let w = self.sliders[ix].track_w;
        let avail_w = (w - 12.0).max(1.0);
        let frac = ((f32::from(x) - left - 6.0) / avail_w).clamp(0.0, 1.0);
        let spec = self.sliders[ix].spec;
        self.sliders[ix].value = spec.value_from_fraction(frac);
        let v = self.sliders[ix].value;
        self.sliders[ix].drag = Some((x, v));
        cx.notify();
    }

    fn on_move(&mut self, ix: usize, x: Pixels, cx: &mut Context<Self>) {
        if self.sliders[ix].drag.is_none() || !self.sliders[ix].enabled {
            return;
        }
        let left = self.sliders[ix].track_left.unwrap_or(48.0);
        let w = self.sliders[ix].track_w;
        let avail_w = (w - 12.0).max(1.0);
        let frac = ((f32::from(x) - left - 6.0) / avail_w).clamp(0.0, 1.0);
        let spec = self.sliders[ix].spec;
        self.sliders[ix].value = spec.value_from_fraction(frac);
        cx.notify();
    }

    fn on_up(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.sliders[ix].drag = None;
        self.sliders[ix].is_pressed = false;
        cx.notify();
    }

    fn hover_slider(&mut self, ix: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.sliders[ix].enabled {
            return;
        }
        self.sliders[ix].is_hovered = hovered;
        cx.notify();
    }

    fn step_by(&mut self, ix: usize, dir: f32, cx: &mut Context<Self>) {
        if !self.sliders[ix].enabled {
            return;
        }
        let s = self.sliders[ix].spec;
        let span = (s.max - s.min).max(1e-6);
        let step = if s.steps == 0 {
            span / 20.0
        } else {
            span / (s.steps + 1) as f32
        };
        let cur = self.sliders[ix].value;
        self.sliders[ix].value = s.value_from_fraction(s.fraction(cur + dir * step));
        cx.notify();
    }

    fn toggle_switch(&mut self, ix: usize, cx: &mut Context<Self>) {
        if !self.switches[ix].enabled {
            return;
        }
        self.switches[ix].toggle();
        cx.notify();
    }

    fn press_switch(&mut self, ix: usize, pressed: bool, cx: &mut Context<Self>) {
        if !self.switches[ix].enabled {
            return;
        }
        self.switches[ix].is_pressed = pressed;
        cx.notify();
    }

    fn hover_switch(&mut self, ix: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.switches[ix].enabled {
            return;
        }
        self.switches[ix].is_hovered = hovered;
        cx.notify();
    }

    fn press_split_leading(&mut self, ix: usize, pressed: bool, cx: &mut Context<Self>) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        self.split_buttons[ix].is_leading_pressed = pressed;
        cx.notify();
    }

    fn hover_split_leading(&mut self, ix: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        self.split_buttons[ix].is_leading_hovered = hovered;
        cx.notify();
    }

    fn click_split_leading(&mut self, ix: usize, cx: &mut Context<Self>) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        cx.notify();
    }

    fn press_split_trailing(&mut self, ix: usize, pressed: bool, cx: &mut Context<Self>) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        self.split_buttons[ix].is_trailing_pressed = pressed;
        cx.notify();
    }

    fn hover_split_trailing(&mut self, ix: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        self.split_buttons[ix].is_trailing_hovered = hovered;
        cx.notify();
    }

    fn toggle_split_dropdown(&mut self, ix: usize, cx: &mut Context<Self>) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        let was_open = self.split_buttons[ix].checked;
        for (i, b) in self.split_buttons.iter_mut().enumerate() {
            if i != ix {
                b.close_dropdown();
            }
        }
        if was_open {
            self.split_buttons[ix].close_dropdown();
        } else {
            self.split_buttons[ix].toggle_dropdown();
        }
        cx.notify();
    }

    fn close_split_dropdown(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.split_buttons[ix].close_dropdown();
        cx.notify();
    }

    fn select_split_menu_item(&mut self, ix: usize, item_ix: usize, cx: &mut Context<Self>) {
        self.split_buttons[ix].select_menu_item(item_ix);
        cx.notify();
    }

    fn render_split_button(
        &self,
        ix: usize,
        pal: &M3Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let btn = &self.split_buttons[ix];
        let metrics = btn.metrics();
        let colors = btn.resolve_colors(pal);
        let (l_tl, l_tr, l_br, l_bl) = btn.leading_corners();
        let (t_tl, t_tr, t_br, t_bl) = btn.trailing_corners();

        let trailing_w = (metrics.trailing_padding_start
            + metrics.trailing_icon_size
            + metrics.trailing_padding_end)
            .max(metrics.container_height * 0.75);

        div()
            .relative()
            .flex()
            .items_center()
            // Leading primary action button
            .child(
                div()
                    .id(("split-leading", ix))
                    .relative()
                    .h(px(metrics.container_height))
                    .w(px(btn.leading_width.pos))
                    .overflow_hidden()
                    .flex()
                    .items_center()
                    .justify_center()
                    .pl(px(metrics.leading_padding_start))
                    .pr(px(metrics.leading_padding_end))
                    .rounded_tl(px(l_tl))
                    .rounded_tr(px(l_tr))
                    .rounded_br(px(l_br))
                    .rounded_bl(px(l_bl))
                    .bg(if btn.enabled {
                        colors.container
                    } else {
                        colors.disabled_container
                    })
                    .when(btn.enabled && colors.has_shadow, |d| d.shadow_sm())
                    .when_some(
                        if btn.enabled {
                            colors.border
                        } else {
                            colors.disabled_border
                        },
                        |d, b| d.border_1().border_color(b),
                    )
                    .when(!btn.enabled, |d| d.cursor_not_allowed())
                    .when(btn.enabled, |d| d.cursor_pointer())
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_leading(ix, true, cx);
                        }),
                    )
                    .on_mouse_up(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_leading(ix, false, cx);
                            this.click_split_leading(ix, cx);
                        }),
                    )
                    .on_mouse_up_out(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_leading(ix, false, cx);
                        }),
                    )
                    .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                        this.hover_split_leading(ix, *hovered, cx);
                        if !*hovered {
                            this.press_split_leading(ix, false, cx);
                        }
                    }))
                    // State layer overlay
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .rounded_tl(px(l_tl))
                            .rounded_tr(px(l_tr))
                            .rounded_br(px(l_br))
                            .rounded_bl(px(l_bl))
                            .bg(colors.state_layer)
                            .opacity(btn.leading_state_layer.pos),
                    )
                    // Optional leading icon
                    .when(btn.icon != SplitIconKind::None, |d| {
                        let icon_sz = (metrics.container_height * 0.38).clamp(14.0, 24.0);
                        let icon_kind = btn.icon;
                        let icon_color = if btn.enabled {
                            colors.content
                        } else {
                            colors.disabled_content
                        };
                        d.child(
                            gpui::canvas(
                                move |bounds, _, _| (bounds, icon_kind, icon_color),
                                move |_, (bounds, k, c), window, _| {
                                    draw_split_icon(window, bounds, k, c);
                                },
                            )
                            .size(px(icon_sz))
                            .flex_shrink_0()
                            .mr_2(),
                        )
                    })
                    // Button Label
                    .child(
                        div()
                            .text_size(px(metrics.font_size))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .flex_shrink_0()
                            .whitespace_nowrap()
                            .text_color(if btn.enabled {
                                colors.content
                            } else {
                                colors.disabled_content
                            })
                            .child(btn.label.clone()),
                    ),
            )
            // 2dp spacing between buttons from M3E tokens
            .child(div().w(px(metrics.between_space)))
            // Trailing toggle/menu button
            .child(
                div()
                    .id(("split-trailing", ix))
                    .relative()
                    .h(px(metrics.container_height))
                    .w(px(trailing_w))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_tl(px(t_tl))
                    .rounded_tr(px(t_tr))
                    .rounded_br(px(t_br))
                    .rounded_bl(px(t_bl))
                    .bg(if btn.enabled {
                        colors.container
                    } else {
                        colors.disabled_container
                    })
                    .when(btn.enabled && colors.has_shadow, |d| d.shadow_sm())
                    .when_some(
                        if btn.enabled {
                            colors.border
                        } else {
                            colors.disabled_border
                        },
                        |d, b| d.border_1().border_color(b),
                    )
                    .when(!btn.enabled, |d| d.cursor_not_allowed())
                    .when(btn.enabled, |d| d.cursor_pointer())
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_trailing(ix, true, cx);
                        }),
                    )
                    .on_mouse_up(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_trailing(ix, false, cx);
                            this.toggle_split_dropdown(ix, cx);
                        }),
                    )
                    .on_mouse_up_out(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_trailing(ix, false, cx);
                        }),
                    )
                    .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                        this.hover_split_trailing(ix, *hovered, cx);
                        if !*hovered {
                            this.press_split_trailing(ix, false, cx);
                        }
                    }))
                    // State layer overlay
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .rounded_tl(px(t_tl))
                            .rounded_tr(px(t_tr))
                            .rounded_br(px(t_br))
                            .rounded_bl(px(t_bl))
                            .bg(colors.state_layer)
                            .opacity(btn.trailing_state_layer.pos),
                    )
                    // Animated vector chevron icon
                    .child({
                        let chevron_pos = btn.chevron_progress.pos;
                        let chevron_color = if btn.enabled {
                            colors.content
                        } else {
                            colors.disabled_content
                        };
                        gpui::canvas(
                            move |bounds, _, _| (bounds, chevron_pos, chevron_color),
                            move |_, (bounds, prog, col), window, _| {
                                draw_chevron_icon(window, bounds, prog, col);
                            },
                        )
                        .size(px(metrics.trailing_icon_size))
                    }),
            )
            // Anchored dropdown menu popover
            .when(
                (btn.checked || btn.menu_progress.pos > 0.01) && !btn.menu_items.is_empty(),
                |d| {
                    let anim = btn.menu_progress.pos.clamp(0.0, 1.0);
                    // Fast, snappy slide down
                    let slide_distance = 16.0;
                    let top_offset = metrics.container_height + 4.0 - (1.0 - anim) * slide_distance;
                    let opacity = (anim * 5.0).clamp(0.0, 1.0);
                    // Auto-scale modal width to match the split button container width exactly (or expand if items require)
                    let total_btn_w = btn.leading_width.pos + metrics.between_space + trailing_w;
                    let menu_w = total_btn_w.max(btn.menu_content_width());
                    d.child(
                        deferred(
                            div()
                                .id(("split-menu", ix))
                                .occlude()
                                .absolute()
                                .top(px(top_offset))
                                .left_0()
                                .w(px(menu_w))
                                .p(px(metrics.menu_item_padding_y * 0.75))
                                .when(btn.size == SplitButtonSize::ExtraSmall, |s| s.rounded_md())
                                .when(btn.size == SplitButtonSize::Small, |s| s.rounded_lg())
                                .when(btn.size == SplitButtonSize::Medium, |s| s.rounded_xl())
                                .when(btn.size >= SplitButtonSize::Large, |s| s.rounded_2xl())
                                .bg(pal.surface_container_highest)
                                .border_1()
                                .border_color(pal.outline_variant)
                                .shadow_lg()
                                .opacity(opacity)
                                .on_mouse_down_out(cx.listener(move |this, _, _, cx| {
                                    this.close_split_dropdown(ix, cx);
                                }))
                                .children(btn.menu_items.iter().enumerate().map(|(item_ix, item)| {
                                    let is_selected = btn.selected_index == Some(item_ix);
                                    div()
                                        .id(("split-menu-item", ix * 100 + item_ix))
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .px(px(metrics.menu_item_padding_x))
                                        .py(px(metrics.menu_item_padding_y))
                                        .when(btn.size == SplitButtonSize::ExtraSmall, |s| s.rounded_sm())
                                        .when(btn.size == SplitButtonSize::Small, |s| s.rounded_md())
                                        .when(btn.size >= SplitButtonSize::Medium, |s| s.rounded_lg())
                                        .cursor_pointer()
                                        .text_size(px(metrics.menu_font_size))
                                        .hover(|s| {
                                            s.bg(pal.secondary_container)
                                                .text_color(pal.on_secondary_container)
                                        })
                                        .when(is_selected, |s| {
                                            s.font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(pal.primary)
                                        })
                                        .when(!is_selected, |s| s.text_color(pal.on_surface))
                                        .child(item.clone())
                                        .when(is_selected, |s| s.child("✓"))
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.select_split_menu_item(ix, item_ix, cx);
                                            }),
                                        )
                                })),
                        )
                        .priority(100),
                    )
                },
            )
    }
}

impl Render for Showcase {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pal = if self.dark_theme {
            dark_scheme()
        } else {
            light_scheme()
        };
        for m in &mut self.sliders {
            let target = m.target_frac();
            let params = if m.expressive {
                SpringParams::expressive()
            } else {
                SpringParams::standard()
            };
            let next = spring_step(params, m.display, target, DT);
            let settled = (next.pos - target).abs() < 0.0005 && next.vel.abs() < 0.0005;
            m.display = if settled {
                SpringState {
                    pos: target,
                    vel: 0.0,
                }
            } else {
                window.request_animation_frame();
                next
            };

            // Press spring: targets 1.0 when pressed or dragging, 0.0 otherwise
            let press_target = if m.enabled && (m.is_pressed || m.drag.is_some()) {
                1.0
            } else {
                0.0
            };
            let press_params = SpringParams {
                stiffness: 450.0,
                damping: 30.0,
                mass: 1.0,
            };
            let next_press = spring_step(press_params, m.press_state, press_target, DT);
            let press_settled = (next_press.pos - press_target).abs() < 0.001 && next_press.vel.abs() < 0.001;
            m.press_state = if press_settled {
                SpringState {
                    pos: press_target,
                    vel: 0.0,
                }
            } else {
                window.request_animation_frame();
                next_press
            };

            // Hover spring: targets 1.0 when hovered, 0.0 otherwise
            let hover_target = if m.enabled && m.is_hovered {
                1.0
            } else {
                0.0
            };
            let hover_params = SpringParams {
                stiffness: 400.0,
                damping: 32.0,
                mass: 1.0,
            };
            let next_hover = spring_step(hover_params, m.hover_state, hover_target, DT);
            let hover_settled = (next_hover.pos - hover_target).abs() < 0.001 && next_hover.vel.abs() < 0.001;
            m.hover_state = if hover_settled {
                SpringState {
                    pos: hover_target,
                    vel: 0.0,
                }
            } else {
                window.request_animation_frame();
                next_hover
            };
        }

        for sw in &mut self.switches {
            sw.advance_springs(window, DT);
        }
        for btn in &mut self.split_buttons {
            btn.advance_springs(window, DT);
        }
        let sw_colors = SwitchColors::from_palette(&pal, self.dark_theme);
        let sw_metrics = SwitchMetrics::default();

        div()
            .id("showcase-scroll")
            .flex()
            .flex_col()
            .gap_6()
            .p_8()
            .size_full()
            .overflow_y_scroll()
            .bg(pal.surface)
            .text_color(pal.on_surface)
            .child(
                div().flex().justify_between().items_center()
                    .child(
                        div().text_xl().font_weight(gpui::FontWeight::SEMIBOLD).child("Sliders")
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_full()
                            .bg(pal.secondary_container)
                            .text_color(pal.on_secondary_container)
                            .text_xs()
                            .cursor_pointer()
                            .child(if self.dark_theme { "Light Theme" } else { "Dark Theme" })
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.dark_theme = !this.dark_theme;
                                    cx.notify();
                                }),
                            ),
                    ),
            )
            .children(self.sliders.iter().enumerate().map(|(ix, m)| {
                let frac = m.display.pos.clamp(0.0, 1.0);
                let press = m.press_state.pos.clamp(0.0, 1.0);
                let hover = m.hover_state.pos.clamp(0.0, 1.0);

                let avail_w = TRACK_W - 12.0;
                let p = frac * avail_w;
                let thumb_cx = p + 6.0;

                // Handle "|" animation: shrinks in width from 4px down to 2px when pressed.
                let thumb_w = 4.0 - 2.0 * press;
                // Height expands: 40px idle -> 44px hover -> 48px pressed.
                let thumb_h = 40.0 + 4.0 * hover * (1.0 - press) + 8.0 * press;
                let thumb_top = 36.0 - thumb_h / 2.0;
                let thumb_left = thumb_cx - thumb_w / 2.0;

                let active_end = p;
                let inactive_start = p + 12.0;
                let show_active = active_end >= 8.0;
                let show_inactive = (TRACK_W - inactive_start) >= 8.0;
                let stop_x = TRACK_W - 8.0;
                let show_stop = show_inactive && (stop_x - thumb_cx).abs() >= 10.0;
                let ticks = m.spec.tick_fractions();
                let truth_frac = m.target_frac();

                let val_str = if !m.enabled {
                    "Disabled".to_string()
                } else if m.spec.steps > 0 {
                    format!("{:.0} / {:.0}", m.value, m.spec.max)
                } else if (m.spec.max - 100.0).abs() < 1e-4 {
                    format!("{:.0}%", m.value)
                } else {
                    format!("{:.2}", m.value)
                };

                div()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .rounded_xl()
                    .bg(gpui::Rgba {
                        a: if self.dark_theme { 0.08 } else { 0.04 },
                        ..pal.on_surface
                    })
                    .child(
                        div().flex().justify_between().items_center()
                            .child(div().text_base().font_weight(gpui::FontWeight::MEDIUM).child(m.label.clone()))
                            .child(div().text_sm().text_color(pal.on_surface_variant).child(val_str)),
                    )
                    .child(
                        div()
                            .relative()
                            .w(px(TRACK_W))
                            .h(px(72.0))
                            .id(("m3-slider", ix))
                            .cursor_pointer()
                            .when(!m.enabled, |d| d.opacity(0.38))
                            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                                this.hover_slider(ix, *hovered, cx);
                            }))
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(move |this, e: &gpui::MouseDownEvent, _, cx| {
                                    this.on_down(ix, e.position.x, cx);
                                }),
                            )
                            .on_mouse_move(cx.listener(move |this, e: &gpui::MouseMoveEvent, _, cx| {
                                this.on_move(ix, e.position.x, cx);
                            }))
                            .on_mouse_up(
                                gpui::MouseButton::Left,
                                cx.listener(move |this, _, _, cx| this.on_up(ix, cx)),
                            )
                            .on_mouse_up_out(
                                gpui::MouseButton::Left,
                                cx.listener(move |this, _, _, cx| this.on_up(ix, cx)),
                            )
                            .on_drag(
                                SliderDrag { ix },
                                |_drag, _, _, cx| cx.new(|_| EmptyDrag),
                            )
                            .on_drag_move(cx.listener(
                                |this, e: &gpui::DragMoveEvent<SliderDrag>, _, cx| {
                                    let ix = e.drag(cx).ix;
                                    let mx = f32::from(e.event.position.x);
                                    let left = f32::from(e.bounds.origin.x);
                                    let w = f32::from(e.bounds.size.width);
                                    this.seek_absolute(ix, mx, left, w, cx);
                                },
                            ))
                            .when(m.drag.is_some(), |d| {
                                d.child(
                                    div()
                                        .absolute()
                                        .top(px(0.0))
                                        .left(px(thumb_cx - 28.0))
                                        .w(px(56.0))
                                        .h(px(24.0))
                                        .flex()
                                        .justify_center()
                                        .items_center()
                                        .rounded_full()
                                        .bg(pal.inverse_surface)
                                        .text_color(pal.inverse_on_surface)
                                        .text_xs()
                                        .child(format!("{:.1}", self.sliders[ix].value)),
                                )
                            })
                            .when(show_active, |d| {
                                d.child(
                                    div()
                                        .absolute()
                                        .top(px(28.0))
                                        .left(px(0.0))
                                        .w(px(active_end))
                                        .h(px(16.0))
                                        .rounded_l_full()
                                        .rounded_tr_xs()
                                        .rounded_br_xs()
                                        .bg(pal.primary),
                                )
                            })
                            .when(show_inactive, |d| {
                                d.child(
                                    div()
                                        .absolute()
                                        .top(px(28.0))
                                        .left(px(inactive_start))
                                        .w(px(TRACK_W - inactive_start))
                                        .h(px(16.0))
                                        .rounded_r_full()
                                        .rounded_tl_xs()
                                        .rounded_bl_xs()
                                        .bg(pal.secondary_container),
                                )
                            })
                            .when(show_stop, |d| {
                                d.child(
                                    div()
                                        .absolute()
                                        .top(px(34.0))
                                        .left(px(stop_x - 2.0))
                                        .size(px(4.0))
                                        .rounded_full()
                                        .bg(pal.primary),
                                )
                            })
                            .children(ticks.into_iter().filter_map(|t| {
                                if (t - 1.0).abs() < 1e-4 {
                                    return None;
                                }
                                let x = t * (TRACK_W - 12.0) + 6.0;
                                // Hide ticks inside the thumb cutout
                                if (x - thumb_cx).abs() < 8.0 {
                                    return None;
                                }
                                let c = if t <= truth_frac + 1e-5 {
                                    pal.secondary_container
                                } else {
                                    pal.primary
                                };
                                Some(
                                    div()
                                        .absolute()
                                        .top(px(34.0))
                                        .left(px(x - 2.0))
                                        .size(px(4.0))
                                        .rounded_full()
                                        .bg(c),
                                )
                            }))
                            .child(
                                div()
                                    .absolute()
                                    .top(px(thumb_top))
                                    .left(px(thumb_left))
                                    .w(px(thumb_w))
                                    .h(px(thumb_h))
                                    .rounded_full()
                                    .bg(pal.primary)
                                    .when(!m.enabled, |d| d.bg(gpui::Rgba { a: 0.38, ..pal.on_surface }))
                                    .when(m.enabled && (m.drag.is_some() || m.is_pressed), |d| d.shadow_md())
                                    .when(m.enabled && m.drag.is_none() && !m.is_pressed && m.is_hovered, |d| d.shadow_sm()),
                            ),
                    )
                    .child(
                        div().flex().gap_2().items_center()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_full()
                                    .bg(pal.secondary_container)
                                    .text_color(pal.on_secondary_container)
                                    .cursor_pointer()
                                    .child("-")
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| this.step_by(ix, -1.0, cx)),
                                    ),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_full()
                                    .bg(pal.secondary_container)
                                    .text_color(pal.on_secondary_container)
                                    .cursor_pointer()
                                    .child("+")
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| this.step_by(ix, 1.0, cx)),
                                    ),
                            )
                    )
            }))
            .child(
                div().mt_4()
                    .child(div().text_xl().font_weight(gpui::FontWeight::SEMIBOLD).child("Switches"))
            )
            .child(
                div()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .rounded_xl()
                    .bg(gpui::Rgba {
                        a: if self.dark_theme { 0.08 } else { 0.04 },
                        ..pal.on_surface
                    })
                    .children(self.switches.iter().enumerate().map(|(ix, sw)| {
                        let pos = sw.display_pos.pos.clamp(0.0, 1.0);
                        let press_factor = sw.press_pos.pos.clamp(0.0, 1.0);
                        let (thumb_left, thumb_w) = thumb_geometry(&sw_metrics, pos, press_factor);
                        let (track_c, thumb_c, icon_c, state_c, state_opacity) = sw.resolve_colors(&sw_colors);
                        let (seg1, seg2) = animated_icon_segments(pos, press_factor);
                        let show_icons = sw.show_icons;

                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .py_1()
                            .child(
                                div().flex_col()
                                    .child(div().text_sm().font_weight(gpui::FontWeight::MEDIUM).child(sw.label.clone()))
                                    .child(
                                        div().text_xs().text_color(pal.on_surface_variant).child(
                                            if !sw.enabled {
                                                "Disabled"
                                            } else if sw.checked {
                                                "On"
                                            } else {
                                                "Off"
                                            },
                                        ),
                                    ),
                            )
                            .child(
                                div()
                                    .id(("m3-switch", ix))
                                    .relative()
                                    .w(px(sw_metrics.track_width))
                                    .h(px(sw_metrics.track_height))
                                    .rounded_full()
                                    .bg(track_c)
                                    .when(!sw.enabled, |d| d.opacity(0.38).cursor_not_allowed())
                                    .when(sw.enabled, |d| d.cursor_pointer())
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.press_switch(ix, true, cx);
                                        }),
                                    )
                                    .on_mouse_up(
                                        gpui::MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.press_switch(ix, false, cx);
                                            this.toggle_switch(ix, cx);
                                        }),
                                    )
                                    .on_mouse_up_out(
                                        gpui::MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.press_switch(ix, false, cx);
                                        }),
                                    )
                                    .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                                        this.hover_switch(ix, *hovered, cx);
                                        if !*hovered {
                                            this.press_switch(ix, false, cx);
                                        }
                                    }))
                                    .child(
                                        div()
                                            .absolute()
                                            .top(px(2.0))
                                            .left(px(thumb_left))
                                            .w(px(thumb_w))
                                            .h(px(sw_metrics.thumb_height))
                                            .rounded_full()
                                            .bg(thumb_c)
                                            .shadow_sm()
                                            .overflow_hidden()
                                            .child(
                                                div()
                                                    .absolute()
                                                    .inset_0()
                                                    .rounded_full()
                                                    .bg(state_c)
                                                    .opacity(state_opacity),
                                            )
                                            .when(show_icons, |d| {
                                                let icon_left = (thumb_w - sw_metrics.icon_size) / 2.0;
                                                let icon_top = (sw_metrics.thumb_height - sw_metrics.icon_size) / 2.0;
                                                d.child(
                                                    gpui::canvas(
                                                        move |bounds, _window, _cx| (bounds, seg1, seg2, icon_c),
                                                        move |_, (bounds, s1, s2, color), window, _cx| {
                                                            draw_switch_icon(window, bounds, s1, s2, color);
                                                        },
                                                    )
                                                    .absolute()
                                                    .left(px(icon_left))
                                                    .top(px(icon_top))
                                                    .size(px(sw_metrics.icon_size)),
                                                )
                                            }),
                                    ),
                            )
                    })),
            )
            .child(
                div().mt_4()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Split Buttons"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(pal.on_surface_variant)
                            .child("Material 3 Expressive • 1:1 Tokens • Damped Spring Shape Morphing"),
                    ),
            )
            // 1. Emphasis Variants Card
            .child(
                div()
                    .flex_col()
                    .gap_4()
                    .p_4()
                    .rounded_xl()
                    .bg(gpui::Rgba {
                        a: if self.dark_theme { 0.08 } else { 0.04 },
                        ..pal.on_surface
                    })
                    .child(
                        div()
                            .flex_col()
                            .gap_0()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("Emphasis Variants (Interactive Dropdowns)"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(pal.on_surface_variant)
                                    .child("Hover or press to morph inner corners (4px → 12px). Click chevron to toggle dropdown and morph to circle (20px)."),
                            ),
                    )
                    // Filled
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .py_1()
                            .child(
                                div()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child("Filled Split Button"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("High-emphasis • Primary container • Medium size (56dp)"),
                                    ),
                            )
                            .child(self.render_split_button(0, &pal, cx)),
                    )
                    // Tonal
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .py_1()
                            .child(
                                div()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child("Tonal Split Button"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("Medium-emphasis • Secondary container • Small size (40dp)"),
                                    ),
                            )
                            .child(self.render_split_button(1, &pal, cx)),
                    )
                    // Elevated
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .py_1()
                            .child(
                                div()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child("Elevated Split Button"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("Elevation shadow • Primary content • Small size (40dp)"),
                                    ),
                            )
                            .child(self.render_split_button(2, &pal, cx)),
                    )
                    // Outlined
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .py_1()
                            .child(
                                div()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child("Outlined Split Button"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("1px Outline border • Transparent container • Small size (40dp)"),
                                    ),
                            )
                            .child(self.render_split_button(3, &pal, cx)),
                    ),
            )
            // 2. Token Sizes Comparison Card
            .child(
                div()
                    .flex_col()
                    .gap_4()
                    .p_4()
                    .rounded_xl()
                    .bg(gpui::Rgba {
                        a: if self.dark_theme { 0.08 } else { 0.04 },
                        ..pal.on_surface
                    })
                    .child(
                        div()
                            .flex_col()
                            .gap_0()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("Token Size Hierarchy (XS, Small, Medium)"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(pal.on_surface_variant)
                                    .child("Exact dp container heights, 2dp between-space, and size-specific inner/outer radii."),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_4()
                            .py_2()
                            // ExtraSmall (32dp)
                            .child(
                                div()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("Extra Small (32dp)"),
                                    )
                                    .child(self.render_split_button(4, &pal, cx)),
                            )
                            // Small (40dp)
                            .child(
                                div()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("Small (40dp)"),
                                    )
                                    .child(self.render_split_button(5, &pal, cx)),
                            )
                            // Medium (56dp)
                            .child(
                                div()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("Medium (56dp)"),
                                    )
                                    .child(self.render_split_button(6, &pal, cx)),
                            ),
                    ),
            )
            // 3. Disabled States Card
            .child(
                div()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .rounded_xl()
                    .bg(gpui::Rgba {
                        a: if self.dark_theme { 0.08 } else { 0.04 },
                        ..pal.on_surface
                    })
                    .child(
                        div()
                            .flex_col()
                            .gap_0()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("Disabled States"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(pal.on_surface_variant)
                                    .child("Container and label colors resolved with official M3E disabled alpha (0.12 container, 0.38 content)."),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_6()
                            .py_1()
                            .child(self.render_split_button(7, &pal, cx))
                            .child(self.render_split_button(8, &pal, cx)),
                    ),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(680.0), px(900.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| Showcase {
                    dark_theme: true,
                    sliders: vec![
                        SliderModel::new(
                            "Media Volume",
                            SliderSpec::new(0.0, 100.0, 0),
                            42.0,
                        ),
                        SliderModel::new(
                            "Display Brightness",
                            SliderSpec::new(0.0, 10.0, 4),
                            6.0,
                        ),
                        SliderModel::new(
                            "Balance",
                            SliderSpec::new(0.0, 1.0, 0),
                            0.5,
                        )
                        .enabled(false)
                        .expressive(false),
                    ],
                    switches: vec![
                        SwitchModel::new("Airplane Mode", true),
                        SwitchModel::new("Bluetooth", false),
                        SwitchModel::new("Haptics", true).show_icons(false),
                        SwitchModel::new("Night Light", true).enabled(false),
                        SwitchModel::new("Mobile Data", false).enabled(false),
                    ],
                    split_buttons: vec![
                        // 0: Filled (Save) with Check icon
                        SplitButtonModel::new("Save")
                            .variant(SplitButtonVariant::Filled)
                            .size(SplitButtonSize::Medium)
                            .icon(SplitIconKind::Check)
                            .menu_items(["Save & Publish", "Save Draft", "Export"]),
                        // 1: Tonal (Edit Document) with Edit icon
                        SplitButtonModel::new("Edit Document")
                            .variant(SplitButtonVariant::Tonal)
                            .size(SplitButtonSize::Small)
                            .icon(SplitIconKind::Edit)
                            .menu_items(["Duplicate", "Rename", "Move"]),
                        // 2: Elevated (Share Project) with Share icon
                        SplitButtonModel::new("Share Project")
                            .variant(SplitButtonVariant::Elevated)
                            .size(SplitButtonSize::Small)
                            .icon(SplitIconKind::Share)
                            .menu_items(["Copy Link", "Email Invite", "Permissions"]),
                        // 3: Outlined (Options)
                        SplitButtonModel::new("Options")
                            .variant(SplitButtonVariant::Outlined)
                            .size(SplitButtonSize::Small)
                            .menu_items(["Settings", "Shortcuts", "Reset"]),
                        // 4: Extra Small (XS)
                        SplitButtonModel::new("XS Action")
                            .variant(SplitButtonVariant::Filled)
                            .size(SplitButtonSize::ExtraSmall)
                            .menu_items(["Action 1", "Action 2"]),
                        // 5: Small Standard
                        SplitButtonModel::new("Small Standard")
                            .variant(SplitButtonVariant::Filled)
                            .size(SplitButtonSize::Small)
                            .menu_items(["Action 1", "Action 2"]),
                        // 6: Medium Action
                        SplitButtonModel::new("Medium Action")
                            .variant(SplitButtonVariant::Tonal)
                            .size(SplitButtonSize::Medium)
                            .menu_items(["Action 1", "Action 2"]),
                        // 7: Disabled Filled
                        SplitButtonModel::new("Disabled Filled")
                            .variant(SplitButtonVariant::Filled)
                            .size(SplitButtonSize::Small)
                            .enabled(false),
                        // 8: Disabled Outlined
                        SplitButtonModel::new("Disabled Outlined")
                            .variant(SplitButtonVariant::Outlined)
                            .size(SplitButtonSize::Small)
                            .enabled(false),
                    ],
                })
            },
        )
        .expect("open window");
        cx.activate(true);
    });
}
