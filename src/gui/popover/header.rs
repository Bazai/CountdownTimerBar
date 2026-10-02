//! Header: clock, Stop / Pause buttons and the status line.

use gpui_kit::component::Icon;
use gpui_kit::prelude::*;
use gpui_kit::{div, px, AnyElement, Context, FontWeight, IntoElement};

use crate::domain::countdown::Phase as CountdownPhase;
use crate::domain::preset::Preset;
use crate::domain::preset_list::PresetSpec;
use crate::gui::state::AppState;
use crate::gui::timer_circle::tabular_numbers;
use crate::theme::{metrics, Palette, ThemeExt as _};

use super::labels::{kind_name, preset_label};
use super::widgets::icon_button;
use super::{FocusTarget, Popover};

impl Popover {
    /// Time with Stop on its left and Pause / Resume on its right, status text below.
    /// Both slots are always laid out, so nothing moves when the state changes.
    pub(super) fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let p = *cx.palette();
        let (phase, active, clock) = {
            let app = self.state.read(cx);
            (app.phase(), app.active_preset(), app.clock_label())
        };
        let paused = phase == CountdownPhase::Paused;
        let time_color = if paused { p.text_muted } else { p.text };
        let (stop, toggle) = match active {
            Some(preset) => {
                let (stop, toggle) = self.transport_buttons(&p, preset, paused, cx);
                (Some(stop), Some(toggle))
            }
            None => (None, None),
        };
        let slot = |child: Option<AnyElement>| {
            div()
                .size(px(metrics::ICON_BUTTON_SIZE))
                .flex_none()
                .children(child)
        };

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .w_full()
                    .h(px(40.))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .justify_end()
                            .child(slot(stop)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(34.))
                            .line_height(px(40.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .font_features(tabular_numbers())
                            .text_color(time_color.hsla())
                            .child(clock),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .justify_start()
                            .child(slot(toggle)),
                    ),
            )
            .child(
                div()
                    .h(px(18.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(
                                if active.is_some() {
                                    p.text_muted
                                } else {
                                    p.text_subtle
                                }
                                .hsla(),
                            )
                            .child(status_text(phase, active)),
                    ),
            )
    }

    fn transport_buttons(
        &self,
        p: &Palette,
        preset: Preset,
        paused: bool,
        cx: &mut Context<Self>,
    ) -> (AnyElement, AnyElement) {
        let pause_state = self.state.clone();
        // Stop disappears from its slot: hand focus to the circle that was
        // running so keyboard navigation continues from there.
        let stop_target = self
            .state
            .read(cx)
            .index_of(preset.kind, preset.id)
            .map(|index| FocusTarget::Circle(preset.kind, index));
        let toggle = if paused {
            icon_button("timer-toggle", "Resume")
                .icon(Icon::new(gpui_kit::assets::IconName::Play))
                .bg(p.accent.alpha(metrics::accent::FILL))
                .border_1()
                .border_color(p.accent.hsla())
        } else {
            icon_button("timer-toggle", "Pause")
                .icon(Icon::new(gpui_kit::assets::IconName::Pause))
                .bg(p.control.hsla())
                .border_1()
                .border_color(p.control_border.hsla())
        };
        let stop = icon_button("stop", "Stop")
            .bg(p.control.hsla())
            .border_1()
            .border_color(p.control_border.hsla())
            .child(div().size(px(10.)).rounded(px(2.)).bg(p.icon.hsla()))
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some(target) = stop_target {
                    this.pending_focus = Some((target, 0));
                }
                this.state.update(cx, AppState::stop_countdown);
            }));
        (
            stop.into_any_element(),
            toggle
                .on_click(move |_, _, cx| {
                    pause_state.update(cx, AppState::pause_or_resume);
                })
                .into_any_element(),
        )
    }
}

fn status_text(phase: CountdownPhase, active: Option<Preset>) -> String {
    match (phase, active) {
        (CountdownPhase::Paused, Some(preset)) => format!("Paused · {}", preset_label(preset)),
        (_, Some(preset)) => {
            let spec = PresetSpec::from_duration(preset.duration);
            format!(
                "{} · {}{}",
                kind_name(preset.kind),
                spec.value(),
                spec.unit().suffix()
            )
        }
        (_, None) => "Click to start · drag to adjust".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::duration::DurationSeconds;
    use crate::domain::preset::{IdGen, PresetKind};

    use super::*;

    fn preset(kind: PresetKind, seconds: u32) -> Preset {
        Preset {
            kind,
            duration: DurationSeconds::new(seconds).unwrap(),
            id: IdGen::default().next_id(),
        }
    }

    #[test]
    fn idle_shows_the_hint() {
        assert_eq!(
            status_text(CountdownPhase::Idle, None),
            "Click to start · drag to adjust"
        );
    }

    #[test]
    fn running_names_the_column_and_the_duration() {
        let rest = preset(PresetKind::Rest, 60);
        assert_eq!(
            status_text(CountdownPhase::Running, Some(rest)),
            "Rest · 1m"
        );
    }

    #[test]
    fn paused_leads_with_the_state() {
        let focus = preset(PresetKind::Focus, 13);
        assert_eq!(
            status_text(CountdownPhase::Paused, Some(focus)),
            "Paused · Focus 13s"
        );
    }
}
