//! Maps the [`StylingChoice`] saved in the config to a [`Styling`]: the
//! shapes, type and ornaments the app is drawn with.
//!
//! A styling never names a colour. It only says which of the palette's
//! colours goes where, so every styling works with every
//! [theme](super::theme), and the two are picked separately.

// Phase 1 of splitting the look from the palette: the views start reading
// their styling in phase 2.
#![expect(dead_code, reason = "the views start reading their styling next")]

use std::fmt::{self, Display};

use iced::Font;

use crate::{
    models::{ConfigError, DEFAULT_STYLING_NAME, StylingChoice},
    presentation::widget::deco,
};

/// How the app is drawn, whatever its colours. Spacing is the same under
/// every styling, so screens keep their layout; only the decoration
/// changes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Styling {
    /// The face for titles, tabs, buttons, captions and field labels.
    pub heading: Font,
    /// Whether tabs, buttons, captions and field labels are in capitals.
    pub caps: bool,
    /// Corner radius of cards, chips, buttons, inputs, list entries, the
    /// unit switch and scrollbars.
    pub radius: f32,
    /// How cards and tiles are edged.
    pub card_frame: CardFrame,
    /// Whether stat tiles have brackets in their corners.
    pub tile_brackets: bool,
    /// The mark under page headers, beside the app's name, on the active
    /// tab and before the selected list entry, if any.
    pub ornament: Option<Ornament>,
    /// How the stock gauge is drawn.
    pub gauge: Gauge,
}

/// How cards and tiles are edged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardFrame {
    /// One line.
    Single,
    /// An outer line and a fainter inner one, a few pixels in.
    Double,
}

/// A mark used to pick things out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ornament {
    /// A small diamond, ◆.
    Diamond,
}

/// How the stock gauge is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gauge {
    /// A row of this many square segments, lit up to the share in stock.
    Segments(u8),
    /// One continuous bar.
    Bar,
}

impl Styling {
    /// The app's own styling, and the default: square corners,
    /// double-edged cards, bracketed tiles, diamonds, and headings in
    /// Federo capitals.
    pub const ART_DECO: Styling = Styling {
        heading: deco::HEADING,
        caps: true,
        radius: 0.0,
        card_frame: CardFrame::Double,
        tile_brackets: true,
        ornament: Some(Ornament::Diamond),
        gauge: Gauge::Segments(20),
    };
}

/// Used whenever the saved styling can't be found.
const FALLBACK: Styling = Styling::ART_DECO;

/// A styling offered in the picker, under the name it's saved by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StylingOption {
    pub name: &'static str,
    pub styling: Styling,
}

impl Display for StylingOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

/// Every styling the user can pick from, the app's own
/// [Art Deco](Styling::ART_DECO) first. User-defined stylings would be
/// added here as well.
#[derive(Debug, Clone)]
pub struct Stylings {
    options: Vec<StylingOption>,
}

impl Stylings {
    pub fn new() -> Self {
        Self {
            options: vec![StylingOption {
                name: DEFAULT_STYLING_NAME,
                styling: Styling::ART_DECO,
            }],
        }
    }

    /// The stylings offered in the picker.
    pub fn options(&self) -> &[StylingOption] {
        &self.options
    }

    /// The styling `choice` names.
    ///
    /// # Errors
    ///
    /// [`ConfigError::UnknownStyling`] if no styling has that name.
    pub fn lookup(&self, choice: &StylingChoice) -> Result<Styling, ConfigError> {
        let StylingChoice::BuiltIn(name) = choice;
        self.options
            .iter()
            .find(|option| option.name == name)
            .map(|option| option.styling)
            .ok_or_else(|| ConfigError::UnknownStyling(choice.clone()))
    }

    /// The styling `choice` names, or the fallback if there is none.
    pub fn resolve(&self, choice: &StylingChoice) -> Styling {
        self.lookup(choice).unwrap_or(FALLBACK)
    }
}

/// The config value that selects `option`.
pub fn choice_for(option: &StylingOption) -> StylingChoice {
    StylingChoice::BuiltIn(option.name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offers_art_deco_first() {
        let stylings = Stylings::new();
        let first = stylings.options().first().unwrap();
        assert_eq!(first.name, DEFAULT_STYLING_NAME);
        assert_eq!(first.styling, Styling::ART_DECO);
        assert_eq!(first.to_string(), DEFAULT_STYLING_NAME);
    }

    #[test]
    fn styling_names_are_unique() {
        let stylings = Stylings::new();
        let mut names: Vec<&str> = stylings.options().iter().map(|o| o.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), stylings.options().len());
    }

    #[test]
    fn art_deco_is_square_double_edged_and_ornamented() {
        let deco = Styling::ART_DECO;
        assert_eq!(deco.heading, deco::HEADING);
        assert!(deco.caps);
        assert!(deco.radius.abs() < f32::EPSILON);
        assert_eq!(deco.card_frame, CardFrame::Double);
        assert!(deco.tile_brackets);
        assert_eq!(deco.ornament, Some(Ornament::Diamond));
        assert_eq!(deco.gauge, Gauge::Segments(20));
    }

    #[test]
    fn every_styling_round_trips_through_its_choice() {
        let stylings = Stylings::new();
        for option in stylings.options() {
            assert_eq!(
                stylings.lookup(&choice_for(option)).ok(),
                Some(option.styling)
            );
        }
    }

    #[test]
    fn the_default_choice_is_the_fallback_styling() {
        let stylings = Stylings::new();
        assert_eq!(
            stylings.lookup(&StylingChoice::default()).ok(),
            Some(FALLBACK)
        );
    }

    #[test]
    fn an_unknown_styling_is_an_error_and_resolves_to_the_fallback() {
        let stylings = Stylings::new();
        let unknown = StylingChoice::BuiltIn("Not A Styling".to_string());

        assert!(matches!(
            stylings.lookup(&unknown),
            Err(ConfigError::UnknownStyling(choice)) if choice == unknown
        ));
        assert_eq!(stylings.resolve(&unknown), FALLBACK);
    }
}
