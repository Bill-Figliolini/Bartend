use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use iced::widget::Id;

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
impl<T, Message> Viewable<Message> for RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn view(&self) -> iced::Element<'_, Message> {
        self.inner.view()
    }
}

impl<T, Message> Input<T, Message> for RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn update(&mut self, input: T) {
        self.inner.input = Some(input);
    }

    fn id(&self) -> &Id {
        self.inner.id()
    }
}
impl<T, Message> InputContents<T> for RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn get_output(&self) -> Result<T, Error> {
        match self.inner.input {
            Some(ref value) => Ok(value.clone()),
            None => Err(Error::MustChooseValue),
        }
    }
}
