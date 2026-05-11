use crate::{
    logic::config::Config,
    presentation::application::{Command, Message},
};
use iced::Element;

pub mod application;
mod constants;
mod screen;
mod widget;

trait Viewable<T: Clone> {
    fn view(&self) -> Element<'_, Message>;
}
trait Updateable<T: Clone> {
    fn update(&mut self, message: T) -> Option<Command>;
}
