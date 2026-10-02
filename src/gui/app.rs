//! Application entry point: key bindings, global actions and the event loop.

use gpui_kit::prelude::*;
use gpui_kit::KeyBinding;

use objc2_foundation::MainThreadMarker;

use crate::diag;

use super::popover::{
    CancelAbout, CancelEdit, TabNext, TabPrev, ABOUT_KEY_CONTEXT, EDIT_KEY_CONTEXT,
};
use super::state::AppState;
use crate::theme;

gpui_kit::actions!(countdown_timer_bar, [DismissPanels, QuitApp]);

/// Key bindings shared by the app and the UI tests.
pub(crate) fn bind_keys(cx: &mut gpui_kit::App) {
    cx.bind_keys([
        KeyBinding::new("escape", DismissPanels, None),
        KeyBinding::new("cmd-q", QuitApp, None),
        // Esc inside the circle's inline edit cancels the edit only.
        KeyBinding::new("escape", CancelEdit, Some(EDIT_KEY_CONTEXT)),
        // Esc with the About card open closes the card, not the popover.
        KeyBinding::new("escape", CancelAbout, Some(ABOUT_KEY_CONTEXT)),
        KeyBinding::new("tab", TabNext, None),
        KeyBinding::new("shift-tab", TabPrev, None),
    ]);
}

pub fn run() {
    gpui_kit::application()
        .with_assets(super::assets::AppAssets)
        .run(|cx| {
            let Some(mtm) = MainThreadMarker::new() else {
                diag::report("GPUI did not start the app on the main thread");
                return;
            };
            // GPUI has just forced the Dock-visible policy; undo it first so the
            // icon never lingers (see macos::activation).
            if !crate::macos::activation::hide_dock_icon() {
                diag::report("Cannot hide the Dock icon");
            }
            gpui_kit::init(cx);
            theme::init(cx);
            let state = cx.new(|cx| AppState::new(mtm, cx));
            bind_keys(cx);
            // The global handler owns the entity so it outlives the windows.
            let quit_state = state.clone();
            cx.on_action(move |_: &DismissPanels, cx| {
                let state = state.clone();
                cx.defer(move |cx| state.update(cx, AppState::dismiss_panels));
            });
            cx.on_action(move |_: &QuitApp, cx| {
                let state = quit_state.clone();
                cx.defer(move |cx| state.update(cx, AppState::quit));
            });
        });
}
