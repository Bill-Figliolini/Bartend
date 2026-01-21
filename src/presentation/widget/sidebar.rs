use iced::{Element, Length::Fill, widget::button};

pub fn item<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: impl Fn() -> Message + 'a,
) -> Element<'a, Message> {
    button(content)
        .on_press_with(on_press)
        .padding(10)
        .width(Fill)
        .into()
}
