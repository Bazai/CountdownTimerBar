// GPUI opens a pop-up as a titled panel, so AppKit draws its own outline and
// shadow next to the card's border (a double border). A borderless window leaves
// only the card GPUI draws; the window background must be transparent.

use objc2::rc::Retained;
use objc2_app_kit::{NSView, NSWindow, NSWindowStyleMask};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// `NSWindowStyleMaskNonactivatingPanel`, which `objc2-app-kit` does not name.
const NONACTIVATING_PANEL: NSWindowStyleMask = NSWindowStyleMask::from_bits_retain(1 << 7);

pub fn borderless_popover_mask() -> NSWindowStyleMask {
    NSWindowStyleMask::Borderless | NONACTIVATING_PANEL
}

pub(super) fn native_window(window: &gpui_kit::Window) -> Option<(&NSView, Retained<NSWindow>)> {
    let handle = HasWindowHandle::window_handle(window).ok()?;
    let RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        return None;
    };
    // SAFETY: `ns_view` is the NSView GPUI created for this window; it stays
    // alive for as long as the window does, and we only borrow it for that long.
    let view: &NSView = unsafe { appkit.ns_view.cast::<NSView>().as_ref() };
    let ns_window = view.window()?;
    Some((view, ns_window))
}

/// Returns `false` while the native window is not reachable yet.
pub fn make_borderless(window: &gpui_kit::Window) -> bool {
    let Some((view, ns_window)) = native_window(window) else {
        return false;
    };

    let was_key = ns_window.isKeyWindow();
    ns_window.setStyleMask(borderless_popover_mask());
    // No AppKit shadow: the window server would also draw a 1 px outline around
    // the alpha shape, which reads as a second border next to the card's own.
    ns_window.setHasShadow(false);
    if was_key {
        // Changing the style mask makes AppKit resign key status and the
        // first responder, so keyboard input would stop reaching GPUI.
        ns_window.makeKeyAndOrderFront(None);
        ns_window.makeFirstResponder(Some(view));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_is_borderless_but_stays_a_nonactivating_panel() {
        let mask = borderless_popover_mask();
        assert!(!mask.contains(NSWindowStyleMask::Titled));
        assert!(!mask.contains(NSWindowStyleMask::Closable));
        assert!(mask.contains(NONACTIVATING_PANEL));
    }
}
