//! Styles for iced's stock controls: text boxes (also the text box of a
//! combo box), pick lists, the menus they open, scrollbars and sliders. Each
//! starts from iced's own style and gives it the [`Styling`]'s corners,
//! edged in the shade of cards' borders, and picked out in the accent
//! color when hovered, focused or open.
//! Every color comes from the theme's palette, so they suit whichever
//! theme is chosen.

use iced::{
    Border, Color, Element, Theme,
    overlay::menu,
    widget::{
        Scrollable, pick_list, scrollable,
        scrollable::{Direction, Scrollbar},
        slider as iced_slider, text_input,
    },
};

use super::styled::accent;
use crate::presentation::styling::Styling;

/// How much of the accent color edges a hovered control.
const HOVER_ALPHA: f32 = 0.6;
/// How much of the accent color a scrollbar's handle keeps when it is
/// not hovered or dragged.
const SCROLLER_ALPHA: f32 = 0.75;
/// Width of a scrollbar, and of its handle.
const SCROLLBAR_WIDTH: f32 = 4.0;
/// Gap between a scrollbar and what it scrolls.
const SCROLLBAR_GAP: f32 = 8.0;
/// Thickness of a slider's rail.
const SLIDER_RAIL: f32 = 4.0;
/// Width of a slider's handle, a bar as tall as the slider.
pub const SLIDER_HANDLE: u16 = 8;
/// The most a slider handle's corners round, so a styling with wide
/// corners still gives a bar rather than a pill.
const SLIDER_HANDLE_ROUNDING: f32 = 3.0;
/// Width of the accent edge around a slider's handle.
const SLIDER_HANDLE_EDGE: f32 = 2.0;

/// `content` in a vertical [`Scrollable`] with a thin [`scrollbar`] that
/// sits beside it rather than over it.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn scrolling<'a, Message: 'a>(
    styling: &Styling,
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    let bar = Scrollbar::new()
        .width(SCROLLBAR_WIDTH)
        .scroller_width(SCROLLBAR_WIDTH)
        .spacing(SCROLLBAR_GAP);
    scrollable(content)
        .direction(Direction::Vertical(bar))
        .style(scrollbar(styling))
}

/// A 1px border in `color` with corners of `radius`.
fn edge(color: Color, radius: f32) -> Border {
    Border {
        width: 1.0,
        radius: radius.into(),
        color,
    }
}

/// The color that edges a control: the shade of cards' borders at rest,
/// a wash of the accent when `hovered`, and the accent when `engaged`
/// (focused or open).
fn edge_color(theme: &Theme, hovered: bool, engaged: bool) -> Color {
    if engaged {
        accent(theme)
    } else if hovered {
        accent(theme).scale_alpha(HOVER_ALPHA)
    } else {
        theme.extended_palette().background.weak.color
    }
}

/// A text box: iced's, with the styling's corners, and edged as in
/// [`edge_color`].
pub fn text_field(
    styling: &Styling,
) -> impl Fn(&Theme, text_input::Status) -> text_input::Style + use<> {
    let radius = styling.radius;
    move |theme, status| {
        let (hovered, engaged) = match status {
            text_input::Status::Active | text_input::Status::Disabled => (false, false),
            text_input::Status::Hovered => (true, false),
            text_input::Status::Focused { .. } => (false, true),
        };
        text_input::Style {
            border: edge(edge_color(theme, hovered, engaged), radius),
            ..text_input::default(theme, status)
        }
    }
}

/// A pick list: on the panel's background like a [`text_field`], with an
/// accent arrow, and edged as in [`edge_color`].
pub fn picker(styling: &Styling) -> impl Fn(&Theme, pick_list::Status) -> pick_list::Style + use<> {
    let radius = styling.radius;
    move |theme, status| {
        let (hovered, engaged) = match status {
            pick_list::Status::Active => (false, false),
            pick_list::Status::Hovered => (true, false),
            pick_list::Status::Opened { .. } => (false, true),
        };
        let base = theme.extended_palette().background.base;
        pick_list::Style {
            background: base.color.into(),
            text_color: base.text,
            handle_color: accent(theme),
            border: edge(edge_color(theme, hovered, engaged), radius),
            ..pick_list::default(theme, status)
        }
    }
}

/// The menu a pick list or combo box opens: on the cards' shade, with the
/// styling's corners, edged in the accent, with the hovered option filled
/// with the accent as in iced's.
pub fn menu(styling: &Styling) -> impl Fn(&Theme) -> menu::Style + use<> {
    let radius = styling.radius;
    move |theme| {
        let surface = theme.extended_palette().background.weakest;
        menu::Style {
            background: surface.color.into(),
            text_color: surface.text,
            border: edge(accent(theme), radius),
            ..menu::default(theme)
        }
    }
}

/// A scrollbar: with the styling's corners, on a track in the shade of
/// cards' borders, with a faint accent handle that turns full accent while
/// hovered or dragged.
pub fn scrollbar(
    styling: &Styling,
) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style + use<> {
    let radius = styling.radius;
    move |theme, status| scrollbar_style(theme, status, radius)
}

fn scrollbar_style(theme: &Theme, status: scrollable::Status, radius: f32) -> scrollable::Style {
    let (horizontal, vertical) = match status {
        scrollable::Status::Active { .. } => (false, false),
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => (
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
        ),
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => (
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
        ),
    };
    scrollable::Style {
        vertical_rail: rail(theme, vertical, radius),
        horizontal_rail: rail(theme, horizontal, radius),
        ..scrollable::default(theme, status)
    }
}

/// One of a [`scrollbar`]'s rails, with its handle `hot` while hovered or
/// dragged, and corners of `radius`.
fn rail(theme: &Theme, hot: bool, radius: f32) -> scrollable::Rail {
    let handle = if hot {
        accent(theme)
    } else {
        accent(theme).scale_alpha(SCROLLER_ALPHA)
    };
    scrollable::Rail {
        background: Some(theme.extended_palette().background.weak.color.into()),
        border: Border::default().rounded(radius),
        scroller: scrollable::Scroller {
            background: handle.into(),
            border: Border::default().rounded(radius),
        },
    }
}

/// A slider: a rail in the accent up to the handle and in the shade of
/// cards' borders after it, and a bar of a handle on the panel's
/// background, edged in the accent and filled with it while hovered or
/// dragged. Its corners follow the styling, held to its narrow shape.
pub fn slider(
    styling: &Styling,
) -> impl Fn(&Theme, iced_slider::Status) -> iced_slider::Style + use<> {
    let radius = styling.radius;
    move |theme, status| {
        let hot = !matches!(status, iced_slider::Status::Active);
        let palette = theme.extended_palette();
        let fill = if hot {
            accent(theme)
        } else {
            palette.background.base.color
        };
        iced_slider::Style {
            rail: iced_slider::Rail {
                backgrounds: (accent(theme).into(), palette.background.weak.color.into()),
                width: SLIDER_RAIL,
                border: Border::default().rounded(radius.min(SLIDER_RAIL / 2.0)),
            },
            handle: iced_slider::Handle {
                shape: iced_slider::HandleShape::Rectangle {
                    width: SLIDER_HANDLE,
                    border_radius: radius.min(SLIDER_HANDLE_ROUNDING).into(),
                },
                background: fill.into(),
                border_width: SLIDER_HANDLE_EDGE,
                border_color: accent(theme),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::{
        styling::{Styling, every_styling},
        theme::Themes,
    };

    const DECO: Styling = Styling::ART_DECO;

    #[test]
    fn controls_have_the_stylings_corners() {
        for styling in every_styling() {
            let radius = styling.radius.into();
            for theme in Themes::new().options() {
                for status in TEXT_STATES {
                    assert_eq!(text_field(&styling)(theme, status).border.radius, radius);
                }
                let list = picker(&styling)(theme, pick_list::Status::Active);
                assert_eq!(list.border.radius, radius);
                assert_eq!(menu(&styling)(theme).border.radius, radius);
                let bar = scrollbar(&styling)(theme, IDLE);
                assert_eq!(bar.vertical_rail.border.radius, radius);
                assert_eq!(bar.vertical_rail.scroller.border.radius, radius);
            }
        }
    }

    const IDLE: scrollable::Status = scrollable::Status::Active {
        is_horizontal_scrollbar_disabled: false,
        is_vertical_scrollbar_disabled: false,
    };

    const TEXT_STATES: [text_input::Status; 4] = [
        text_input::Status::Active,
        text_input::Status::Hovered,
        text_input::Status::Focused { is_hovered: false },
        text_input::Status::Disabled,
    ];

    #[test]
    fn text_fields_light_up_when_focused() {
        for theme in Themes::new().options() {
            let steel = theme.extended_palette().background.weak.color;
            let field = |status| text_field(&DECO)(theme, status).border.color;
            assert_eq!(field(text_input::Status::Active), steel, "{theme}");
            assert_eq!(
                field(text_input::Status::Hovered),
                accent(theme).scale_alpha(HOVER_ALPHA)
            );
            assert_eq!(
                field(text_input::Status::Focused { is_hovered: true }),
                accent(theme)
            );
            assert_eq!(field(text_input::Status::Disabled), steel);
        }
    }

    #[test]
    fn pick_lists_match_text_fields() {
        for theme in Themes::new().options() {
            let list = |status| picker(&DECO)(theme, status);
            let active = list(pick_list::Status::Active);
            assert_eq!(
                active.background,
                theme.extended_palette().background.base.color.into()
            );
            assert_eq!(active.handle_color, accent(theme));
            assert_eq!(
                active.border,
                text_field(&DECO)(theme, text_input::Status::Active).border
            );
            assert_eq!(
                list(pick_list::Status::Hovered).border.color,
                accent(theme).scale_alpha(HOVER_ALPHA)
            );
            assert_eq!(
                list(pick_list::Status::Opened { is_hovered: false })
                    .border
                    .color,
                accent(theme)
            );
        }
    }

    #[test]
    fn menus_are_on_the_card_shade() {
        for theme in Themes::new().options() {
            let style = menu(&DECO)(theme);
            assert_eq!(
                style.background,
                theme.extended_palette().background.weakest.color.into()
            );
            assert_eq!(style.border, edge(accent(theme), DECO.radius));
        }
    }

    #[test]
    fn scrollbar_handles_light_up_while_used() {
        for theme in Themes::new().options() {
            let idle = scrollbar(&DECO)(theme, IDLE);
            let faint = accent(theme).scale_alpha(SCROLLER_ALPHA).into();
            assert_eq!(idle.vertical_rail.scroller.background, faint);

            let dragged = scrollbar(&DECO)(
                theme,
                scrollable::Status::Dragged {
                    is_horizontal_scrollbar_dragged: false,
                    is_vertical_scrollbar_dragged: true,
                    is_horizontal_scrollbar_disabled: false,
                    is_vertical_scrollbar_disabled: false,
                },
            );
            assert_eq!(
                dragged.vertical_rail.scroller.background,
                accent(theme).into()
            );
            assert_eq!(dragged.horizontal_rail.scroller.background, faint);

            let hovered = scrollbar(&DECO)(
                theme,
                scrollable::Status::Hovered {
                    is_horizontal_scrollbar_hovered: true,
                    is_vertical_scrollbar_hovered: false,
                    is_horizontal_scrollbar_disabled: false,
                    is_vertical_scrollbar_disabled: false,
                },
            );
            assert_eq!(
                hovered.horizontal_rail.scroller.background,
                accent(theme).into()
            );
            assert_eq!(hovered.vertical_rail.scroller.background, faint);
        }
    }

    const SLIDER_STATES: [iced_slider::Status; 3] = [
        iced_slider::Status::Active,
        iced_slider::Status::Hovered,
        iced_slider::Status::Dragged,
    ];

    #[test]
    fn slider_corners_follow_the_styling_but_stay_narrow() {
        for styling in every_styling() {
            let style = slider(&styling)(&Theme::Dark, iced_slider::Status::Active);
            assert_eq!(
                style.rail.border.radius,
                styling.radius.min(SLIDER_RAIL / 2.0).into()
            );
            assert_eq!(
                style.handle.shape,
                iced_slider::HandleShape::Rectangle {
                    width: SLIDER_HANDLE,
                    border_radius: styling.radius.min(SLIDER_HANDLE_ROUNDING).into(),
                }
            );
        }
    }

    #[test]
    fn slider_rails_run_from_the_accent_to_the_card_border_shade() {
        for theme in Themes::new().options() {
            for status in SLIDER_STATES {
                let style = slider(&DECO)(theme, status);
                let steel = theme.extended_palette().background.weak.color;
                assert_eq!(style.rail.backgrounds, (accent(theme).into(), steel.into()));
                assert_eq!(style.handle.border_color, accent(theme));
            }
        }
    }

    #[test]
    fn slider_handles_fill_with_the_accent_while_used() {
        for theme in Themes::new().options() {
            let fill = |status| slider(&DECO)(theme, status).handle.background;
            let base = theme.extended_palette().background.base.color;
            assert_eq!(fill(iced_slider::Status::Active), base.into());
            assert_eq!(fill(iced_slider::Status::Hovered), accent(theme).into());
            assert_eq!(fill(iced_slider::Status::Dragged), accent(theme).into());
        }
    }
}
