use iced::{
    Element,
    widget::{column, row},
};

use crate::presentation::{application, widget::text_style::title};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug)]
pub struct Recipe {}

impl Recipe {
    pub fn new() -> Self {
        Self {}
    }
    pub fn view(&self) -> Element<'_, application::Message> {
        let title = title("Recipes");
        let body = "Heres where your recipes go!";
        column![title, body].into()
    }
}
