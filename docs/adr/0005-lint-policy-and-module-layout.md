# ADR 0005. Lint policy and module layout

- Status: accepted
- Date: 2026-10-01
- Affects: `Cargo.toml` (`[lints]`, `[features]`), `clippy.toml`, `rustfmt.toml`, `Makefile`, all of `src/`

## Context

An audit against the rust-best-practices guide found: `cargo fmt --check` failed, there were no lints, three compiler warnings (dead code, `lockFocus`), and 20+ groups of pedantic findings. The crate was a single binary, so `rustc` could not find unused `pub` items and the core could not be tested from outside. `popover.rs` was 1800 lines with four functions longer than 100 lines, presets were stored twice, and diagnostics with unsafe FFI ended up in the release binary.

## Decision

**Lints** live in `Cargo.toml`, not in crate attributes:

- `clippy::all = deny`, `clippy::pedantic = warn`; `make lint` runs clippy with `-D warnings`, so any finding is an error.
- Only three lints are allowed globally, each with the reason in a comment: `unreadable_literal` (`0xRRGGBB` colours), `must_use_candidate` and `return_self_not_must_use` (an application crate, where `#[must_use]` on every getter is noise).
- In production code `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, `dbg!` and `eprintln!` warn; they are allowed in tests (`clippy.toml`). The only place that writes to stderr is `diag::report`.
- Local exceptions only through `#[expect(clippy::…, reason = "…")]`: if the reason goes away, the compiler says so.
- The `block v0.1.6` warning (future-incompat) arrives transitively through `gpui-kit → gpui-pre-macos → gpui-pre-apple → cocoa → block`. It cannot be fixed locally; `-D warnings` does not apply to dependencies.

**lib + bin.** `src/lib.rs` declares `pub mod domain; pub mod theme;` and the private `gui`, `macos`, `diag`; `main.rs` calls `countdown_timer_bar::run()`. This gives `tests/` and doc tests for the core. The cost: in a library `rustc` does not warn about unused `pub` items, so by default everything inside `gui` and `macos` is private or `pub(super)`, and unused core code is caught by the tests.

**Layers.**

- `domain`: the countdown, durations, presets (`PresetList` owns slots, ids and the limit of 8), settings. No GPUI or AppKit. Zero durations are unrepresentable (`NonZeroU32`), the countdown has three phases (`Phase`), and there is no separate `Finished` state.
- `theme`: palettes, metrics, application to `gpui-component` (ADR 0004).
- `gui`: `app.rs` (entry point and bindings), `state.rs` (`AppState`, fields closed), `panel.rs` (the popover window), `popover/` (views), `gesture.rs`, `inline_edit.rs`, `timer_circle.rs`.
- `macos`: everything that knows about AppKit. `MainThreadMarker` is passed in as a parameter instead of being taken with `expect`.

**Popover state** is a single value `Interaction { Idle, Press, Edit }`; a press on the edit field lives inside `Edit`, so invalid combinations are unrepresentable.

**Deliberately not done:** `thiserror` for `ParseError`. The message depends on two fields (the token and the reason), a hand-written `Display` takes 20 lines, and that is not worth a new dependency.

## Verification

`make check` (= `fmt-check`, `lint`, `test`). Appearance is verified by state snapshots in both themes and by interaction scenarios (ADR 0006).
