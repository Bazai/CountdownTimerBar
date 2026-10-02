//! The single popover: header, Focus/Rest columns, collapsible settings. Window-level
//! mouse listeners keep a knob drag alive wherever the pointer goes; presets are
//! never removed by dragging (docs/adr/0001).

mod circles;
mod edit;
mod header;
mod labels;
mod model;
mod pointer;
mod settings;
mod widgets;

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::component::{
    button::ButtonVariants as _,
    input::{InputEvent, InputState},
    Icon,
};
use gpui_kit::prelude::*;
use gpui_kit::{
    div, px, Bounds, Context, Entity, FocusHandle, IntoElement, Pixels, Point, Render,
    Subscription, Task, Window,
};

use crate::domain::{
    preset::PresetKind,
    preset_list::{format_list, Unit},
};
use crate::gui::gesture::{Gesture, Phase, Source};
use crate::gui::inline_edit::EditBuffer;
use crate::gui::native_popover::NativePopoverFrame;
use crate::gui::state::AppState;
use crate::theme::{metrics, ThemeExt as _};

use labels::column_index;
use widgets::icon_button;

gpui_kit::actions!(
    countdown_timer_bar,
    [CancelEdit, CancelAbout, TabNext, TabPrev]
);

/// Focus target applied after the next render, once the element exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FocusTarget {
    Edit,
    Circle(PresetKind, usize),
    Add(PresetKind),
    AboutLink,
}

/// Key context of the inline edit field; `escape` there cancels the edit
/// instead of closing the popover.
pub const EDIT_KEY_CONTEXT: &str = "TimerEdit";

/// Key context set on the popover while the About card is open; `escape`
/// there closes the card instead of the whole popover.
pub const ABOUT_KEY_CONTEXT: &str = "AboutOpen";

const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(250);
/// Row = 44 px circle + 5 px above and below, so the hover area also covers
/// the × badge that sticks out of the circle.
const ROW_HEIGHT: f32 = 54.;
const ROW_PAD: f32 = 5.;

struct Press {
    kind: PresetKind,
    /// `None`: the new circle after "+".
    index: Option<usize>,
    source: Source,
    /// Value the knob started from (the typed number for the edit sources;
    /// 0 for an empty new field).
    base_value: i64,
    unit: Unit,
    gesture: Gesture,
    pos: Point<Pixels>,
}

impl Press {
    fn preview(&self) -> u32 {
        match self.gesture.steps() {
            0 => u32::try_from(self.base_value).unwrap_or(0),
            steps => self.unit.clamp(self.base_value + i64::from(steps)),
        }
    }
}

struct Edit {
    kind: PresetKind,
    /// `None`: a new preset being typed at the end of the column.
    index: Option<usize>,
    unit: Unit,
    buffer: EditBuffer,
    drag: Option<Press>,
}

/// What the pointer or keyboard is doing to a circle right now. One value, so
/// a press on a circle and an open edit field cannot coexist by accident.
enum Interaction {
    Idle,
    Press(Press),
    Edit(Edit),
}

impl Interaction {
    fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    fn edit(&self) -> Option<&Edit> {
        match self {
            Self::Edit(edit) => Some(edit),
            _ => None,
        }
    }

    fn edit_mut(&mut self) -> Option<&mut Edit> {
        match self {
            Self::Edit(edit) => Some(edit),
            _ => None,
        }
    }

    fn press(&self) -> Option<&Press> {
        match self {
            Self::Press(press) => Some(press),
            Self::Edit(edit) => edit.drag.as_ref(),
            Self::Idle => None,
        }
    }

    fn press_mut(&mut self) -> Option<&mut Press> {
        match self {
            Self::Press(press) => Some(press),
            Self::Edit(edit) => edit.drag.as_mut(),
            Self::Idle => None,
        }
    }

    fn take_press(&mut self) -> Option<Press> {
        match std::mem::replace(self, Self::Idle) {
            Self::Press(press) => Some(press),
            Self::Edit(mut edit) => {
                let drag = edit.drag.take();
                *self = Self::Edit(edit);
                drag
            }
            Self::Idle => None,
        }
    }

    fn take_edit(&mut self) -> Option<Edit> {
        match std::mem::replace(self, Self::Idle) {
            Self::Edit(edit) => Some(edit),
            other => {
                *self = other;
                None
            }
        }
    }
}

pub struct Popover {
    state: Entity<AppState>,
    visible_bounds: Bounds<Pixels>,
    settings_open: bool,
    hovered: Option<(PresetKind, usize)>,
    interaction: Interaction,
    pending_click: Option<Task<()>>,
    edit_focus: FocusHandle,
    /// Moves focus after the next render; the `i8` then steps the tab order (+1 next,
    /// -1 previous, 0 stay).
    pending_focus: Option<(FocusTarget, i8)>,
    /// Catch-all focus target, so Tab and the key bindings keep working when the
    /// focused element disappears.
    root_focus: FocusHandle,
    circle_focus: HashMap<(usize, usize), FocusHandle>,
    add_focus: [FocusHandle; 2],
    /// Filled while painting.
    edit_row_bounds: Rc<Cell<Bounds<Pixels>>>,
    about_open: bool,
    about_focus: FocusHandle,
    about_link_bounds: Rc<Cell<Bounds<Pixels>>>,
    about_card_bounds: Rc<Cell<Bounds<Pixels>>>,
    list_inputs: [Entity<InputState>; 2],
    list_errors: [Option<String>; 2],
    /// Text of the list last applied from the circles or the input. Used to
    /// tell a circle edit (rewrite the input) from the input's own change.
    applied_text: [String; 2],
    _subscriptions: Vec<Subscription>,
}

impl Popover {
    pub fn new(
        state: Entity<AppState>,
        visible_bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let kinds = [PresetKind::Focus, PresetKind::Rest];
        let applied_text = kinds.map(|kind| format_list(state.read(cx).specs(kind)));
        let placeholders = ["e.g. 15, 30, 45s", "e.g. 1, 5, 90s"];
        let list_inputs: [Entity<InputState>; 2] = std::array::from_fn(|i| {
            let text = applied_text[i].clone();
            cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(text)
                    .placeholder(placeholders[i])
            })
        });
        let mut subscriptions = Vec::new();
        subscriptions.push(cx.observe_in(&state, window, |this, _, window, cx| {
            this.sync_inputs_from_state(window, cx);
            cx.notify();
        }));
        for (i, kind) in kinds.into_iter().enumerate() {
            subscriptions.push(cx.subscribe(
                &list_inputs[i],
                move |this, input, event: &InputEvent, cx| {
                    if matches!(event, InputEvent::Change) {
                        let text = input.read(cx).value().to_string();
                        this.list_input_changed(kind, &text, cx);
                    }
                },
            ));
        }
        Self {
            state,
            visible_bounds,
            settings_open: false,
            hovered: None,
            interaction: Interaction::Idle,
            pending_click: None,
            edit_focus: cx.focus_handle(),
            pending_focus: None,
            root_focus: cx.focus_handle(),
            circle_focus: HashMap::new(),
            add_focus: [
                cx.focus_handle().tab_stop(true),
                cx.focus_handle().tab_stop(true),
            ],
            edit_row_bounds: Rc::default(),
            about_open: false,
            about_focus: cx.focus_handle().tab_stop(true),
            about_link_bounds: Rc::default(),
            about_card_bounds: Rc::default(),
            list_inputs,
            list_errors: [None, None],
            applied_text,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for Popover {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = *cx.palette();
        let settings_open = self.settings_open;
        let this = cx.entity();
        let header = self.header(cx);
        let focus = self.column(PresetKind::Focus, cx);
        let rest = self.column(PresetKind::Rest, cx);
        if let Some((target, advance)) = self.pending_focus.take() {
            let handle = match target {
                FocusTarget::Edit => self
                    .interaction
                    .edit()
                    .is_some()
                    .then(|| self.edit_focus.clone()),
                FocusTarget::Circle(kind, index) => {
                    self.circle_focus.get(&(column_index(kind), index)).cloned()
                }
                FocusTarget::Add(kind) => Some(self.add_focus[column_index(kind)].clone()),
                FocusTarget::AboutLink => Some(self.about_focus.clone()),
            };
            if let Some(handle) = handle {
                window.defer(cx, move |window, cx| {
                    window.focus(&handle, cx);
                    match advance {
                        1 => window.focus_next(cx),
                        -1 => window.focus_prev(cx),
                        _ => {}
                    }
                });
            }
        }
        if self.pending_focus.is_none() && window.focused(cx).is_none() {
            let root = self.root_focus.clone();
            window.defer(cx, move |window, cx| window.focus(&root, cx));
        }
        let listeners = Self::mouse_listeners(cx.entity());
        let knob_drag = self
            .interaction
            .press()
            .is_some_and(|press| press.gesture.phase() == Phase::Knob);

        let gear = icon_button("settings", "Settings")
            .icon(Icon::new(gpui_kit::assets::IconName::Settings))
            .ghost()
            .text_color(if settings_open { p.text } else { p.text_muted }.hsla())
            .when(settings_open, |button| button.bg(p.control_hover.hsla()))
            .on_click(move |_, _, cx| {
                this.update(cx, |popover, cx| {
                    popover.settings_open = !popover.settings_open;
                    if !popover.settings_open {
                        popover.about_open = false;
                    }
                    cx.notify();
                });
            });

        NativePopoverFrame::new(
            self.visible_bounds,
            div()
                .relative()
                .w(px(metrics::POPOVER_WIDTH))
                .flex()
                .flex_col()
                .gap(px(14.))
                .pt(px(18.))
                .px(px(16.))
                .pb(px(10.))
                .text_color(p.text.hsla())
                .track_focus(&self.root_focus)
                .when(knob_drag, gpui_kit::Styled::cursor_ns_resize)
                .when(self.about_open, |this| {
                    this.key_context(ABOUT_KEY_CONTEXT).on_action(cx.listener(
                        |this, _: &CancelAbout, _, cx| this.close_about_from_keyboard(cx),
                    ))
                })
                .child(listeners)
                .child(header)
                .child(div().flex().child(focus).child(rest))
                .when(settings_open, |this| this.child(self.settings(cx)))
                .child(
                    div()
                        .flex()
                        .justify_center()
                        .pt(px(8.))
                        .border_t_1()
                        .border_color(p.divider.hsla())
                        .child(gear),
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use gpui_kit::{point, px};

    use super::*;

    fn press() -> Press {
        Press {
            kind: PresetKind::Focus,
            index: Some(0),
            source: Source::Circle,
            base_value: 5,
            unit: Unit::Minutes,
            gesture: Gesture::new(0.),
            pos: point(px(0.), px(0.)),
        }
    }

    fn edit_with_drag() -> Edit {
        Edit {
            kind: PresetKind::Focus,
            index: Some(0),
            unit: Unit::Minutes,
            buffer: EditBuffer::with_value(5),
            drag: Some(press()),
        }
    }

    #[test]
    fn a_plain_press_ends_idle() {
        let mut interaction = Interaction::Press(press());
        assert!(interaction.take_press().is_some());
        assert!(interaction.is_idle());
    }

    #[test]
    fn ending_a_drag_on_the_edit_field_keeps_the_field_open() {
        let mut interaction = Interaction::Edit(edit_with_drag());
        assert!(interaction.take_press().is_some());
        assert!(interaction.edit().is_some());
    }

    #[test]
    fn the_drag_on_an_edit_field_is_the_press_in_progress() {
        let interaction = Interaction::Edit(edit_with_drag());
        assert!(interaction.press().is_some());
    }

    #[test]
    fn taking_the_edit_of_a_press_leaves_the_press_alone() {
        let mut interaction = Interaction::Press(press());
        assert!(interaction.take_edit().is_none());
        assert!(interaction.press().is_some());
    }

    #[test]
    fn taking_the_edit_drops_its_drag() {
        let mut interaction = Interaction::Edit(edit_with_drag());
        assert!(interaction.take_edit().is_some());
        assert!(interaction.is_idle());
    }
    use gpui_kit::assets::{Assets, IconName};
    use gpui_kit::AssetSource as _;

    #[test]
    fn icons_used_by_the_popover_are_embedded_in_the_asset_bundle() {
        for icon in [IconName::Play, IconName::Pause, IconName::Settings] {
            let path = icon.path();
            assert!(
                Assets
                    .load(&path)
                    .expect("asset lookup must succeed")
                    .is_some(),
                "{path} must be embedded in gpui_kit::assets::Assets"
            );
        }
    }
}
