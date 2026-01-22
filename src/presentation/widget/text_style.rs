use iced::{
    Element,
    Length::Fill,
    border::width,
    widget::{self, text::IntoFragment},
};

use crate::presentation::application::Message;

pub fn title<'a>(text: impl IntoFragment<'a>) -> Element<'a, Message> {
    widget::text(text)
        .size(30)
        .width(Fill)
        .height(45)
        .center()
        .into()
}
