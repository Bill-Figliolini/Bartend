use iced::Element;

pub mod application;
mod constants;
mod error;
mod input_handling;
mod screen;
mod widget;

trait Viewable<MessageOut: Clone> {
    fn view(&self) -> Element<'_, MessageOut>;
}
