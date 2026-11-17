//! Building blocks for detail panels: double-bordered cards (plain or
//! [toned](Tone)), stat tiles, with corner brackets if the [`Styling`] has
//! them (showing a figure or holding a control), a stock gauge (segmented
//! or one bar, as the styling says),
//! chips (plain or toned), link rows (optionally with a row action or a
//! toned status), column headings and labelled fields. Cards, tiles and
//! chips have the styling's corners.
//! Every colour comes from the theme's palette, so they suit whichever
//! theme is chosen.

use iced::{
    Alignment::Center,
    Border, Color, Element,
    Length::{Fill, Fixed},
    Theme,
    alignment::Horizontal::Right,
    theme::palette::Pair,
    widget::{
        Button, Column, Row, Text, button, column, container, progress_bar, row, space, text,
    },
};

use crate::{
    logic::StockLevel,
    presentation::{
        constants,
        styling::{GaugeKind, Styling},
        widget::{
            framed_card::{FRAME_PADDING, FramedCard},
            styled,
        },
    },
};

/// How much of the accent colour the inner line of a plain card keeps.
const INNER_LINE_ALPHA: f32 = 0.35;
/// Space between a card's inner line and its content.
const CARD_PADDING: f32 = constants::PADDING as f32 + FRAME_PADDING;
/// Size of the headline figure in a stat tile.
const VALUE_SIZE: f32 = 28.0;
/// How much of a toned card's text colour fills a button on it when
/// hovered.
const ON_TONE_WASH: f32 = 0.12;
/// How much of the text colour muted text keeps.
const MUTED_ALPHA: f32 = 0.65;
/// Width of the status column of a [`status_link_row`], so the details
/// before it line up from row to row. Fits "None in stock" in small type.
const STATUS_WIDTH: f32 = 100.0;

/// How a chip or a status reads: plain, or in the theme's colour for good
/// news, a warning, or a problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Success,
    Warning,
    Danger,
}

/// A box with a double border, holding `content` under a small `title` if
/// it has one.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn card<'a, Message: 'a>(
    styling: &Styling,
    title: Option<&str>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    card_box(styling, title, content).into()
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn card_box<'a, Message: 'a>(
    styling: &Styling,
    title: Option<&str>,
    content: impl Into<Element<'a, Message>>,
) -> FramedCard<'a, Message> {
    let body = Column::new()
        .push(title.map(|title| caption(styling, title)))
        .push(content)
        .spacing(constants::SPACING);
    let radius = styling.radius;
    let box_ = container(body)
        .width(Fill)
        .padding(CARD_PADDING)
        .style(move |theme: &Theme| card_style(theme, radius));
    FramedCard::new(styling, box_, |theme| inner_line(theme, Tone::Neutral))
}

/// A [`card_box`] for a tile, with accent brackets in its corners if the
/// styling has them.
#[cfg_attr(coverage_nightly, coverage(off))]
fn tile_box<'a, Message: 'a>(
    styling: &Styling,
    content: impl Into<Element<'a, Message>>,
) -> FramedCard<'a, Message> {
    let card = card_box(styling, None, content);
    if styling.tile_brackets {
        card.brackets(styled::accent)
    } else {
        card
    }
}

/// A card showing one figure: a small `label`, the `value` in large type,
/// then anything `below` it, such as a [`gauge`].
///
/// To line tiles up in a row, give the shorter ones `.height(Fill)`: they
/// grow to match the tallest tile that keeps its own height and comes
/// before them in the row. So put a tile that keeps its own height first:
/// a `.height(Fill)` tile with none before it has nothing to match and
/// collapses to nothing. The exception is a tile with a fixed width: it is
/// laid out after all the others, so it matches the row's height wherever
/// it sits.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn stat_tile<'a, Message: 'a>(
    styling: &Styling,
    label: &str,
    value: impl text::IntoFragment<'a>,
    below: Option<Element<'a, Message>>,
) -> FramedCard<'a, Message> {
    let body = column![caption(styling, label), text(value).size(VALUE_SIZE)]
        .push(below)
        .spacing(constants::SPACING / 2);
    tile_box(styling, body)
}

/// A [`stat_tile`] whose figure is a control, such as a stepper, rather
/// than a value: a small `label` over the `control`. Lines up in a row the
/// same way.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn control_tile<'a, Message: 'a>(
    styling: &Styling,
    label: &str,
    control: impl Into<Element<'a, Message>>,
) -> FramedCard<'a, Message> {
    let body = column![caption(styling, label), control.into()].spacing(constants::SPACING / 2);
    tile_box(styling, body)
}

/// A card filled with the weak shade of `tone`'s colour and double
/// outlined in it, for a message about the whole page, such as what was
/// just served. A neutral one is a plain [`card`].
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn toned_card<'a, Message: 'a>(
    styling: &Styling,
    tone: Tone,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let radius = styling.radius;
    let box_ = container(content)
        .width(Fill)
        .padding(CARD_PADDING)
        .style(move |theme: &Theme| toned_card_style(theme, tone, radius));
    FramedCard::new(styling, box_, move |theme| inner_line(theme, tone)).into()
}

/// A button for a [`toned_card`] of the same `tone`, such as Dismiss: a
/// [secondary button](styled::secondary_button) outlined in the card's text
/// colour, which reads on the card where the accent may not.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn toned_card_button<'a, Message: 'a>(
    styling: &Styling,
    tone: Tone,
    label: &str,
) -> Button<'a, Message> {
    let secondary = styled::secondary(styling);
    styled::secondary_button(styling, label)
        .style(move |theme: &Theme, status| on_tone(secondary(theme, status), theme, status, tone))
}

/// How full something is: `fraction` (0 to 1) of the gauge filled in the
/// colour of `level`, followed by the share as a percentage. The gauge is
/// a row of square segments (see [`lit_segments`]) or one bar with the
/// styling's corners, as thick as the styling says.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn gauge<'a, Message: 'a>(
    styling: &Styling,
    fraction: f32,
    level: StockLevel,
) -> Element<'a, Message> {
    let girth = styling.gauge.girth;
    let bar: Element<'a, Message> = match styling.gauge.kind {
        GaugeKind::Segments { count, gap } => {
            let lit = lit_segments(count, fraction);
            let segments = (0..count).map(|index| {
                let on = usize::from(index) < lit;
                container(space())
                    .width(Fill)
                    .height(girth)
                    .style(move |theme: &Theme| segment_style(theme, level, on))
                    .into()
            });
            Row::with_children(segments).spacing(gap).width(Fill).into()
        }
        GaugeKind::Bar => {
            let radius = styling.radius;
            progress_bar(0.0..=1.0, fraction)
                .girth(girth)
                .style(move |theme: &Theme| bar_style(theme, level, radius))
                .into()
        }
    };
    let share = text(percent(fraction))
        .size(constants::SMALL_TEXT)
        .style(muted);
    row![bar, share]
        .spacing(constants::SPACING)
        .align_y(Center)
        .into()
}

/// How many of a gauge's `count` segments are lit for a `fraction` of
/// stock, from 0 to 1: rounded up, so any stock at all lights at least
/// one, and none lights none.
pub fn lit_segments(count: u8, fraction: f32) -> usize {
    // Leeway for fractions such as 0.15 that come out a hair over their
    // true value, so they don't light a segment too many.
    const LEEWAY: f32 = 1e-4;
    let filled = fraction * f32::from(count) - LEEWAY;
    (0..count)
        .filter(|&index| f32::from(index) < filled)
        .count()
        .max(usize::from(fraction > 0.0))
        .min(usize::from(count))
}

/// A small label, e.g. for a category.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn chip<'a, Message: 'a>(
    styling: &Styling,
    label: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
    let radius = styling.radius;
    container(text(label).size(constants::SMALL_TEXT))
        .padding([2, 10])
        .style(move |theme: &Theme| chip_style(theme, radius))
        .into()
}

/// A [`chip`] coloured by `tone`, e.g. to say whether a recipe can be
/// made.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn toned_chip<'a, Message: 'a>(
    styling: &Styling,
    label: impl text::IntoFragment<'a>,
    tone: Tone,
) -> Element<'a, Message> {
    let radius = styling.radius;
    container(text(label).size(constants::SMALL_TEXT))
        .padding([2, 10])
        .style(move |theme: &Theme| toned_chip_style(theme, tone, radius))
        .into()
}

/// A full-width row that can be clicked, e.g. to open a recipe: `label`
/// on the left and a muted `detail` on the right.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn link_row<'a, Message: Clone + 'a>(
    label: impl text::IntoFragment<'a>,
    detail: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message> {
    let content = row![text(label), space().width(Fill), text(detail).style(muted)];
    button(content)
        .width(Fill)
        .style(button::text)
        .on_press(on_press)
        .into()
}

/// A [`link_row`] with a separate `action` control, such as a Remove
/// button, at its right end. The action sits outside the link, so
/// pressing it does not also follow the link.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn link_row_with_action<'a, Message: Clone + 'a>(
    label: impl text::IntoFragment<'a>,
    detail: impl text::IntoFragment<'a>,
    on_press: Message,
    action: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![link_row(label, detail, on_press), action.into()]
        .spacing(constants::SPACING)
        .align_y(Center)
        .into()
}

/// A [`link_row`] that ends in a short `status` coloured by its tone, such
/// as whether an ingredient is in stock. The status sits in a fixed-width
/// column so each row's `detail` lines up with the others.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn status_link_row<'a, Message: Clone + 'a>(
    label: impl text::IntoFragment<'a>,
    detail: impl text::IntoFragment<'a>,
    (status, tone): (impl text::IntoFragment<'a>, Tone),
    on_press: Message,
) -> Element<'a, Message> {
    let status = text(status)
        .size(constants::SMALL_TEXT)
        .width(Fixed(STATUS_WIDTH))
        .align_x(Right)
        .style(move |theme: &Theme| toned_text(theme, tone));
    let content = row![
        text(label),
        space().width(Fill),
        text(detail).style(muted),
        status
    ]
    .spacing(constants::SPACING)
    .align_y(Center);
    button(content)
        .width(Fill)
        .style(button::text)
        .on_press(on_press)
        .into()
}

/// A compact [`styled::quiet_danger`] button, for an action on one row of
/// a list rather than on the whole page.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn row_danger_button<'a, Message: Clone + 'a>(
    styling: &Styling,
    label: &str,
    on_press: Message,
) -> Element<'a, Message> {
    styled::small_button(styling, label)
        .style(styled::quiet_danger(styling))
        .on_press(on_press)
        .into()
}

/// A form field with its `label` above it as a [`column_heading`].
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn labeled<'a, Message: 'a>(
    styling: &Styling,
    label: &str,
    field: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![column_heading(styling, label), field.into()]
        .spacing(4)
        .into()
}

/// A small, muted [label](styled::label) over a field or a column of a
/// table.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn column_heading<'a>(styling: &Styling, label: &str) -> Text<'a> {
    styled::label(styling, label)
        .size(constants::SMALL_TEXT)
        .style(muted)
}

/// A small [label](styled::label) in the accent colour, as on cards and
/// stat tiles.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn caption<'a>(styling: &Styling, label: &str) -> Text<'a> {
    styled::label(styling, label)
        .size(constants::SMALL_TEXT)
        .style(styled::accented)
}

/// A small, muted note, e.g. explaining why something is missing.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn hint<'a>(note: impl text::IntoFragment<'a>) -> Text<'a> {
    text(note).size(constants::SMALL_TEXT).style(muted)
}

/// Text in a fainter shade of the theme's text colour, for secondary
/// details.
pub fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().text.scale_alpha(MUTED_ALPHA)),
    }
}

/// Text coloured by `tone`: [muted](muted) when neutral, otherwise in the
/// theme's success, warning or danger colour.
pub fn toned_text(theme: &Theme, tone: Tone) -> text::Style {
    let palette = theme.extended_palette();
    let color = match tone {
        Tone::Neutral => return muted(theme),
        Tone::Success => palette.success.base.color,
        Tone::Warning => palette.warning.base.color,
        Tone::Danger => palette.danger.base.color,
    };
    text::Style { color: Some(color) }
}

/// `fraction` as a whole percentage, rounded down so that an item just
/// under a threshold never reads as being on it.
pub fn percent(fraction: f32) -> String {
    format!("{:.0}%", (fraction * 100.0).floor())
}

/// A card with corners of `radius`.
fn card_style(theme: &Theme, radius: f32) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.weakest.color.into()),
        text_color: Some(palette.background.weakest.text),
        border: Border {
            width: 1.0,
            radius: radius.into(),
            color: palette.background.weak.color,
        },
        ..container::Style::default()
    }
}

/// A chip with corners of `radius`.
fn chip_style(theme: &Theme, radius: f32) -> container::Style {
    let pair = theme.extended_palette().background.weak;
    container::Style {
        background: Some(pair.color.into()),
        text_color: Some(pair.text),
        border: Border::default().rounded(radius),
        ..container::Style::default()
    }
}

/// A [`chip_style`] filled with the weak shade of `tone`'s colour, or the
/// plain chip when neutral.
fn toned_chip_style(theme: &Theme, tone: Tone, radius: f32) -> container::Style {
    let chip = chip_style(theme, radius);
    let Some(shade) = tone_shades(theme, tone) else {
        return chip;
    };
    container::Style {
        background: Some(shade.weak.color.into()),
        text_color: Some(shade.weak.text),
        ..chip
    }
}

/// A [`card_style`] filled with the weak shade of `tone`'s colour and
/// outlined in its base colour, or the plain card when neutral.
fn toned_card_style(theme: &Theme, tone: Tone, radius: f32) -> container::Style {
    let card = card_style(theme, radius);
    let Some(shade) = tone_shades(theme, tone) else {
        return card;
    };
    container::Style {
        background: Some(shade.weak.color.into()),
        text_color: Some(shade.weak.text),
        border: Border {
            color: shade.base.color,
            ..card.border
        },
        ..card
    }
}

/// The colour of the inner line of a card of `tone`: a faint wash of the
/// accent on a plain card, or the base colour of the tone, as its outer
/// line is.
fn inner_line(theme: &Theme, tone: Tone) -> Color {
    match tone_shades(theme, tone) {
        Some(shade) => shade.base.color,
        None => styled::accent(theme).scale_alpha(INNER_LINE_ALPHA),
    }
}

/// A [`toned_card_button`]: its [`styled::secondary`] `style` recoloured
/// in the text colour of a `tone` card, with a faint wash of it on hover.
/// A neutral one keeps the plain secondary style.
fn on_tone(
    mut style: button::Style,
    theme: &Theme,
    status: button::Status,
    tone: Tone,
) -> button::Style {
    if let Some(shade) = tone_shades(theme, tone) {
        let ink = shade.weak.text;
        style.background = matches!(status, button::Status::Hovered | button::Status::Pressed)
            .then(|| ink.scale_alpha(ON_TONE_WASH).into());
        style.text_color = ink;
        style.border.color = ink;
    }
    style
}

/// The base and weak shades of `tone`'s colour in the theme, or `None`
/// when neutral.
fn tone_shades(theme: &Theme, tone: Tone) -> Option<Shades> {
    let palette = theme.extended_palette();
    let (base, weak) = match tone {
        Tone::Neutral => return None,
        Tone::Success => (palette.success.base, palette.success.weak),
        Tone::Warning => (palette.warning.base, palette.warning.weak),
        Tone::Danger => (palette.danger.base, palette.danger.weak),
    };
    Some(Shades { base, weak })
}

/// Two shades of one of the theme's colours, each with the text colour
/// that reads on it.
struct Shades {
    base: Pair,
    weak: Pair,
}

/// One segment of the stock gauge: in the colour of `level` when `lit`,
/// or the shade of cards' borders when not.
fn segment_style(theme: &Theme, level: StockLevel, lit: bool) -> container::Style {
    let color = if lit {
        level_color(theme, level)
    } else {
        unlit(theme)
    };
    container::Style {
        background: Some(color.into()),
        ..container::Style::default()
    }
}

/// The stock gauge as one bar: filled in the colour of `level` on a track
/// in the shade of cards' borders, with corners of `radius`.
fn bar_style(theme: &Theme, level: StockLevel, radius: f32) -> progress_bar::Style {
    progress_bar::Style {
        background: unlit(theme).into(),
        bar: level_color(theme, level).into(),
        border: Border::default().rounded(radius),
    }
}

/// The colour of the filled part of the stock gauge at `level`.
fn level_color(theme: &Theme, level: StockLevel) -> Color {
    let palette = theme.extended_palette();
    match level {
        StockLevel::Healthy => palette.success.base.color,
        StockLevel::Low => palette.warning.base.color,
        StockLevel::Critical => palette.danger.base.color,
    }
}

/// The colour of the empty part of the stock gauge.
fn unlit(theme: &Theme) -> Color {
    theme.extended_palette().background.weak.color
}

#[cfg(test)]
mod tests {
    use iced::Background;

    use super::*;
    use crate::presentation::{
        styling::{PLAIN, Styling},
        theme::Themes,
    };

    const DECO: Styling = Styling::ART_DECO;
    const SQUARE: f32 = Styling::ART_DECO.radius;

    #[test]
    fn percentages_round_down() {
        assert_eq!(percent(0.0), "0%");
        assert_eq!(percent(0.099), "9%");
        assert_eq!(percent(0.25), "25%");
        assert_eq!(percent(0.999), "99%");
        assert_eq!(percent(1.0), "100%");
    }

    #[test]
    fn gauge_colour_follows_the_stock_level_in_every_theme() {
        for theme in Themes::new().options() {
            let palette = theme.extended_palette();
            let cases = [
                (StockLevel::Healthy, palette.success.base.color),
                (StockLevel::Low, palette.warning.base.color),
                (StockLevel::Critical, palette.danger.base.color),
            ];
            for (level, expected) in cases {
                let lit = segment_style(theme, level, true);
                assert_eq!(lit.background, Some(expected.into()), "{theme} {level:?}");
                assert_eq!(
                    segment_style(theme, level, false).background,
                    Some(Background::Color(palette.background.weak.color))
                );
                let bar = bar_style(theme, level, PLAIN.radius);
                assert_eq!(bar.bar, expected.into(), "{theme} {level:?}");
                assert_eq!(bar.background, palette.background.weak.color.into());
                assert_eq!(bar.border.radius, PLAIN.radius.into());
            }
        }
    }

    #[test]
    fn gauge_segments_light_up_rounding_up() {
        let lit = |fraction| lit_segments(20, fraction);
        assert_eq!(lit(0.0), 0);
        assert_eq!(lit(-0.5), 0);
        assert_eq!(lit(0.001), 1);
        assert_eq!(lit(0.05), 1);
        assert_eq!(lit(0.051), 2);
        assert_eq!(lit(0.15), 3);
        assert_eq!(lit(0.75), 15);
        assert_eq!(lit(0.999), 20);
        assert_eq!(lit(1.0), 20);
        assert_eq!(lit(1.5), 20);
    }

    #[test]
    fn gauges_with_fewer_segments_light_proportionally() {
        assert_eq!(lit_segments(5, 0.0), 0);
        assert_eq!(lit_segments(5, 0.01), 1);
        assert_eq!(lit_segments(5, 0.4), 2);
        assert_eq!(lit_segments(5, 0.41), 3);
        assert_eq!(lit_segments(5, 1.0), 5);
        assert_eq!(lit_segments(0, 0.5), 0);
    }

    #[test]
    fn cards_and_chips_have_the_given_corners() {
        for radius in [SQUARE, PLAIN.radius] {
            for theme in Themes::new().options() {
                assert_eq!(card_style(theme, radius).border.radius, radius.into());
                assert_eq!(chip_style(theme, radius).border.radius, radius.into());
                assert_eq!(
                    toned_chip_style(theme, Tone::Success, radius).border.radius,
                    radius.into()
                );
            }
        }
    }

    #[test]
    fn inner_lines_are_faint_accent_or_the_tone_colour() {
        for theme in Themes::new().options() {
            let palette = theme.extended_palette();
            assert_eq!(
                inner_line(theme, Tone::Neutral),
                theme.palette().primary.scale_alpha(INNER_LINE_ALPHA)
            );
            assert_eq!(inner_line(theme, Tone::Success), palette.success.base.color);
            assert_eq!(inner_line(theme, Tone::Warning), palette.warning.base.color);
            assert_eq!(inner_line(theme, Tone::Danger), palette.danger.base.color);
        }
    }

    #[test]
    fn cards_and_chips_use_the_theme_background() {
        for theme in Themes::new().options() {
            let palette = theme.extended_palette();

            let card = card_style(theme, SQUARE);
            assert_eq!(
                card.background,
                Some(Background::Color(palette.background.weakest.color))
            );
            assert_eq!(card.border.color, palette.background.weak.color);

            let chip = chip_style(theme, SQUARE);
            assert_eq!(chip.background, Some(palette.background.weak.color.into()));
            assert_eq!(chip.text_color, Some(palette.background.weak.text));
        }
    }

    #[test]
    fn muted_text_is_a_fainter_theme_text_colour() {
        for theme in Themes::new().options() {
            let text = theme.palette().text;
            let color = muted(theme).color.unwrap();
            assert_eq!([color.r, color.g, color.b], [text.r, text.g, text.b]);
            assert!(color.a < text.a, "{theme}");
        }
    }

    #[test]
    fn toned_chips_use_the_weak_shade_of_their_tone() {
        for theme in Themes::new().options() {
            let palette = theme.extended_palette();
            assert_eq!(
                toned_chip_style(theme, Tone::Neutral, SQUARE),
                chip_style(theme, SQUARE)
            );
            let cases = [
                (Tone::Success, palette.success.weak),
                (Tone::Warning, palette.warning.weak),
                (Tone::Danger, palette.danger.weak),
            ];
            for (tone, pair) in cases {
                let style = toned_chip_style(theme, tone, SQUARE);
                assert_eq!(
                    style.background,
                    Some(pair.color.into()),
                    "{theme} {tone:?}"
                );
                assert_eq!(style.text_color, Some(pair.text));
                assert_eq!(style.border, chip_style(theme, SQUARE).border);
            }
        }
    }

    #[test]
    fn toned_cards_are_filled_with_the_weak_shade_and_outlined_in_the_base() {
        for theme in Themes::new().options() {
            let palette = theme.extended_palette();
            assert_eq!(
                toned_card_style(theme, Tone::Neutral, SQUARE),
                card_style(theme, SQUARE)
            );
            let cases = [
                (Tone::Success, palette.success.base, palette.success.weak),
                (Tone::Warning, palette.warning.base, palette.warning.weak),
                (Tone::Danger, palette.danger.base, palette.danger.weak),
            ];
            for (tone, base, weak) in cases {
                let style = toned_card_style(theme, tone, SQUARE);
                assert_eq!(
                    style.background,
                    Some(weak.color.into()),
                    "{theme} {tone:?}"
                );
                assert_eq!(style.text_color, Some(weak.text));
                assert_eq!(style.border.color, base.color);
                assert_eq!(style.border.radius, card_style(theme, SQUARE).border.radius);
            }
        }
    }

    #[test]
    fn buttons_on_toned_cards_are_in_the_cards_text_colour() {
        for theme in Themes::new().options() {
            let active = button::Status::Active;
            let hovered = button::Status::Hovered;
            let secondary = styled::secondary(&DECO);
            let on_tone = |status, tone| on_tone(secondary(theme, status), theme, status, tone);
            assert_eq!(on_tone(active, Tone::Neutral), secondary(theme, active));
            for tone in [Tone::Success, Tone::Warning, Tone::Danger] {
                let ink = tone_shades(theme, tone).unwrap().weak.text;
                let style = on_tone(active, tone);
                assert_eq!(style.text_color, ink, "{theme} {tone:?}");
                assert_eq!(style.border.color, ink);
                assert_eq!(style.background, None);
                assert_eq!(
                    on_tone(hovered, tone).background,
                    Some(ink.scale_alpha(ON_TONE_WASH).into())
                );
            }
        }
    }

    #[test]
    fn toned_text_is_muted_or_the_tone_colour() {
        for theme in Themes::new().options() {
            let palette = theme.extended_palette();
            assert_eq!(toned_text(theme, Tone::Neutral).color, muted(theme).color);
            let cases = [
                (Tone::Success, palette.success.base.color),
                (Tone::Warning, palette.warning.base.color),
                (Tone::Danger, palette.danger.base.color),
            ];
            for (tone, color) in cases {
                assert_eq!(
                    toned_text(theme, tone).color,
                    Some(color),
                    "{theme} {tone:?}"
                );
            }
        }
    }
}
