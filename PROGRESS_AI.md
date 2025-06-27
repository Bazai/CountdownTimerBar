# AI Progress Tracker: CountdownTimerBar (macOS)

## Product Goal

Configurable countdown timer in the macOS StatusBar. Fast preset switching, simple options, all UI/logic described for deterministic AI work.

---

## Progress Checklist

### 1. UI/UX

- [x] StatusBar icon with timer (rounded square, clock inside)
- [x] Click icon → popover with two columns: Focus/Rest timers
- [x] Gear button for options
- [ ] Options UI: About, Sound, Checkbox Option, Focus Timers (input), Rest Timers (input), Quit
- [x] Inactive timer: primary System Theme color (dark), background system theme color (white)
- [x] Active timer: purple border appears
- [x] Clicking active timer stops timer
- [x] StatusBar: show 00:00 when no active timer
- [x] StatusBar timer displayed in outlined pill with perfect UX (no aliasing, no cut edges, matches reference)

### 2. Timer Logic

- [x] Countdown timer, display in StatusBar
- [x] Start timer from preset
- [x] Auto-stop/reset on finish

### 3. Configurability

- [x] Change Focus/Rest timer values via options (input string → array)
- [ ] Persist user settings (UserDefaults)

### 4. Options

- [ ] About (app info)
- [ ] Sound (toggle)
- [ ] Checkbox Option (TBD)
- [ ] Quit (exit app)

### 5. Extras

- [ ] Statistics (optional)
- [ ] Notifications/sounds on timer end

---

## AI Task Breakdown

### Step 1: Xcode Setup

- [ ] Ensure Xcode project is SwiftUI, macOS, AppDelegate present

### Step 2: StatusBar App

- [x] NSStatusItem created, icon set
- [x] Popover on click

### Step 3: Timer Model

- [x] ObservableObject, countdown logic, published state

### Step 4: Popover UI

- [x] Two columns: Focus/Rest, preset buttons
- [x] Timer display in rounded square
- [x] Gear/options button

### Step 5: Options UI

- [ ] Modal/sheet with:
  - [ ] About
  - [ ] Sound toggle
  - [ ] Checkbox Option
  - [ ] Focus Timers input (comma-separated)
  - [ ] Rest Timers input (comma-separated)
  - [ ] Quit

### Step 6: Settings Logic

- [ ] Parse input strings to arrays
- [ ] Save/load settings (UserDefaults)

### Step 7: Polish

- [ ] UX review, bugfixes
- [ ] Release build

---

## Notes for AI Agents

- Use explicit, deterministic logic for all UI/logic
- All state changes must be observable and testable
- UI structure must match attached reference image
- All options must be accessible via popover
- No hidden state, all config via UI
