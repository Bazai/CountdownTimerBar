//! The popover driven the way a user drives it: real hit testing, key bindings
//! and a manual clock, with no platform behind it.

#![expect(
    clippy::expect_used,
    reason = "test helpers state their preconditions with expect"
)]

use std::time::Duration;

use countdown_timer_bar::domain::countdown::{Event, Phase};
use countdown_timer_bar::domain::preset::PresetKind;
use countdown_timer_bar::testing::{MemoryStore, Session};
use countdown_timer_bar::theme::Appearance;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{AppContext as _, ElementId, TestAppContext};

fn open(cx: &mut TestAppContext) -> Session {
    let session = cx.update(|cx| Session::open(cx, Appearance::Dark, MemoryStore::default()));
    frame(cx, &session);
    session
}

fn frame(cx: &mut TestAppContext, session: &Session) {
    cx.update_window(session.window, |_, window, cx| window.render_frame(cx))
        .expect("window is open");
}

fn click(cx: &mut TestAppContext, session: &Session, id: impl Into<ElementId>) {
    let id = id.into();
    cx.update_window(session.window, |_, window, cx| window.click(id, cx))
        .expect("window is open");
    frame(cx, session);
}

fn double_click(cx: &mut TestAppContext, session: &Session, id: impl Into<ElementId>) {
    let id = id.into();
    cx.update_window(session.window, |_, window, cx| window.double_click(id, cx))
        .expect("window is open");
    frame(cx, session);
}

fn hover(cx: &mut TestAppContext, session: &Session, id: impl Into<ElementId>) {
    let id = id.into();
    cx.update_window(session.window, |_, window, cx| window.hover(id, cx))
        .expect("window is open");
    frame(cx, session);
}

fn press(cx: &mut TestAppContext, session: &Session, keys: &[&str]) {
    for key in keys {
        cx.update_window(session.window, |_, window, cx| window.press(key, cx))
            .expect("window is open");
        frame(cx, session);
    }
}

/// Moves time forward one second at a time so every tick fires.
fn wait(cx: &mut TestAppContext, session: &Session, seconds: u64) {
    for _ in 0..seconds {
        session.clock.advance(Duration::from_secs(1));
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
    }
}

/// A click on an idle circle waits out the double-click window before it starts.
fn settle_click(cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
}

fn phase(cx: &TestAppContext, session: &Session) -> Phase {
    cx.read(|cx| session.phase(cx))
}

fn column(cx: &TestAppContext, session: &Session, kind: PresetKind) -> String {
    cx.read(|cx| session.column_text(cx, kind))
}

const FIRST_FOCUS: (&str, usize) = ("circle", 0);
const FIRST_REST: (&str, usize) = ("circle", 100);

#[gpui_kit::test]
fn clicking_a_circle_starts_it_after_the_double_click_window(cx: &mut TestAppContext) {
    let session = open(cx);
    assert_eq!(phase(cx, &session), Phase::Idle);

    click(cx, &session, FIRST_FOCUS);
    settle_click(cx);

    assert_eq!(phase(cx, &session), Phase::Running);
}

#[gpui_kit::test]
fn the_header_shows_the_started_preset(cx: &mut TestAppContext) {
    let session = open(cx);

    click(cx, &session, FIRST_FOCUS);
    settle_click(cx);

    assert_eq!(cx.read(|cx| session.clock_label(cx)), "15:00");
}

#[gpui_kit::test]
fn clicking_the_running_circle_pauses_it(cx: &mut TestAppContext) {
    let session = open(cx);
    click(cx, &session, FIRST_FOCUS);
    settle_click(cx);

    click(cx, &session, FIRST_FOCUS);

    assert_eq!(phase(cx, &session), Phase::Paused);
}

#[gpui_kit::test]
fn clicking_the_paused_circle_resumes_it(cx: &mut TestAppContext) {
    let session = open(cx);
    click(cx, &session, FIRST_FOCUS);
    settle_click(cx);
    click(cx, &session, FIRST_FOCUS);

    click(cx, &session, FIRST_FOCUS);

    assert_eq!(phase(cx, &session), Phase::Running);
}

#[gpui_kit::test]
fn the_menu_bar_item_dims_while_paused(cx: &mut TestAppContext) {
    let session = open(cx);
    click(cx, &session, FIRST_FOCUS);
    settle_click(cx);

    click(cx, &session, FIRST_FOCUS);

    assert_eq!(session.status_label(), Some(("15:00".to_owned(), true)));
}

#[gpui_kit::test]
fn time_passing_while_paused_is_not_counted(cx: &mut TestAppContext) {
    let session = open(cx);
    click(cx, &session, FIRST_REST);
    settle_click(cx);
    wait(cx, &session, 10);
    click(cx, &session, FIRST_REST);

    wait(cx, &session, 30);

    assert_eq!(cx.read(|cx| session.clock_label(cx)), "00:50");
}

#[gpui_kit::test]
fn a_finished_timer_goes_idle_and_notifies_once(cx: &mut TestAppContext) {
    let session = open(cx);
    click(cx, &session, FIRST_REST);
    settle_click(cx);

    wait(cx, &session, 61);

    let finished = session
        .events()
        .iter()
        .filter(|(event, _)| matches!(event, Event::Finished(_)))
        .count();
    assert_eq!((phase(cx, &session), finished), (Phase::Idle, 1));
}

#[gpui_kit::test]
fn turning_sound_on_is_remembered_and_reaches_the_notifier(cx: &mut TestAppContext) {
    let session = open(cx);
    click(cx, &session, "settings");
    click(cx, &session, "sound-toggle");
    click(cx, &session, FIRST_REST);
    settle_click(cx);

    wait(cx, &session, 61);

    let sound_at_finish = session
        .events()
        .into_iter()
        .find_map(|(event, sound)| matches!(event, Event::Finished(_)).then_some(sound));
    assert_eq!(sound_at_finish, Some(true));
}

#[gpui_kit::test]
fn double_click_edits_a_value_and_enter_saves_it(cx: &mut TestAppContext) {
    let session = open(cx);

    double_click(cx, &session, FIRST_FOCUS);
    press(cx, &session, &["2", "enter"]);

    assert_eq!(column(cx, &session, PresetKind::Focus), "2, 30, 35");
}

#[gpui_kit::test]
fn a_saved_edit_reaches_the_store(cx: &mut TestAppContext) {
    let session = open(cx);

    double_click(cx, &session, FIRST_FOCUS);
    press(cx, &session, &["2", "enter"]);

    assert_eq!(
        session.store.array("focusTimers"),
        Some(vec!["120".into(), "1800".into(), "2100".into()])
    );
}

#[gpui_kit::test]
fn escape_cancels_an_edit(cx: &mut TestAppContext) {
    let session = open(cx);

    double_click(cx, &session, FIRST_FOCUS);
    press(cx, &session, &["2", "escape"]);

    assert_eq!(column(cx, &session, PresetKind::Focus), "15, 30, 35");
}

#[gpui_kit::test]
fn the_plus_button_adds_a_preset(cx: &mut TestAppContext) {
    let session = open(cx);

    click(cx, &session, ("add", 0usize));
    press(cx, &session, &["9", "enter"]);

    assert_eq!(column(cx, &session, PresetKind::Focus), "15, 30, 35, 9");
}

#[gpui_kit::test]
fn the_remove_badge_deletes_a_preset(cx: &mut TestAppContext) {
    let session = open(cx);

    hover(cx, &session, FIRST_FOCUS);
    click(cx, &session, ("remove", 0usize));

    assert_eq!(column(cx, &session, PresetKind::Focus), "30, 35");
}

#[gpui_kit::test]
fn the_running_circle_has_no_remove_badge(cx: &mut TestAppContext) {
    let session = open(cx);
    click(cx, &session, FIRST_FOCUS);
    settle_click(cx);

    hover(cx, &session, FIRST_FOCUS);

    let badge = cx
        .update_window(session.window, |_, window, _| {
            window.try_find(("remove", 0usize)).is_some()
        })
        .expect("window is open");
    assert!(!badge);
}

#[gpui_kit::test]
fn dragging_a_circle_up_raises_its_value(cx: &mut TestAppContext) {
    let session = open(cx);

    cx.update_window(session.window, |_, window, cx| {
        let center = window.find(FIRST_FOCUS).bounds().center();
        let mut to = center;
        to.y -= gpui_kit::px(12.);
        window.drag(center, to, cx);
    })
    .expect("window is open");
    frame(cx, &session);

    assert_eq!(column(cx, &session, PresetKind::Focus), "17, 30, 35");
}

fn exists(cx: &mut TestAppContext, session: &Session, id: impl Into<ElementId>) -> bool {
    let id = id.into();
    cx.update_window(session.window, |_, window, _| window.try_find(id).is_some())
        .expect("window is open")
}

#[gpui_kit::test]
fn tab_then_space_starts_the_first_circle(cx: &mut TestAppContext) {
    let session = open(cx);

    press(cx, &session, &["tab", "space"]);

    assert_eq!(phase(cx, &session), Phase::Running);
}

#[gpui_kit::test]
fn enter_on_a_focused_circle_opens_the_editor(cx: &mut TestAppContext) {
    let session = open(cx);

    press(cx, &session, &["tab", "enter", "4", "enter"]);

    assert_eq!(column(cx, &session, PresetKind::Focus), "4, 30, 35");
}

#[gpui_kit::test]
fn a_digit_on_a_focused_circle_starts_typing(cx: &mut TestAppContext) {
    let session = open(cx);

    press(cx, &session, &["tab", "4", "5", "enter"]);

    assert_eq!(column(cx, &session, PresetKind::Focus), "45, 30, 35");
}

#[gpui_kit::test]
fn the_arrow_keys_adjust_a_focused_circle(cx: &mut TestAppContext) {
    let session = open(cx);

    press(cx, &session, &["tab", "up", "up", "down"]);

    assert_eq!(column(cx, &session, PresetKind::Focus), "16, 30, 35");
}

#[gpui_kit::test]
fn delete_removes_a_focused_circle(cx: &mut TestAppContext) {
    let session = open(cx);

    press(cx, &session, &["tab", "delete"]);

    assert_eq!(column(cx, &session, PresetKind::Focus), "30, 35");
}

#[gpui_kit::test]
fn typing_s_in_the_editor_switches_to_seconds(cx: &mut TestAppContext) {
    let session = open(cx);

    double_click(cx, &session, FIRST_FOCUS);
    press(cx, &session, &["9", "s", "enter"]);

    assert_eq!(column(cx, &session, PresetKind::Focus), "9s, 30, 35");
}

#[gpui_kit::test]
fn the_unit_toggle_switches_a_hovered_circle_to_seconds(cx: &mut TestAppContext) {
    let session = open(cx);

    hover(cx, &session, FIRST_FOCUS);
    click(cx, &session, ("unit", 1usize));

    assert_eq!(column(cx, &session, PresetKind::Focus), "15s, 30, 35");
}

#[gpui_kit::test]
fn the_running_circle_cannot_be_edited(cx: &mut TestAppContext) {
    let session = open(cx);
    click(cx, &session, FIRST_FOCUS);
    settle_click(cx);

    double_click(cx, &session, FIRST_FOCUS);
    press(cx, &session, &["5", "enter"]);

    assert_eq!(column(cx, &session, PresetKind::Focus), "15, 30, 35");
}

#[gpui_kit::test]
fn a_full_column_hides_the_plus_button(cx: &mut TestAppContext) {
    let session = open(cx);

    for _ in 0..5 {
        click(cx, &session, ("add", 0usize));
        press(cx, &session, &["7", "enter"]);
    }

    assert!(!exists(cx, &session, ("add", 0usize)));
}

#[gpui_kit::test]
fn the_about_card_opens_from_its_link_and_closes_on_escape(cx: &mut TestAppContext) {
    let session = open(cx);
    click(cx, &session, "settings");
    click(cx, &session, "about-link");
    let opened = exists(cx, &session, "about-card");

    press(cx, &session, &["escape"]);

    assert_eq!((opened, exists(cx, &session, "about-card")), (true, false));
}

#[gpui_kit::test]
fn a_valid_list_in_the_settings_input_rewrites_the_circles(cx: &mut TestAppContext) {
    let session = open(cx);
    click(cx, &session, "settings");

    click(cx, &session, ("list-input", 0usize));
    press(cx, &session, &["cmd-a"]);
    cx.update_window(session.window, |_, window, cx| window.input("5, 45s", cx))
        .expect("window is open");
    frame(cx, &session);

    assert_eq!(column(cx, &session, PresetKind::Focus), "5, 45s");
}

#[gpui_kit::test]
fn an_invalid_list_keeps_the_last_valid_circles(cx: &mut TestAppContext) {
    let session = open(cx);
    click(cx, &session, "settings");

    click(cx, &session, ("list-input", 0usize));
    press(cx, &session, &["cmd-a"]);
    cx.update_window(session.window, |_, window, cx| window.input("5, abc", cx))
        .expect("window is open");
    frame(cx, &session);

    assert_eq!(column(cx, &session, PresetKind::Focus), "15, 30, 35");
}
