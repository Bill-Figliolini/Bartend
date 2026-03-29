use iced::{
    Element,
    Length::Fill,
    widget::{self, row},
};

use crate::presentation::constants;

pub struct Sidebar<'a, Message: Clone + 'a> {
    buttons: Vec<Element<'a, Message>>,
}

impl<'a, Message: Clone + 'a> Sidebar<'a, Message> {
    pub const fn new() -> Self {
        Self {
            buttons: Vec::new(),
        }
    }

    pub fn button(
        mut self,
        content: impl Into<Element<'a, Message>>,
        on_press: impl Fn() -> Message + 'a,
    ) -> Self {
        let button = widget::button(content)
            .on_press_with(on_press)
            .padding(10)
            .width(Fill)
            .into();
        self.buttons.push(button);
        self
    }
    pub fn into(self) -> Element<'a, Message> {
        let sidebar_buttons = iced::widget::Column::from_iter(self.buttons);
        let sidebar_contents = sidebar_buttons.width(150).height(Fill);
        let sidebar_div = iced::widget::rule::vertical(constants::DIV_SIZE);
        iced::widget::container(row![sidebar_contents, sidebar_div]).into()
    }
}
