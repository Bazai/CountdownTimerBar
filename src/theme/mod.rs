//! The app's theme and design tokens, and the single source of truth for them.
//!
//! - [`color`]: the `Rgb` value type and colour maths.
//! - [`palette`]: semantic colour roles (`Palette`).
//! - [`metrics`]: sizes, radii, opacity levels, type sizes.
//! - `component` (private): applies the palette to the `gpui-component` theme.
//!
//! Views read colours from the active palette (`cx.palette()`) and sizes from
//! [`metrics`]; they never write a hex value.

pub mod color;
pub mod metrics;
pub mod palette;

mod component;

use gpui_kit::{App, Global, WindowAppearance};

pub use palette::Palette;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appearance {
    Dark,
    Light,
}

impl Appearance {
    pub fn palette(self) -> Palette {
        match self {
            Self::Dark => Palette::DARK,
            Self::Light => Palette::light(),
        }
    }

    /// Vibrant variants (menus, popovers) follow their plain counterpart.
    pub fn from_window_appearance(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::Dark,
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::Light,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AppTheme {
    pub palette: Palette,
}

impl Global for AppTheme {}

fn current_appearance(cx: &App) -> Appearance {
    Appearance::from_window_appearance(cx.window_appearance())
}

/// Call once at startup, after `gpui_kit::init`.
pub fn init(cx: &mut App) {
    refresh(cx);
}

pub fn refresh(cx: &mut App) {
    let appearance = current_appearance(cx);
    set_appearance(cx, appearance);
}

pub fn set_appearance(cx: &mut App, appearance: Appearance) {
    let palette = appearance.palette();
    cx.set_global(AppTheme { palette });
    component::apply(cx, &palette, appearance);
    cx.refresh_windows();
}

pub trait ThemeExt {
    fn palette(&self) -> &Palette;
}

impl ThemeExt for App {
    fn palette(&self) -> &Palette {
        &self.global::<AppTheme>().palette
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_appearance_maps_to_a_palette_choice() {
        use WindowAppearance::*;
        assert_eq!(Appearance::from_window_appearance(Dark), Appearance::Dark);
        assert_eq!(
            Appearance::from_window_appearance(VibrantDark),
            Appearance::Dark
        );
        assert_eq!(Appearance::from_window_appearance(Light), Appearance::Light);
        assert_eq!(
            Appearance::from_window_appearance(VibrantLight),
            Appearance::Light
        );
    }

    #[test]
    fn each_appearance_has_its_own_palette() {
        assert_eq!(Appearance::Dark.palette(), Palette::DARK);
        assert_eq!(Appearance::Light.palette(), Palette::light());
    }
}
