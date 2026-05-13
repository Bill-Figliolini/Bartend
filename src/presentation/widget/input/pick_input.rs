use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use iced::widget::Id;

use crate::presentation::{Viewable, application::Message, widget::input::Input};

use super::Error;

pub struct PickInput<T>
where
    T: Debug + Clone + Display,
{
    id: Id,
    input: Option<T>,
    options: Vec<T>,
    message: Rc<dyn Fn(Id, Option<T>) -> Message>,
}

impl<T> PickInput<T>
where
    T: Debug + Clone + Display,
{
    pub fn new<F: Fn(Id, Option<T>) -> Message + 'static>(
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
    T: Debug + Clone + Display,
{
    fn view(&self) -> iced::Element<'_, Message> {
        todo!()
    }
}

impl<T> Input<T, T, Message> for PickInput<T>
where
    T: Debug + Clone + Display,
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
    fn get_optional_output(&self) -> Result<Option<T>, Error> {
        return Ok(self.input.clone());
    }
    fn id(&self) -> &Id {
        &self.id
    }
}

impl<T> Debug for PickInput<T>
where
    T: Debug + Clone + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PickInput")
            .field("id", &self.id)
            .field("input", &self.input)
            .field("options", &self.options)
            .finish()
    }
}
