use std::{fmt::Display, rc::Rc};

use iced::widget::Id;

use crate::presentation::{Viewable, application::Message, widget::input::Input};

use super::Error;

pub struct PickInput<T>
where
    T: Clone + Display,
{
    id: Id,
    input: Option<T>,
    options: Vec<T>,
    message: Rc<dyn Fn(Id, T) -> Message>,
}

impl<T> PickInput<T>
where
    T: Clone + Display,
{
    pub fn new<F: Fn(Id, T) -> Message + 'static>(
        msg: F,
        initial_value: Option<T>,
        options: Vec<T>,
    ) -> Self {
        Self {
            id: Id::unique(),
            input: initial_value,
            options,
            message: Rc::new(msg),
        }
    }
}

impl<T> Viewable<Message> for PickInput<T>
where
    T: Clone + Display,
{
    fn view(&self) -> iced::Element<'_, Message> {
        todo!()
    }
}

impl<T> Input<T, T, Message> for PickInput<T>
where
    T: Clone + Display,
{
    fn update(&mut self, input: T) {
        self.input = Some(input);
    }
    fn get_output(&self) -> Result<T, Error> {
        match self.input {
            Some(ref value) => Ok(value.clone()),
            None => Err(Error::MustChooseValue),
        }
    }
    fn id(&self) -> &Id {
        &self.id
    }
}
