use crate::presentation::application;

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {}

#[derive(Debug)]
pub(in crate::presentation) struct Serving {}

impl Serving {
    pub fn new() -> Self {
        Self {}
    }
    pub fn view(&self) -> iced::Element<'_, application::Message> {
        todo!()
    }
    pub fn update(&mut self, message: Message) -> Option<application::Command> {
        todo!()
    }
}
