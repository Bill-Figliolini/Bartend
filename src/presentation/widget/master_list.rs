//! The master panel on the left of every screen: an optional "+ New" button,
//! then a scrolling list with one button per entry.

use iced::{
    Element,
    Length::Fill,
    widget::{Column, button, column, container, scrollable, text},
};

use crate::presentation::{application::Message, constants};

/// One row of a [`master_list`].
#[derive(Debug)]
pub struct Entry {
    pub label: String,
    /// Whether the detail panel is showing this entry.
    pub selected: bool,
    pub on_press: Message,
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn master_list<'a>(
    new: Option<(String, Message)>,
    entries: impl IntoIterator<Item = Entry>,
) -> Element<'a, Message> {
    let entries = Column::with_children(entries.into_iter().map(|entry| {
        let style = if entry.selected {
            button::primary
        } else {
            button::text
        };
        button(text(entry.label))
            .width(Fill)
            .style(style)
            .on_press(entry.on_press)
            .into()
    }))
    .spacing(2);

    let mut contents = column![].spacing(constants::SPACING);
    if let Some((label, on_press)) = new {
        let new_button = button(text(label).width(Fill).center())
            .width(Fill)
            .style(button::secondary)
            .on_press(on_press);
        contents = contents.push(new_button);
    }
    contents = contents.push(scrollable(entries).height(Fill));

    container(contents)
        .width(constants::MASTER_WIDTH)
        .height(Fill)
        .padding(constants::PADDING)
        .into()
}
