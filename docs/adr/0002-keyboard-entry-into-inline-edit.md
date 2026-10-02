# ADR 0002. Entering inline edit from the keyboard: Enter and a digit

- Status: accepted
- Date: 2026-10-01
- Affects: `src/gui/popover/circles.rs` (`circle_key_down`)

## Context

Editing a circle's value could only be opened by double-click. There was no way into it from the keyboard: on a focused circle Enter and Space started the timer, ↑/↓ changed the value by 1, and Delete removed it. The original specification offered only an optional `⌥Enter`.

## Options

1. A digit on a focused circle opens the field with that digit.
2. `⌥Enter`.
3. `F2`.
4. Enter edits, Space starts (as in Finder).
5. A visible ✎ button next to the circle, reachable with Tab.

## Decision

Adopt **1 and 4**:

- A **digit** (no modifiers) opens the field; the first digit replaces the current value. Esc cancels.
- **Enter** opens the field with the current value selected for replacement. **Space** starts, pauses and resumes the timer.
- The value of a running or paused circle cannot be changed (a design decision), so there a digit is ignored, and Enter, like Space, pauses or resumes. This keeps the circle operable from the keyboard.
- The same goes for "+": a digit opens the new-preset field with that digit, Enter and Space open an empty field.
- The circle's aria description lists the keys so the way in can be discovered.

After Enter, Tab or Esc, focus returns to the circle (after committing, Tab moves on along the tab order); see `end_edit`.

## Consequences

- **A departure from the original specification:** "Space/Enter = start" stays true only for Space and for the active circle. For an ordinary circle, Enter now edits the value.
- No modifiers and no fn key are needed; the way in works with one hand.
- Rejected: `⌥Enter` and F2 (less discoverable; F2 needs fn on a MacBook), and a ✎ button (one more element on every circle, while × and m/s are currently visible only on hover).
