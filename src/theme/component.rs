//! Maps the palette onto the `gpui-component` theme: the one place that knows
//! about that library's theme slots. Widgets from the library (Switch, Input,
//! Button, tooltips) then take our colours.

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::App;

use super::{palette::Palette, Appearance};

pub fn apply(cx: &mut App, palette: &Palette, appearance: Appearance) {
    let p = *palette;
    let mode = match appearance {
        Appearance::Dark => ThemeMode::Dark,
        Appearance::Light => ThemeMode::Light,
    };
    Theme::change(mode, None, cx);
    Theme::update(cx, |theme| {
        // `gpui_kit::open_window` wraps the view in `Root`, which paints this colour
        // over the whole window. The window must stay transparent so only the rounded
        // card of `NativePopoverFrame` shows.
        theme.background = gpui_kit::transparent_black();
        theme.popover = p.popover_bg.hsla();
        theme.popover_foreground = p.text.hsla();
        theme.foreground = p.text.hsla();
        theme.muted_foreground = p.text_muted.hsla();
        theme.border = p.control_border.hsla();
        theme.input = p.input_bg.hsla();
        theme.primary = p.accent.hsla();
        theme.primary_hover = p.accent.hsla();
        theme.primary_active = p.accent.hsla();
        theme.ring = p.accent.hsla();
        theme.caret = p.accent.hsla();
        theme.danger = p.danger.hsla();
        theme.secondary = p.control.hsla();
        theme.secondary_hover = p.control_hover.hsla();
        theme.secondary_active = p.control_hover.hsla();
    });
}
