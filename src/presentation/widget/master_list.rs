//! The master panel on the left of every screen: an optional "+ New" button,
//! then a scrolling list with one button per entry, or a note when the list
//! is empty. It sits on the frame's raised surface.

use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Theme,
    widget::{Column, button, column, container, row, scrollable, space, text},
};

use crate::presentation::{
    application::Message,
    constants,
    widget::{detail::hint, frame},
};

/// One row of a [`master_list`].
#[derive(Debug)]
pub struct Entry {
    pub label: String,
    /// Whether the detail panel is showing this entry.
    pub selected: bool,
    pub on_press: Message,
    /// Shows the label faded, e.g. for a recipe that cannot be made right
    /// now. The entry can still be chosen. A selected entry is never faded,
    /// so that its label stays readable on the highlight.
    pub dimmed: bool,
}

/// How much of the text colour a dimmed entry keeps.
const DIMMED_ALPHA: f32 = 0.45;
/// Size of the accent bar beside the selected entry.
const MARKER_WIDTH: f32 = 3.0;
const MARKER_HEIGHT: f32 = 16.0;

/// The list, under a `new` button if given. With no entries, `empty` is
/// shown in their place.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn master_list<'a>(
    new: Option<(String, Message)>,
    entries: impl IntoIterator<Item = Entry>,
    empty: &'a str,
) -> Element<'a, Message> {
    let rows: Vec<Element<'a, Message>> = entries.into_iter().map(entry).collect();
    let list: Element<'a, Message> = if rows.is_empty() {
        container(hint(empty)).padding([4, 6]).into()
    } else {
        scrollable(Column::with_children(rows).spacing(2))
            .height(Fill)
            .into()
    };

    let mut contents = column![].spacing(constants::SPACING);
    if let Some((label, on_press)) = new {
        let new_button = button(text(label).width(Fill).center())
            .width(Fill)
            .style(frame::outlined)
            .on_press(on_press);
        contents = contents.push(new_button);
    }
    contents = contents.push(list);

    container(contents)
        .width(constants::MASTER_WIDTH)
        .height(Fill)
        .padding(constants::PADDING)
        .style(frame::surface)
        .into()
}

/// One entry: its label beside an accent bar that shows while it is
/// selected, on the frame's tint.
#[cfg_attr(coverage_nightly, coverage(off))]
fn entry<'a>(entry: Entry) -> Element<'a, Message> {
    let Entry {
        label,
        selected,
        on_press,
        dimmed,
    } = entry;
    let marker = container(space())
        .width(MARKER_WIDTH)
        .height(MARKER_HEIGHT)
        .style(move |theme: &Theme| frame::marker(theme, selected));
    let dimmed = dimmed.then_some(DIMMED_ALPHA);
    button(row![marker, text(label)].spacing(8).align_y(Center))
        .width(Fill)
        .padding([6, 6])
        .style(move |theme: &Theme, status| frame::entry(theme, status, selected, dimmed))
        .on_press(on_press)
        .into()
}
