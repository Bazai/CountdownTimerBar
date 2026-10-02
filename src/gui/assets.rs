//! Asset source: the bundled `gpui_kit` icons plus the app's own glyphs.

use std::borrow::Cow;

use gpui_kit::assets::Assets;
use gpui_kit::{AssetSource, Result, SharedString};

pub const TIMER_GLYPH: &str = "icons/timer-glyph.svg";

const TIMER_GLYPH_SVG: &[u8] = include_bytes!("../../assets/icons/timer-glyph.svg");

/// The default `gpui_kit` bundle does not include a timer icon (only the full
/// catalog does), so the one glyph the app needs is embedded here.
pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == TIMER_GLYPH {
            return Ok(Some(Cow::Borrowed(TIMER_GLYPH_SVG)));
        }
        Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut items = Assets.list(path)?;
        if TIMER_GLYPH.starts_with(path) {
            items.push(TIMER_GLYPH.into());
        }
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_the_timer_glyph_and_falls_back_to_the_bundle() {
        let glyph = AppAssets.load(TIMER_GLYPH).unwrap().expect("glyph");
        assert!(std::str::from_utf8(&glyph).unwrap().contains("<circle"));
        assert!(AppAssets.load("icons/settings.svg").unwrap().is_some());
    }
}
