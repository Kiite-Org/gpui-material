# gpui-m3e

M3E slider port for GPUI. Lib is `src/`, preview is `examples/showcase.rs`.

## Commands

- Fast check: `cargo check --example showcase` (about 5s cached)
- Run preview: `cargo run --example showcase` (needs display, Wayland or X11)
- Lib only: `cargo check`
- Avoid `cargo test` / full `cargo clippy` for iteration. First build pulls about 710 crates and stalls the session.

## Layout

- `src/lib.rs`: facade re-exports (`components`, `motion`, `theme`) plus backward-compatible module aliases.
- `src/components/`: modular components (`slider.rs`, `switch.rs`, `split_button.rs`, `mod.rs`).
- `src/motion.rs`: `SpringParams`, `SpringState`, and `spring_step` physics engine.
- `src/theme.rs`: light and dark palettes transcribed from `PaletteTokens.kt`.
- `examples/showcase.rs`: `Showcase` view with slider, switch, and split button models.
- `references/androidx/`: sparse checkout, official `Slider.kt` plus `tokens/SliderTokens.kt`. Read only, never edit.
- `references/zed/`: sparse checkout, `crates/gpui` examples for interaction patterns. Read only.

Git in `references/` needs `git -C references/<name>` because of the mount filesystem boundary. Do not re-clone.

## GPUI 0.2.2 quirks, crates.io pinned

- No `gpui::SpringConfig`, no `with_spring`. That API exists only in zed main. Implement springs by hand with `spring_step` plus `window.request_animation_frame()`.
- `Rgba` has no `.opacity()`. That method is only on `Hsla`. Use `gpui::Rgba { a, ..color }`.
- `Pixels` field is private. Never use `.0`. Use `f32::from(px)` and `Pixels / Pixels -> f32`.
- `MouseDownEvent` has window position only, no element bounds. Click-to-seek needs `on_drag` plus `DragMoveEvent.bounds` to cache `track_left` and width. Keep the relative-delta fallback for the first click before bounds exist.
- Per-corner rounding exists via macro: `rounded_l_full`, `rounded_tr_xs`, and so on. M3E tracks need outer full and inner 2px (`xs`), not `rounded_full` everywhere.

## M3E numbers, do not guess

- Track 16px high, thumb 4px wide idle and 2px pressed, 44px tall, gap 6px, tick dots 4px.
- Hide ticks inside `thumb_w / 2 + gap + 2` around the thumb center, like `drawTrack` skips `tickEndGap`.
- Hide active or inactive track when its width is under 8px.
- Stepped ticks are `0..=steps + 1` over `steps + 1`, so 4 steps gives 0, 2, 4, 6, 8, 10 on a 0..10 range.
