use iced::{Element, widget::row};

use crate::presentation::application;

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug)]
pub struct Recipe {}

impl Recipe {
    pub fn new() -> Self {
        Self {}
    }
    pub fn view(&self) -> Element<'_, application::Message> {
        row!["Heres where your recipes go!"].into()
    }
}
