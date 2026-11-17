//! The bar across the top of the window: the app's name in the corner above
//! the master list, then one tab per screen, and at the far end the switch
//! between metric and imperial amounts.

use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Theme,
    widget::{Stack, button, container, row, space, text},
};

use crate::{
    models::UnitSystem,
    presentation::{
        application::Message,
        constants,
        screen::ScreenKind,
        styling::Styling,
        widget::{frame, styled},
    },
};

/// Height of the bar.
const HEIGHT: f32 = 48.0;
/// Size of the app's name in the corner.
const BRAND_SIZE: f32 = 22.0;
/// Size of the ornaments either side of the app's name.
const BRAND_ORNAMENT: f32 = 8.0;
/// Size of a tab's label.
const TAB_SIZE: f32 = 15.0;
/// Size of the ornament at the centre of the active tab's underline, and
/// the underline's height with or without one.
const TAB_ORNAMENT: f32 = 7.0;

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn tab_bar<'a>(
    styling: &Styling,
    active: ScreenKind,
    units: UnitSystem,
) -> Element<'a, Message> {
    let name = styled::label(styling, "Bartend")
        .size(BRAND_SIZE)
        .style(styled::accented);
    // The name between two of the styling's ornaments, if it has one.
    let brand = row![
        styled::ornament(styling, BRAND_ORNAMENT),
        name,
        styled::ornament(styling, BRAND_ORNAMENT)
    ]
    .spacing(10)
    .align_y(Center);
    let brand = container(brand)
        .width(constants::MASTER_WIDTH)
        .center_x(constants::MASTER_WIDTH)
        .center_y(Fill);

    let tabs = row(ScreenKind::ALL
        .into_iter()
        .map(|kind| tab(styling, kind, kind == active)))
    .spacing(4);

    let switch = container(unit_switch(styling, units))
        .padding([0, constants::PADDING])
        .center_y(Fill);

    let bar = row![brand, tabs, space().width(Fill), switch].height(HEIGHT);
    container(bar).width(Fill).style(frame::surface).into()
}

/// One screen's tab: its name in capitals, and while it is the `active`
/// one, underlined in the accent colour with the styling's ornament, if
/// any, at the centre.
#[cfg_attr(coverage_nightly, coverage(off))]
fn tab<'a>(styling: &Styling, kind: ScreenKind, active: bool) -> Element<'a, Message> {
    let name = styled::label(styling, kind.label()).size(TAB_SIZE);
    let label = button(container(name).center_y(Fill))
        .height(Fill)
        .padding([0, 16])
        .style(move |theme: &Theme, status| frame::tab(theme, status, active))
        .on_press(Message::OpenScreen(kind));
    let mut layers = Stack::new().push(label);
    if active {
        let underline = match styled::ornament(styling, TAB_ORNAMENT) {
            Some(mark) => row![frame::underline(), mark, frame::underline()],
            None => row![frame::underline()],
        }
        .spacing(4)
        .height(TAB_ORNAMENT)
        .align_y(Center);
        // Laid over the tab, so that it takes the tab's width.
        let underline = container(underline)
            .padding([0, 10])
            .width(Fill)
            .align_bottom(Fill);
        layers = layers.push(underline);
    }
    layers.into()
}

/// Two segments, one per unit system, with the one in use tinted. Picking
/// the other switches every screen to it.
#[cfg_attr(coverage_nightly, coverage(off))]
fn unit_switch<'a>(styling: &Styling, units: UnitSystem) -> Element<'a, Message> {
    let segment = |system: UnitSystem| {
        let chosen = system == units;
        button(text(short_label(system)).size(14))
            .padding([4, 12])
            .style(frame::entry(styling, chosen, None))
            .on_press_maybe((!chosen).then_some(Message::SetUnits(system)))
    };
    let segments = row![segment(UnitSystem::Metric), segment(UnitSystem::Imperial)].spacing(2);
    container(segments)
        .padding(2)
        .style(frame::switch_outline(styling))
        .into()
}

/// The unit a system measures drinks in, as named on its segment.
fn short_label(system: UnitSystem) -> &'static str {
    match system {
        UnitSystem::Metric => "ml",
        UnitSystem::Imperial => "fl oz",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_are_named_by_their_volume_unit() {
        assert_eq!(short_label(UnitSystem::Metric), "ml");
        assert_eq!(short_label(UnitSystem::Imperial), "fl oz");
    }
}
