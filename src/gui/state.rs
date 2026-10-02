//! Shared application state: settings, the countdown, the status item and the
//! popover window's lifecycle.

use std::rc::Rc;
use std::time::Duration;

use gpui_kit::prelude::*;
use gpui_kit::{
    AnyWindowHandle, Context, Task, WindowBackgroundAppearance, WindowBounds, WindowKind,
    WindowOptions,
};
use objc2_foundation::MainThreadMarker;

use crate::diag;
use crate::domain::countdown::{Clock, Countdown, Event, Phase, SystemClock};
use crate::domain::duration::{format_clock_fixed, uses_hours};
use crate::domain::preset::{IdGen, Preset, PresetId, PresetKind, PresetList};
use crate::domain::preset_list::PresetSpec;
use crate::domain::settings::{
    load_settings, save_presets, save_sound_on, KeyValueStore, Settings,
};
use crate::macos::appearance::SystemAppearanceObserver;
use crate::macos::notifications::NotificationController;
use crate::macos::status_item::StatusItemController;
use crate::macos::user_defaults_store::UserDefaultsStore;
use crate::theme;

use super::panel::{panel_bounds, Panel};
use super::ports::{Notifier, StatusDisplay};

/// Shared GPUI entity; closing the window neither stops the timer nor resets settings.
pub struct AppState {
    settings: Settings,
    /// Editable presets `[Focus, Rest]`. The active timer, highlight and edit lock go
    /// by slot id, not by value.
    lists: [PresetList; 2],
    ids: IdGen,
    /// Clock format (`h:mm:ss`), fixed when a timer starts and kept for the whole run.
    clock_hours: bool,
    countdown: Countdown<Rc<dyn Clock>>,
    store: Box<dyn KeyValueStore>,
    panel_window: Option<AnyWindowHandle>,
    status_item: Option<Box<dyn StatusDisplay>>,
    notifier: Box<dyn Notifier>,
    _appearance_observer: Option<SystemAppearanceObserver>,
    _tick_task: Task<()>,
}

impl AppState {
    /// The app as shipped: `NSUserDefaults`, the system clock, the real status
    /// item, notifications and the appearance observer.
    pub(super) fn new(mtm: MainThreadMarker, cx: &mut Context<Self>) -> Self {
        let state = cx.weak_entity();
        let async_cx = cx.to_async();
        let status_item = StatusItemController::new(mtm, move || {
            let state = state.clone();
            // AppKit calls target/action outside GPUI; hop back onto its executor.
            async_cx
                .spawn(async move |cx| {
                    let _ = state.update(cx, Self::toggle_panel);
                })
                .detach();
        });
        let appearance_state = cx.weak_entity();
        let appearance_cx = cx.to_async();
        let appearance_observer = SystemAppearanceObserver::new(mtm, move || {
            let state = appearance_state.clone();
            appearance_cx
                .spawn(async move |cx| {
                    let _ = state.update(cx, Self::appearance_changed);
                })
                .detach();
        });
        Self::with_parts(
            Box::new(UserDefaultsStore::standard()),
            Rc::new(SystemClock),
            Some(Box::new(status_item)),
            Box::new(NotificationController::new()),
            Some(appearance_observer),
            cx,
        )
    }

    /// The app over substitutable parts; tests pass in-memory ones.
    pub(crate) fn with_parts(
        store: Box<dyn KeyValueStore>,
        clock: Rc<dyn Clock>,
        status_item: Option<Box<dyn StatusDisplay>>,
        notifier: Box<dyn Notifier>,
        appearance_observer: Option<SystemAppearanceObserver>,
        cx: &mut Context<Self>,
    ) -> Self {
        let settings = load_settings(&*store);
        let tick_task = cx.spawn(async move |state, cx| loop {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            if state.update(cx, Self::tick).is_err() {
                break;
            }
        });

        let state = cx.weak_entity();
        cx.on_window_closed(move |cx, closed| {
            let state = state.clone();
            // `remove_window()` fires this synchronously while `dismiss_panels` holds
            // the lease on `AppState`; updating directly would double-lease and panic.
            cx.defer(move |cx| {
                let _ = state.update(cx, |state, cx| {
                    if state
                        .panel_window
                        .is_some_and(|panel| panel.window_id() == closed)
                    {
                        state.dismiss_panels(cx);
                    }
                });
            });
        })
        .detach();

        let mut ids = IdGen::default();
        let lists = [PresetKind::Focus, PresetKind::Rest]
            .map(|kind| PresetList::new(settings.preset_specs(kind), &mut ids));
        Self {
            settings,
            lists,
            ids,
            clock_hours: false,
            countdown: Countdown::new(clock),
            store,
            panel_window: None,
            status_item,
            notifier,
            _appearance_observer: appearance_observer,
            _tick_task: tick_task,
        }
    }

    fn tick(&mut self, cx: &mut Context<Self>) {
        if let Some(event) = self.countdown.tick() {
            self.handle_event(&event);
        }
        self.sync_status_item();
        cx.notify();
    }

    fn sync_status_item(&mut self) {
        let label = self.clock_label();
        let dimmed = self.countdown.phase() == Phase::Paused;
        if let Some(status_item) = &mut self.status_item {
            status_item.set_label(&label, dimmed);
        }
    }

    fn appearance_changed(&mut self, cx: &mut Context<Self>) {
        theme::refresh(cx);
        if let Some(status_item) = &mut self.status_item {
            status_item.redraw_for_appearance();
        }
        cx.notify();
    }

    pub fn phase(&self) -> Phase {
        self.countdown.phase()
    }

    pub fn active_preset(&self) -> Option<Preset> {
        self.countdown.active_preset()
    }

    pub fn sound_on(&self) -> bool {
        self.settings.sound_on
    }

    pub fn progress(&self) -> f32 {
        self.countdown.remaining_fraction()
    }

    pub fn clock_label(&self) -> String {
        format_clock_fixed(self.countdown.remaining(), self.clock_hours)
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Started(preset) => self.clock_hours = uses_hours(preset.duration.as_secs()),
            Event::Stopped | Event::Finished(_) => self.clock_hours = false,
            Event::Paused(_) | Event::Resumed(_) => {}
        }
        self.notifier.on_event(event, self.settings.sound_on);
    }

    fn apply(&mut self, event: Option<Event>, cx: &mut Context<Self>) {
        if let Some(event) = event {
            self.handle_event(&event);
        }
        self.sync_status_item();
        cx.notify();
    }

    pub fn press_preset(&mut self, preset: Preset, cx: &mut Context<Self>) {
        let event = self.countdown.press(preset);
        self.apply(event, cx);
    }

    pub fn pause_or_resume(&mut self, cx: &mut Context<Self>) {
        let event = if self.countdown.phase() == Phase::Paused {
            self.countdown.resume()
        } else {
            self.countdown.pause()
        };
        self.apply(event, cx);
    }

    pub fn stop_countdown(&mut self, cx: &mut Context<Self>) {
        let event = self.countdown.stop();
        self.apply(event, cx);
    }

    fn list(&self, kind: PresetKind) -> &PresetList {
        &self.lists[spec_index(kind)]
    }

    pub fn specs(&self, kind: PresetKind) -> &[PresetSpec] {
        self.list(kind).specs()
    }

    pub fn is_full(&self, kind: PresetKind) -> bool {
        self.list(kind).is_full()
    }

    pub fn preset(&self, kind: PresetKind, index: usize) -> Option<Preset> {
        self.list(kind).preset(kind, index)
    }

    pub fn index_of(&self, kind: PresetKind, id: PresetId) -> Option<usize> {
        self.list(kind).index_of(id)
    }

    /// Whether this exact slot is active; twins with the same value do not count.
    pub fn is_slot_active(&self, kind: PresetKind, index: usize) -> bool {
        match (self.preset(kind, index), self.countdown.active_preset()) {
            (Some(slot), Some(active)) => slot.id == active.id,
            _ => false,
        }
    }

    fn persist(&mut self, kind: PresetKind, cx: &mut Context<Self>) {
        let durations = self.list(kind).durations();
        save_presets(&mut *self.store, kind, &durations);
        cx.notify();
    }

    /// Replaces the whole list (text input) and persists it. Ids of surviving presets
    /// carry over, so the running timer keeps its highlight.
    pub fn set_specs(&mut self, kind: PresetKind, specs: Vec<PresetSpec>, cx: &mut Context<Self>) {
        self.lists[spec_index(kind)].replace_all(specs, &mut self.ids);
        self.persist(kind, cx);
    }

    pub fn replace_spec_at(
        &mut self,
        kind: PresetKind,
        index: usize,
        spec: PresetSpec,
        cx: &mut Context<Self>,
    ) {
        if self.lists[spec_index(kind)].replace_at(index, spec) {
            self.persist(kind, cx);
        }
    }

    pub fn remove_spec_at(&mut self, kind: PresetKind, index: usize, cx: &mut Context<Self>) {
        if self.lists[spec_index(kind)].remove_at(index) {
            self.persist(kind, cx);
        }
    }

    pub fn push_spec(&mut self, kind: PresetKind, spec: PresetSpec, cx: &mut Context<Self>) {
        if self.lists[spec_index(kind)]
            .push(spec, &mut self.ids)
            .is_ok()
        {
            self.persist(kind, cx);
        }
    }

    pub fn toggle_sound(&mut self, cx: &mut Context<Self>) {
        self.settings.sound_on = !self.settings.sound_on;
        save_sound_on(&mut *self.store, self.settings.sound_on);
        cx.notify();
    }

    fn toggle_panel(&mut self, cx: &mut Context<Self>) {
        if self.panel_window.is_some() {
            self.dismiss_panels(cx);
            return;
        }
        let Some(status_item) = &self.status_item else {
            return;
        };
        let Some(anchor) = status_item.screen_anchor() else {
            diag::report("Cannot locate the status item button in the menu bar");
            return;
        };
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(panel_bounds(
                anchor.bounds,
                anchor.visible_bounds,
            ))),
            display_id: Some(anchor.display_id),
            titlebar: None,
            kind: WindowKind::PopUp,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            ..WindowOptions::default()
        };
        let state = cx.entity();
        let panel_state = state.clone();
        // `open_window` draws the root view synchronously and that view reads this
        // entity, so calling it inside this `update()` double-leases and panics (see
        // `panel::tests`). Defer it and store the result in a separate update.
        cx.defer(move |cx| {
            let opened = gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| Panel::new(panel_state, anchor.visible_bounds, window, cx))
            });
            state.update(cx, |state, _cx| match opened {
                Ok((window, _)) => state.panel_window = Some(window),
                Err(error) => diag::report(format_args!("Cannot open the timer panel: {error:#}")),
            });
        });
    }

    pub fn quit(&mut self, cx: &mut Context<Self>) {
        self.dismiss_panels(cx);
        if let Some(status_item) = self.status_item.take() {
            status_item.remove();
        }
        cx.quit();
    }

    pub fn dismiss_panels(&mut self, cx: &mut Context<Self>) {
        if let Some(handle) = self.panel_window.take() {
            // The system may have closed the window before this deferred callback ran.
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        }
    }

    pub(super) fn dismiss_if_inactive(&mut self, cx: &mut Context<Self>) {
        let active = self.panel_window.is_some_and(|handle| {
            handle
                .update(cx, |_, window, _| window.is_window_active())
                .unwrap_or(false)
        });
        if !active {
            self.dismiss_panels(cx);
        }
    }
}

fn spec_index(kind: PresetKind) -> usize {
    match kind {
        PresetKind::Focus => 0,
        PresetKind::Rest => 1,
    }
}
