use std::{
    collections::HashSet,
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
    errors: HashSet<Error>,
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
            errors: HashSet::new(),
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
                .errors
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
    fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}
impl<T, Message> InputContents<T> for RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn get_output(&mut self) -> Result<T, ()> {
        self.errors.clear();
        match self.inner.selection {
            Some(ref value) => Ok(value.clone()),
            None => {
                self.errors.insert(Error::MustChooseValue);
                Err(())
            }
        }
    }
}
