pub mod optional;
pub mod required;

use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use iced::widget::{Id, pick_list};

use crate::presentation::{Viewable, widget::input::Input};

struct PickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    id: Id,
    input: Option<T>,
    options: Vec<T>,
    message: Rc<dyn Fn(Id, T) -> Message>,
}

impl<T, Message> Viewable<Message> for PickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn view(&self) -> iced::Element<'_, Message> {
        let message = self.message.clone();
        pick_list(self.options.clone(), self.input.clone(), move |input: T| {
            message(self.id.clone(), input)
        })
        .into()
    }
}

impl<T, Message> Input<T, Message> for PickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn update(&mut self, _input: T) {
        unreachable!()
    }
    fn id(&self) -> &Id {
        &self.id
    }
}

impl<T, Message> Debug for PickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
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
