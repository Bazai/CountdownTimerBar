//! Pure naming and key helpers shared by the popover views.

use crate::domain::preset::{Preset, PresetKind};
use crate::domain::preset_list::{PresetSpec, Unit};
use gpui_kit::KeyDownEvent;

pub(super) fn typed_digit(event: &KeyDownEvent) -> Option<char> {
    let m = &event.keystroke.modifiers;
    if m.control || m.alt || m.platform || m.shift || m.function {
        return None;
    }
    let mut chars = event.keystroke.key.chars();
    match (chars.next(), chars.next()) {
        (Some(ch), None) if ch.is_ascii_digit() => Some(ch),
        _ => None,
    }
}

pub(super) fn column_index(kind: PresetKind) -> usize {
    match kind {
        PresetKind::Focus => 0,
        PresetKind::Rest => 1,
    }
}

pub(super) fn kind_name(kind: PresetKind) -> &'static str {
    match kind {
        PresetKind::Focus => "Focus",
        PresetKind::Rest => "Rest",
    }
}

pub(super) fn unit_word(unit: Unit) -> &'static str {
    match unit {
        Unit::Minutes => "minutes",
        Unit::Seconds => "seconds",
    }
}

pub(super) fn preset_label(preset: Preset) -> String {
    let spec = PresetSpec::from_duration(preset.duration);
    format!(
        "{} {}{}",
        kind_name(preset.kind),
        spec.value(),
        spec.unit().suffix()
    )
}
