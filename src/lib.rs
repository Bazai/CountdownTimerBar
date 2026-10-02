//! `CountdownTimerBar`: a configurable countdown timer for the macOS status bar.
//!
//! [`domain`] is the pure model, [`theme`] the design tokens; the `GPUI` views
//! (`gui`) and the `AppKit` boundary (`macos`) are private. The binary only calls [`run`].

pub mod domain;
pub mod theme;

#[cfg(feature = "test-support")]
pub mod testing;

mod diag;
mod gui;
mod macos;

pub use gui::app::run;

/// The version from `Cargo.toml`, shown in the About card and stamped into the bundle.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The years and holder from the `LICENSE` notice, e.g. `2025-2026 Pavel Bubentsov`.
/// `make license` keeps the years current, and the About card shows this text.
#[must_use]
pub fn copyright() -> &'static str {
    include_str!("../LICENSE")
        .lines()
        .find_map(|line| line.strip_prefix("Copyright (c) "))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_license_notice_starts_with_a_year() {
        assert!(copyright().starts_with(|c: char| c.is_ascii_digit()));
    }

    #[test]
    fn the_license_notice_names_a_holder() {
        assert!(copyright()
            .split_once(' ')
            .is_some_and(|(_, holder)| !holder.is_empty()));
    }
}
