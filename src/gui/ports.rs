//! What `AppState` needs from the platform, so tests can substitute it.

use crate::domain::countdown::Event;
use crate::macos::notifications::NotificationController;
use crate::macos::status_item::{StatusItemAnchor, StatusItemController};

/// The menu-bar item that shows the remaining time.
pub(crate) trait StatusDisplay {
    fn set_label(&mut self, text: &str, dimmed: bool);
    fn redraw_for_appearance(&mut self);
    fn screen_anchor(&self) -> Option<StatusItemAnchor>;
    fn remove(self: Box<Self>);
}

/// Notification and sound side effects of countdown events.
pub(crate) trait Notifier {
    fn on_event(&mut self, event: &Event, sound_on: bool);
}

impl StatusDisplay for StatusItemController {
    fn set_label(&mut self, text: &str, dimmed: bool) {
        Self::set_label(self, text, dimmed);
    }

    fn redraw_for_appearance(&mut self) {
        Self::redraw_for_appearance(self);
    }

    fn screen_anchor(&self) -> Option<StatusItemAnchor> {
        Self::screen_anchor(self)
    }

    fn remove(self: Box<Self>) {
        Self::remove(*self);
    }
}

impl Notifier for NotificationController {
    fn on_event(&mut self, event: &Event, sound_on: bool) {
        self.handle_event(event, sound_on);
    }
}
