//! In-memory harness for UI tests: the real popover over a manual clock, an
//! in-memory store and spies for the platform side effects.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui_kit::{
    point, px, size, AnyWindowHandle, App, AppContext as _, Bounds, Entity, WindowBounds,
    WindowOptions,
};

use crate::domain::countdown::{Clock, Event, Phase};
use crate::domain::preset::PresetKind;
use crate::domain::preset_list::format_list;
use crate::domain::settings::KeyValueStore;
use crate::gui::panel::Panel;
use crate::gui::ports::{Notifier, StatusDisplay};
use crate::gui::state::AppState;
use crate::macos::status_item::StatusItemAnchor;
use crate::theme::{self, Appearance};

/// The asset source the app uses, for harnesses that build their own context.
pub fn assets() -> impl gpui_kit::AssetSource {
    crate::gui::assets::AppAssets
}

/// A clock that only moves when told to.
#[derive(Clone)]
pub struct ManualClock(Rc<Cell<Instant>>);

impl ManualClock {
    pub fn new() -> Self {
        Self(Rc::new(Cell::new(Instant::now())))
    }

    pub fn advance(&self, by: Duration) {
        self.0.set(self.0.get() + by);
    }
}

impl Default for ManualClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Instant {
        self.0.get()
    }
}

#[derive(Default)]
struct StoreData {
    arrays: HashMap<String, Vec<String>>,
    flags: HashMap<String, bool>,
}

/// A `KeyValueStore` in memory; clones share the same data.
#[derive(Clone, Default)]
pub struct MemoryStore(Rc<RefCell<StoreData>>);

impl MemoryStore {
    /// A store that already holds `values` under `key`.
    #[must_use]
    pub fn with_array(mut self, key: &str, values: &[&str]) -> Self {
        let values: Vec<String> = values.iter().map(ToString::to_string).collect();
        self.set_string_array(key, &values);
        self
    }

    pub fn array(&self, key: &str) -> Option<Vec<String>> {
        self.get_string_array(key)
    }
}

impl KeyValueStore for MemoryStore {
    fn get_string_array(&self, key: &str) -> Option<Vec<String>> {
        self.0.borrow().arrays.get(key).cloned()
    }

    fn get_bool(&self, key: &str) -> bool {
        self.0.borrow().flags.get(key).copied().unwrap_or(false)
    }

    fn set_string_array(&mut self, key: &str, values: &[String]) {
        self.0
            .borrow_mut()
            .arrays
            .insert(key.to_owned(), values.to_vec());
    }

    fn set_bool(&mut self, key: &str, value: bool) {
        self.0.borrow_mut().flags.insert(key.to_owned(), value);
    }
}

#[derive(Default)]
struct Log {
    labels: Vec<(String, bool)>,
    events: Vec<(Event, bool)>,
}

/// Records what the app would show in the menu bar and notify.
#[derive(Clone, Default)]
struct Spy(Rc<RefCell<Log>>);

impl StatusDisplay for Spy {
    fn set_label(&mut self, text: &str, dimmed: bool) {
        self.0.borrow_mut().labels.push((text.to_owned(), dimmed));
    }

    fn redraw_for_appearance(&mut self) {}

    fn screen_anchor(&self) -> Option<StatusItemAnchor> {
        None
    }

    fn remove(self: Box<Self>) {}
}

impl Notifier for Spy {
    fn on_event(&mut self, event: &Event, sound_on: bool) {
        self.0.borrow_mut().events.push((*event, sound_on));
    }
}

/// The popover open in a window, with handles to observe and drive it.
pub struct Session {
    pub window: AnyWindowHandle,
    pub clock: ManualClock,
    pub store: MemoryStore,
    state: Entity<AppState>,
    log: Rc<RefCell<Log>>,
}

impl Session {
    /// Opens the popover in a 264 pt wide window in `appearance`.
    ///
    /// # Panics
    ///
    /// If the test platform cannot open a window.
    #[expect(
        clippy::expect_used,
        reason = "a harness that cannot open its window has nothing to test"
    )]
    pub fn open(cx: &mut App, appearance: Appearance, store: MemoryStore) -> Self {
        gpui_kit::init(cx);
        theme::set_appearance(cx, appearance);
        crate::gui::app::bind_keys(cx);
        cx.set_global(crate::gui::about::AboutOverride {
            version: "1.2.3".into(),
            copyright: "2025-2026 Test Author".into(),
        });
        let clock = ManualClock::new();
        let spy = Spy::default();
        let log = spy.0.clone();
        let state = {
            let (store, clock, spy) = (store.clone(), clock.clone(), spy);
            cx.new(|cx| {
                AppState::with_parts(
                    Box::new(store),
                    Rc::new(clock),
                    Some(Box::new(spy.clone())),
                    Box::new(spy),
                    None,
                    cx,
                )
            })
        };
        let visible = Bounds::new(point(px(0.), px(0.)), size(px(1440.), px(900.)));
        let panel_state = state.clone();
        let (window, _) = gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    point(px(0.), px(0.)),
                    size(px(264.), px(640.)),
                ))),
                ..WindowOptions::default()
            },
            cx,
            move |window, cx| cx.new(|cx| Panel::new(panel_state, visible, window, cx)),
        )
        .expect("the test platform opens windows");
        Self {
            window,
            clock,
            store,
            state,
            log,
        }
    }

    pub fn phase(&self, cx: &App) -> Phase {
        self.state.read(cx).phase()
    }

    /// The clock text shown in the header.
    pub fn clock_label(&self, cx: &App) -> String {
        self.state.read(cx).clock_label()
    }

    /// The column as the settings input shows it, e.g. `15, 30, 35`.
    pub fn column_text(&self, cx: &App, kind: PresetKind) -> String {
        format_list(self.state.read(cx).specs(kind))
    }

    /// The last label and dimming sent to the menu-bar item.
    pub fn status_label(&self) -> Option<(String, bool)> {
        self.log.borrow().labels.last().cloned()
    }

    /// Every countdown event the notifier saw, with the sound setting at the time.
    pub fn events(&self) -> Vec<(Event, bool)> {
        self.log.borrow().events.clone()
    }
}

/// Low-level pointer events, for states a click or drag helper cannot hold
/// (for example a knob drag in progress).
pub mod pointer {
    use gpui_kit::test::TestWindowExt as _;
    use gpui_kit::{
        App, InputEvent as _, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels,
        Point, Window,
    };

    pub fn down(window: &mut Window, at: Point<Pixels>, cx: &mut App) {
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: at,
                modifiers: gpui_kit::Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        window.render_frame(cx);
    }

    /// Moves the pointer with the left button held.
    pub fn drag_to(window: &mut Window, at: Point<Pixels>, cx: &mut App) {
        window.dispatch_event(
            MouseMoveEvent {
                position: at,
                pressed_button: Some(MouseButton::Left),
                modifiers: gpui_kit::Modifiers::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.render_frame(cx);
    }

    pub fn up(window: &mut Window, at: Point<Pixels>, cx: &mut App) {
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: at,
                modifiers: gpui_kit::Modifiers::default(),
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
        window.render_frame(cx);
    }
}
