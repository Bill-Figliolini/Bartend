//! Maps the [`StylingChoice`] saved in the config to a [`Styling`]: the
//! shapes, type and ornaments the app is drawn with.
//!
//! A styling never names a colour. It only says which of the palette's
//! colours goes where, so every styling works with every
//! [theme](super::theme), and the two are picked separately.

use std::fmt::{self, Display};

use iced::Font;

use crate::models::{ConfigError, DEFAULT_STYLING_NAME, StylingChoice};

/// Federo (SIL Open Font License; see `assets/fonts/Federo-OFL.txt`), the
/// [Art Deco](Styling::ART_DECO) heading face. Loaded into iced at
/// startup, with the other [fonts](Stylings::fonts).
pub const FEDERO_BYTES: &[u8] = include_bytes!("../../assets/fonts/Federo-Regular.ttf");
/// Federo, by the family name its file declares.
const FEDERO: Font = Font::with_name("Federo");

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
    /// What marks the corners of stat tiles.
    pub tile_corners: TileCorners,
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
    /// A thick outer line and a fainter thin one, a few pixels in.
    Heavy,
}

impl CardFrame {
    /// Thickness of the outer line.
    pub fn outline(self) -> f32 {
        match self {
            CardFrame::Single | CardFrame::Double => 1.0,
            CardFrame::Heavy => 2.0,
        }
    }
}

/// What marks the corners of stat tiles, drawn in the accent colour along
/// the card's inner line (where a double-edged card has one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileCorners {
    /// Nothing.
    None,
    /// An L-shaped bracket in each corner.
    Brackets,
    /// A small filled square in each corner, like a rivet.
    Studs,
}

/// A mark used to pick things out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ornament {
    /// A small diamond, ◆.
    Diamond,
    /// A candle flame: a teardrop, point up.
    Flame,
}

/// How the stock gauge is drawn. Its `girth` sets the height of the stock
/// tile, so a styling that changes it moves what sits below.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gauge {
    /// Thickness of the gauge.
    pub girth: f32,
    pub kind: GaugeKind,
}

/// Whether the stock gauge is split into segments or is one bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GaugeKind {
    /// A row of `count` square segments `gap` apart, lit up to the share
    /// in stock.
    Segments { count: u8, gap: f32 },
    /// One continuous bar, with the styling's corners.
    Bar,
}

impl Styling {
    /// The app's own styling, and the default: square corners,
    /// double-edged cards, bracketed tiles, diamonds, and headings in
    /// Federo capitals.
    pub const ART_DECO: Styling = Styling {
        heading: FEDERO,
        caps: true,
        radius: 0.0,
        card_frame: CardFrame::Double,
        tile_corners: TileCorners::Brackets,
        ornament: Some(Ornament::Diamond),
        gauge: Gauge {
            girth: 8.0,
            kind: GaugeKind::Segments {
                count: 20,
                gap: 2.0,
            },
        },
    };

    /// Clean and contemporary: rounded corners, single-edged cards, no
    /// brackets or ornaments, headings in the body face as typed, and a
    /// continuous gauge as thick as Art Deco's, so pages keep their layout.
    pub const MODERN: Styling = Styling {
        heading: Font::DEFAULT,
        caps: false,
        radius: 8.0,
        card_frame: CardFrame::Single,
        tile_corners: TileCorners::None,
        ornament: None,
        gauge: Gauge {
            girth: 8.0,
            kind: GaugeKind::Bar,
        },
    };
}

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

/// The app's own styling, and the one used whenever the saved styling
/// can't be found.
const FALLBACK: StylingOption = StylingOption {
    name: DEFAULT_STYLING_NAME,
    styling: Styling::ART_DECO,
};

/// Every styling the user can pick from, the app's own
/// [Art Deco](Styling::ART_DECO) first, then [Modern](Styling::MODERN),
/// and the font files their heading
/// faces come from. User-defined stylings would be added here as well.
#[derive(Debug, Clone)]
pub struct Stylings {
    options: Vec<StylingOption>,
    /// Kept apart from the options, so that comparing options (as the
    /// picker does) never compares font files.
    fonts: Vec<&'static [u8]>,
}

impl Stylings {
    pub fn new() -> Self {
        Self {
            options: vec![
                FALLBACK,
                StylingOption {
                    name: "Modern",
                    styling: Styling::MODERN,
                },
            ],
            // Modern's heading face is the body face, already loaded.
            fonts: vec![FEDERO_BYTES],
        }
    }

    /// The stylings offered in the picker.
    pub fn options(&self) -> &[StylingOption] {
        &self.options
    }

    /// The font files the stylings' heading faces come from, to load at
    /// startup.
    pub fn fonts(&self) -> &[&'static [u8]] {
        &self.fonts
    }

    /// The styling `choice` names.
    ///
    /// # Errors
    ///
    /// [`ConfigError::UnknownStyling`] if no styling has that name.
    pub fn lookup(&self, choice: &StylingChoice) -> Result<StylingOption, ConfigError> {
        let StylingChoice::BuiltIn(name) = choice;
        self.options
            .iter()
            .find(|option| option.name == name)
            .copied()
            .ok_or_else(|| ConfigError::UnknownStyling(choice.clone()))
    }

    /// The styling `choice` names, or the fallback if there is none.
    pub fn resolve(&self, choice: &StylingChoice) -> StylingOption {
        self.lookup(choice).unwrap_or(FALLBACK)
    }
}

/// The config value that selects `option`.
pub fn choice_for(option: &StylingOption) -> StylingChoice {
    StylingChoice::BuiltIn(option.name.to_string())
}

/// A styling unlike [Art Deco](Styling::ART_DECO) in every field, for
/// checking that the views follow the styling.
#[cfg(test)]
pub const PLAIN: Styling = Styling {
    heading: Font::DEFAULT,
    caps: false,
    radius: 6.0,
    card_frame: CardFrame::Single,
    tile_corners: TileCorners::None,
    ornament: None,
    gauge: Gauge {
        girth: 6.0,
        kind: GaugeKind::Bar,
    },
};

/// [`PLAIN`] as it would be offered in the picker.
#[cfg(test)]
pub const PLAIN_OPTION: StylingOption = StylingOption {
    name: "Plain",
    styling: PLAIN,
};

/// Every built-in styling and [`PLAIN`], for tests that check the views
/// follow whichever styling they're given.
#[cfg(test)]
pub fn every_styling() -> Vec<Styling> {
    Stylings::with_plain()
        .options()
        .iter()
        .map(|option| option.styling)
        .collect()
}

#[cfg(test)]
impl Stylings {
    /// The built-in stylings and [`PLAIN`], so tests can pick and save a
    /// styling other than the default.
    pub fn with_plain() -> Self {
        let mut stylings = Self::new();
        stylings.options.push(PLAIN_OPTION);
        stylings
    }
}

#[cfg(test)]
mod tests {
    use iced::font::Family;

    use super::*;

    #[test]
    fn federo_is_bundled_under_its_family_name() {
        // A TrueType file starts with version 1.0, and names its family
        // in its name table.
        assert_eq!(&FEDERO_BYTES[..4], &[0, 1, 0, 0]);
        let family = b"Federo";
        assert!(
            FEDERO_BYTES
                .windows(family.len())
                .any(|window| window == family)
        );
        assert_eq!(FEDERO, Font::with_name("Federo"));
    }

    #[test]
    fn offers_art_deco_first_then_modern() {
        let stylings = Stylings::new();
        let names: Vec<String> = stylings.options().iter().map(ToString::to_string).collect();
        assert_eq!(names, [DEFAULT_STYLING_NAME, "Modern"]);
        let first = stylings.options()[0];
        assert_eq!(first.styling, Styling::ART_DECO);
        assert_eq!(stylings.options()[1].styling, Styling::MODERN);
    }

    #[test]
    fn modern_is_rounded_plain_and_keeps_art_decos_layout() {
        let modern = Styling::MODERN;
        assert_eq!(modern.heading, Font::DEFAULT);
        assert!(!modern.caps);
        assert!((modern.radius - 8.0).abs() < f32::EPSILON);
        assert_eq!(modern.card_frame, CardFrame::Single);
        assert_eq!(modern.tile_corners, TileCorners::None);
        assert_eq!(modern.ornament, None);
        assert_eq!(modern.gauge.kind, GaugeKind::Bar);
        // Girth is the one field that moves layout; Modern matches.
        assert!((modern.gauge.girth - Styling::ART_DECO.gauge.girth).abs() < f32::EPSILON);
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
        assert_eq!(deco.heading, FEDERO);
        assert!(deco.caps);
        assert!(deco.radius.abs() < f32::EPSILON);
        assert_eq!(deco.card_frame, CardFrame::Double);
        assert_eq!(deco.tile_corners, TileCorners::Brackets);
        assert_eq!(deco.ornament, Some(Ornament::Diamond));
        assert_eq!(
            deco.gauge,
            Gauge {
                girth: 8.0,
                kind: GaugeKind::Segments {
                    count: 20,
                    gap: 2.0
                },
            }
        );
    }

    #[test]
    fn the_plain_test_styling_differs_from_art_deco_in_every_field() {
        // Destructured, so a new field fails to compile here until PLAIN
        // differs in it too.
        let Styling {
            heading,
            caps,
            radius,
            card_frame,
            tile_corners,
            ornament,
            gauge: Gauge { girth, kind },
        } = PLAIN;
        let deco = Styling::ART_DECO;
        assert_ne!(heading, deco.heading);
        assert_ne!(caps, deco.caps);
        assert!((radius - deco.radius).abs() > f32::EPSILON);
        assert_ne!(card_frame, deco.card_frame);
        assert_ne!(tile_corners, deco.tile_corners);
        assert_ne!(ornament, deco.ornament);
        assert!((girth - deco.gauge.girth).abs() > f32::EPSILON);
        assert_ne!(kind, deco.gauge.kind);
    }

    #[test]
    fn heavy_frames_have_a_thicker_outer_line() {
        assert!((CardFrame::Single.outline() - 1.0).abs() < f32::EPSILON);
        assert!((CardFrame::Double.outline() - 1.0).abs() < f32::EPSILON);
        assert!(CardFrame::Heavy.outline() > CardFrame::Double.outline());
    }

    #[test]
    fn every_styling_round_trips_through_its_choice() {
        let stylings = Stylings::new();
        for option in stylings.options() {
            assert_eq!(stylings.lookup(&choice_for(option)).ok(), Some(*option));
        }
    }

    #[test]
    fn every_heading_face_is_the_default_or_has_its_font_loaded() {
        let stylings = Stylings::new();
        for option in stylings.options() {
            match option.styling.heading.family {
                Family::Name(family) => assert!(
                    stylings
                        .fonts()
                        .iter()
                        .any(|font| font.windows(family.len()).any(|w| w == family.as_bytes())),
                    "{family} is not loaded for {option}"
                ),
                other => assert_eq!(other, Font::DEFAULT.family, "{option}"),
            }
        }
    }

    #[test]
    fn the_test_registry_adds_plain_after_the_built_in_stylings() {
        let stylings = Stylings::with_plain();
        assert_eq!(stylings.options().last(), Some(&PLAIN_OPTION));
        let plain = choice_for(&PLAIN_OPTION);
        assert_eq!(stylings.resolve(&plain), PLAIN_OPTION);
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
