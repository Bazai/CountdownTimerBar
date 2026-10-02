# ADR 0006. UI testing on headless GPUI

- Status: accepted
- Date: 2026-10-01
- Affects: `src/testing.rs`, `src/gui/ports.rs`, `src/gui/state.rs`, `tests/ui_flows.rs`, `tests/visual.rs`, `tests/golden/`

## Context

Appearance and gestures were checked with a home-made rig of bash, Swift (synthetic mouse, PNG comparison), AppleScript through Accessibility, and a window self-capture through CoreGraphics inside the app. It depended on coordinates, Accessibility permission and the user's real settings, produced differences at the end of the progress arc because of timing, and did not run under `cargo test`.

GPUI already contains everything needed: `TestAppContext` and `HeadlessAppContext` with controllable time, `gpui_kit::test::TestWindowExt` (click, double click, hover, drag, keys, finding an element by id with real hit testing) and a headless Metal renderer that draws the scene into a texture with no window and no permissions.

## Decision

- `AppState` knows nothing about AppKit: the store (`KeyValueStore`), the clock (`Clock`), the menu-bar item (`StatusDisplay`) and notifications (`Notifier`) come in through `with_parts`. Production assembles the real adapters in `AppState::new`; tests substitute memory, a manual clock and spies.
- `testing` (cargo feature `test-support`) builds a `Session`: the real `Panel` over those parts, with the theme set in code. Key elements register with `.test_support()`; without the feature it is an empty wrapper.
- `tests/ui_flows.rs`: scenarios on `TestAppContext` (start, pause, finish and notification, editing a value, "+", removal, knob, keyboard, the list field, About). They run on any thread and are part of the ordinary `cargo test`.
- `tests/visual.rs`: golden snapshots `tests/golden/*.png` for idle, settings, about, running, paused, edit and a knob drag, in the dark and light themes. `harness = false`, because Metal and the text system need the main thread. Tolerance: a channel may differ by up to 2, and up to 0.05 % of pixels may change. `UPDATE_GOLDEN=1` rewrites the goldens; on a mismatch `target/visual/` holds the actual image and the difference.
- Time is fake, so the progress arc in the snapshots is exact.

## Consequences

- Removed: `scripts/dev`, the `dev-tools` feature, `macos/diagnostics.rs` (about 150 lines of unsafe FFI to CoreGraphics), the `CTB_CAPTURE_*`, `CTB_DEBUG_WINDOW` and `CTB_APPEARANCE` variables, and the `make build-dev` target.
- Goldens depend on the macOS version and fonts; the visual test is skipped outside macOS. After an intended visual change the goldens are updated with a command and reviewed by eye.
- Not covered: the behaviour of the real `NSWindow` (borderless, key status, activation), `NSStatusItem` and system notifications. That is the thin `macos/` layer, checked by unit tests and manual runs.
