//! The inline edit field of a circle.

use gpui_kit::{Context, KeyDownEvent, Window};

use crate::domain::{preset::PresetKind, preset_list::Unit};
use crate::gui::inline_edit::EditBuffer;

use super::{Edit, FocusTarget, Interaction, Popover, Press};

pub(super) fn edit_text(edit: &Edit, knob: Option<&Press>) -> String {
    match knob {
        Some(press) if press.preview() > 0 => press.preview().to_string(),
        Some(_) => String::new(),
        None => edit.buffer.text().to_owned(),
    }
}

impl Popover {
    pub(super) fn begin_edit(
        &mut self,
        kind: PresetKind,
        index: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        self.commit_edit(cx);
        let edit = if let Some(i) = index {
            let Some(spec) = self.spec_at(kind, i, cx) else {
                return;
            };
            Edit {
                kind,
                index,
                unit: spec.unit(),
                buffer: EditBuffer::with_value(spec.value()),
                drag: None,
            }
        } else {
            if self.state.read(cx).is_full(kind) {
                return;
            }
            Edit {
                kind,
                index,
                unit: Unit::Minutes,
                buffer: EditBuffer::empty(),
                drag: None,
            }
        };
        self.interaction = Interaction::Edit(edit);
        self.hovered = None;
        self.pending_focus = Some((FocusTarget::Edit, 0));
        cx.notify();
    }

    /// Stores a valid typed value; empty or zero cancels (and drops a
    /// brand-new preset). Returns the index of the stored preset.
    pub(super) fn store_edit(&mut self, edit: &Edit, cx: &mut Context<Self>) -> Option<usize> {
        let spec = edit.buffer.commit(edit.unit)?;
        if let Some(index) = edit.index {
            self.replace_spec(edit.kind, index, spec, cx);
            Some(index)
        } else {
            self.push_spec(edit.kind, spec, cx);
            Some(self.state.read(cx).specs(edit.kind).len() - 1)
        }
    }

    pub(super) fn commit_edit(&mut self, cx: &mut Context<Self>) {
        if let Some(edit) = self.interaction.take_edit() {
            cx.notify();
            self.store_edit(&edit, cx);
        }
    }

    /// Keyboard end of an edit: Enter (`commit`, stay), Esc (cancel, stay),
    /// Tab / Shift+Tab (commit, then step the tab order). Focus returns to the
    /// circle that was edited, or to "+" when nothing was stored for a new one.
    pub(super) fn end_edit(&mut self, commit: bool, advance: i8, cx: &mut Context<Self>) {
        let Some(edit) = self.interaction.take_edit() else {
            return;
        };
        let mut target = match edit.index {
            Some(index) => FocusTarget::Circle(edit.kind, index),
            None => FocusTarget::Add(edit.kind),
        };
        if commit {
            if let Some(index) = self.store_edit(&edit, cx) {
                target = FocusTarget::Circle(edit.kind, index);
            }
        }
        self.pending_focus = Some((target, advance));
        cx.notify();
    }

    pub fn tab(&mut self, direction: i8, window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.edit().is_some() {
            self.end_edit(true, direction, cx);
        } else if direction > 0 {
            window.focus_next(cx);
        } else {
            window.focus_prev(cx);
        }
    }

    pub(super) fn edit_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let Some(edit) = self.interaction.edit_mut() else {
            return;
        };
        let key = event.keystroke.key.as_str();
        match key {
            "enter" => self.end_edit(true, 0, cx),
            "backspace" => {
                edit.buffer.backspace();
                cx.notify();
            }
            "m" | "s" => {
                edit.unit = if key == "m" {
                    Unit::Minutes
                } else {
                    Unit::Seconds
                };
                cx.notify();
            }
            _ => {
                let mut chars = key.chars();
                if let (Some(ch), None) = (chars.next(), chars.next()) {
                    if edit.buffer.push_digit(ch) {
                        cx.notify();
                    }
                }
            }
        }
    }

    /// Appends a digit to the edit that was just opened; the first digit
    /// replaces the pre-filled value.
    pub(super) fn type_into_edit(&mut self, digit: char) {
        if let Some(edit) = self.interaction.edit_mut() {
            edit.buffer.push_digit(digit);
        }
    }
}
