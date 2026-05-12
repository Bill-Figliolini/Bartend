use std::{fmt::Display, rc::Rc};

use iced::widget::Id;

use crate::presentation::{Viewable, application::Message, widget::input::Input};

use super::Error;

pub struct PickInput<T>
where
    T: Clone + Display,
{
    id: Id,
    input: T,
    message: Rc<dyn Fn(Id, T) -> Message>,
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
    T: Clone + Display + Default,
{
    fn new<F: Fn(Id, T) -> Message + 'static>(msg: F) -> Self {
        Self {
            id: Id::unique(),
            input: T::default(),
            message: Rc::new(msg),
        }
    }
    fn update(&mut self, input: T) {
        self.input = input;
    }
    fn get_output(&self) -> Result<T, Error> {
        Ok(self.input.clone())
    }
}
