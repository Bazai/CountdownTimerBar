//! The popover's window: placement under the status item, the root view and
//! closing when the window loses focus.

use gpui_kit::prelude::*;
use gpui_kit::{
    div, point, px, size, Bounds, Context, Entity, FocusHandle, IntoElement, Pixels, Render,
    WeakEntity, Window,
};

use crate::theme::metrics;

use super::popover::{TabNext, TabPrev};
use super::state::AppState;

const INITIAL_POPOVER_HEIGHT: f32 = 1.;

pub fn watch_panel_activation<T: 'static>(
    window: &mut Window,
    cx: &mut Context<T>,
    state: WeakEntity<AppState>,
) {
    cx.observe_window_activation(window, move |_, window, cx| {
        if !window.is_window_active() {
            let state = state.clone();
            // GPUI delivers resign / become-key as separate foreground tasks; check
            // activity after them.
            cx.spawn(async move |_, cx| {
                let _ = state.update(cx, AppState::dismiss_if_inactive);
            })
            .detach();
        }
    })
    .detach();
}

pub(crate) struct Panel {
    popover: Entity<super::popover::Popover>,
    focus: FocusHandle,
}

impl Panel {
    pub(crate) fn new(
        state: Entity<AppState>,
        visible_bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        watch_panel_activation(window, cx, state.downgrade());
        // Drop the AppKit window frame so only the popover's own rounded
        // border is visible; retry once a frame later if the NSWindow is not
        // attached to the view yet.
        if !crate::macos::popover_window::make_borderless(window) {
            window.defer(cx, |window, _| {
                crate::macos::popover_window::make_borderless(window);
            });
        }
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let popover = cx.new(|cx| super::popover::Popover::new(state, visible_bounds, window, cx));
        Self { popover, focus }
    }
}

impl Render for Panel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Tab handling sits here, above the popover, so it also works while
        // nothing inside the popover has focus yet.
        div()
            .size_full()
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &TabNext, window, cx| {
                this.popover
                    .update(cx, |popover, cx| popover.tab(1, window, cx));
            }))
            .on_action(cx.listener(|this, _: &TabPrev, window, cx| {
                this.popover
                    .update(cx, |popover, cx| popover.tab(-1, window, cx));
            }))
            .child(self.popover.clone())
    }
}

pub(super) fn panel_bounds(anchor: Bounds<Pixels>, visible: Bounds<Pixels>) -> Bounds<Pixels> {
    let panel_size = size(px(metrics::POPOVER_WIDTH), px(INITIAL_POPOVER_HEIGHT));
    let x = (anchor.center().x - panel_size.width / 2.)
        .min(visible.right() - panel_size.width)
        .max(visible.left());
    Bounds::new(point(x, anchor.bottom()), panel_size)
}

#[cfg(test)]
mod tests {
    use gpui_kit::{point, px, size, WindowOptions};

    use super::*;

    #[test]
    fn panel_is_centered_below_the_status_button() {
        let anchor = Bounds::new(point(px(500.), px(0.)), size(px(60.), px(24.)));
        let visible = Bounds::new(point(px(0.), px(24.)), size(px(1440.), px(876.)));

        assert_eq!(
            panel_bounds(anchor, visible),
            Bounds::new(point(px(398.), px(24.)), size(px(264.), px(1.)))
        );
    }

    #[test]
    fn measured_popover_height_preserves_top_anchor_and_clamps_to_screen() {
        let current = Bounds::new(point(px(900.), px(500.)), size(px(300.), px(200.)));
        let visible = Bounds::new(point(px(0.), px(24.)), size(px(1000.), px(700.)));

        assert_eq!(
            super::super::native_popover::resized_bounds(current, visible, px(360.)),
            Some(Bounds::new(
                point(px(700.), px(364.)),
                size(px(300.), px(360.))
            ))
        );
        assert_eq!(
            super::super::native_popover::resized_bounds(current, visible, px(200.5)),
            None
        );
    }

    #[test]
    fn panel_stays_inside_the_right_screen_edge() {
        let anchor = Bounds::new(point(px(1380.), px(0.)), size(px(60.), px(24.)));
        let visible = Bounds::new(point(px(0.), px(24.)), size(px(1440.), px(876.)));

        assert_eq!(
            panel_bounds(anchor, visible).origin,
            point(px(1176.), px(24.))
        );
    }

    #[test]
    fn panel_respects_the_visible_left_screen_edge() {
        let anchor = Bounds::new(point(px(15.), px(0.)), size(px(60.), px(24.)));
        let visible = Bounds::new(point(px(80.), px(24.)), size(px(1360.), px(876.)));

        assert_eq!(
            panel_bounds(anchor, visible).origin,
            point(px(80.), px(24.))
        );
    }

    // `AppState::toggle_panel` and `dismiss_panels` open and close the window from
    // outside GPUI's own update, via `cx.defer`. These two tests pin down why: both
    // operations panic with a double lease when done synchronously inside
    // `AppState`'s own update. If GPUI ever stops panicking, the deferrals can go.
    // `AppState` cannot be built in a unit test (live `NSStatusItem`), so a
    // stand-in with the same shape is used.
    struct Source;
    struct Child(gpui_kit::Entity<Source>);

    impl Render for Child {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let _ = self.0.read(cx);
            div()
        }
    }

    #[gpui_kit::test]
    fn opening_a_window_inside_the_owners_update_panics(cx: &mut gpui_kit::TestAppContext) {
        let source = cx.update(|cx| cx.new(|_| Source));
        let child_source = source.clone();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cx.update(|cx| {
                source.update(cx, |_this, cx| {
                    let child_source = child_source.clone();
                    gpui_kit::open_window(WindowOptions::default(), cx, move |_, cx| {
                        cx.new(|_| Child(child_source))
                    })
                    .unwrap();
                });
            });
        }));

        assert!(
            result.is_err(),
            "expected the double-lease panic this test reproduces — if this no longer \
             panics, GPUI's re-entrancy behavior changed and the cx.defer fix below may no \
             longer be necessary"
        );
    }

    // `dismiss_panels` removes windows inside `AppState`'s update, and removal
    // synchronously fires `on_window_closed` handlers; a handler that updates the
    // entity directly is a second, nested lease.
    #[gpui_kit::test]
    fn closing_a_window_inside_the_owners_update_panics_when_the_handler_reenters(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        let source = cx.update(|cx| cx.new(|_| Source));
        let child_source = source.clone();
        let (window, _) = cx
            .update(|cx| {
                gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
                    cx.new(|_| Child(child_source))
                })
            })
            .unwrap();

        let handler_source = source.clone();
        cx.update(|cx| {
            cx.on_window_closed(move |cx, _closed| {
                let () = handler_source.update(cx, |_this, _cx| {});
            })
            .detach();
        });

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cx.update(|cx| {
                source.update(cx, |_this, cx| {
                    let _ = window.update(cx, |_, win, _| win.remove_window());
                });
            });
        }));

        assert!(
            result.is_err(),
            "expected removing a window from inside self.update() to panic when a registered \
             on_window_closed handler synchronously re-enters the same entity — this is the \
             real crash dismiss_panels triggers via AppState::new's on_window_closed handler"
        );
    }
}
