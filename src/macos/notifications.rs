use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_app_kit::NSSound;
use objc2_foundation::{NSError, NSString};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotificationRequest,
    UNUserNotificationCenter,
};

use crate::domain::countdown::Event;

const FINISHED_NOTIFICATION_ID: &str = "baz.CountdownTimerBar.finished";

/// Owns the notification and sound side effects. `center` is created on first
/// use: `UNUserNotificationCenter.currentNotificationCenter` raises for an
/// executable without a bundle identifier (a bare `cargo run`).
pub struct NotificationController {
    center: Option<Retained<UNUserNotificationCenter>>,
}

impl NotificationController {
    pub fn new() -> Self {
        Self { center: None }
    }

    fn center(&mut self) -> Retained<UNUserNotificationCenter> {
        self.center
            .get_or_insert_with(UNUserNotificationCenter::currentNotificationCenter)
            .clone()
    }

    /// `Started` / `Stopped` cancel a pending notification. `Finished` requests
    /// authorization, posts the notification if granted and, independently of
    /// that, plays the sound if `sound_on`.
    pub fn handle_event(&mut self, event: &Event, sound_on: bool) {
        match event {
            Event::Started(_) | Event::Stopped => {
                if let Some(center) = &self.center {
                    center.removeAllPendingNotificationRequests();
                }
            }
            Event::Paused(_) | Event::Resumed(_) => {}
            Event::Finished(_) => {
                self.request_and_post_finished_notification();
                if sound_on {
                    play_glass_sound();
                }
            }
        }
    }

    fn request_and_post_finished_notification(&mut self) {
        let center = self.center();
        let post_center = center.clone();
        let options = UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound;
        let completion_handler = RcBlock::new(move |granted: Bool, error: *mut NSError| {
            if !granted.as_bool() {
                if !error.is_null() {
                    crate::diag::report(format_args!(
                        "Notification authorization request failed: {error:?}"
                    ));
                }
                return;
            }

            let content = UNMutableNotificationContent::new();
            content.setTitle(&NSString::from_str("Timer finished"));
            content.setBody(&NSString::from_str("Countdown complete."));
            content.setSound(None);

            let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
                &NSString::from_str(FINISHED_NOTIFICATION_ID),
                &content,
                None,
            );
            post_center.addNotificationRequest_withCompletionHandler(&request, None);
        });
        center.requestAuthorizationWithOptions_completionHandler(options, &completion_handler);
    }
}

/// Plays the "Glass" system sound; does nothing if it is not found.
fn play_glass_sound() {
    if let Some(sound) = NSSound::soundNamed(&NSString::from_str("Glass")) {
        sound.play();
    }
}
