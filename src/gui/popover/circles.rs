//! The Focus / Rest columns: circle rows and the "+" row.

use gpui_kit::base::ObservedElement;
use gpui_kit::prelude::*;
use gpui_kit::TestSupportExt as _;
use gpui_kit::{
    div, px, svg, AnyElement, Context, Div, FocusHandle, FontWeight, IntoElement, KeyDownEvent,
    MouseButton, MouseDownEvent, Stateful,
};

use crate::domain::countdown::Phase as CountdownPhase;
use crate::domain::{
    preset::PresetKind,
    preset_list::{PresetSpec, Unit, MAX_PRESETS},
};
use crate::gui::gesture::{Phase, Source};
use crate::gui::timer_circle::{
    self, border_width, edit_label, progress_ring, value_label, Visual,
};
use crate::theme::{metrics, Palette, ThemeExt as _};

use super::edit::edit_text;
use super::labels::{column_index, kind_name, typed_digit, unit_word};
use super::widgets::{
    knob_marks, measure, row_shell, small_square, LABEL_BESIDE_CIRCLE, LABEL_PAST_TOGGLES,
};
use super::{CancelEdit, Edit, FocusTarget, Popover, Press, EDIT_KEY_CONTEXT, ROW_PAD};

impl Popover {
    pub(super) fn circle_key_down(
        &mut self,
        kind: PresetKind,
        index: usize,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) {
        let Some(preset) = self.preset_at(kind, index, cx) else {
            return;
        };
        let active = self.is_active(kind, index, cx);
        // Enter or a digit starts inline edit; the active timer cannot be re-valued,
        // so there Enter pauses / resumes.
        if let Some(digit) = typed_digit(event) {
            if !active {
                self.begin_edit(kind, Some(index), cx);
                self.type_into_edit(digit);
            }
            return;
        }
        match event.keystroke.key.as_str() {
            "space" => self.press_preset(preset, cx),
            "enter" if active => self.press_preset(preset, cx),
            "enter" => self.begin_edit(kind, Some(index), cx),
            "up" => self.step_spec(kind, index, 1, cx),
            "down" => self.step_spec(kind, index, -1, cx),
            "delete" | "backspace" => {
                let before = self.state.read(cx).specs(kind).len();
                self.remove_spec(kind, index, cx);
                let after = self.state.read(cx).specs(kind).len();
                if after < before {
                    let target = match after {
                        0 => FocusTarget::Add(kind),
                        n => FocusTarget::Circle(kind, index.min(n - 1)),
                    };
                    self.pending_focus = Some((target, 0));
                    cx.notify();
                }
            }
            _ => {}
        }
    }

    pub(super) fn circle_handle(
        &mut self,
        kind: PresetKind,
        index: usize,
        cx: &mut Context<Self>,
    ) -> FocusHandle {
        self.circle_focus
            .entry((column_index(kind), index))
            .or_insert_with(|| cx.focus_handle().tab_stop(true))
            .clone()
    }

    fn pick_unit(
        &mut self,
        kind: PresetKind,
        index: Option<usize>,
        unit: Unit,
        cx: &mut Context<Self>,
    ) {
        if let Some(edit) = self.interaction.edit_mut() {
            edit.unit = unit;
            // Clicking the toggle takes focus off the field.
            self.pending_focus = Some((FocusTarget::Edit, 0));
            cx.notify();
        } else if let Some(index) = index {
            self.set_unit(kind, index, unit, cx);
        }
    }

    pub(super) fn set_unit(
        &mut self,
        kind: PresetKind,
        index: usize,
        unit: Unit,
        cx: &mut Context<Self>,
    ) {
        if let Some(spec) = self.spec_at(kind, index, cx) {
            self.replace_spec(kind, index, spec.with_unit(unit), cx);
        }
    }

    pub(super) fn circle_row(
        &self,
        kind: PresetKind,
        index: usize,
        spec: PresetSpec,
        handle: &FocusHandle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let p = *cx.palette();
        let active = self.is_active(kind, index, cx);
        let (slot_phase, progress) = {
            let app = self.state.read(cx);
            let phase = if active {
                app.phase()
            } else {
                CountdownPhase::Idle
            };
            (phase, app.progress())
        };
        let editing = self
            .interaction
            .edit()
            .filter(|e| e.kind == kind && e.index == Some(index));
        let press = self
            .interaction
            .press()
            .filter(|p| p.kind == kind && p.index == Some(index) && !active);
        let knob = press.is_some_and(|p| p.gesture.phase() == Phase::Knob);
        let hovered = self.hovered == Some((kind, index)) && !active && self.interaction.is_idle();

        let shown = match press {
            Some(p) if knob => p.preview(),
            _ => spec.value(),
        };
        let visual = circle_visual(editing.is_some(), knob, slot_phase);

        let id_base = column_index(kind) * 100 + index;
        let aria = format!("Start {} {}", spec.value(), unit_word(spec.unit()));
        let circle = timer_circle::shell(&p, ("circle", id_base), visual, aria);
        let circle = match editing {
            Some(edit) => edit_circle(
                circle,
                &p,
                (kind, Some(index)),
                edit,
                press.filter(|_| knob),
                &self.edit_focus,
                cx,
            ),
            None => circle
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                        this.mouse_down_on_circle(kind, index, event, window, cx);
                    }),
                )
                .track_focus(handle)
                .aria_description(if active {
                    "Space or Enter: pause or resume"
                } else {
                    "Space: start. Enter or a digit: edit the value. Up and Down: adjust. \
                     Delete: remove"
                })
                .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                    this.circle_key_down(kind, index, event, cx);
                }))
                .when(
                    matches!(visual, Visual::Active | Visual::Paused),
                    |circle| circle.child(progress_ring(&p, progress, visual == Visual::Paused)),
                )
                .child(value_label(&p, shown, spec.unit().suffix(), visual)),
        };

        let row = row_shell()
            .id(("row", id_base))
            .on_hover(cx.listener(move |this, over: &bool, _, cx| {
                if *over {
                    this.hovered = Some((kind, index));
                } else if this.hovered == Some((kind, index)) {
                    this.hovered = None;
                }
                cx.notify();
            }))
            .child(circle)
            .when(editing.is_some(), |row| {
                row.child(measure(&self.edit_row_bounds))
            })
            .when(hovered || editing.is_some(), |row| {
                let unit_now = editing.map_or_else(|| spec.unit(), |edit| edit.unit);
                row.child(unit_toggles(
                    &p,
                    "unit",
                    id_base,
                    unit_now,
                    kind,
                    Some(index),
                    cx,
                ))
            })
            .when(hovered, |row| {
                row.child(remove_badge(&p, id_base, kind, index, cx))
            });
        if knob {
            let base = press.map_or(i64::from(spec.value()), |p| p.base_value);
            let delta = i64::from(shown) - base;
            knob_marks(&p, row, delta, knob_label_left(editing.is_some())).into_any_element()
        } else {
            row.into_any_element()
        }
    }

    pub(super) fn add_row(&self, kind: PresetKind, cx: &mut Context<Self>) -> Option<AnyElement> {
        let p = *cx.palette();
        let count = self.state.read(cx).specs(kind).len();
        let adding = self
            .interaction
            .edit()
            .filter(|e| e.kind == kind && e.index.is_none());
        if let Some(edit) = adding {
            return Some(self.new_circle_row(&p, kind, count, edit, cx));
        }
        if count >= MAX_PRESETS {
            return None;
        }
        Some(self.plus_row(&p, kind, cx))
    }

    fn new_circle_row(
        &self,
        p: &Palette,
        kind: PresetKind,
        count: usize,
        edit: &Edit,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id_base = column_index(kind) * 100 + count;
        let press = self.interaction.press().filter(|p| {
            p.kind == kind && p.source == Source::EditNew && p.gesture.phase() == Phase::Knob
        });
        let circle = timer_circle::shell(p, ("circle", id_base), Visual::Edit, "New timer");
        let circle = edit_circle(circle, p, (kind, None), edit, press, &self.edit_focus, cx);
        let row = row_shell()
            .child(measure(&self.edit_row_bounds))
            .child(circle)
            .child(unit_toggles(
                p,
                "unit-new",
                column_index(kind),
                edit.unit,
                kind,
                None,
                cx,
            ));
        match press {
            Some(press) => knob_marks(
                p,
                row,
                i64::from(press.preview()) - press.base_value,
                LABEL_PAST_TOGGLES,
            )
            .into_any_element(),
            None => row.into_any_element(),
        }
    }

    fn plus_row(&self, p: &Palette, kind: PresetKind, cx: &mut Context<Self>) -> AnyElement {
        let label = match kind {
            PresetKind::Focus => "Add focus timer",
            PresetKind::Rest => "Add rest timer",
        };
        let plus = border_width(
            div()
                .id(("add", column_index(kind)))
                .test_support()
                .aria_label(label)
                .size(px(metrics::CIRCLE_SIZE))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .cursor_pointer()
                .track_focus(&self.add_focus[column_index(kind)]),
            1.5,
        )
        .border_dashed()
        .border_color(p.dashed.hsla())
        .hover(|s| s.border_color(p.text_subtle.hsla()))
        .focus_visible(|s| s.border_color(p.accent.hsla()))
        .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
            if let Some(digit) = typed_digit(event) {
                this.begin_edit(kind, None, cx);
                this.type_into_edit(digit);
            } else if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                this.begin_edit(kind, None, cx);
            }
        }))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                this.mouse_down_on_plus(kind, event, cx);
            }),
        )
        .child(
            svg()
                .path(gpui_kit::assets::IconName::Plus.path())
                .size(px(14.))
                .text_color(p.text_subtle.hsla()),
        );
        let dragged = self.interaction.press().filter(|p| {
            p.kind == kind && p.source == Source::Plus && p.gesture.phase() == Phase::Knob
        });
        let row = row_shell();
        match dragged {
            Some(press) => {
                let value = press.preview();
                let knob = timer_circle::shell(p, ("add", column_index(kind)), Visual::Knob, label)
                    .child(value_label(p, value, Unit::Minutes.suffix(), Visual::Knob));
                knob_marks(p, row.child(knob), i64::from(value), LABEL_BESIDE_CIRCLE)
                    .into_any_element()
            }
            None => row.child(plus).into_any_element(),
        }
    }

    pub(super) fn column(&mut self, kind: PresetKind, cx: &mut Context<Self>) -> impl IntoElement {
        let p = *cx.palette();
        let specs = self.state.read(cx).specs(kind).to_vec();
        let mut rows: Vec<AnyElement> = Vec::with_capacity(specs.len());
        for (index, spec) in specs.into_iter().enumerate() {
            let handle = self.circle_handle(kind, index, cx);
            rows.push(self.circle_row(kind, index, spec, &handle, cx));
        }
        let add = self.add_row(kind, cx);

        div()
            .relative()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(2.))
            .child(
                div()
                    .mb(px(ROW_PAD + 2.))
                    .text_size(px(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(p.text_muted.hsla())
                    .child(kind_name(kind)),
            )
            .children(rows)
            .children(add)
    }
}

/// The delta label sits past the `m` / `s` toggles while an edit field is dragged.
fn knob_label_left(editing: bool) -> f32 {
    if editing {
        LABEL_PAST_TOGGLES
    } else {
        LABEL_BESIDE_CIRCLE
    }
}

/// Which look a circle has. Editing wins over a knob drag, which wins over the
/// running / paused look. `slot_phase` is the countdown's phase when this very
/// slot is the active one, `Idle` for every other slot.
fn circle_visual(editing: bool, knob: bool, slot_phase: CountdownPhase) -> Visual {
    if editing {
        Visual::Edit
    } else if knob {
        Visual::Knob
    } else {
        match slot_phase {
            CountdownPhase::Idle => Visual::Normal,
            CountdownPhase::Running => Visual::Active,
            CountdownPhase::Paused => Visual::Paused,
        }
    }
}

fn edit_circle(
    circle: ObservedElement<Stateful<Div>>,
    p: &Palette,
    (kind, index): (PresetKind, Option<usize>),
    edit: &Edit,
    knob: Option<&Press>,
    focus: &FocusHandle,
    cx: &mut Context<Popover>,
) -> ObservedElement<Stateful<Div>> {
    circle
        .track_focus(focus)
        .key_context(EDIT_KEY_CONTEXT)
        .on_action(cx.listener(|this, _: &CancelEdit, _, cx| this.end_edit(false, 0, cx)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                this.mouse_down_on_edit(kind, index, event, cx);
            }),
        )
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
            this.edit_key_down(event, cx);
        }))
        .child(edit_label(p, &edit_text(edit, knob), edit.unit.suffix()))
}

/// The `m` / `s` toggle beside a circle. Ids are `(id_prefix, id_base * 2 + n)`.
fn unit_toggles(
    p: &Palette,
    id_prefix: &'static str,
    id_base: usize,
    selected: Unit,
    kind: PresetKind,
    index: Option<usize>,
    cx: &mut Context<Popover>,
) -> Div {
    div()
        .absolute()
        .left(px(86.))
        .top(px(ROW_PAD + 2.))
        .flex()
        .flex_col()
        .gap(px(2.))
        .children([Unit::Minutes, Unit::Seconds].map(|unit| {
            let (label, name) = match unit {
                Unit::Minutes => ("m", "Minutes"),
                Unit::Seconds => ("s", "Seconds"),
            };
            small_square(
                p,
                (id_prefix, id_base * 2 + usize::from(unit == Unit::Seconds)),
                name,
                selected == unit,
            )
            .on_click(cx.listener(move |this, _, _, cx| this.pick_unit(kind, index, unit, cx)))
            .child(label)
        }))
}

fn remove_badge(
    p: &Palette,
    id_base: usize,
    kind: PresetKind,
    index: usize,
    cx: &mut Context<Popover>,
) -> ObservedElement<Stateful<Div>> {
    div()
        .id(("remove", id_base))
        .test_support()
        .aria_label("Remove timer")
        .absolute()
        .left(px(67.))
        .top(px(0.))
        .size(px(18.))
        .rounded_full()
        .bg(p.badge_bg.hsla())
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        // The badge overlaps the circle's corner: keep the press from also
        // starting a circle gesture (which would hide the badge before the
        // click completes).
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(cx.listener(move |this, _, _, cx| this.remove_spec(kind, index, cx)))
        .child(
            svg()
                .path(gpui_kit::assets::IconName::Close.path())
                .size(px(8.))
                .text_color(p.badge_fg.hsla()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_wins_over_every_other_look() {
        assert_eq!(
            circle_visual(true, true, CountdownPhase::Running),
            Visual::Edit
        );
    }

    #[test]
    fn a_knob_drag_wins_over_the_running_look() {
        assert_eq!(
            circle_visual(false, true, CountdownPhase::Running),
            Visual::Knob
        );
    }

    #[test]
    fn an_active_circle_is_paused_or_running() {
        assert_eq!(
            circle_visual(false, false, CountdownPhase::Paused),
            Visual::Paused
        );
        assert_eq!(
            circle_visual(false, false, CountdownPhase::Running),
            Visual::Active
        );
    }

    #[test]
    fn a_circle_that_is_not_active_looks_normal() {
        assert_eq!(
            circle_visual(false, false, CountdownPhase::Idle),
            Visual::Normal
        );
    }
}
