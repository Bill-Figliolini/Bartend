use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use iced::widget::Id;

use crate::presentation::{
    Viewable,
    widget::input::{Error, Input, InputOptionalContents, pick_input::PickInput},
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
    ) -> Self {
        Self {
            inner: PickInput {
                id: Id::unique(),
                message: Rc::new(msg),
                input: initial_value,
                options,
            },
        }
    }
}

impl<T, Message> Viewable<Message> for OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn view(&self) -> iced::Element<'_, Message> {
        self.inner.view()
    }
}
impl<T, Message> Input<T, Message> for OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn update(&mut self, input: T) {
        let next = Some(input);
        if self.inner.input == next {
            self.inner.input = None;
        } else {
            self.inner.input = next
        }
    }

    fn id(&self) -> &Id {
        self.inner.id()
    }
}
impl<T, Message> InputOptionalContents<T> for OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn get_output(&self) -> Result<Option<T>, Error> {
        Ok(self.inner.input.clone())
    }
}
