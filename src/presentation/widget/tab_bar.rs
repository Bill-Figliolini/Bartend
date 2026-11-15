//! The bar across the top of the window: the app's name in the corner above
//! the master list, then one tab per screen, and at the far end the switch
//! between metric and imperial amounts.

use iced::{
    Element,
    Length::Fill,
    Theme,
    widget::{button, container, row, space, stack, text},
};

use crate::{
    models::UnitSystem,
    presentation::{application::Message, constants, screen::ScreenKind, widget::frame},
};

/// Height of the bar.
const HEIGHT: f32 = 48.0;
/// Thickness of the line under the active tab.
const UNDERLINE: f32 = 3.0;

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn tab_bar<'a>(active: ScreenKind, units: UnitSystem) -> Element<'a, Message> {
    let brand = container(text("Bartend 🍸").size(22))
        .width(constants::MASTER_WIDTH)
        .padding([0, constants::PADDING])
        .center_y(Fill);

    let tabs = row(ScreenKind::ALL
        .into_iter()
        .map(|kind| tab(kind, kind == active)))
    .spacing(4);

    let switch = container(unit_switch(units))
        .padding([0, constants::PADDING])
        .center_y(Fill);

    let bar = row![brand, tabs, space().width(Fill), switch].height(HEIGHT);
    container(bar).width(Fill).style(frame::surface).into()
}

/// One screen's tab: its name, underlined in the accent colour while it is
/// the `active` one.
#[cfg_attr(coverage_nightly, coverage(off))]
fn tab<'a>(kind: ScreenKind, active: bool) -> Element<'a, Message> {
    let label = button(container(text(kind.label())).center_y(Fill))
        .height(Fill)
        .padding([0, 16])
        .style(move |theme: &Theme, status| frame::tab(theme, status, active))
        .on_press(Message::OpenScreen(kind));
    let underline = container(space())
        .width(Fill)
        .height(UNDERLINE)
        .style(move |theme: &Theme| frame::marker(theme, active));
    // Laid over the tab, so that it takes the tab's width.
    let underline = container(underline)
        .padding([0, 10])
        .width(Fill)
        .align_bottom(Fill);
    stack![label, underline].into()
}

/// Two segments, one per unit system, with the one in use tinted. Picking
/// the other switches every screen to it.
#[cfg_attr(coverage_nightly, coverage(off))]
fn unit_switch<'a>(units: UnitSystem) -> Element<'a, Message> {
    let segment = |system: UnitSystem| {
        let chosen = system == units;
        button(text(short_label(system)).size(14))
            .padding([4, 12])
            .style(move |theme: &Theme, status| frame::entry(theme, status, chosen, None))
            .on_press_maybe((!chosen).then_some(Message::SetUnits(system)))
    };
    let segments = row![segment(UnitSystem::Metric), segment(UnitSystem::Imperial)].spacing(2);
    container(segments)
        .padding(2)
        .style(frame::switch_outline)
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
