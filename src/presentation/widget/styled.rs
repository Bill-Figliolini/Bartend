//! The touches every screen shares, drawn as the [`Styling`] says: titles
//! and labels in its heading face, its ornament, the rule under page
//! headers, and buttons labeled in its heading face. Every color comes
//! from the theme's palette, so they suit whichever theme is chosen.

use iced::{
    Alignment::Center,
    Border, Color, Element,
    Length::Fill,
    Padding, Theme,
    widget::{Button, Text, button, row, rule, svg, text},
};

use crate::presentation::{
    constants,
    styling::{Ornament, Styling},
    widget::frame,
};

/// Size of the name at the top of a detail panel or form.
pub const TITLE_SIZE: f32 = 28.0;

/// A diamond, drawn rather than typed because Federo has no ◆.
const DIAMOND: &[u8] = include_bytes!("../../../assets/diamond.svg");
/// A candle flame, which no font has.
const FLAME: &[u8] = include_bytes!("../../../assets/flame.svg");
/// Size of the ornament at the center of a [`header_rule`], and the
/// rule's height with or without one.
const RULE_ORNAMENT: f32 = 7.0;
/// How much of the accent color the lines of a [`header_rule`] keep.
const RULE_ALPHA: f32 = 0.45;
/// Thickness of the lines of a [`header_rule`].
const RULE_GIRTH: u32 = 1;
/// Size of a button's label.
const BUTTON_TEXT: f32 = 14.0;
/// Space around a button's label.
const BUTTON_PADDING: Padding = Padding {
    top: 7.0,
    bottom: 7.0,
    left: 16.0,
    right: 16.0,
};
/// Space around a [`small_button`]'s label.
const SMALL_BUTTON_PADDING: Padding = Padding {
    top: 3.0,
    bottom: 3.0,
    left: 10.0,
    right: 10.0,
};
/// How much of its color a disabled button keeps.
const DISABLED_ALPHA: f32 = 0.5;
/// Width of an outlined button's border.
const OUTLINE: f32 = 1.0;

/// `name` as a page title: large, in the heading face, cased as typed.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn title<'a>(styling: &Styling, name: impl text::IntoFragment<'a>) -> Text<'a> {
    text(name)
        .font(styling.heading)
        .size(styling.heading_size(TITLE_SIZE))
}

/// `label` as on tabs, captions and buttons: in the heading face at
/// `size` (scaled as the styling says), and in capitals if the styling
/// has them.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn label<'a>(styling: &Styling, label: &str, size: f32) -> Text<'a> {
    text(cased(styling, label))
        .font(styling.heading)
        .size(styling.heading_size(size))
}

/// `label` in capitals if the styling has them, otherwise as written.
fn cased(styling: &Styling, label: &str) -> String {
    if styling.caps {
        label.to_uppercase()
    } else {
        label.to_string()
    }
}

/// The styling's ornament in the accent color, `size` pixels across as
/// [scaled](Styling::ornament_size) by the styling, or `None` if it has
/// none.
pub fn ornament<'a, Message: 'a>(styling: &Styling, size: f32) -> Option<Element<'a, Message>> {
    let size = styling.ornament_size(size);
    styling
        .ornament
        .map(|ornament| mark(ornament_svg(ornament), size))
}

/// The SVG `ornament` is drawn from.
fn ornament_svg(ornament: Ornament) -> &'static [u8] {
    match ornament {
        Ornament::Diamond => DIAMOND,
        Ornament::Flame => FLAME,
    }
}

/// The SVG `image`, `size` pixels square, in the accent color.
#[cfg_attr(coverage_nightly, coverage(off))]
fn mark<'a, Message: 'a>(image: &'static [u8], size: f32) -> Element<'a, Message> {
    svg(svg::Handle::from_memory(image))
        .width(size)
        .height(size)
        .style(|theme: &Theme, _| svg::Style {
            color: Some(accent(theme)),
        })
        .into()
}

/// The faint accent line across the panel under a page's header, with the
/// styling's ornament at its center, ── ◆ ──, if it has one. It takes the
/// same height either way, so pages keep their layout.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn header_rule<'a, Message: 'a>(styling: &Styling) -> Element<'a, Message> {
    let line = || rule::horizontal(RULE_GIRTH).style(rule_line);
    let lines = match ornament(styling, RULE_ORNAMENT) {
        Some(mark) => row![line(), mark, line()],
        None => row![line()],
    };
    lines
        .spacing(8)
        .height(styling.ornament_size(RULE_ORNAMENT))
        .align_y(Center)
        .width(Fill)
        .into()
}

/// The main action on a page, such as Save, Edit or Serve: a solid block
/// of the accent color with its `label`. Give each page one.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn primary_button<'a, Message: 'a>(styling: &Styling, label: &str) -> Button<'a, Message> {
    labeled(styling, label, BUTTON_TEXT)
        .padding(BUTTON_PADDING)
        .style(primary(styling))
}

/// A lesser action, such as Cancel or "+ Add ingredient": its `label` in
/// an accent outline.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn secondary_button<'a, Message: 'a>(styling: &Styling, label: &str) -> Button<'a, Message> {
    labeled(styling, label, BUTTON_TEXT)
        .padding(BUTTON_PADDING)
        .style(secondary(styling))
}

/// A destructive action on the whole page, such as Delete: its `label` in
/// a [`quiet_danger`] outline.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn danger_button<'a, Message: 'a>(styling: &Styling, label: &str) -> Button<'a, Message> {
    labeled(styling, label, BUTTON_TEXT)
        .padding(BUTTON_PADDING)
        .style(quiet_danger(styling))
}

/// A compact button with a small label, for an action on one row of a
/// list, such as Remove. Unstyled; give it [`secondary`] or
/// [`quiet_danger`].
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn small_button<'a, Message: 'a>(styling: &Styling, label: &str) -> Button<'a, Message> {
    labeled(styling, label, constants::SMALL_TEXT).padding(SMALL_BUTTON_PADDING)
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn labeled<'a, Message: 'a>(styling: &Styling, text: &str, size: f32) -> Button<'a, Message> {
    button(label(styling, text, size))
}

/// A [`primary_button`]: filled with the accent color, a shade stronger
/// on hover, faded when disabled.
pub fn primary(styling: &Styling) -> impl Fn(&Theme, button::Status) -> button::Style + use<> {
    let radius = styling.radius;
    move |theme, status| {
        let shades = theme.extended_palette().primary;
        let fill = match status {
            button::Status::Hovered | button::Status::Pressed => shades.strong,
            button::Status::Active | button::Status::Disabled => shades.base,
        };
        let mut style = button::Style {
            background: Some(fill.color.into()),
            text_color: fill.text,
            border: Border::default().rounded(radius),
            ..button::Style::default()
        };
        if status == button::Status::Disabled {
            fade(&mut style);
        }
        style
    }
}

/// A [`secondary_button`], and the master list's "+ New" button: accent
/// text and outline on no background, filling with the frame's
/// [tint](frame::tint) on hover, faded when disabled.
pub fn secondary(styling: &Styling) -> impl Fn(&Theme, button::Status) -> button::Style + use<> {
    let radius = styling.radius;
    move |theme, status| {
        let background = matches!(status, button::Status::Hovered | button::Status::Pressed)
            .then(|| frame::tint(theme).into());
        let mut style = button::Style {
            background,
            text_color: accent(theme),
            border: Border {
                width: OUTLINE,
                radius: radius.into(),
                color: accent(theme),
            },
            ..button::Style::default()
        };
        if status == button::Status::Disabled {
            fade(&mut style);
        }
        style
    }
}

/// A button for a destructive action that is less loud than
/// [`button::danger`] until hovered: danger-colored text and outline on
/// no background, filling in on hover, faded when disabled.
pub fn quiet_danger(styling: &Styling) -> impl Fn(&Theme, button::Status) -> button::Style + use<> {
    let radius = styling.radius;
    move |theme, status| {
        let danger = theme.extended_palette().danger;
        let border = Border {
            width: OUTLINE,
            radius: radius.into(),
            color: danger.base.color,
        };
        let outlined = button::Style {
            background: None,
            text_color: danger.base.color,
            border,
            ..button::Style::default()
        };
        match status {
            button::Status::Active => outlined,
            button::Status::Disabled => {
                let mut style = outlined;
                fade(&mut style);
                style
            }
            button::Status::Hovered | button::Status::Pressed => button::Style {
                border,
                ..button::danger(theme, status)
            },
        }
    }
}

/// Fades `style` out, as a disabled button is.
fn fade(style: &mut button::Style) {
    style.background = style
        .background
        .map(|background| background.scale_alpha(DISABLED_ALPHA));
    style.text_color = style.text_color.scale_alpha(DISABLED_ALPHA);
    style.border.color = style.border.color.scale_alpha(DISABLED_ALPHA);
}

/// Text in the accent color, as captions are.
pub fn accented(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(accent(theme)),
    }
}

/// The theme's accent color.
pub fn accent(theme: &Theme) -> Color {
    theme.palette().primary
}

/// A line of a [`header_rule`]: a faint wash of the accent color.
fn rule_line(theme: &Theme) -> rule::Style {
    rule::Style {
        color: accent(theme).scale_alpha(RULE_ALPHA),
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::{
        styling::{PLAIN, Styling, every_styling},
        theme::Themes,
    };

    const DECO: Styling = Styling::ART_DECO;

    type ButtonStyle = dyn Fn(&Theme, button::Status) -> button::Style;

    #[test]
    fn ornaments_are_in_the_accent_color() {
        for theme in Themes::new().options() {
            let primary = theme.palette().primary;
            assert_eq!(accented(theme).color, Some(primary));
            assert_eq!(rule_line(theme).color, primary.scale_alpha(RULE_ALPHA));
        }
    }

    const ACTIVE: button::Status = button::Status::Active;
    const HOVERED: button::Status = button::Status::Hovered;
    const DISABLED: button::Status = button::Status::Disabled;

    #[test]
    fn buttons_have_the_stylings_corners() {
        for styling in every_styling() {
            let styles: [&ButtonStyle; 3] = [
                &primary(&styling),
                &secondary(&styling),
                &quiet_danger(&styling),
            ];
            for theme in Themes::new().options() {
                for style in styles {
                    for status in [ACTIVE, HOVERED, DISABLED] {
                        assert_eq!(
                            style(theme, status).border.radius,
                            styling.radius.into(),
                            "{theme}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_ornament_is_drawn_only_if_the_styling_has_one() {
        assert!(ornament::<()>(&DECO, 7.0).is_some());
        assert!(ornament::<()>(&PLAIN, 7.0).is_none());
    }

    #[test]
    fn labels_are_in_capitals_only_if_the_styling_has_them() {
        assert_eq!(cased(&DECO, "Add bottle"), "ADD BOTTLE");
        assert_eq!(cased(&PLAIN, "Add bottle"), "Add bottle");
    }

    #[test]
    fn primary_buttons_are_filled_with_the_accent_and_stronger_on_hover() {
        for theme in Themes::new().options() {
            let shades = theme.extended_palette().primary;
            let active = primary(&DECO)(theme, ACTIVE);
            assert_eq!(active.background, Some(shades.base.color.into()));
            assert_eq!(active.text_color, shades.base.text);
            assert_eq!(
                primary(&DECO)(theme, HOVERED).background,
                Some(shades.strong.color.into())
            );
            assert_eq!(
                primary(&DECO)(theme, DISABLED).text_color,
                shades.base.text.scale_alpha(DISABLED_ALPHA)
            );
        }
    }

    #[test]
    fn secondary_buttons_are_outlined_and_fill_with_the_tint_on_hover() {
        for theme in Themes::new().options() {
            let active = secondary(&DECO)(theme, ACTIVE);
            assert_eq!(active.background, None);
            assert_eq!(active.border.color, accent(theme));
            assert_eq!(active.text_color, accent(theme));
            assert_eq!(
                secondary(&DECO)(theme, HOVERED).background,
                Some(frame::tint(theme).into())
            );
            let disabled = secondary(&DECO)(theme, DISABLED);
            assert_eq!(
                disabled.text_color,
                accent(theme).scale_alpha(DISABLED_ALPHA)
            );
            assert_eq!(
                disabled.border.color,
                accent(theme).scale_alpha(DISABLED_ALPHA)
            );
        }
    }

    #[test]
    fn quiet_danger_is_outlined_until_hovered() {
        for theme in Themes::new().options() {
            let danger = theme.extended_palette().danger.base.color;

            let active = quiet_danger(&DECO)(theme, ACTIVE);
            assert_eq!(active.background, None, "{theme}");
            assert_eq!(active.text_color, danger);
            assert_eq!(active.border.color, danger);

            let hovered = quiet_danger(&DECO)(theme, HOVERED);
            let loud = button::danger(theme, HOVERED);
            assert_eq!(hovered.background, loud.background, "{theme}");
            assert_eq!(hovered.text_color, loud.text_color);

            let disabled = quiet_danger(&DECO)(theme, DISABLED);
            assert_eq!(disabled.background, None, "{theme}");
            assert_eq!(disabled.text_color, danger.scale_alpha(DISABLED_ALPHA));
            assert_eq!(disabled.border.color, danger.scale_alpha(DISABLED_ALPHA));
        }
    }

    #[test]
    fn every_ornament_is_drawn_from_its_own_svg() {
        let diamond = ornament_svg(Ornament::Diamond);
        let flame = ornament_svg(Ornament::Flame);
        assert!(diamond.starts_with(b"<svg"));
        assert!(flame.starts_with(b"<svg"));
        assert_ne!(diamond, flame);
    }
}
