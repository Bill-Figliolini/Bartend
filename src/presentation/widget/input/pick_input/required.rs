use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use iced::widget::{Id, column, combo_box, row, text};

use crate::presentation::{
    Viewable,
    widget::input::{Error, Input, InputContents, pick_input::PickInput},
};

#[derive(Debug)]
pub struct RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    inner: PickInput<T, Message>,
    error: Option<Error>,
}
impl<T, Message> RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    pub fn new<F: Fn(Id, T) -> Message + 'static>(
        msg: F,
        options: Vec<T>,
        initial_value: Option<T>,
        placeholder: String,
    ) -> Self {
        Self {
            inner: PickInput {
                id: Id::unique(),
                message: Rc::new(msg),
                options: combo_box::State::new(options),
                selection: initial_value,
                placeholder,
            },
            error: None,
        }
    }
}
impl<T, Message> Viewable<Message> for RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq + 'static,
    Message: Clone + 'static,
{
    fn view(&self) -> iced::Element<'_, Message> {
        column![
            self.inner.view(),
            row(self
                .error
                .iter()
                .map(|error| text(error.to_string()).into()))
        ]
        .into()
    }
}

impl<T, Message> Input<T, Message> for RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq + 'static,
    Message: Clone + 'static,
{
    fn update(&mut self, input: T) {
        self.inner.selection = Some(input);
    }

    fn id(&self) -> &Id {
        self.inner.id()
    }
    fn clear(&mut self) {
        self.inner.selection = None;
        self.error = None;
    }
}
impl<T, Message> InputContents<T> for RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn get_output(&mut self) -> Result<T, Error> {
        let result = self.inner.selection.clone().ok_or(Error::MustChooseValue);
        self.error = result.as_ref().err().copied();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(initial: Option<&str>) -> RequiredPickInput<String, ()> {
        RequiredPickInput::new(
            |_, _| (),
            vec!["Gin".to_string(), "Rum".to_string()],
            initial.map(str::to_string),
            "Spirit".to_string(),
        )
    }

    #[test]
    fn nothing_selected_is_rejected_and_shown() {
        let mut input = input(None);
        assert_eq!(input.get_output(), Err(Error::MustChooseValue));
        assert_eq!(input.error, Some(Error::MustChooseValue));
    }

    #[test]
    fn returns_selection() {
        let mut input = input(None);
        input.update("Rum".to_string());
        assert_eq!(input.get_output(), Ok("Rum".to_string()));
        assert_eq!(input.error, None);
    }

    #[test]
    fn clear_removes_selection_and_error() {
        let mut input = input(None);
        _ = input.get_output();
        input.update("Gin".to_string());
        input.clear();
        assert_eq!(input.error, None);
        assert_eq!(input.get_output(), Err(Error::MustChooseValue));
    }
}
