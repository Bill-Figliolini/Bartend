use iced::{self, Element, widget::column};

use crate::presentation::constants;

pub fn header<'a, Message: Clone + 'a>(
    contents: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let contents = contents.into();
    let divider = iced::widget::rule::horizontal(constants::DIV_SIZE);
    let divided_contents = column![contents, divider];
    iced::widget::container(divided_contents).into()
}
