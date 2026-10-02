//! Observes the system appearance (`NSApp.effectiveAppearance`) through KVO.

use std::ffi::c_void;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{define_class, msg_send, AnyThread, DefinedClass};
use objc2_app_kit::NSApplication;
use objc2_foundation::{
    MainThreadMarker, NSKeyValueObservingOptions, NSObjectNSKeyValueObserverRegistration, NSString,
};

const EFFECTIVE_APPEARANCE_KEY_PATH: &str = "effectiveAppearance";
static APPEARANCE_OBSERVER_CONTEXT: u8 = 0;

fn appearance_observer_context() -> *mut c_void {
    (&raw const APPEARANCE_OBSERVER_CONTEXT).cast_mut().cast()
}

struct AppearanceObserverIvars {
    on_change: Box<dyn Fn()>,
}

define_class!(
    // `NSApplication` does not retain a KVO observer; the Rust owner keeps it alive.
    //
    // SAFETY: `NSObject` has no subclassing requirements and the ivar is released
    // with the object.
    #[unsafe(super(NSObject))]
    #[ivars = AppearanceObserverIvars]
    struct AppearanceObserver;

    impl AppearanceObserver {
        #[unsafe(method(observeValueForKeyPath:ofObject:change:context:))]
        fn observe_value_for_key_path(
            &self,
            _key_path: Option<&NSString>,
            _object: Option<&AnyObject>,
            _change: Option<&AnyObject>,
            context: *mut c_void,
        ) {
            if context == appearance_observer_context() {
                (self.ivars().on_change)();
            }
        }
    }
);

impl AppearanceObserver {
    fn new(on_change: Box<dyn Fn()>) -> Retained<Self> {
        let observer = Self::alloc().set_ivars(AppearanceObserverIvars { on_change });
        // SAFETY: `NSObject`'s initializer has no additional requirements.
        unsafe { msg_send![super(observer), init] }
    }
}

/// KVO registration for `NSApp.effectiveAppearance`; main thread only, removed on drop.
pub struct SystemAppearanceObserver {
    application: Retained<NSApplication>,
    observer: Retained<AppearanceObserver>,
    _main_thread: MainThreadMarker,
}

impl SystemAppearanceObserver {
    pub fn new(mtm: MainThreadMarker, on_change: impl Fn() + 'static) -> Self {
        let application = NSApplication::sharedApplication(mtm);
        let observer = AppearanceObserver::new(Box::new(on_change));
        let key_path = NSString::from_str(EFFECTIVE_APPEARANCE_KEY_PATH);

        // SAFETY: `observer` implements the exact KVO callback selector and
        // remains retained until this registration is removed in `Drop`.
        unsafe {
            application.addObserver_forKeyPath_options_context(
                &observer,
                &key_path,
                NSKeyValueObservingOptions::empty(),
                appearance_observer_context(),
            );
        }

        Self {
            application,
            observer,
            _main_thread: mtm,
        }
    }
}

impl Drop for SystemAppearanceObserver {
    fn drop(&mut self) {
        debug_assert!(MainThreadMarker::new().is_some());
        let key_path = NSString::from_str(EFFECTIVE_APPEARANCE_KEY_PATH);
        // SAFETY: this is the matching application, observer, key path and
        // context from `new`; the AppState owner drops it on the main thread.
        unsafe {
            self.application.removeObserver_forKeyPath_context(
                &self.observer,
                &key_path,
                appearance_observer_context(),
            );
        }
    }
}
