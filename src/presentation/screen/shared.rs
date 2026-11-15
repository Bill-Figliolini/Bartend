//! Pieces the detail panels and forms of several screens share, so that
//! items, categories, recipes and servings read alike: the header with a
//! name, a subtitle and buttons such as Edit / Delete, the form page with
//! Save / Cancel, whether a recipe can be made, its drink volume, and counts
//! in words.

use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Task,
    widget::{button, column, row, scrollable, space, text},
};

use crate::{
    logic::{CategoryService, IngredientStatus},
    models::{BartendError, CategoryID, UnitSystem, VolumeSummary},
    presentation::{
        application::Message,
        constants,
        widget::detail::{Tone, card, chip, hint, quiet_danger, toned_chip},
    },
};

/// Size of the name at the top of a detail panel or form.
pub(super) const NAME_SIZE: f32 = 28.0;

/// What follows a write to the database: reloading the screen once it
/// succeeds, or reporting why it failed.
pub(super) fn reload_or_error<T>(result: Result<T, BartendError>) -> Task<Message> {
    match result {
        Ok(_) => Task::done(Message::ReloadScreen),
        Err(error) => Task::done(Message::Error(error)),
    }
}

/// What follows a write to the database that needs no reload: nothing once
/// it succeeds, or reporting why it failed.
pub(super) fn report_error<T>(result: Result<T, BartendError>) -> Task<Message> {
    match result {
        Ok(_) => Task::none(),
        Err(error) => Task::done(Message::Error(error)),
    }
}

/// A detail panel's header: `name` in large type over a `subtitle` line on
/// the left, and on the right any `lead` button (such as Serve) followed by
/// Edit and Delete.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn detail_header<'a>(
    name: String,
    subtitle: Element<'a, Message>,
    lead: Option<Element<'a, Message>>,
    edit: Message,
    delete: Message,
) -> Element<'a, Message> {
    let edit = button("Edit").on_press(edit);
    let delete = button("Delete").style(quiet_danger).on_press(delete);
    let buttons = row![lead, edit, delete].spacing(constants::SPACING);
    page_header(name, subtitle, buttons.into())
}

/// A panel's header: `name` in large type over a `subtitle` line on the
/// left, and `buttons` on the right. [`detail_header`] is this with Edit
/// and Delete.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn page_header<'a>(
    name: String,
    subtitle: Element<'a, Message>,
    buttons: Element<'a, Message>,
) -> Element<'a, Message> {
    let title = column![text(name).size(NAME_SIZE), subtitle].spacing(6);
    row![title, space().width(Fill), buttons].into()
}

/// Whether a recipe can be made from what is in stock, as shown under its
/// name on Recipes and Serving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Availability {
    /// The recipe has no ingredients yet.
    Draft,
    Ready,
    Missing,
}

impl Availability {
    /// Ready when every ingredient is [`IngredientStatus::InStock`], as
    /// Serving's list judges it.
    pub(super) fn of(statuses: &[IngredientStatus]) -> Self {
        if statuses.is_empty() {
            Self::Draft
        } else if statuses
            .iter()
            .all(|&status| status == IngredientStatus::InStock)
        {
            Self::Ready
        } else {
            Self::Missing
        }
    }

    /// The line under the recipe's name: a chip saying whether it can be
    /// made, or a note that it is still a draft.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn view<'a>(self) -> Element<'a, Message> {
        match self {
            Self::Draft => hint("Draft: no ingredients yet").into(),
            Self::Ready => toned_chip("Ready to make", Tone::Success),
            Self::Missing => toned_chip("Missing ingredients", Tone::Warning),
        }
    }
}

/// A drink volume tile's figure, and any line under it: the amounts that
/// can't join the volume, or why there is no volume to show.
pub(super) fn volume_figure(
    summary: &VolumeSummary,
    units: UnitSystem,
) -> (String, Option<String>) {
    let others: Vec<String> = summary
        .others
        .iter()
        .map(|&quantity| quantity.readable(units))
        .collect();
    let plus = (!others.is_empty()).then(|| format!("plus {}", others.join(", ")));
    match summary.volume {
        Some(volume) => (volume.readable(units), plus),
        None => (
            "—".to_string(),
            Some(plus.unwrap_or_else(|| "No ingredients measured by volume".to_string())),
        ),
    }
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
    form_page_with_cards(title, note, buttons, card(None, fields))
}

/// A [`form_page`] whose form lays out its own cards, e.g. one for the name
/// and one for a list of rows, so `cards` are shown as they are.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn form_page_with_cards<'a>(
    title: String,
    note: &'a str,
    buttons: Element<'a, Message>,
    cards: Element<'a, Message>,
) -> Element<'a, Message> {
    let title = column![text(title).size(NAME_SIZE), hint(note)].spacing(6);
    let header = row![title, space().width(Fill), buttons];
    scrollable(column![header, cards].spacing(16)).into()
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
        .map(|id| category_service.name_of(*id))
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
    fn a_recipe_is_ready_only_with_every_ingredient_in_stock() {
        use IngredientStatus::{InStock, NoneInStock, Short};
        assert_eq!(Availability::of(&[]), Availability::Draft);
        assert_eq!(Availability::of(&[InStock, InStock]), Availability::Ready);
        assert_eq!(Availability::of(&[InStock, Short]), Availability::Missing);
        assert_eq!(Availability::of(&[NoneInStock]), Availability::Missing);
    }

    #[test]
    fn volume_figure_shows_the_volume_and_whatever_cannot_join_it() {
        use crate::models::{Quantity, Unit};
        let summary = |volume, others| VolumeSummary { volume, others };
        let ml = |amount| Quantity::new(amount, Unit::Milliliter);
        let dashes = Quantity::new(2.0, Unit::Dash);
        let grams = Quantity::new(5.0, Unit::Gram);

        assert_eq!(
            volume_figure(&summary(Some(ml(90.0)), vec![]), UnitSystem::Metric),
            ("90 ml".to_string(), None)
        );
        assert_eq!(
            volume_figure(
                &summary(Some(ml(90.0)), vec![dashes, grams]),
                UnitSystem::Metric
            ),
            ("90 ml".to_string(), Some("plus 2 dash, 5 g".to_string()))
        );
        assert_eq!(
            volume_figure(&summary(None, vec![dashes]), UnitSystem::Metric),
            ("—".to_string(), Some("plus 2 dash".to_string()))
        );
        assert_eq!(
            volume_figure(&summary(None, vec![]), UnitSystem::Metric),
            (
                "—".to_string(),
                Some("No ingredients measured by volume".to_string())
            )
        );
    }

    #[test]
    fn volume_figure_follows_the_unit_system() {
        use crate::models::{Quantity, Unit};
        let summary = VolumeSummary {
            volume: Some(Quantity::new(2.0, Unit::FluidOunce)),
            others: Vec::new(),
        };
        assert_eq!(
            volume_figure(&summary, UnitSystem::Imperial).0,
            "2 fl oz".to_string()
        );
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
