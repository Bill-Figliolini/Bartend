//! The bar across the top of the window: the app's name in the corner above
//! the master list, then one tab per screen.

use iced::{
    Alignment, Element,
    Length::Fill,
    widget::{button, container, row, text},
};

use crate::presentation::{application::Message, constants, screen::ScreenKind};

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn tab_bar<'a>(active: ScreenKind) -> Element<'a, Message> {
    let brand = container(text("Bartend 🍸").size(22))
        .width(constants::MASTER_WIDTH)
        .padding([0, constants::PADDING])
        .center_y(Fill);

    let tabs = row(ScreenKind::ALL.into_iter().map(|kind| {
        let style = if kind == active {
            button::primary
        } else {
            button::text
        };
        button(text(kind.label()))
            .padding([8, 16])
            .style(style)
            .on_press(Message::OpenScreen(kind))
            .into()
    }))
    .spacing(constants::SPACING / 2)
    .align_y(Alignment::Center);

    row![brand, container(tabs).center_y(Fill)]
        .height(48)
        .width(Fill)
        .into()
}
