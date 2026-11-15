//! Building blocks for detail panels: cards, stat tiles, a stock gauge,
//! chips, link rows and labelled fields. Every colour comes from the theme's palette,
//! so they suit whichever theme is chosen.

use iced::{
    Alignment::Center,
    Border, Element,
    Length::Fill,
    Theme,
    widget::{Column, Container, Text, button, column, container, progress_bar, row, space, text},
};

use crate::{logic::StockLevel, presentation::constants};

/// Corner radius of cards.
const CARD_RADIUS: f32 = 8.0;
/// Size of the headline figure in a stat tile.
const VALUE_SIZE: f32 = 28.0;
/// How much of the text colour muted text keeps.
const MUTED_ALPHA: f32 = 0.65;
/// Thickness of the stock gauge.
const GAUGE_GIRTH: f32 = 8.0;

/// A rounded, bordered box holding `content`, under a small muted `title`
/// if it has one.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn card<'a, Message: 'a>(
    title: Option<&str>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    card_box(title, content).into()
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn card_box<'a, Message: 'a>(
    title: Option<&str>,
    content: impl Into<Element<'a, Message>>,
) -> Container<'a, Message> {
    let body = Column::new()
        .push(title.map(caption))
        .push(content)
        .spacing(constants::SPACING);
    container(body)
        .width(Fill)
        .padding(constants::PADDING)
        .style(card_style)
}

/// A card showing one figure: a small `label`, the `value` in large type,
/// then anything `below` it, such as a [`gauge`].
///
/// To line tiles up in a row, give the shorter ones `.height(Fill)`: they
/// grow to match the tallest tile that keeps its own height. At least one
/// tile must keep its own height, or the row has none to match and
/// collapses.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn stat_tile<'a, Message: 'a>(
    label: &str,
    value: impl text::IntoFragment<'a>,
    below: Option<Element<'a, Message>>,
) -> Container<'a, Message> {
    let body = column![caption(label), text(value).size(VALUE_SIZE)]
        .push(below)
        .spacing(constants::SPACING / 2);
    card_box(None, body)
}

/// A bar filled to `fraction` (0 to 1), coloured by `level`, followed by
/// the share as a percentage.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn gauge<'a, Message: 'a>(fraction: f32, level: StockLevel) -> Element<'a, Message> {
    let bar = progress_bar(0.0..=1.0, fraction)
        .girth(GAUGE_GIRTH)
        .style(move |theme: &Theme| gauge_style(theme, level));
    let share = text(percent(fraction))
        .size(constants::SMALL_TEXT)
        .style(muted);
    row![bar, share]
        .spacing(constants::SPACING)
        .align_y(Center)
        .into()
}

/// A small pill-shaped label, e.g. for a category.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn chip<'a, Message: 'a>(label: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    container(text(label).size(constants::SMALL_TEXT))
        .padding([2, 10])
        .style(chip_style)
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

/// A form field with its `label` above it.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn labeled<'a, Message: 'a>(
    label: &str,
    field: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let label = text(label.to_owned())
        .size(constants::SMALL_TEXT)
        .style(muted);
    column![label, field.into()].spacing(4).into()
}

/// A small, muted, upper-case heading, as on cards and stat tiles.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn caption<'a>(label: &str) -> Text<'a> {
    text(label.to_uppercase())
        .size(constants::SMALL_TEXT)
        .style(muted)
}

/// A small, muted note, e.g. explaining why something is missing.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn hint<'a>(note: impl text::IntoFragment<'a>) -> Text<'a> {
    text(note).size(constants::SMALL_TEXT).style(muted)
}

/// A button for a destructive action that is less loud than
/// [`button::danger`] until hovered: danger-coloured text and outline on
/// no background, filling in on hover.
pub fn quiet_danger(theme: &Theme, status: button::Status) -> button::Style {
    let danger = theme.extended_palette().danger;
    let border = Border {
        width: 1.0,
        radius: 2.0.into(),
        color: danger.base.color,
    };
    match status {
        button::Status::Active | button::Status::Disabled => button::Style {
            background: None,
            text_color: danger.base.color,
            border,
            ..button::Style::default()
        },
        button::Status::Hovered | button::Status::Pressed => button::Style {
            border,
            ..button::danger(theme, status)
        },
    }
}

/// Text in a fainter shade of the theme's text colour, for secondary
/// details.
pub fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().text.scale_alpha(MUTED_ALPHA)),
    }
}

/// `fraction` as a whole percentage, rounded down so that an item just
/// under a threshold never reads as being on it.
pub fn percent(fraction: f32) -> String {
    format!("{:.0}%", (fraction * 100.0).floor())
}

fn card_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.weakest.color.into()),
        text_color: Some(palette.background.weakest.text),
        border: Border {
            width: 1.0,
            radius: CARD_RADIUS.into(),
            color: palette.background.weak.color,
        },
        ..container::Style::default()
    }
}

fn chip_style(theme: &Theme) -> container::Style {
    let pair = theme.extended_palette().background.weak;
    container::Style {
        background: Some(pair.color.into()),
        text_color: Some(pair.text),
        border: Border {
            radius: f32::MAX.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

fn gauge_style(theme: &Theme, level: StockLevel) -> progress_bar::Style {
    let palette = theme.extended_palette();
    let bar = match level {
        StockLevel::Healthy => palette.success.base.color,
        StockLevel::Low => palette.warning.base.color,
        StockLevel::Critical => palette.danger.base.color,
    };
    progress_bar::Style {
        background: palette.background.weak.color.into(),
        bar: bar.into(),
        border: Border {
            radius: (GAUGE_GIRTH / 2.0).into(),
            ..Border::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use iced::Background;

    use super::*;

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
        for theme in Theme::ALL {
            let palette = theme.extended_palette();
            let cases = [
                (StockLevel::Healthy, palette.success.base.color),
                (StockLevel::Low, palette.warning.base.color),
                (StockLevel::Critical, palette.danger.base.color),
            ];
            for (level, expected) in cases {
                let style = gauge_style(theme, level);
                assert_eq!(style.bar, Background::Color(expected), "{theme} {level:?}");
                assert_eq!(
                    style.background,
                    Background::Color(palette.background.weak.color)
                );
            }
        }
    }

    #[test]
    fn cards_and_chips_use_the_theme_background() {
        for theme in Theme::ALL {
            let palette = theme.extended_palette();

            let card = card_style(theme);
            assert_eq!(
                card.background,
                Some(Background::Color(palette.background.weakest.color))
            );
            assert_eq!(card.border.color, palette.background.weak.color);

            let chip = chip_style(theme);
            assert_eq!(chip.background, Some(palette.background.weak.color.into()));
            assert_eq!(chip.text_color, Some(palette.background.weak.text));
        }
    }

    #[test]
    fn quiet_danger_is_outlined_until_hovered() {
        for theme in Theme::ALL {
            let danger = theme.extended_palette().danger.base.color;

            let active = quiet_danger(theme, button::Status::Active);
            assert_eq!(active.background, None, "{theme}");
            assert_eq!(active.text_color, danger);
            assert_eq!(active.border.color, danger);

            let hovered = quiet_danger(theme, button::Status::Hovered);
            let loud = button::danger(theme, button::Status::Hovered);
            assert_eq!(hovered.background, loud.background, "{theme}");
            assert_eq!(hovered.text_color, loud.text_color);
        }
    }

    #[test]
    fn muted_text_is_a_fainter_theme_text_colour() {
        for theme in Theme::ALL {
            let text = theme.palette().text;
            let color = muted(theme).color.unwrap();
            assert_eq!([color.r, color.g, color.b], [text.r, text.g, text.b]);
            assert!(color.a < text.a, "{theme}");
        }
    }
}
