//! Adapts a content-sized GPUI layout to a separate native popover window.

use gpui_kit::prelude::*;
use gpui_kit::{div, px, size, AnyElement, App, Bounds, IntoElement, Pixels, RenderOnce, Window};

use crate::theme::{metrics, ThemeExt as _};

const TOP_GAP: f32 = 6.;

const RESIZE_THRESHOLD: f32 = 1.;

/// New bounds for the measured height, or `None` if no resize is needed.
pub(crate) fn resized_bounds(
    current: Bounds<Pixels>,
    visible: Bounds<Pixels>,
    measured_height: Pixels,
) -> Option<Bounds<Pixels>> {
    if (current.size.height - measured_height).abs().as_f32() < RESIZE_THRESHOLD {
        return None;
    }

    let width = current.size.width.min(visible.size.width);
    let height = measured_height.min(visible.size.height);
    let x = current
        .origin
        .x
        .min(visible.right() - width)
        .max(visible.left());
    let y = current
        .origin
        .y
        .min(visible.bottom() - height)
        .max(visible.top());
    Some(Bounds::new(gpui_kit::point(x, y), size(width, height)))
}

/// Chrome of the single transparent popover window: a rounded body with the palette's tokens.
#[derive(IntoElement)]
pub(crate) struct NativePopoverFrame {
    visible_bounds: Bounds<Pixels>,
    content: AnyElement,
}

impl NativePopoverFrame {
    pub(crate) fn new(visible_bounds: Bounds<Pixels>, content: impl IntoElement) -> Self {
        Self {
            visible_bounds,
            content: content.into_any_element(),
        }
    }
}

impl RenderOnce for NativePopoverFrame {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let visible_bounds = self.visible_bounds;
        let p = *cx.palette();

        div()
            .w_full()
            .flex()
            .flex_col()
            .child(div().h(px(TOP_GAP)))
            .child(
                div()
                    .w_full()
                    .rounded(px(metrics::POPOVER_RADIUS))
                    .bg(p.popover_bg.hsla())
                    .border_1()
                    .border_color(p.popover_border.hsla())
                    .child(self.content),
            )
            .on_children_prepainted(move |bounds, window, cx| {
                let measured_height = bounds
                    .iter()
                    .map(|bounds| bounds.size.height)
                    .fold(Pixels::ZERO, |height, child| height + child);
                let current = window.bounds();
                if let Some(next) = resized_bounds(current, visible_bounds, measured_height) {
                    window.defer(cx, move |window, _| window.resize(next.size));
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use gpui_kit::{point, size, Context, Render, WindowOptions};

    use super::*;

    struct Host;

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            NativePopoverFrame::new(
                Bounds::new(point(px(0.), px(0.)), size(px(2000.), px(2000.))),
                div().w(px(264.)).h(px(100.)),
            )
        }
    }

    /// Regression: `Root` paints `theme.background` over the whole window; an opaque
    /// one fills the gap above the card and its corners. Only the body may paint an
    /// opaque quad.
    // One test per appearance: each needs its own app context, because
    // `gpui_kit::init` starts a theme-reload task that must not run twice.
    #[gpui_kit::test]
    fn dark_theme_paints_nothing_opaque_above_the_rounded_body(cx: &mut gpui_kit::TestAppContext) {
        assert_nothing_opaque_above_the_body(cx, crate::theme::Appearance::Dark);
    }

    #[gpui_kit::test]
    fn light_theme_paints_nothing_opaque_above_the_rounded_body(cx: &mut gpui_kit::TestAppContext) {
        assert_nothing_opaque_above_the_body(cx, crate::theme::Appearance::Light);
    }

    fn assert_nothing_opaque_above_the_body(
        cx: &mut gpui_kit::TestAppContext,
        appearance: crate::theme::Appearance,
    ) {
        let (window, _) = cx
            .update(|cx| {
                gpui_kit::init(cx);
                crate::theme::set_appearance(cx, appearance);
                gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Host))
            })
            .unwrap();
        cx.run_until_parked();
        let (quads, scale) = window
            .update(cx, |_, window, _| {
                (window.painted_quads(), window.scale_factor())
            })
            .unwrap();

        let body_top = TOP_GAP * scale;
        let opaque: Vec<_> = quads
            .iter()
            .filter(|quad| !quad.background.is_transparent())
            .collect();
        assert!(!opaque.is_empty(), "the body must paint a background");
        for quad in opaque {
            assert!(
                quad.bounds.origin.y.0 >= body_top - 0.5,
                "{appearance:?}: an opaque quad starts at y={} above the body (y={body_top}): {:?} bg={:?}",
                quad.bounds.origin.y.0,
                quad.bounds,
                quad.background
            );
        }
    }
}
