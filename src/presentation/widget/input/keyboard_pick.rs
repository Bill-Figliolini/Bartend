//! [`KeyboardPick`]: a combo box that takes part in Tab navigation.
//!
//! iced's `ComboBox` has two gaps for keyboard use. It does not report its
//! text box to focus operations, so `focus_next`/`focus_previous` skip it;
//! and while open it captures Tab to cycle its options, so Tab cannot leave
//! it. The wrapper fixes both and passes everything else straight through.
//!
//! Picking an option with Enter makes the combo box unfocus itself, which
//! would send the next Tab back to the first field. So the wrapper keeps
//! counting as focused (for focus operations only) until focus moves on or
//! the mouse is clicked.

use std::fmt::{self, Debug};

use iced::{
    Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, text,
        widget::{Operation, Tree, operation::Focusable, tree},
    },
    keyboard::{self, Key, key::Named},
    widget::text_input,
};

/// The state of the text box inside an iced `ComboBox`.
type InputState = text_input::State<<Renderer as text::Renderer>::Paragraph>;

/// Wraps a `ComboBox` element; see the [module docs](self).
pub(super) struct KeyboardPick<'a, M> {
    combo: Element<'a, M>,
}

impl<'a, M> KeyboardPick<'a, M> {
    /// `combo` must be an iced `ComboBox`. Anything else is shown as is,
    /// but is not a tab stop.
    pub(super) fn new(combo: impl Into<Element<'a, M>>) -> Self {
        Self {
            combo: combo.into(),
        }
    }
}

impl<M> Debug for KeyboardPick<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyboardPick").finish_non_exhaustive()
    }
}

/// The wrapper's own state, kept in the widget tree between frames.
#[derive(Debug, Default)]
struct State {
    /// An option was just picked with Enter: count as focused so Tab
    /// carries on from here.
    resume: bool,
}

/// The combo box's text box state, found in the tree as
/// `wrapper -> combo box -> text box`. `None` if `combo` was not a
/// `ComboBox`.
fn input_state(children: &mut [Tree]) -> Option<&mut InputState> {
    let input = children.first_mut()?.children.first_mut()?;
    (input.tag == tree::Tag::of::<InputState>()).then(|| input.state.downcast_mut::<InputState>())
}

fn input_focused(tree: &mut Tree) -> bool {
    input_state(&mut tree.children).is_some_and(|input| input.is_focused())
}

fn is_key(event: &Event, named: Named) -> bool {
    matches!(
        event,
        Event::Keyboard(keyboard::Event::KeyPressed { key: Key::Named(key), .. }) if *key == named
    )
}

/// What focus operations see: the text box, plus the `resume` flag.
struct PickFocus<'a, F> {
    input: &'a mut F,
    resume: &'a mut bool,
}

impl<F: Focusable> Focusable for PickFocus<'_, F> {
    fn is_focused(&self) -> bool {
        *self.resume || self.input.is_focused()
    }

    fn focus(&mut self) {
        *self.resume = false;
        self.input.focus();
    }

    fn unfocus(&mut self) {
        *self.resume = false;
        self.input.unfocus();
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<M> Widget<M, Theme, Renderer> for KeyboardPick<'_, M> {
    fn size(&self) -> Size<Length> {
        self.combo.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.combo.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.combo
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.combo.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.combo)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.combo));
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let Tree {
            state, children, ..
        } = tree;
        let resume = &mut state.downcast_mut::<State>().resume;
        if let Some(input) = input_state(children) {
            operation.focusable(None, layout.bounds(), &mut PickFocus { input, resume });
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        let was_focused = input_focused(tree);
        if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))) {
            tree.state.downcast_mut::<State>().resume = false;
        }
        // Left uncaptured, so the app's Tab handling moves focus on.
        if was_focused && is_key(event, Named::Tab) {
            return;
        }

        self.combo.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if was_focused && is_key(event, Named::Enter) && !input_focused(tree) {
            tree.state.downcast_mut::<State>().resume = true;
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.combo.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.combo.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: 'a> From<KeyboardPick<'a, M>> for Element<'a, M> {
    fn from(pick: KeyboardPick<'a, M>) -> Self {
        Element::new(pick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct FakeInput {
        focused: bool,
    }

    impl Focusable for FakeInput {
        fn is_focused(&self) -> bool {
            self.focused
        }
        fn focus(&mut self) {
            self.focused = true;
        }
        fn unfocus(&mut self) {
            self.focused = false;
        }
    }

    /// Whether the picker counts as focused, and what `act` leaves of the
    /// text box's focus and the resume flag.
    fn after(
        focused: bool,
        resume: bool,
        act: impl FnOnce(&mut PickFocus<'_, FakeInput>),
    ) -> (bool, bool) {
        let mut input = FakeInput { focused };
        let mut resume = resume;
        act(&mut PickFocus {
            input: &mut input,
            resume: &mut resume,
        });
        (input.focused, resume)
    }

    fn reported(focused: bool, resume: bool) -> bool {
        let mut input = FakeInput { focused };
        let mut resume = resume;
        PickFocus {
            input: &mut input,
            resume: &mut resume,
        }
        .is_focused()
    }

    #[test]
    fn counts_as_focused_when_typing_or_resuming() {
        assert!(reported(true, false));
        assert!(reported(false, true), "a pending resume counts");
        assert!(!reported(false, false));
    }

    #[test]
    fn moving_focus_away_clears_the_resume() {
        assert_eq!(after(false, true, |pick| pick.unfocus()), (false, false));
        assert_eq!(after(true, false, |pick| pick.unfocus()), (false, false));
    }

    #[test]
    fn focusing_opens_the_text_box_and_clears_the_resume() {
        assert_eq!(after(false, true, |pick| pick.focus()), (true, false));
    }

    #[test]
    fn only_key_presses_of_that_key_match() {
        let press = |key| {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key),
                modified_key: Key::Named(key),
                physical_key: keyboard::key::Physical::Unidentified(
                    keyboard::key::NativeCode::Unidentified,
                ),
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::default(),
                text: None,
                repeat: false,
            })
        };
        assert!(is_key(&press(Named::Tab), Named::Tab));
        assert!(!is_key(&press(Named::Enter), Named::Tab));
        let click = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        assert!(!is_key(&click, Named::Tab));
    }
}
