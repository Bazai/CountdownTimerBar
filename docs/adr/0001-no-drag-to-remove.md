# ADR 0001. Presets are removed with × and Delete, not by dragging

- Status: accepted
- Date: 2026-10-01
- Affects: `src/gui/gesture.rs`, `src/gui/popover/`

## Context

In the redesign a preset circle supported two drag gestures at once:

- **knob** (vertical drag): changes the value, one step per 6 px;
- **drag-out**: pull the circle out of its column and release to delete the preset. A ghost circle with a "Remove" chip was shown, and a dashed placeholder took the circle's place.

Both gestures start from the same press on the circle, so there is no boundary between them: only where the pointer ends up decides.

## Problem

Drag-out got in the way of the main scenario, adjusting a value:

1. **Accidental deletion while adjusting.** A long vertical drag from the top or bottom circle took the pointer past the column edge, and the preset was deleted instead of changed. The first implementation judged "outside the column" by the column rectangle; the check had to be limited to the horizontal axis, but that does not solve it either: a small sideways drift during a vertical adjustment also deletes the preset.
2. **Irreversible without confirmation.** The product decision is that deletion is immediate, with no confirmation. A gesture that fires "on the way" is too dangerous for such an action.
3. **Extra complexity.** The gesture needed column bounds, a ghost overlay, a dashed placeholder, and special outcomes for an ordinary circle, a circle in edit mode and a new circle. The tests and checks around it were noticeably larger than those of the knob itself.
4. **Unavailable from the keyboard.** Drag-out is impossible without a mouse, so deletion had to be duplicated anyway (× and Delete).

## Decision

Remove drag-to-remove completely. Deletion stays available in two explicit ways:

- the **×** button on a hovered circle (hidden on a running or paused timer);
- the **Delete / Backspace** key on a focused circle (does nothing on the active timer).

The knob gesture (vertical drag) no longer has side outcomes and is not limited to the column area: the pointer can go anywhere, and the value depends only on the vertical offset.

Removed from the code: the `DragOut` phase, the `Remove` / `CancelNew` outcomes, `Action::RemovePreset`, the ghost with the "Remove" chip, the dashed placeholder (`Visual::Placeholder`), column bounds measurement, and the `DANGER_CHIP_*` tokens.

## Consequences

- Adjusting a value is safe: no drag deletes a preset.
- Dragging in edit mode and on "+" gets simpler: only "click" and "knob" remain. Dragging "+" sideways and dragging the edit field sideways no longer delete or cancel anything (Esc still cancels the edit).
- Confirmed product decisions do not change: a running or paused timer has neither × nor knob; at most 8 presets; deletion is immediate.
- The "drag-out remove" item of the redesign acceptance checklist is no longer checked.
- If a quick gesture for deletion is wanted later, that is a new decision with its own ADR (for example a swipe with an explicit threshold and a way to cancel), not a return to this gesture.

## Rejected alternatives

- **Keep drag-out but tighten the threshold** (move ≥ N px sideways): lowers the rate of false triggers but does not remove them, and keeps the ghost, the placeholder and all related branches.
- **Deletion with confirmation or an undo toast:** contradicts the decision that deletion is immediate and adds UI the popover does not have.
- **A dedicated "trash" zone:** an extra element in a compact 264 px popover.
