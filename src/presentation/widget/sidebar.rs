use iced::{Element, Length::Fill, widget};

pub fn button<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: impl Fn() -> Message + 'a,
) -> Element<'a, Message> {
    widget::button(content)
        .on_press_with(on_press)
        .padding(10)
        .width(Fill)
        .into()
}
