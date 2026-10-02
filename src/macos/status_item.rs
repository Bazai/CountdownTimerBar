use block2::RcBlock;
use gpui_kit::{point, px, size, Bounds, DisplayId, Pixels};
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{define_class, msg_send, sel, AnyThread, DefinedClass};
use objc2_app_kit::{
    NSBezierPath, NSCellImagePosition, NSColor, NSFont, NSFontAttributeName, NSFontWeightMedium,
    NSForegroundColorAttributeName, NSGraphicsContext, NSImage, NSImageInterpolation, NSStatusBar,
    NSStatusItem, NSStringDrawing, NSVariableStatusItemLength,
};
use objc2_foundation::{
    MainThreadMarker, NSAttributedStringKey, NSDictionary, NSNumber, NSPoint, NSRect, NSSize,
    NSString,
};

use super::pill::pill_geometry;

/// Button geometry in logical points from the top-left of its screen.
pub struct StatusItemAnchor {
    pub bounds: Bounds<Pixels>,
    pub visible_bounds: Bounds<Pixels>,
    pub display_id: DisplayId,
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "AppKit geometry is f64, GPUI pixels are f32; screen coordinates fit exactly"
)]
fn screen_bounds(rect: NSRect, screen: NSRect) -> Bounds<Pixels> {
    Bounds::new(
        point(
            px((rect.origin.x - screen.origin.x) as f32),
            px((screen.origin.y + screen.size.height - rect.origin.y - rect.size.height) as f32),
        ),
        size(px(rect.size.width as f32), px(rect.size.height as f32)),
    )
}

struct ClickHandlerIvars {
    callback: Box<dyn Fn()>,
}

define_class!(
    // SAFETY:
    // - The superclass `NSObject` has no subclassing requirements.
    // - `ClickHandler` does not implement `Drop`.
    #[unsafe(super(NSObject))]
    #[ivars = ClickHandlerIvars]
    struct ClickHandler;

    impl ClickHandler {
        #[unsafe(method(handleClick:))]
        fn handle_click(&self, _sender: Option<&AnyObject>) {
            (self.ivars().callback)();
        }
    }
);

impl ClickHandler {
    fn new(callback: Box<dyn Fn()>) -> Retained<Self> {
        let this = Self::alloc().set_ivars(ClickHandlerIvars { callback });
        // SAFETY: `NSObject`'s `init` has no special requirements.
        unsafe { msg_send![super(this), init] }
    }
}

/// Owns the `NSStatusItem` in the system menu bar (GPUI has no API for it) and
/// calls back into Rust when its button is clicked.
pub struct StatusItemController {
    status_item: Retained<NSStatusItem>,
    mtm: MainThreadMarker,
    current_label: String,
    current_dimmed: bool,
    // `NSControl.target` is weak, so the handler is kept alive here.
    _click_handler: Retained<ClickHandler>,
}

impl StatusItemController {
    pub fn new(mtm: MainThreadMarker, on_click: impl Fn() + 'static) -> Self {
        let status_bar = NSStatusBar::systemStatusBar();
        let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);
        let click_handler = ClickHandler::new(Box::new(on_click));

        if let Some(button) = status_item.button(mtm) {
            button.setImage(Some(&make_pill_image("00:00", false)));
            button.setImagePosition(NSCellImagePosition::ImageOnly);
            // SAFETY: `click_handler` is a live `NSObject` we retain for as
            // long as `self`, and `handleClick:` is the selector defined on
            // it above, matching the single-argument action signature.
            unsafe {
                button.setTarget(Some(&click_handler));
                button.setAction(Some(sel!(handleClick:)));
            }
        }

        Self {
            status_item,
            mtm,
            current_label: "00:00".to_owned(),
            current_dimmed: false,
            _click_handler: click_handler,
        }
    }

    /// `dimmed` draws the pill fainter (paused timer).
    pub fn set_label(&mut self, text: &str, dimmed: bool) {
        if self.current_label == text && self.current_dimmed == dimmed {
            return;
        }
        self.current_label.clear();
        self.current_label.push_str(text);
        self.current_dimmed = dimmed;
        self.redraw();
    }

    fn redraw(&self) {
        if let Some(button) = self.status_item.button(self.mtm) {
            button.setImage(Some(&make_pill_image(
                &self.current_label,
                self.current_dimmed,
            )));
        }
    }

    /// Refreshes the pill after a system appearance change; being a template
    /// image it is re-tinted by the system on its own.
    pub fn redraw_for_appearance(&mut self) {
        self.redraw();
    }

    pub fn screen_anchor(&self) -> Option<StatusItemAnchor> {
        let button = self.status_item.button(self.mtm)?;
        let window = button.window()?;
        let screen = window.screen()?;
        let frame = screen.frame();
        let button_rect =
            window.convertRectToScreen(button.convertRect_toView(button.bounds(), None));
        let screen_number = screen
            .deviceDescription()
            .objectForKey(&NSString::from_str("NSScreenNumber"))?;
        let display_id = screen_number.downcast_ref::<NSNumber>()?.unsignedIntValue();
        Some(StatusItemAnchor {
            bounds: screen_bounds(button_rect, frame),
            visible_bounds: screen_bounds(screen.visibleFrame(), frame),
            display_id: DisplayId::new(u64::from(display_id)),
        })
    }

    /// Removes the status item; consumes the controller so the item and its click
    /// closure are released together.
    pub fn remove(self) {
        NSStatusBar::systemStatusBar().removeStatusItem(&self.status_item);
    }
}

/// Text attributes shared by measuring and drawing. Digits are monospaced so the
/// pill's width does not jitter as they change.
fn text_attributes(dimmed: bool) -> Retained<NSDictionary<NSAttributedStringKey, AnyObject>> {
    // SAFETY: reading a well-known, always-initialized AppKit constant.
    let weight = unsafe { NSFontWeightMedium };
    let font = NSFont::monospacedDigitSystemFontOfSize_weight(13.0, weight);
    let color = pill_ink(dimmed);
    let font_ref: &AnyObject = &font;
    let color_ref: &AnyObject = &color;

    // SAFETY: reading well-known, always-initialized AppKit constants.
    let (font_key, color_key) = unsafe { (NSFontAttributeName, NSForegroundColorAttributeName) };

    NSDictionary::from_slices(&[font_key, color_key], &[font_ref, color_ref])
}

fn make_pill_image(text: &str, dimmed: bool) -> Retained<NSImage> {
    let attributes = text_attributes(dimmed);
    let ns_text = NSString::from_str(text);

    // SAFETY: `attributes`' generic parameter (`NSFont`/`NSColor` values) is
    // the correct type for these text-drawing attribute keys.
    let text_size = unsafe { ns_text.sizeWithAttributes(Some(&attributes)) };
    let geometry = pill_geometry(text_size.width, text_size.height);

    let image_size = NSSize::new(geometry.width, geometry.height);
    // The handler runs whenever AppKit needs pixels, so the drawing stays sharp
    // on any display (`lockFocus` rasterised once at 1x).
    let drawing = RcBlock::new(move |_: NSRect| -> Bool {
        if let Some(ctx) = NSGraphicsContext::currentContext() {
            ctx.setShouldAntialias(true);
            ctx.setImageInterpolation(NSImageInterpolation::High);
        }

        let stroke_rect = NSRect::new(
            NSPoint::new(0.5, 0.5),
            NSSize::new(geometry.width - 1.0, geometry.height - 1.0),
        );
        let path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
            stroke_rect,
            geometry.corner_radius,
            geometry.corner_radius,
        );

        NSColor::clearColor().setFill();
        path.fill();

        pill_ink(dimmed).setStroke();
        path.setLineWidth(1.0);
        path.stroke();

        let text_rect = NSRect::new(
            NSPoint::new(
                (geometry.width - text_size.width) / 2.0,
                (geometry.height - text_size.height) / 2.0,
            ),
            text_size,
        );
        // SAFETY: same attributes used to measure `text_size` above.
        unsafe { ns_text.drawInRect_withAttributes(text_rect, Some(&attributes)) };
        Bool::YES
    });
    let image = NSImage::imageWithSize_flipped_drawingHandler(image_size, false, &drawing);
    // A template image is tinted by the system for the menu bar it sits on. The
    // menu bar's appearance is not the app's: a dynamic colour resolved against a
    // light app appearance drew a black pill on a dark bar.
    image.setTemplate(true);
    image
}

/// Only the alpha matters: a template image is a mask the system tints.
fn pill_ink(dimmed: bool) -> Retained<NSColor> {
    let ink = NSColor::blackColor();
    if dimmed {
        ink.colorWithAlphaComponent(0.5)
    } else {
        ink
    }
}

#[cfg(test)]
mod tests {
    use gpui_kit::{point, px, size, Bounds};

    use super::*;

    #[test]
    fn appkit_anchor_is_flipped_to_top_left_screen_coordinates() {
        let screen = NSRect::new(NSPoint::new(0., 0.), NSSize::new(1440., 900.));
        let button = NSRect::new(NSPoint::new(800., 876.), NSSize::new(60., 24.));

        assert_eq!(
            screen_bounds(button, screen),
            Bounds::new(point(px(800.), px(0.)), size(px(60.), px(24.)))
        );
    }

    #[test]
    fn the_pill_is_a_template_image_so_the_system_tints_it_for_the_menu_bar() {
        assert!(make_pill_image("12:34", false).isTemplate());
        assert!(make_pill_image("12:34", true).isTemplate());
    }

    fn left_edge_alpha(dimmed: bool) -> f64 {
        use objc2_app_kit::NSBitmapImageRep;
        let image = make_pill_image("12:34", dimmed);
        let tiff = image
            .TIFFRepresentation()
            .expect("the pill renders to TIFF");
        let rep = NSBitmapImageRep::imageRepWithData(&tiff).expect("TIFF data decodes");
        let middle = rep.pixelsHigh() / 2;
        rep.colorAtX_y(0, middle)
            .expect("pixel is inside the bitmap")
            .alphaComponent()
    }

    #[test]
    fn the_rendered_pill_has_an_opaque_outline_that_dims_when_paused() {
        let (normal, dimmed) = (left_edge_alpha(false), left_edge_alpha(true));
        assert!(normal > 0.9, "outline alpha {normal}");
        assert!(dimmed > 0.3 && dimmed < normal, "dimmed alpha {dimmed}");
    }

    #[test]
    fn a_dimmed_pill_is_drawn_fainter_than_a_normal_one() {
        assert!(pill_ink(true).alphaComponent() < pill_ink(false).alphaComponent());
    }

    #[test]
    fn anchor_on_offset_display_uses_that_displays_origin() {
        let screen = NSRect::new(NSPoint::new(-1920., 900.), NSSize::new(1920., 1080.));
        let button = NSRect::new(NSPoint::new(-200., 1956.), NSSize::new(60., 24.));

        assert_eq!(
            screen_bounds(button, screen),
            Bounds::new(point(px(1720.), px(0.)), size(px(60.), px(24.)))
        );
    }

    #[test]
    fn pill_text_width_is_stable_across_all_digit_combinations() {
        let attributes = text_attributes(false);
        let samples = ["00:00", "11:11", "88:88", "01:23", "59:59", "10:07"];

        let widths: Vec<f64> = samples
            .iter()
            .map(|s| {
                // SAFETY: same attribute types as `make_pill_image`.
                unsafe { NSString::from_str(s).sizeWithAttributes(Some(&attributes)) }.width
            })
            .collect();

        let first = widths[0];
        for (sample, width) in samples.iter().zip(&widths) {
            assert!(
                (width - first).abs() < 0.01,
                "width for {sample:?} was {width}, expected {first} (all \"MM:SS\" strings \
                 must measure identically with monospaced digits) — widths: {widths:?}"
            );
        }
    }
}
