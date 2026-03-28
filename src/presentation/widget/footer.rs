use iced::{self, Element, Length::Fill, widget::column};

use crate::presentation::constants;

pub fn footer<'a, Message: Clone + 'a>(
    contents: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let contents = contents.into();
    let divider = iced::widget::rule::horizontal(constants::DIV_SIZE);
    let divided_contents = column![divider, contents].spacing(5);
    iced::widget::container(divided_contents)
        .align_left(Fill)
        .height(50)
        .into()
}
