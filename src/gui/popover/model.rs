//! Edits to the preset lists, routed through `AppState`.

use gpui_kit::{App, Context};

use crate::domain::{
    preset::{Preset, PresetKind},
    preset_list::PresetSpec,
};

use super::Popover;

impl Popover {
    /// Is this exact slot the running (or paused) timer? Decided by id, so a
    /// twin with the same kind and duration is not active, nor locked.
    pub(super) fn is_active(&self, kind: PresetKind, index: usize, cx: &App) -> bool {
        self.state.read(cx).is_slot_active(kind, index)
    }

    pub(super) fn spec_at(&self, kind: PresetKind, index: usize, cx: &App) -> Option<PresetSpec> {
        self.state.read(cx).specs(kind).get(index).copied()
    }

    pub(super) fn preset_at(&self, kind: PresetKind, index: usize, cx: &App) -> Option<Preset> {
        self.state.read(cx).preset(kind, index)
    }

    pub(super) fn replace_spec(
        &mut self,
        kind: PresetKind,
        index: usize,
        spec: PresetSpec,
        cx: &mut Context<Self>,
    ) {
        let Some(old) = self.spec_at(kind, index, cx) else {
            return;
        };
        if old == spec || self.is_active(kind, index, cx) {
            return;
        }
        self.state
            .update(cx, |state, cx| state.replace_spec_at(kind, index, spec, cx));
    }

    pub(super) fn remove_spec(&mut self, kind: PresetKind, index: usize, cx: &mut Context<Self>) {
        if self.is_active(kind, index, cx) {
            return;
        }
        self.hovered = None;
        self.state
            .update(cx, |state, cx| state.remove_spec_at(kind, index, cx));
    }

    pub(super) fn push_spec(&mut self, kind: PresetKind, spec: PresetSpec, cx: &mut Context<Self>) {
        if !self.state.read(cx).is_full(kind) {
            self.state
                .update(cx, |state, cx| state.push_spec(kind, spec, cx));
        }
    }

    pub(super) fn step_spec(
        &mut self,
        kind: PresetKind,
        index: usize,
        delta: i64,
        cx: &mut Context<Self>,
    ) {
        if let Some(spec) = self.spec_at(kind, index, cx) {
            let next = spec.with_value_clamped(i64::from(spec.value()) + delta);
            self.replace_spec(kind, index, next, cx);
        }
    }

    pub(super) fn press_preset(&mut self, preset: Preset, cx: &mut Context<Self>) {
        self.state
            .update(cx, |state, cx| state.press_preset(preset, cx));
    }
}
