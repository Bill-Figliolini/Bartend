use iced::{Element, widget::column};

use crate::presentation::{application, widget::text_style::title};

#[derive(Debug)]
pub struct Settings {}

#[derive(Debug, Clone)]
pub enum Message {}

impl Settings {
    pub(super) const fn new() -> Self {
        Self {}
    }
    pub(super) fn view(&self) -> Element<'_, application::Message> {
        let title = title("Settings");

        column![title].into()
    }
}
