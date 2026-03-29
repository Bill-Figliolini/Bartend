use iced::{self, Element, Length::Fill, widget::column};

use crate::presentation::{constants, widget::Alignment};

pub fn footer<'a, Message: Clone + 'a>(
    contents: impl Into<Element<'a, Message>>,
    align: Alignment,
) -> Element<'a, Message> {
    let contents = contents.into();
    let footer = iced::widget::container(contents).height(50).width(Fill);
    let aligned_footer = match align {
        Alignment::Left => footer.align_left(Fill),
        Alignment::Center => footer.center_x(Fill),
        Alignment::Right => footer.align_right(Fill),
    };
    let divider = iced::widget::rule::horizontal(constants::DIV_SIZE);
    column![divider, aligned_footer].into()
}
