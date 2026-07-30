use std::{collections::HashSet, hash::Hash, rc::Rc};

use iced::{
    Element,
    widget::{Id, button, column, text},
};
use iced_aw::DropDown;

use crate::presentation::{application::Command, input_handling::InputMessage};

// could I turn the structure  {id, body} into a trait, to make things smoother here?
pub struct MultipickInput<T, Message>
where
    T: Clone + PartialEq + Eq + Hash,
    Message: Clone,
{
    id: Id,
    choices: Vec<InputRow<T>>,
    message: Rc<dyn Fn(Id, InputMessage) -> Message>,
    expanded: bool,
}

struct InputRow<T> {
    id: Id,
    pub contents: T,
    pub selected: bool,
}

impl<T, Message> MultipickInput<T, Message>
where
    T: Clone + PartialEq + Eq + Hash,
    Message: Clone,
{
    pub fn new<F: Fn(Id, InputMessage) -> Message + 'static>(
        choices: Vec<T>,
        already_selected: &HashSet<T>,
        message: F,
    ) -> Self {
        let choices = choices.into_iter().fold(Vec::new(), |mut acc, contents| {
            let selected = already_selected.contains(&contents);
            acc.push(InputRow::new(contents, selected));
            acc
        });
        let message = Rc::new(message);
        Self {
            id: Id::unique(),
            choices,
            message,
            expanded: false,
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn output(&self) -> HashSet<T> {
        self.choices.iter().fold(HashSet::new(), |mut acc, row| {
            if let Some(content) = row.output() {
                acc.insert(content);
            }
            acc
        })
    }

    pub fn view(&self) -> Element<'_, Message> {
        let underlay = button(text!("Expand me!"));
        let overlay = column![];
        DropDown::new(underlay, overlay, self.expanded).into()
    }
    pub fn update(&mut self, message: Message) -> Option<Command> {
        todo!()
    }
}

impl<T> InputRow<T>
where
    T: Clone + PartialEq,
{
    fn new(contents: T, selected: bool) -> Self {
        Self {
            id: Id::unique(),
            contents,
            selected,
        }
    }
    fn output(&self) -> Option<T> {
        if self.selected {
            Some(self.contents.clone())
        } else {
            None
        }
    }
}
