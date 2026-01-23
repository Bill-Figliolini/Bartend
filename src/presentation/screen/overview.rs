use iced::{Element, widget::column};

use crate::{application, presentation::widget::text_style::title};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug)]
pub struct Overview {}

impl Overview {
    pub fn new() -> Self {
        Self {}
    }
    pub fn view(&self) -> Element<'_, application::Message> {
        let title = title("Overview");
        let body = column!["Welcome to Bartending!"];

        column![title, body].into()
    }
}
