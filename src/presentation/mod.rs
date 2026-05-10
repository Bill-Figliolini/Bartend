use crate::{
    logic::config::Config,
    presentation::application::{Command, Message},
};
use iced::Element;

pub mod application;
mod constants;
mod screen;
mod widget;

trait Composition<T: Clone> {
    fn new(config: &Config) -> Self;
    fn view(&self) -> Element<'_, Message>;
    fn update(&mut self, message: T) -> Option<Command>;
}
