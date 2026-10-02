# CountdownTimerBar

Minimalist, hackable countdown timer for your macOS status bar. Built for coders, makers, and productivity geeks who want a Pomodoro-style timer always at hand — but hate bloat.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset=".github/hero-dark.png">
    <img alt="CountdownTimerBar: a timer pill in the menu bar and its popover with Focus and Rest presets" src=".github/hero-light.png" width="360">
  </picture>
</p>

## Features

- ⚡️ **One click to start:** Pick a Focus or Rest preset in the popover; click it again to pause or resume
- ⏱ **The time where you look:** A pill in the menu bar counts down, and dims when paused
- 🎚 **Presets you can reshape in seconds:** Drag a circle up or down to adjust it, double-click or just type to set an exact value, add up to eight per column
- 🕒 **Minutes and seconds:** Write your own list, e.g. `25, 5m, 30s`
- 🟣 **A ring that shows what's left:** The running preset drains clockwise, once per second
- ⌨️ **Keyboard-first when you want it:** `Tab` through the circles, `Space` to start, `Enter` or a digit to edit, `↑`/`↓` to nudge, `Delete` to remove
- 🔔 **Native notification & sound:** Get told when time is up; the sound is optional
- 🎨 **Distraction-free:** No Dock icon, no windows, no nags
- 🌗 **Follows your Mac:** Dark and light, switching with the system
- 💾 **Remembers everything:** Presets and options persist between launches

## Install

**Download:** grab the latest build from [Releases](https://github.com/Bazai/CountdownTimerBar/releases), unzip it and drag `CountdownTimerBar.app` to Applications. Builds are unsigned unless a release says otherwise: on first launch, right-click the app and choose **Open**.

**Or build it yourself** (needs macOS 15+ and [mise](https://mise.jdx.dev)):

```sh
git clone https://github.com/Bazai/CountdownTimerBar.git
cd CountdownTimerBar
mise install
make build        # creates releases/CountdownTimerBar.app
open releases/CountdownTimerBar.app
```

> **Note:** On first finish, macOS asks to allow notifications. If you said no, enable them in System Settings → Notifications.

## Usage

- Click the pill in your status bar to open the popover
- Click a preset to start it. Click the running one to pause, again to resume. The ■ button stops
- Adjust a preset by dragging its circle up or down; double-click it (or focus it and type digits) to enter an exact value. `m` and `s` switch minutes and seconds
- Hover a circle for the × to remove it; **+** adds a new one
- Open the gear ⚙️ for options:
  - Edit the Focus and Rest lists as text (`10`, `30s`, `5m`, …), kept in sync with the circles
  - Toggle the sound
  - About and Quit (`⌘Q`)
- The running preset can't be edited or removed, so you never lose a countdown by accident

## Why?

- No bloated Pomodoro apps
- No Dock clutter
- No distractions
- Just a timer, always where you need it

## License

MIT — do what you want, but don't blame me if you miss a meeting.

---

# For contributors and agents

PRs welcome! If you have an idea or spot a bug, open an issue or a pull request. This half of the README is the working manual for humans and AI agents alike.

## Quick start

```sh
mise install       # Rust 1.98.1, pinned in mise.toml
cargo run          # run from the working tree
make check         # fmt check + clippy (-D warnings) + all tests; run before every commit
```

`cargo run` works without a bundle; notifications need the bundled app (`make build`) because macOS requires a bundle identifier for them.

| Command | What it does |
| --- | --- |
| `make build` | Release bundle `releases/CountdownTimerBar.app` (unsigned unless signing variables are set) |
| `make dist` | `make build`, then `releases/CountdownTimerBar-v<version>.zip` for the GitHub release |
| `make license` | Set the LICENSE years to `<first year>-<current year>`; the About card shows the same text |
| `make agents` | Link `.agents/skills` into `.claude/skills`; create `CLAUDE.md` (`@AGENTS.md`) if missing |
| `make run` | Build, stop a running copy, open the new one |
| `make check` | `fmt-check`, `lint`, `test` |
| `make release` | Tag `v<Cargo.toml version>` and push branch and tag to `origin` (`DRY_RUN=1` to rehearse) |
| `make fmt` / `make lint` / `make test` | The parts of `check` |
| `UPDATE_GOLDEN=1 cargo test --test visual` | Rewrite the golden images after an intended visual change |

Signing and notarization happen in `packaging/build-app.sh` when `APPLE_SIGNING_IDENTITY`, `APPLE_TEAM_ID` and `APPLE_NOTARY_KEYCHAIN_PROFILE` are all set. `APPLE_NOTARY_KEYCHAIN_PROFILE` names credentials stored with `xcrun notarytool store-credentials`; never put Apple ID passwords or API keys in scripts or arguments.

## Source layout

Rust 2021, GPUI through `gpui-kit`. The crate is a library plus a one-line binary.

| Path | Responsibility |
| --- | --- |
| `src/domain` | Pure model: countdown state machine over an injectable `Clock`, durations, presets (`PresetList` owns slots, ids and the limit of eight), settings over a `KeyValueStore`. No UI, no platform |
| `src/theme` | Palettes (dark is the reference, light is derived from it), metrics, and their application to `gpui-component` |
| `src/gui` | `state.rs` (`AppState` behind ports), `panel.rs` (the popover window), `popover/` (views), `gesture.rs`, `inline_edit.rs`, `timer_circle.rs` |
| `src/macos` | Everything that touches AppKit: status item, notifications, user defaults, appearance observer, activation policy |
| `src/testing.rs` | Test harness (`test-support` feature): the real popover over a manual clock, an in-memory store and spies |
| `tests/` | `ui_flows` (interactions), `visual` + `golden/` (screenshots), and tests of the public `domain` API |
| `docs/adr` | Architecture decision records |

Design rules worth knowing before you change anything:

- `AppState` reaches the platform only through `StatusDisplay`, `Notifier`, `KeyValueStore` and `Clock`. New side effects get a port, not a direct call.
- Illegal states are unrepresentable where it is cheap: `NonZeroU32` durations, one `Interaction` enum for press and edit, `PresetList` for ids and limits.
- Colours and sizes come from `cx.palette()` and `theme::metrics`; views never write a hex value.

## Testing

Three layers, all in `cargo test`:

1. **Unit and integration tests** for `domain`, `theme` and gestures, plus doc tests on the public API.
2. **`tests/ui_flows.rs`** drives the real popover on GPUI's headless test platform with real hit testing and key bindings: clicks, double clicks, drags, keys, the settings input. Time is a `ManualClock`, so countdowns are exact. Elements are found by id; give new interactive elements an id and `.test_support()`.
3. **`tests/visual.rs`** renders every state in both themes on GPUI's headless Metal renderer and compares with `tests/golden/*.png` (channel tolerance 2, at most 0.05 % of pixels). It runs with `harness = false` on the main thread and is skipped outside macOS. On a mismatch, `target/visual/` holds the actual image and a diff.

Rules for tests:

- One behaviour per test, named for the behaviour; no narrating comments.
- A UI change comes with a flow test, or a changed golden image that you looked at before committing it.
- Tests never touch real user defaults or the real menu bar: use `MemoryStore` and the spies from `testing`.
- Details and the reasoning: `docs/adr/0006-ui-testing-on-headless-gpui.md`.

The two images at the top of this README (`.github/hero-*.png`) are composites: a headless render of the running popover over a generated wallpaper and menu bar, with the pill drawn to match. Refresh them when the look changes.

## Code standards

- Everything in the repository is English: code, comments, commit messages, docs. (Conversation with the maintainer may be in Russian, see below.)
- Comments only where code cannot say it: platform or dependency quirks, `SAFETY` notes, public API contracts, ADR references. No narration, no history, no references to documents that may disappear.
- Lints are policy, not advice: `clippy::all` is an error, `pedantic` a warning, `-D warnings` in `make lint`. Silence a lint locally with `#[expect(clippy::x, reason = "…")]`, never globally. No `unwrap`/`expect`/`panic` outside tests and the test harness. Stderr goes through `diag::report`. See `docs/adr/0005-lint-policy-and-module-layout.md`.
- Commits follow the Sentry convention (`feat(scope): Subject`, imperative, ≤ 70 chars, body explains why). The `commit` skill in `.claude/skills` writes them.
- Decisions that change behaviour or structure get an ADR in `docs/adr`.

## Notes for AI agents

- Read `AGENTS.md` first. Talk to the maintainer in Russian; write code, comments, commits and docs in English. Do not spawn `gpt-6-astra`, and never escalate the subagent model after failures.
- Skills live in `.agents/skills`; run `make agents` to link them into `.claude/skills` (generated, not committed) and to create `CLAUDE.md`, which imports `AGENTS.md`. Use `rust-best-practices` for Rust work and `commit` for commits.
- Before saying a task is done, run `make check`. For visual work, also look at the changed golden images, not just the test result.
- Don't commit or push unless asked. Don't leave experiments in the tree; keep throwaway code out of commits.
- Don't add dependencies without a reason that survives review; this project values few of them.

## Architecture decisions

| ADR | Decision |
| --- | --- |
| [0001](docs/adr/0001-no-drag-to-remove.md) | No drag-out removal: remove with × or Delete |
| [0002](docs/adr/0002-keyboard-entry-into-inline-edit.md) | Enter or a digit opens the inline editor from the keyboard |
| [0003](docs/adr/0003-radial-progress-on-active-timer.md) | The running circle shows a radial progress ring |
| [0004](docs/adr/0004-theme-layer-and-system-appearance.md) | A typed theme layer; light derived from dark; follows the system |
| [0005](docs/adr/0005-lint-policy-and-module-layout.md) | Lint policy, lib + bin split, module layout |
| [0006](docs/adr/0006-ui-testing-on-headless-gpui.md) | UI tests on GPUI's headless platform and renderer |

## Maintenance

- **Known warning:** builds print a future-incompatibility note about `block v0.1.6`. It comes from `gpui-kit`'s dependency chain and cannot be fixed here; it does not affect the build.
- **Dependencies:** GPUI is consumed through `gpui-kit` 0.7. After `cargo update`, run `make check`; golden images can shift with a new GPUI, macOS or font version. Review the diffs, then regenerate with `UPDATE_GOLDEN=1`.
- **Copyright:** the notice in `LICENSE` is the only place the years are written. `make license` updates them, the About card reads the same line (`countdown_timer_bar::copyright()`), and `make release` refuses a release whose LICENSE years are not current.
- **Versioning:** `Cargo.toml` is the only place a version is written. The About card shows it (`countdown_timer_bar::VERSION`) and `build-app.sh` stamps it into the bundle's `Info.plist`.
- **Releasing:** bump `version` in `Cargo.toml` (and `Cargo.lock` follows), commit, then `make release`. It refuses a dirty tree or an existing tag, runs `make check`, creates the annotated tag `v<version>` and pushes the branch and the tag to `origin`. `make release DRY_RUN=1` stops before tagging. Then `make dist` with the signing variables set and attach `releases/CountdownTimerBar-v<version>.zip` to the GitHub release for that tag.
- **Storage:** settings live in the `baz.CountdownTimerBar` user-defaults domain under `focusTimers`, `restTimers` and `soundOn`, the same keys as the earlier Swift version, so its settings carry over. Don't rename them.
- **Platform minimum:** macOS 15 (`LSMinimumSystemVersion`).
