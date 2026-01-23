use iced::{
    Element,
    widget::{column, container, row, text},
};

use crate::{application, presentation::widget::text_style::title};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug)]
pub struct Inventory {}

impl Inventory {
    pub fn new() -> Self {
        Self {}
    }
    pub fn view(&self) -> Element<'_, application::Message> {
        let title = title("Inventory");
        let body = text("Welcome to the Inventory!");
        column![title, body].into()
    }
}
