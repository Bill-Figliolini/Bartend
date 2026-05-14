use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use iced::widget::Id;

use crate::presentation::{
    Viewable,
    widget::input::{Input, InputContents, InputOptionalContents},
};

use super::Error;

struct PickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    id: Id,
    input: Option<T>,
    options: Vec<T>,
    message: Rc<dyn Fn(Id, Option<T>) -> Message>,
}
#[derive(Debug)]
pub struct RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    inner: PickInput<T, Message>,
}

#[derive(Debug)]
pub struct OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    inner: PickInput<T, Message>,
}

impl<T, Message> PickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
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

impl<T, Message> Viewable<Message> for PickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    fn view(&self) -> iced::Element<'_, Message> {
        todo!()
    }
}

impl<T, Message> Input<T, T, Message> for PickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    fn update(&mut self, input: T) {
        self.input = Some(input);
    }
    fn id(&self) -> &Id {
        &self.id
    }
}

impl<T, Message> Debug for PickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PickInput")
            .field("id", &self.id)
            .field("input", &self.input)
            .field("options", &self.options)
            .finish()
    }
}

impl<T, Message> Viewable<Message> for RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    fn view(&self) -> iced::Element<'_, Message> {
        self.inner.view()
    }
}
impl<T, Message> Viewable<Message> for OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    fn view(&self) -> iced::Element<'_, Message> {
        self.inner.view()
    }
}
impl<T, Message> Input<T, T, Message> for RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    fn update(&mut self, input: T) {
        self.inner.update(input);
    }

    fn id(&self) -> &Id {
        self.inner.id()
    }
}
impl<T, Message> InputContents<T> for RequiredPickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    fn get_output(&self) -> Result<T, Error> {
        match self.inner.input {
            Some(ref value) => Ok(value.clone()),
            None => Err(Error::MustChooseValue),
        }
    }
}

impl<T, Message> Input<T, T, Message> for OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    fn update(&mut self, input: T) {
        self.inner.update(input);
    }

    fn id(&self) -> &Id {
        self.inner.id()
    }
}
impl<T, Message> InputOptionalContents<T> for OptionalPickInput<T, Message>
where
    T: Debug + Clone + Display,
    Message: Clone,
{
    fn get_output(&self) -> Result<Option<T>, Error> {
        Ok(self.inner.input.clone())
    }
}
