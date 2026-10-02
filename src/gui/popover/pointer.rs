//! Pointer gestures on the circles and the "+" button.

use gpui_kit::prelude::*;
use gpui_kit::{
    canvas, point, px, size, Bounds, Context, DispatchPhase, Entity, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Window,
};

use crate::domain::{
    preset::PresetKind,
    preset_list::{PresetSpec, Unit},
};
use crate::gui::gesture::{resolve, Action, Gesture, Source};

use super::{FocusTarget, Interaction, Popover, Press, DOUBLE_CLICK_WINDOW, ROW_HEIGHT};

impl Popover {
    pub(super) fn mouse_down_on_circle(
        &mut self,
        kind: PresetKind,
        index: usize,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.commit_edit(cx);
        let Some(spec) = self.spec_at(kind, index, cx) else {
            return;
        };
        if event.click_count >= 2 {
            // Second click of a double-click: edit, and cancel the first click's start.
            self.pending_click = None;
            if !self.is_active(kind, index, cx) {
                self.begin_edit(kind, Some(index), cx);
                // The outside-click check sees this click next, before the row is
                // measured; treat the row around the pointer as the edited one.
                self.edit_row_bounds.set(Bounds::new(
                    point(
                        event.position.x - px(58.),
                        event.position.y - px(ROW_HEIGHT / 2.),
                    ),
                    size(px(116.), px(ROW_HEIGHT)),
                ));
            }
            return;
        }
        self.interaction = Interaction::Press(Press {
            kind,
            index: Some(index),
            source: Source::Circle,
            base_value: i64::from(spec.value()),
            unit: spec.unit(),
            gesture: Gesture::new(event.position.y.as_f32()),
            pos: event.position,
        });
        cx.notify();
    }

    pub(super) fn mouse_down_on_edit(
        &mut self,
        kind: PresetKind,
        index: Option<usize>,
        event: &MouseDownEvent,
        cx: &mut Context<Self>,
    ) {
        let Some(edit) = self.interaction.edit_mut() else {
            return;
        };
        if event.click_count >= 2 {
            return;
        }
        edit.drag = Some(Press {
            kind,
            index,
            source: if index.is_some() {
                Source::EditExisting
            } else {
                Source::EditNew
            },
            base_value: i64::from(edit.buffer.value()),
            unit: edit.unit,
            gesture: Gesture::new(event.position.y.as_f32()),
            pos: event.position,
        });
        cx.notify();
    }

    pub(super) fn mouse_down_on_plus(
        &mut self,
        kind: PresetKind,
        event: &MouseDownEvent,
        cx: &mut Context<Self>,
    ) {
        self.commit_edit(cx);
        if self.state.read(cx).is_full(kind) {
            return;
        }
        self.interaction = Interaction::Press(Press {
            kind,
            index: None,
            source: Source::Plus,
            base_value: 0,
            unit: Unit::Minutes,
            gesture: Gesture::new(event.position.y.as_f32()),
            pos: event.position,
        });
        cx.notify();
    }

    pub(super) fn mouse_moved(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if self.interaction.press().is_none() {
            return;
        }
        if event.pressed_button != Some(MouseButton::Left) {
            // The button was released where we could not see it.
            self.finish_press(event.position, cx);
            return;
        }
        if let Some(press) = self.interaction.press_mut() {
            press.gesture.moved(event.position.y.as_f32());
            press.pos = event.position;
        }
        cx.notify();
    }

    pub(super) fn finish_press(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let Some(press) = self.interaction.take_press() else {
            return;
        };
        let outcome = press.gesture.released(position.y.as_f32());
        let action = resolve(press.source, outcome, press.base_value);
        let kind = press.kind;
        match press.source {
            Source::Circle => self.finish_circle_press(kind, press.index, action, cx),
            Source::Plus => self.finish_plus_press(kind, action, cx),
            Source::EditExisting | Source::EditNew => self.finish_edit_press(press.unit, action),
        }
        cx.notify();
    }

    pub(super) fn finish_circle_press(
        &mut self,
        kind: PresetKind,
        index: Option<usize>,
        action: Action,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = index else { return };
        let Some(spec) = self.spec_at(kind, index, cx) else {
            return;
        };
        let Some(preset) = self.preset_at(kind, index, cx) else {
            return;
        };
        let active = self.is_active(kind, index, cx);
        match action {
            Action::PressTimer if active => self.press_preset(preset, cx),
            Action::PressTimer => {
                self.pending_click = Some(cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(DOUBLE_CLICK_WINDOW).await;
                    let _ = this.update(cx, |this, cx| {
                        this.pending_click = None;
                        this.press_preset(preset, cx);
                    });
                }));
            }
            _ if active => {}
            Action::SetValue(value) => {
                self.replace_spec(kind, index, spec.with_value_clamped(value), cx);
            }
            Action::BeginNew | Action::Nothing => {}
        }
    }

    pub(super) fn finish_plus_press(
        &mut self,
        kind: PresetKind,
        action: Action,
        cx: &mut Context<Self>,
    ) {
        match action {
            Action::BeginNew => self.begin_edit(kind, None, cx),
            Action::SetValue(value) => {
                let unit = Unit::Minutes;
                let value = unit.clamp(value);
                if let Some(spec) = PresetSpec::new(value, unit) {
                    self.push_spec(kind, spec, cx);
                    let last = self.state.read(cx).specs(kind).len() - 1;
                    self.pending_focus = Some((FocusTarget::Circle(kind, last), 0));
                }
            }
            _ => {}
        }
    }

    /// A knob drag on the edit field changes the typed number only; storing
    /// it is still Enter / Tab / click outside.
    pub(super) fn finish_edit_press(&mut self, unit: Unit, action: Action) {
        if let (Action::SetValue(value), Some(edit)) = (action, self.interaction.edit_mut()) {
            edit.buffer.set_value(unit.clamp(value));
        }
    }

    /// Window-level listeners, so a drag or an outside click is seen anywhere.
    pub(super) fn mouse_listeners(this: Entity<Self>) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |_, (), window, _| {
                let moves = this.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                    if phase == DispatchPhase::Bubble {
                        moves.update(cx, |this, cx| this.mouse_moved(event, cx));
                    }
                });
                let ups = this.clone();
                window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                    if phase == DispatchPhase::Bubble && event.button == MouseButton::Left {
                        ups.update(cx, |this, cx| this.finish_press(event.position, cx));
                    }
                });
                let downs = this;
                window.on_mouse_event(move |event: &MouseDownEvent, phase, _, cx| {
                    if phase == DispatchPhase::Bubble {
                        downs.update(cx, |this, cx| {
                            if this.about_open
                                && !this.about_card_bounds.get().contains(&event.position)
                                && !this.about_link_bounds.get().contains(&event.position)
                            {
                                this.about_open = false;
                                cx.notify();
                            }
                            if this.interaction.edit().is_some()
                                && !this.edit_row_bounds.get().contains(&event.position)
                            {
                                this.commit_edit(cx);
                            }
                        });
                    }
                });
            },
        )
        .absolute()
        .size_0()
    }
}
