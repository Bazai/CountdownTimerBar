# ADR 0003. Radial progress on the active timer's circle

- Status: accepted
- Date: 2026-10-01
- Replaces: the earlier decision "no additional progress indicators"
- Affects: `src/domain/countdown.rs`, `src/gui/timer_circle.rs`, `src/gui/popover/`

## Context

A running timer's circle differed only by an accent border and a highlight. How much time was left could be read only from the digits in the header. In the first version of the design progress indicators were excluded on purpose, to keep the popover calm.

## Decision

The active timer's circle gets a radial progress ring (requested after review). Parameters:

- **Instead of the border.** The 2 px border of the active circle becomes the track (accent at 25 %, 20 % when paused). An arc of the same radius and thickness is drawn on top: the circle does not grow and the column layout does not change. The outer 4 px halo of a running timer stays.
- **It drains.** The arc shows the share that is left: from 12 o'clock clockwise, like the iOS timer. Full at the start, empty at the end.
- **Once per second.** The ring updates together with the clock; there are no extra timers or redraws. The share is computed in milliseconds (`Countdown::remaining_fraction`).
- **Pause.** The arc freezes and is drawn at 50 % accent, with no halo; Resume continues from the same place.
- Editing the value, deleting and the knob on the active circle remain forbidden (this design decision does not change).

## Consequences

- The "no progress indicators" rule no longer applies; the only progress indicator is the ring on the active circle. There is no progress in the status-bar pill or in the header.
- The arc is drawn with `PathBuilder::stroke` and `arc_to`; its ends are flat (the GPUI API does not expose rounded ends without a direct dependency on lyon).
- Short timers (tens of seconds) move in visible steps once per second, like the digits.

## Rejected alternatives

- **A separate outer ring around the circle** (instead of the halo): keeps the border, but needs space around the circle and removes the halo the compact row is built around.
- **A growing arc (elapsed share):** the opposite metaphor; a draining arc reads better as "what is left".
- **Smooth ~10 Hz updates:** the arc moves continuously, but costs ten redraws per second for an effect that is barely visible on minute-long timers.
