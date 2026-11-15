//! The window's frame around the panels: the top bar and the master list
//! sit on a raised surface, edged from the detail panel by hairlines, and
//! mark what is chosen in them with a faint accent tint.
//! Every colour comes from the theme's palette, like the panels' own.

use iced::{
    Border, Color, Element, Theme,
    widget::{button, container, rule},
};

/// Thickness of the lines between the frame and the detail panel.
const HAIRLINE: u32 = 1;
/// How much of the accent colour the tint behind a chosen entry keeps.
const TINT_ALPHA: f32 = 0.16;
/// How much of the text colour an unchosen tab keeps.
const QUIET_ALPHA: f32 = 0.7;
/// Corner radius of the frame's buttons.
pub(super) const RADIUS: f32 = 6.0;

/// The raised surface the top bar and master list sit on: the shade cards
/// are filled with.
pub fn surface(theme: &Theme) -> container::Style {
    let pair = theme.extended_palette().background.weakest;
    container::Style {
        background: Some(pair.color.into()),
        text_color: Some(pair.text),
        ..container::Style::default()
    }
}

/// A line across the window, under the top bar.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn hairline_across<'a, Message: 'a>() -> Element<'a, Message> {
    rule::horizontal(HAIRLINE).style(hairline).into()
}

/// A line down the window, between the master list and the detail panel.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn hairline_down<'a, Message: 'a>() -> Element<'a, Message> {
    rule::vertical(HAIRLINE).style(hairline).into()
}

/// A [`hairline_across`] or [`hairline_down`]: in the colour of cards'
/// borders.
fn hairline(theme: &Theme) -> rule::Style {
    rule::Style {
        color: theme.extended_palette().background.weak.color,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

/// The faint wash of the accent colour behind a chosen entry, tab segment
/// or hovered outline button.
pub fn tint(theme: &Theme) -> Color {
    theme.palette().primary.scale_alpha(TINT_ALPHA)
}

/// The colour of the bar beside a chosen entry and under the active tab.
pub fn accent(theme: &Theme) -> Color {
    theme.palette().primary
}

/// A row of the master list, or a segment of the unit switch: plain text
/// until hovered, and on the [`tint`] with full-strength text when
/// `chosen`. A `dimmed` row that isn't chosen has faded text.
pub fn entry(
    theme: &Theme,
    status: button::Status,
    chosen: bool,
    dimmed: Option<f32>,
) -> button::Style {
    let palette = theme.extended_palette();
    let text = palette.background.weakest.text;
    let background = if chosen {
        Some(tint(theme).into())
    } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        Some(palette.background.weak.color.into())
    } else {
        None
    };
    let text_color = match dimmed {
        Some(alpha) if !chosen => text.scale_alpha(alpha),
        _ => text,
    };
    button::Style {
        background,
        text_color,
        border: Border {
            radius: RADIUS.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

/// A tab in the top bar: plain, slightly faded text, full strength when
/// hovered, and in the accent colour when `active`. The active tab is also
/// underlined, separately.
pub fn tab(theme: &Theme, status: button::Status, active: bool) -> button::Style {
    let text = theme.extended_palette().background.weakest.text;
    let text_color = if active {
        accent(theme)
    } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        text
    } else {
        text.scale_alpha(QUIET_ALPHA)
    };
    button::Style {
        background: None,
        text_color,
        ..button::Style::default()
    }
}

/// A button for adding something, such as "+ New Item": accent-coloured
/// text and outline on no background, filling with the [`tint`] on hover.
pub fn outlined(theme: &Theme, status: button::Status) -> button::Style {
    let background = matches!(status, button::Status::Hovered | button::Status::Pressed)
        .then(|| tint(theme).into());
    button::Style {
        background,
        text_color: accent(theme),
        border: Border {
            width: 1.0,
            radius: RADIUS.into(),
            color: accent(theme),
        },
        ..button::Style::default()
    }
}

/// The bar beside a chosen entry or under the active tab: the accent
/// colour, or nothing so that others keep the same layout.
pub fn marker(theme: &Theme, shown: bool) -> container::Style {
    container::Style {
        background: shown.then(|| accent(theme).into()),
        border: Border {
            radius: f32::MAX.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// The outline around the unit switch's segments.
pub fn switch_outline(theme: &Theme) -> container::Style {
    container::Style {
        border: Border {
            width: 1.0,
            radius: RADIUS.into(),
            color: theme.extended_palette().background.weak.color,
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTIVE: button::Status = button::Status::Active;
    const HOVERED: button::Status = button::Status::Hovered;

    #[test]
    fn the_frame_is_on_the_card_shade_and_edged_like_cards() {
        for theme in Theme::ALL {
            let palette = theme.extended_palette();
            assert_eq!(
                surface(theme).background,
                Some(palette.background.weakest.color.into())
            );
            assert_eq!(hairline(theme).color, palette.background.weak.color);
            assert_eq!(
                switch_outline(theme).border.color,
                palette.background.weak.color
            );
        }
    }

    #[test]
    fn chosen_entries_are_tinted_and_never_dimmed() {
        for theme in Theme::ALL {
            let text = theme.extended_palette().background.weakest.text;
            let chosen = entry(theme, ACTIVE, true, Some(0.5));
            assert_eq!(chosen.background, Some(tint(theme).into()));
            assert_eq!(chosen.text_color, text);

            let dimmed = entry(theme, ACTIVE, false, Some(0.5));
            assert_eq!(dimmed.background, None);
            assert_eq!(dimmed.text_color, text.scale_alpha(0.5));

            let hovered = entry(theme, HOVERED, false, None);
            assert!(hovered.background.is_some());
            assert_eq!(hovered.text_color, text);
        }
    }

    #[test]
    fn the_active_tab_is_in_the_accent_colour_and_others_are_quieter() {
        for theme in Theme::ALL {
            let text = theme.extended_palette().background.weakest.text;
            assert_eq!(tab(theme, ACTIVE, true).text_color, accent(theme));
            assert_eq!(
                tab(theme, ACTIVE, false).text_color,
                text.scale_alpha(QUIET_ALPHA)
            );
            assert_eq!(tab(theme, HOVERED, false).text_color, text);
            assert_eq!(tab(theme, ACTIVE, true).background, None);
        }
    }

    #[test]
    fn outlined_buttons_fill_with_the_tint_on_hover() {
        for theme in Theme::ALL {
            let active = outlined(theme, ACTIVE);
            assert_eq!(active.background, None);
            assert_eq!(active.border.color, accent(theme));
            assert_eq!(active.text_color, accent(theme));
            assert_eq!(
                outlined(theme, HOVERED).background,
                Some(tint(theme).into())
            );
        }
    }

    #[test]
    fn markers_show_only_when_asked() {
        for theme in Theme::ALL {
            assert_eq!(marker(theme, true).background, Some(accent(theme).into()));
            assert_eq!(marker(theme, false).background, None);
        }
    }
}
