use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use iced::widget::{Id, column, combo_box, row};

use crate::presentation::{
    Viewable,
    widget::input::{Input, InputOptionalContents, pick_input::PickInput},
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
}

impl<T, Message> Viewable<Message> for OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq + 'static,
    Message: Clone + 'static,
{
    fn view(&self) -> iced::Element<'_, Message> {
        let error_row = row![];
        column![self.inner.view(), error_row].into()
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
impl<T, Message> InputOptionalContents<T> for OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn get_output(&mut self) -> Result<Option<T>, ()> {
        Ok(self.inner.selection.clone())
    }
}
