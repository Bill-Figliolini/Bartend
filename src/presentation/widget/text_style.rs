/// mod text_style
/// Contains functions for standardizing the display of text throughout the program.
use iced::{
    Element,
    Length::Fill,
    widget::{self, text::IntoFragment},
};

use crate::presentation::application::Message;

pub fn title<'a>(text: impl IntoFragment<'a>) -> Element<'a, Message> {
    widget::text(text)
        .size(30)
        .line_height(1.5)
        .width(Fill)
        .center()
        .into()
}
