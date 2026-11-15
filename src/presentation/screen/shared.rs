//! Pieces the detail panels and forms of several screens share, so that
//! items and categories read alike: the header with a name, a breadcrumb
//! and Edit / Delete, the form page with Save / Cancel, and counts in
//! words.

use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    widget::{button, column, row, scrollable, space, text},
};

use crate::{
    logic::CategoryService,
    models::CategoryID,
    presentation::{
        application::Message,
        constants,
        widget::detail::{card, chip, hint, quiet_danger},
    },
};

/// Size of the name at the top of a detail panel or form.
pub(super) const NAME_SIZE: f32 = 28.0;

/// A detail panel's header: `name` in large type over a `subtitle` line on
/// the left, and Edit and Delete buttons on the right.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn detail_header(
    name: String,
    subtitle: Element<'_, Message>,
    edit: Message,
    delete: Message,
) -> Element<'_, Message> {
    let title = column![text(name).size(NAME_SIZE), subtitle].spacing(6);
    let edit = button("Edit").on_press(edit);
    let delete = button("Delete").style(quiet_danger).on_press(delete);
    let buttons = row![edit, delete].spacing(constants::SPACING);
    row![title, space().width(Fill), buttons].into()
}

/// A new- or edit form laid out like a detail panel: `title` in large type
/// over a muted `note`, with the form's `buttons` (Save, and Cancel while
/// editing) to the right, then the `fields` in a card.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn form_page<'a>(
    title: String,
    note: &'a str,
    buttons: Element<'a, Message>,
    fields: Element<'a, Message>,
) -> Element<'a, Message> {
    let title = column![text(title).size(NAME_SIZE), hint(note)].spacing(6);
    let header = row![title, space().width(Fill), buttons];
    scrollable(column![header, card(None, fields)].spacing(16)).into()
}

/// A breadcrumb from [`breadcrumb_label`] as a chip, with a muted "+N
/// more" when other paths lead there too, or `none` as a muted note when
/// there is no breadcrumb.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn breadcrumb_line(
    breadcrumb: Option<(String, usize)>,
    none: &str,
) -> Element<'_, Message> {
    match breadcrumb {
        Some((path, others)) => row![chip(path)]
            .push((others > 0).then(|| hint(format!("+{others} more"))))
            .spacing(constants::SPACING)
            .align_y(Center)
            .into(),
        None => hint(none).into(),
    }
}

/// The path down to `category`, as names joined by `›`, and how many other
/// paths lead to it.
pub(super) fn breadcrumb_label(
    category: CategoryID,
    category_service: &CategoryService,
) -> Option<(String, usize)> {
    let breadcrumb = category_service.breadcrumb(category)?;
    let names: Vec<String> = breadcrumb
        .path
        .iter()
        .map(|id| category_service.get(*id).cloned().unwrap_or_default().name)
        .collect();
    Some((names.join(" › "), breadcrumb.other_paths))
}

/// `count` of `noun` in words, e.g. "No recipes", "1 recipe", "3 recipes".
/// `noun` must make its plural with a plain "s".
pub(super) fn counted(count: usize, noun: &str) -> String {
    match count {
        0 => format!("No {noun}s"),
        1 => format!("1 {noun}"),
        n => format!("{n} {noun}s"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::screen::test_support::World;

    #[test]
    fn counts_read_naturally() {
        assert_eq!(counted(0, "recipe"), "No recipes");
        assert_eq!(counted(1, "recipe"), "1 recipe");
        assert_eq!(counted(4, "item"), "4 items");
    }

    #[test]
    fn breadcrumb_label_joins_names_down_the_tree() {
        let mut world = World::new();
        let spirit = world.add_category("Spirit");
        let gin = world.add_category("Gin");
        let botanical = world.add_category("Botanical");
        world.relate(spirit, gin);
        world.relate(botanical, gin);

        assert_eq!(
            breadcrumb_label(gin, &world.categories),
            Some(("Botanical › Gin".to_string(), 1))
        );
        assert_eq!(
            breadcrumb_label(spirit, &world.categories),
            Some(("Spirit".to_string(), 0))
        );
        assert_eq!(breadcrumb_label(CategoryID(99), &world.categories), None);
    }
}
