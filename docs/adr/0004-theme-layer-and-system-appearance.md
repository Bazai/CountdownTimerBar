# ADR 0004. The `src/theme` layer and the system light/dark theme

- Status: accepted
- Date: 2026-10-01
- Affects: `src/theme/*`, `src/macos/appearance.rs`

## Context

The popover was hard-wired to dark. Design tokens lived in `gui/tokens.rs` as `u32` constants, the application to `gpui-component` and the theme observer in `gui/theme.rs`, and eight colours were written as literals right inside the views (`0xe4e4e7`, `0x52525b`, `0xffffff`, `0xd4d4d8`, `0x0f0f10`). There was nowhere to plug in a light theme, and the values had drifted between the mock, the documentation and the code.

## Decision

**The theme layer `src/theme/` is the single source of truth.**

- `color.rs`: the `Rgb` type (a newtype over `u32`, `const fn hex`) and pure colour maths: HSL, compositing, WCAG contrast, lightness adjustment to a minimum contrast.
- `palette.rs`: `Palette` made of semantic roles (`control_hover_border`, `text_active`, `badge_bg`, `inverse_fg`, …) with no `Default`, so a new role must get a value in every palette. `Palette::DARK` is a constant holding the reference dark design values; a test pins them. Shadow opacity (`Elevation`) is part of the palette too.
- `metrics.rs`: sizes, radii, accent opacity levels, text sizes.
- `component.rs`: the only place that knows the `gpui-component` theme slots.
- `mod.rs`: `Appearance { Dark, Light }`, the `AppTheme` global, `cx.palette()`, `theme::refresh(cx)`.
- Views take colours from `cx.palette()` and sizes from `metrics`; no hex is written in views.

**The light palette is derived from the dark one** (`Palette::light()`):

1. Neutral roles: HSL lightness is mirrored inside the palette's range, `L' = L_min + L_max − L`. Hue and saturation are kept, the order of the layers flips, and text becomes almost black.
2. Chromatic roles (`accent`, `danger`, `danger_text`): the same hue, with lightness lowered until the contrast with the popover body reaches the threshold (3:1 for UI colours, 4.5:1 for text).
3. Shadows stay black but weaker (a factor of 0.35).
4. Fills, the halo and the ring are computed from the `accent` role through opacity, so no separate light values are needed.

Changing the dark palette changes the light one automatically; there are no exceptions except three explicit ones (the shadow, the contrast thresholds, the shadow factor).

**The theme is system-only.** `NSApp.effectiveAppearance` is observed through KVO (`macos/appearance.rs`); on a change `theme::refresh` is called and the windows are redrawn. There is no manual switch. Tests set the theme in code (`theme::set_appearance`), so overriding it through an environment variable is unnecessary. The status-bar pill uses system colours and is not governed by the palette.

**Drift is caught by tests:**

- the reference values of `Palette::DARK` are pinned in a test;
- the role list is complete (by struct size) and has no duplicate names;
- in the light palette the layer order is the reverse of the dark one;
- the contrast of text and UI colours on their backgrounds passes the threshold in both themes (including text on the highlighted active circle);
- in both themes the popover frame paints no opaque quads above the body (a regression test for the double-border bug).

## Consequences

- Adding a colour: a new role in `Palette` and a value in `DARK`; the light one is derived, and the tests immediately show insufficient contrast.
- The light palette is algorithmic: there may be places a designer wants to adjust; that calls for an explicit exception, not a second table.
- A table of both palettes is printed by `cargo test print_palettes -- --ignored --nocapture`.

## Rejected alternatives

- **A hand-written light table:** exact control, but two sources of truth, and every change to the dark one needs a manual change to the light one.
- **A TOML token file:** good for exchange with design tools, but adds runtime parsing and parse errors; typed constants are checked by the compiler, and a test pins the reference values.
- **A manual System/Light/Dark switch:** not needed now; the architecture (`AppTheme`, `set_appearance`) does not rule it out for the future.
