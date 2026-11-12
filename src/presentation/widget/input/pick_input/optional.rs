use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use iced::widget::{Id, combo_box};

use crate::presentation::{
    Viewable,
    widget::input::{Input, pick_input::PickInput},
};

#[derive(Debug)]
pub struct OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    inner: PickInput<T, Message>,
}

impl<T, Message> OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    pub fn new<F: Fn(Id, T) -> Message + 'static>(
        msg: F,
        options: Vec<T>,
        initial_value: Option<T>,
        mut placeholder: String,
    ) -> Self {
        placeholder.push_str(" (Optional)");
        Self {
            inner: PickInput {
                id: Id::unique(),
                message: Rc::new(msg),
                options: combo_box::State::new(options),
                selection: initial_value,
                placeholder,
            },
        }
    }

    /// The current selection. Choosing nothing is valid, so this cannot fail.
    pub fn get_output(&self) -> Option<T> {
        self.inner.selection.clone()
    }
}

impl<T, Message> Viewable<Message> for OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq + 'static,
    Message: Clone + 'static,
{
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn view(&self) -> iced::Element<'_, Message> {
        self.inner.view()
    }
}
impl<T, Message> Input<T, Message> for OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq + 'static,
    Message: Clone + 'static,
{
    fn update(&mut self, input: T) {
        let next = Some(input);
        if self.inner.selection == next {
            self.inner.selection = None;
        } else {
            self.inner.selection = next;
        }
    }

    fn id(&self) -> &Id {
        self.inner.id()
    }
    fn clear(&mut self) {
        self.inner.selection = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> OptionalPickInput<String, ()> {
        OptionalPickInput::new(
            |_, _| (),
            vec!["Gin".to_string()],
            None,
            "Spirit".to_string(),
        )
    }

    #[test]
    fn nothing_selected_is_valid() {
        assert_eq!(input().get_output(), None);
    }

    #[test]
    fn choosing_the_same_value_again_deselects_it() {
        let mut input = input();
        input.update("Gin".to_string());
        assert_eq!(input.get_output(), Some("Gin".to_string()));
        input.update("Gin".to_string());
        assert_eq!(input.get_output(), None);
    }

    #[test]
    fn clear_removes_selection() {
        let mut input = input();
        input.update("Gin".to_string());
        input.clear();
        assert_eq!(input.get_output(), None);
    }
}
