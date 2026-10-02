//! Small stateless building blocks of the popover.

use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::base::ObservedElement;
use gpui_kit::component::{button::Button, Sizable as _, Size};
use gpui_kit::prelude::*;
use gpui_kit::TestSupportExt as _;
use gpui_kit::{canvas, div, px, svg, Bounds, Div, FontWeight, IntoElement, Pixels};

use crate::theme::{metrics, Palette};

use super::{ROW_HEIGHT, ROW_PAD};

pub(super) fn icon_button(id: &'static str, label: &'static str) -> Button {
    Button::new(id)
        .with_size(Size::Small)
        .w(px(metrics::ICON_BUTTON_SIZE))
        .h(px(metrics::ICON_BUTTON_SIZE))
        .rounded(px(metrics::ICON_BUTTON_RADIUS))
        .accessibility_label(label)
        .tooltip(label)
}

pub(super) fn row_shell() -> Div {
    div()
        .relative()
        .w_full()
        .h(px(ROW_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
}

/// Zero-size element that records its parent's bounds while painting.
pub(super) fn measure(cell: &Rc<Cell<Bounds<Pixels>>>) -> impl IntoElement {
    let cell = cell.clone();
    canvas(move |bounds, _, _| cell.set(bounds), |_, (), _, _| {})
        .absolute()
        .top_0()
        .left_0()
        .size_full()
}

pub(super) fn small_square(
    p: &Palette,
    id: impl Into<gpui_kit::ElementId>,
    label: &'static str,
    selected: bool,
) -> ObservedElement<gpui_kit::Stateful<gpui_kit::Div>> {
    let base = div()
        .id(id)
        .test_support()
        .aria_label(label)
        .size(px(18.))
        .rounded(px(5.))
        .border_1()
        .border_color(p.control_border.hsla())
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer();
    if selected {
        base.bg(p.inverse_bg.hsla())
            .text_color(p.inverse_fg.hsla())
            .border_color(p.inverse_bg.hsla())
    } else {
        base.bg(p.control.hsla()).text_color(p.text_muted.hsla())
    }
}

/// Where the `+7` label starts: right beside the circle, or past the `m` / `s`
/// toggles when they are shown (an edit field being dragged).
pub(super) const LABEL_BESIDE_CIRCLE: f32 = 90.;
pub(super) const LABEL_PAST_TOGGLES: f32 = 108.;

pub(super) fn knob_marks<E: ParentElement + IntoElement>(
    p: &Palette,
    row: E,
    delta: i64,
    label_left: f32,
) -> E {
    row.child(
        svg()
            .absolute()
            .left(px(53.))
            .top(px(-9.))
            .path(gpui_kit::assets::IconName::ChevronUp.path())
            .size(px(10.))
            .text_color(p.accent.hsla()),
    )
    .child(
        svg()
            .absolute()
            .left(px(53.))
            .top(px(ROW_HEIGHT))
            .path(gpui_kit::assets::IconName::ChevronDown.path())
            .size(px(10.))
            .text_color(p.dashed.hsla()),
    )
    .when(delta != 0, |row| {
        row.child(
            div()
                .absolute()
                .left(px(label_left))
                .top(px(ROW_PAD + 13.))
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(p.accent.hsla())
                .child(format!("{delta:+}")),
        )
    })
}
