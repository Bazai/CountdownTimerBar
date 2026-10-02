//! The preset text format and the preset columns through the public API.

#![expect(
    clippy::expect_used,
    reason = "test helpers state their preconditions with expect"
)]

use countdown_timer_bar::domain::preset::{IdGen, PresetKind, PresetList};
use countdown_timer_bar::domain::preset_list::{
    format_list, parse_list, ParseErrorReason, PresetSpec, Unit, MAX_PRESETS,
};

fn column(text: &str) -> (PresetList, IdGen) {
    let mut ids = IdGen::default();
    let list = PresetList::new(parse_list(text).expect("valid"), &mut ids);
    (list, ids)
}

#[test]
fn mixed_minutes_and_seconds_round_trip() {
    let specs = parse_list("13s, 1, 10, 2").expect("valid");
    assert_eq!(format_list(&specs), "13s, 1, 10, 2");
}

#[test]
fn a_bad_token_is_named_in_the_error() {
    let error = parse_list("5, 35фыва").expect_err("invalid");
    assert_eq!(error.token, "35фыва");
}

#[test]
fn nine_presets_are_too_many() {
    let error = parse_list("1,2,3,4,5,6,7,8,9").expect_err("too many");
    assert_eq!(error.reason, ParseErrorReason::TooMany);
}

#[test]
fn a_column_never_grows_past_the_limit() {
    let (mut list, mut ids) = column("1,2,3,4,5,6,7,8");
    let extra = PresetSpec::new(9, Unit::Minutes).expect("in range");
    assert!(list.push(extra, &mut ids).is_err());
    assert_eq!(list.len(), MAX_PRESETS);
}

#[test]
fn editing_the_text_keeps_the_identity_of_unchanged_presets() {
    let (mut list, mut ids) = column("5, 7");
    let kept = list.preset(PresetKind::Rest, 0).expect("slot 0").id;

    list.replace_all(parse_list("5, 9").expect("valid"), &mut ids);

    assert_eq!(list.index_of(kept), Some(0));
}

#[test]
fn a_stored_duration_comes_back_with_its_unit() {
    let (list, _) = column("90s, 15");
    let recovered: Vec<_> = list
        .durations()
        .into_iter()
        .map(PresetSpec::from_duration)
        .collect();
    assert_eq!(format_list(&recovered), "90s, 15");
}
