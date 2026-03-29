use iced::{
    self, Element,
    Length::Fill,
    widget::{Container, column},
};

use crate::presentation::constants;

pub fn footer<'a, Message: Clone + 'a>(container: Container<'a, Message>) -> Element<'a, Message> {
    let footer = container.height(50).width(Fill);
    let divider = iced::widget::rule::horizontal(constants::DIV_SIZE);
    column![divider, footer].into()
}
