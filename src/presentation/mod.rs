use crate::presentation::application::Command;
use iced::Element;

pub mod application;
mod constants;
mod screen;
mod widget;

trait Viewable<MessageOut: Clone> {
    fn view(&self) -> Element<'_, Message>;
}
trait Updateable<MessageIn: Clone> {
    fn update(&mut self, message: T) -> Option<Command>;
}
