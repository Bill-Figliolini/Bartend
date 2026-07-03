use crate::presentation::{Updateable, Viewable, application};

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {}

#[derive(Debug)]
pub(in crate::presentation) struct Serving {}

impl Viewable<application::Message> for Serving {
    fn view(&self) -> iced::Element<'_, application::Message> {
        todo!()
    }
}
impl Updateable<Message> for Serving {
    fn update(&mut self, message: Message) -> Option<application::Command> {
        todo!()
    }
}
