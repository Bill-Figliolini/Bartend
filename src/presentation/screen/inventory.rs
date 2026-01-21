use iced::{Element, widget::row};

use crate::application;

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug)]
pub struct Inventory {}

impl Inventory {
    pub fn new() -> Self {
        Self {}
    }
    pub fn view(&self) -> Element<'_, application::Message> {
        row!["Welcome to the Inventory!"].into()
    }
}
