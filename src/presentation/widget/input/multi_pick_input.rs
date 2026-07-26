use std::{collections::HashSet, rc::Rc};

use iced::{Element, widget::Id};

use crate::presentation::application::Command;

// could I turn the structure  {id, body} into a trait, to make things smoother here?
struct MultipickInput<T, Message>
where
    T: Clone,
    Message: Clone,
{
    id: Id,
    choices: Vec<T>,
    selected: HashSet<T>,
    message: Rc<dyn Fn(Id, T) -> Message>,
}

impl<T, Message> MultipickInput<T, Message>
where
    T: Clone,
    Message: Clone,
{
    pub fn new(
        choices: Vec<T>,
        already_selected: HashSet<T>,
        message: Rc<dyn Fn(Id, T) -> Message>,
    ) -> Self {
        Self {
            id: Id::unique(),
            choices,
            selected: already_selected,
            message,
        }
    }
    pub fn output(&mut self) -> HashSet<T> {
        self.selected.clone()
    }

    pub fn view(&self) -> Element<'_, Message> {
        todo!()
    }
    pub fn update(&mut self, message: Message) -> Option<Command> {
        todo!()
    }
}
