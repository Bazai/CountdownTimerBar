//! Pixel snapshots of the popover, rendered headlessly by GPUI's Metal renderer
//! and compared with the images in `tests/golden/`.
//!
//! `harness = false`: Metal and the text system need the main thread.
//! `UPDATE_GOLDEN=1 cargo test --test visual` rewrites the images; on a mismatch
//! `target/visual/` gets the actual image and a diff.

#![cfg_attr(
    target_os = "macos",
    expect(
        clippy::expect_used,
        reason = "a snapshot run that cannot open its window or write a file has nothing to report"
    )
)]

#[cfg(target_os = "macos")]
fn main() -> std::process::ExitCode {
    macos::run()
}

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("visual: skipped; the headless renderer is only used on macOS");
}

#[cfg(target_os = "macos")]
mod macos {
    use std::path::PathBuf;
    use std::process::ExitCode;
    use std::time::Duration;

    use countdown_timer_bar::testing::{pointer, MemoryStore, Session};
    use countdown_timer_bar::theme::Appearance;
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{ElementId, HeadlessAppContext};
    use image::RgbaImage;

    /// A channel may differ by this much before a pixel counts as changed.
    const CHANNEL_TOLERANCE: u8 = 2;
    /// Share of changed pixels that is still a pass.
    const MAX_CHANGED_SHARE: f64 = 0.0005;

    struct Case {
        name: &'static str,
        run: fn(&mut Ui, Appearance) -> RgbaImage,
    }

    const FIRST_FOCUS: (&str, usize) = ("circle", 0);
    const FIRST_REST: (&str, usize) = ("circle", 100);

    const CASES: &[Case] = &[
        Case {
            name: "idle",
            run: |ui, theme| ui.shot(theme, |_| {}),
        },
        Case {
            name: "settings",
            run: |ui, theme| ui.shot(theme, |ui| ui.click("settings")),
        },
        Case {
            name: "about",
            run: |ui, theme| {
                ui.shot(theme, |ui| {
                    ui.click("settings");
                    ui.click("about-link");
                })
            },
        },
        Case {
            name: "running",
            run: |ui, theme| ui.shot(theme, Ui::start_rest_and_wait),
        },
        Case {
            name: "paused",
            run: |ui, theme| {
                ui.shot(theme, |ui| {
                    ui.start_rest_and_wait();
                    ui.click(FIRST_REST);
                })
            },
        },
        Case {
            name: "edit",
            run: |ui, theme| ui.shot(theme, |ui| ui.double_click(FIRST_FOCUS)),
        },
        Case {
            name: "edit-knob",
            run: |ui, theme| {
                ui.shot(theme, |ui| {
                    ui.double_click(FIRST_FOCUS);
                    ui.hold_knob_up(FIRST_FOCUS, 12.);
                })
            },
        },
    ];

    struct Ui {
        // Dropped before `cx`: the leak check runs when the context goes away.
        session: Option<Session>,
        cx: HeadlessAppContext,
    }

    impl Ui {
        fn shot(&mut self, appearance: Appearance, setup: impl FnOnce(&mut Self)) -> RgbaImage {
            let session = self
                .cx
                .update(|cx| Session::open(cx, appearance, MemoryStore::default()));
            let window = session.window;
            self.session = Some(session);
            self.frame();
            setup(self);
            self.cx.run_until_parked();
            self.frame();
            self.cx
                .capture_screenshot(window)
                .expect("the Metal renderer is available")
        }

        fn with_window<R>(
            &mut self,
            f: impl FnOnce(&mut gpui_kit::Window, &mut gpui_kit::App) -> R,
        ) -> R {
            let window = self.session.as_ref().expect("a session is open").window;
            let result = self
                .cx
                .update_window(window, |_, window, cx| f(window, cx))
                .expect("window is open");
            self.frame();
            result
        }

        fn click(&mut self, id: impl Into<ElementId>) {
            let id = id.into();
            self.with_window(|window, cx| window.click(id, cx));
        }

        fn double_click(&mut self, id: impl Into<ElementId>) {
            let id = id.into();
            self.with_window(|window, cx| window.double_click(id, cx));
        }

        /// Presses a circle and drags `points` up without releasing.
        fn hold_knob_up(&mut self, id: impl Into<ElementId>, points: f32) {
            let id = id.into();
            self.with_window(|window, cx| {
                let center = window.find(id).bounds().center();
                pointer::down(window, center, cx);
                let mut target = center;
                target.y -= gpui_kit::px(points);
                pointer::drag_to(window, target, cx);
            });
        }

        /// Runs the 1-minute rest timer for 10 seconds.
        fn start_rest_and_wait(&mut self) {
            self.click(FIRST_REST);
            self.advance(Duration::from_millis(300));
            for _ in 0..10 {
                self.advance(Duration::from_secs(1));
            }
        }

        fn advance(&mut self, by: Duration) {
            self.session
                .as_ref()
                .expect("a session is open")
                .clock
                .advance(by);
            self.cx.advance_clock(by);
            self.cx.run_until_parked();
        }

        fn frame(&mut self) {
            let window = self.session.as_ref().expect("a session is open").window;
            self.cx
                .update_window(window, |_, window, cx| window.render_frame(cx))
                .expect("window is open");
        }
    }

    fn new_ui() -> Ui {
        let cx = HeadlessAppContext::with_platform(
            gpui_kit::platform::current_platform(true).text_system(),
            std::sync::Arc::new(countdown_timer_bar::testing::assets()),
            gpui_kit::platform::current_headless_renderer,
        );
        Ui { cx, session: None }
    }

    fn golden_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
    }

    fn output_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/visual")
    }

    /// Pixels whose channels differ by more than the tolerance, and a red-on-black diff.
    fn compare(expected: &RgbaImage, actual: &RgbaImage) -> (usize, RgbaImage) {
        let mut diff = RgbaImage::new(actual.width(), actual.height());
        if expected.dimensions() != actual.dimensions() {
            return (usize::MAX, diff);
        }
        let mut changed = 0;
        for (x, y, pixel) in actual.enumerate_pixels() {
            let other = expected.get_pixel(x, y);
            let off = pixel
                .0
                .iter()
                .zip(other.0)
                .any(|(a, b)| a.abs_diff(b) > CHANNEL_TOLERANCE);
            if off {
                changed += 1;
                diff.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
            }
        }
        (changed, diff)
    }

    pub fn run() -> ExitCode {
        let args: Vec<String> = std::env::args().skip(1).collect();
        if args.iter().any(|arg| arg == "--list") {
            for case in CASES {
                for theme in ["dark", "light"] {
                    println!("{}-{theme}: test", case.name);
                }
            }
            return ExitCode::SUCCESS;
        }
        let filter = args.iter().find(|arg| !arg.starts_with('-'));
        let update = std::env::var_os("UPDATE_GOLDEN").is_some();
        let mut failures = Vec::new();
        for case in CASES {
            for (theme, appearance) in [("dark", Appearance::Dark), ("light", Appearance::Light)] {
                let name = format!("{}-{theme}", case.name);
                if filter.is_some_and(|filter| !name.contains(filter.as_str())) {
                    continue;
                }
                let mut ui = new_ui();
                let actual = (case.run)(&mut ui, appearance);
                let path = golden_dir().join(format!("{name}.png"));
                if update {
                    std::fs::create_dir_all(golden_dir()).expect("create golden dir");
                    actual.save(&path).expect("write golden image");
                    println!("updated {name}");
                    continue;
                }
                let Ok(expected) = image::open(&path).map(|image| image.to_rgba8()) else {
                    failures.push(format!("{name}: no golden image; run with UPDATE_GOLDEN=1"));
                    continue;
                };
                let (changed, diff) = compare(&expected, &actual);
                let total = f64::from(actual.width()) * f64::from(actual.height());
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "pixel counts are far below 2^52"
                )]
                let share = changed as f64 / total;
                if share > MAX_CHANGED_SHARE {
                    std::fs::create_dir_all(output_dir()).expect("create output dir");
                    actual
                        .save(output_dir().join(format!("{name}.actual.png")))
                        .expect("write actual image");
                    diff.save(output_dir().join(format!("{name}.diff.png")))
                        .expect("write diff image");
                    failures.push(format!("{name}: {changed} pixels differ ({share:.3})"));
                } else {
                    println!("ok {name}");
                }
            }
        }
        if failures.is_empty() {
            ExitCode::SUCCESS
        } else {
            for failure in &failures {
                println!("FAILED {failure}");
            }
            ExitCode::FAILURE
        }
    }
}
