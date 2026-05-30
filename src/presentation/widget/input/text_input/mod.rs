pub(super) mod number_input;
pub(super) mod string_input;

use std::{fmt::Debug, rc::Rc};

use iced::widget::{Id, text_input};

use crate::presentation::{Viewable, widget::input::Input};
struct TextInput<Message>
where
    Message: Clone,
{
    id: Id,
    text: String,
    placeholder: String,
    message: Rc<dyn Fn(Id, String) -> Message>,
}

impl<Message> Viewable<Message> for TextInput<Message>
where
    Message: Clone,
{
    fn view(&self) -> iced::Element<'_, Message> {
        let message = self.message.clone();
        text_input(&self.placeholder, &self.text)
            .id(self.id.clone())
            .on_input(move |str: String| message(self.id().clone(), str))
            .into()
    }
}

impl<Message> Input<String, Message> for TextInput<Message>
where
    Message: Clone,
{
    fn update(&mut self, input: String) {
        self.text = input;
    }

    fn clear(&mut self) {
        self.text.clear();
    }
    fn id(&self) -> &Id {
        &self.id
    }
}

impl<Message> Debug for TextInput<Message>
where
    Message: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NameInput")
            .field("id", &self.id)
            .field("placeholder", &self.placeholder)
            .field("text", &self.text)
            .finish()
    }
}
