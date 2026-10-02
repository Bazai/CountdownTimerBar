//! Settings section, the Focus / Rest text inputs and the About card.

use gpui_kit::component::{button::Button, input::Input, switch::Switch, Sizable as _, Size};
use gpui_kit::prelude::*;
use gpui_kit::TestSupportExt as _;
use gpui_kit::{
    canvas, div, point, px, svg, BoxShadow, Context, Div, FontWeight, IntoElement, KeyDownEvent,
    Window,
};

use crate::domain::{
    preset::PresetKind,
    preset_list::{format_list, parse_list},
};
use crate::gui::state::AppState;
use crate::theme::{metrics, Palette, ThemeExt as _};

use super::labels::column_index;
use super::widgets::measure;
use super::{FocusTarget, Popover};

impl Popover {
    pub(super) fn toggle_about(&mut self, cx: &mut Context<Self>) {
        self.about_open = !self.about_open;
        // Make sure the link holds focus, so Esc reaches the card's key context.
        self.pending_focus = Some((FocusTarget::AboutLink, 0));
        cx.notify();
    }

    pub(super) fn close_about_from_keyboard(&mut self, cx: &mut Context<Self>) {
        self.about_open = false;
        self.pending_focus = Some((FocusTarget::AboutLink, 0));
        cx.notify();
    }

    /// Typing: the whole string must parse before the circles change; an
    /// invalid string only shows the error and keeps the last valid list.
    pub(super) fn list_input_changed(
        &mut self,
        kind: PresetKind,
        text: &str,
        cx: &mut Context<Self>,
    ) {
        let i = column_index(kind);
        match parse_list(text) {
            Ok(specs) => {
                self.list_errors[i] = None;
                // Record the normalized text first so the resulting state
                // change does not rewrite what the user is typing.
                self.applied_text[i] = format_list(&specs);
                if self.state.read(cx).specs(kind) != specs.as_slice() {
                    self.state
                        .update(cx, |state, cx| state.set_specs(kind, specs, cx));
                }
            }
            Err(error) => self.list_errors[i] = Some(error.to_string()),
        }
        cx.notify();
    }

    pub(super) fn sync_inputs_from_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for kind in [PresetKind::Focus, PresetKind::Rest] {
            let i = column_index(kind);
            let text = format_list(self.state.read(cx).specs(kind));
            if text != self.applied_text[i] {
                self.applied_text[i].clone_from(&text);
                self.list_errors[i] = None;
                self.list_inputs[i].update(cx, |input, cx| input.set_value(text, window, cx));
            }
        }
    }

    pub(super) fn list_field(
        &self,
        p: &Palette,
        kind: PresetKind,
        label: &'static str,
    ) -> impl IntoElement {
        let i = column_index(kind);
        let error = self.list_errors[i].clone();
        let invalid = error.is_some();
        let ring = BoxShadow {
            color: p.danger.alpha(metrics::DANGER_RING),
            offset: point(px(0.), px(0.)),
            blur_radius: px(0.),
            spread_radius: px(3.),
            inset: false,
        };
        div()
            .id(("list-field", i))
            .aria_label(label)
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .text_size(px(12.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(p.text_muted.hsla())
                    .child(label),
            )
            .child(
                div()
                    .h(px(30.))
                    .flex()
                    .items_center()
                    .rounded(px(7.))
                    .bg(p.input_bg.hsla())
                    .border_1()
                    .border_color(if invalid { p.danger } else { p.control_border }.hsla())
                    .when(invalid, |field| field.shadow(vec![ring]))
                    .text_size(px(12.))
                    .child(
                        Input::new(&self.list_inputs[i])
                            .id(("list-input", i))
                            .appearance(false)
                            .xsmall()
                            .w_full(),
                    ),
            )
            .children(error.map(|message| {
                div()
                    .text_size(px(11.))
                    .line_height(px(15.))
                    .text_color(p.danger_text.hsla())
                    .child(message)
            }))
    }

    pub(super) fn about_link(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let p = *cx.palette();
        let open = self.about_open;
        div()
            .id("about-link")
            .test_support()
            .relative()
            .track_focus(&self.about_focus)
            .role(gpui_kit::Role::Button)
            .aria_label("About CountdownTimerBar")
            .aria_expanded(open)
            .cursor_pointer()
            .ml(px(-6.))
            .px(px(6.))
            .py(px(3.))
            .rounded(px(6.))
            .text_size(px(11.))
            .text_color(p.text_muted.hsla())
            .hover(|s| s.text_color(p.text.hsla()))
            .focus_visible(|s| s.bg(p.control_hover.hsla()))
            .when(open, |link| {
                link.bg(p.control_hover.hsla()).text_color(p.text.hsla())
            })
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    this.toggle_about(cx);
                }
            }))
            .on_click(cx.listener(|this, _, _, cx| this.toggle_about(cx)))
            .child(measure(&self.about_link_bounds))
            .child("About")
    }

    /// The card above the link: it floats over the popover (deferred, absolute),
    /// so opening it never changes the popover's size.
    pub(super) fn about_card(
        &self,
        p: &Palette,
        version: &str,
        copyright: &str,
    ) -> impl IntoElement {
        gpui_kit::deferred(
            div()
                .absolute()
                .left(px(-6.))
                .bottom(px(34.))
                .w(px(236.))
                .child(
                    div()
                        .id("about-card")
                        .test_support()
                        .relative()
                        .role(gpui_kit::Role::Dialog)
                        .aria_label("About CountdownTimerBar")
                        .w_full()
                        .p(px(14.))
                        .flex()
                        .flex_col()
                        .gap(px(10.))
                        .rounded(px(12.))
                        .bg(p.card_bg.hsla())
                        .border_1()
                        .border_color(p.card_border.hsla())
                        .shadow(vec![BoxShadow {
                            color: p.shadow.alpha(p.elevation.card_opacity()),
                            offset: point(px(0.), px(14.)),
                            blur_radius: px(36.),
                            spread_radius: px(0.),
                            inset: false,
                        }])
                        .child(measure(&self.about_card_bounds))
                        .child(about_identity(p, version))
                        .child(
                            div()
                                .text_size(px(12.))
                                .line_height(px(17.))
                                .text_color(p.text_body.hsla())
                                .child(
                                    "Minimalist configurable timer for the macOS status bar. \
                                     Inspired by Hourglass.",
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(p.text_subtle.hsla())
                                .child(format!("© {copyright}")),
                        )
                        .child(about_arrow(p)),
                ),
        )
        .with_priority(1)
    }

    pub(super) fn settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let version = crate::gui::about::version(cx);
        let copyright = crate::gui::about::copyright(cx);
        let p = *cx.palette();
        let sound_on = self.state.read(cx).sound_on();
        let sound_state = self.state.clone();
        let quit_state = self.state.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .pt(px(14.))
            .border_t_1()
            .border_color(p.divider.hsla())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::MEDIUM)
                            .child("Sound"),
                    )
                    .child(
                        Switch::new("sound-toggle")
                            .accessibility_label("Sound")
                            .checked(sound_on)
                            .on_change(move |_, _, cx| {
                                sound_state.update(cx, AppState::toggle_sound);
                            }),
                    ),
            )
            .child(self.list_field(&p, PresetKind::Focus, "Focus timers"))
            .child(self.list_field(&p, PresetKind::Rest, "Rest timers"))
            .child(
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(self.about_link(cx))
                    .children(
                        self.about_open
                            .then(|| self.about_card(&p, &version, &copyright)),
                    )
                    .child(
                        Button::new("quit")
                            .outline()
                            .with_size(Size::Small)
                            .accessibility_label("Quit")
                            .on_click(move |_, _, cx| {
                                quit_state.update(cx, AppState::quit);
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .text_size(px(12.))
                                    .child("Quit")
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(p.text_subtle.hsla())
                                            .child("⌘Q"),
                                    ),
                            ),
                    ),
            )
    }
}

fn about_identity(p: &Palette, version: &str) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(
            div()
                .size(px(36.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(9.))
                .bg(p.accent.alpha(metrics::accent::FILL))
                .border_1()
                .border_color(p.accent.hsla())
                .child(
                    svg()
                        .path(crate::gui::assets::TIMER_GLYPH)
                        .size(px(18.))
                        .text_color(p.accent.hsla()),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.text.hsla())
                        .child("CountdownTimerBar"),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(p.text_muted.hsla())
                        .child(format!("Version {version}")),
                ),
        )
}

fn about_arrow(p: &Palette) -> impl IntoElement {
    let arrow_border = p.card_border.hsla();
    let arrow_fill = p.card_bg.hsla();
    canvas(
        |_, _, _| {},
        move |bounds, (), window, _| {
            // The border colour first, then the fill pulled up by 1 px so it
            // continues the card's own border.
            let paint = |window: &mut Window, inset: f32, color: gpui_kit::Hsla| {
                let mut path = gpui_kit::PathBuilder::fill();
                path.move_to(point(bounds.left() + px(inset), bounds.top() - px(inset)));
                path.line_to(point(bounds.right() - px(inset), bounds.top() - px(inset)));
                path.line_to(point(bounds.center().x, bounds.bottom() - px(inset * 1.4)));
                path.close();
                if let Ok(path) = path.build() {
                    window.paint_path(path, color);
                }
            };
            paint(window, 0., arrow_border);
            paint(window, 1., arrow_fill);
        },
    )
    .absolute()
    .left(px(16.))
    .bottom(px(-6.))
    .w(px(12.))
    .h(px(6.))
}
