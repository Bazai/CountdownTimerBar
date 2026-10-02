// GPUI sets the activation policy to `Regular` in `applicationDidFinishLaunching`,
// which overrides `LSUIElement` and brings the Dock icon back; setting
// `Accessory` from the startup callback restores it.

use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;

/// Makes the process an accessory app (no Dock icon, no menu bar). Main thread
/// only. Returns whether `AppKit` accepted the policy.
pub fn hide_dock_icon() -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    NSApplication::sharedApplication(mtm)
        .setActivationPolicy(NSApplicationActivationPolicy::Accessory)
}
