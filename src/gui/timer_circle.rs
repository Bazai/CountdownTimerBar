//! Visual building blocks of one round preset button.
//!
//! Interaction (click / knob / edit / hover) lives in `popover.rs`; this
//! module only knows how a circle looks in each state.

use std::sync::Arc;

use gpui_kit::base::ObservedElement;
use gpui_kit::prelude::*;
use gpui_kit::TestSupportExt as _;
use gpui_kit::{
    canvas, div, point, px, AbsoluteLength, BoxShadow, Div, ElementId, FontFeatures, FontWeight,
    Hsla, PathBuilder, SharedString, Stateful,
};

use crate::theme::{metrics, Palette};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visual {
    Normal,
    Active,
    Paused,
    Knob,
    Edit,
}

pub fn tabular_numbers() -> FontFeatures {
    FontFeatures(Arc::new(vec![("tnum".into(), 1)]))
}

/// Sets the same border width on all four sides (GPUI only has presets).
pub fn border_width<E: Styled>(mut element: E, width: f32) -> E {
    let style = element.style();
    let width = Some(AbsoluteLength::Pixels(px(width)));
    style.border_widths.top = width;
    style.border_widths.right = width;
    style.border_widths.bottom = width;
    style.border_widths.left = width;
    element
}

fn text_colors(p: &Palette, visual: Visual) -> (Hsla, Hsla) {
    match visual {
        Visual::Normal => (p.text.hsla(), p.text_subtle.hsla()),
        Visual::Active | Visual::Knob | Visual::Edit => (p.text_active.hsla(), p.icon.hsla()),
        Visual::Paused => (p.text_body.hsla(), p.text_subtle.hsla()),
    }
}

pub fn shell(
    p: &Palette,
    id: impl Into<ElementId>,
    visual: Visual,
    aria_label: impl Into<SharedString>,
) -> ObservedElement<Stateful<Div>> {
    let p = *p;
    let (text, _) = text_colors(&p, visual);
    let base = div()
        .id(id)
        .aria_label(aria_label)
        .size(px(metrics::CIRCLE_SIZE))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .text_color(text);
    let styled = match visual {
        Visual::Normal => base
            .cursor_pointer()
            .bg(p.control.hsla())
            .border_1()
            .border_color(p.control_border.hsla())
            .hover(|s| {
                s.bg(p.control_hover.hsla())
                    .border_color(p.control_hover_border.hsla())
            })
            .focus_visible(|s| s.border_color(p.accent.hsla())),
        // The 2 px border of an active circle is the progress ring's track;
        // `progress_ring` paints the remaining arc over it.
        Visual::Active => base
            .cursor_pointer()
            .bg(p.accent.alpha(metrics::accent::FILL))
            .border_2()
            .border_color(p.accent.alpha(metrics::accent::TRACK))
            .focus_visible(|s| s.border_color(p.focus_ring.hsla()))
            .shadow(vec![BoxShadow {
                color: p.accent.alpha(metrics::accent::HALO),
                offset: point(px(0.), px(0.)),
                blur_radius: px(0.),
                spread_radius: px(4.),
                inset: false,
            }]),
        Visual::Paused => base
            .cursor_pointer()
            .bg(p.accent.alpha(metrics::accent::PAUSED_FILL))
            .border_2()
            .border_color(p.accent.alpha(metrics::accent::PAUSED_TRACK))
            .focus_visible(|s| s.border_color(p.focus_ring.hsla())),
        Visual::Knob => base
            .cursor_ns_resize()
            .bg(p.control_hover.hsla())
            .border_2()
            .border_color(p.accent.hsla())
            .shadow(vec![BoxShadow {
                color: p.shadow.alpha(p.elevation.knob_opacity()),
                offset: point(px(0.), px(8.)),
                blur_radius: px(20.),
                spread_radius: px(0.),
                inset: false,
            }]),
        Visual::Edit => base
            .cursor_text()
            .bg(p.input_bg.hsla())
            .border_2()
            .border_color(p.accent.hsla())
            .shadow(vec![BoxShadow {
                color: p.accent.alpha(metrics::accent::EDIT_HALO),
                offset: point(px(0.), px(0.)),
                blur_radius: px(0.),
                spread_radius: px(4.),
                inset: false,
            }]),
    };
    styled.test_support()
}

const RING_WIDTH: f32 = 2.;
const RING_RADIUS: f32 = 21.;

/// End point of an arc that starts at 12 o'clock and runs clockwise over
/// `fraction` of the circle (`0.25` is 3 o'clock, `0.5` is 6 o'clock).
pub fn arc_end(center: (f32, f32), radius: f32, fraction: f32) -> (f32, f32) {
    let angle = fraction * std::f32::consts::TAU;
    (
        center.0 + radius * angle.sin(),
        center.1 - radius * angle.cos(),
    )
}

/// Radial progress of the active timer: the share still left, drawn from
/// 12 o'clock clockwise over the circle's border (which acts as the track).
/// Paused timers keep the frozen arc, dimmed.
pub fn progress_ring(p: &Palette, fraction: f32, paused: bool) -> impl IntoElement {
    let arc_color = if paused {
        p.accent.alpha(metrics::accent::PAUSED_ARC)
    } else {
        p.accent.hsla()
    };
    canvas(
        |_, _, _| {},
        move |bounds, (), window, _| {
            if fraction <= 0.002 {
                return;
            }
            let center = bounds.center();
            let radius = px(RING_RADIUS);
            let top = point(center.x, center.y - radius);
            let radii = point(radius, radius);
            let mut path = PathBuilder::stroke(px(RING_WIDTH));
            path.move_to(top);
            if fraction >= 0.999 {
                // An arc cannot start and end on the same point: two halves.
                let bottom = point(center.x, center.y + radius);
                path.arc_to(radii, px(0.), false, true, bottom);
                path.arc_to(radii, px(0.), false, true, top);
            } else {
                let (x, y) = arc_end(
                    (center.x.as_f32(), center.y.as_f32()),
                    RING_RADIUS,
                    fraction,
                );
                path.arc_to(radii, px(0.), fraction > 0.5, true, point(px(x), px(y)));
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, arc_color);
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

fn value_text(value: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(15.))
        .font_weight(FontWeight::SEMIBOLD)
        .font_features(tabular_numbers())
        .child(value.into())
}

fn unit_text(p: &Palette, unit: &'static str, visual: Visual) -> Div {
    div()
        .text_size(px(10.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(text_colors(p, visual).1)
        .child(unit)
}

pub fn value_label(p: &Palette, value: u32, unit: &'static str, visual: Visual) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(1.))
        .child(value_text(value.to_string()))
        .child(unit_text(p, unit, visual))
}

pub fn edit_label(p: &Palette, text: &str, unit: &'static str) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(1.))
        .child(value_text(text.to_owned()))
        .child(div().w(px(1.5)).h(px(17.)).mx(px(1.)).bg(p.accent.hsla()))
        .child(unit_text(p, unit, Visual::Edit))
}

#[cfg(test)]
mod tests {
    use super::arc_end;

    fn close(a: (f32, f32), b: (f32, f32)) -> bool {
        (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
    }

    #[test]
    fn arc_starts_at_twelve_and_runs_clockwise() {
        let c = (100., 100.);
        assert!(close(arc_end(c, 20., 0.), (100., 80.))); // 12 o'clock
        assert!(close(arc_end(c, 20., 0.25), (120., 100.))); // 3 o'clock
        assert!(close(arc_end(c, 20., 0.5), (100., 120.))); // 6 o'clock
        assert!(close(arc_end(c, 20., 0.75), (80., 100.))); // 9 o'clock
    }
}
