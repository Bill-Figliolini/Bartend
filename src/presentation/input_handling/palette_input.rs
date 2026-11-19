//! [`PaletteInput`]: the form for making or editing a custom palette, a
//! name and a [`ColorField`] for each of the eight [`Role`]s; and the
//! [`Contrast`] checks that warn when text would be hard to read.

use std::fmt::Display;

use iced::{Element, widget::column};

use crate::{
    models::{CustomPalette, PaletteColors, PaletteNameError, Rgb, palette_name},
    presentation::{
        color::{MIN_CONTRAST, contrast_ratio},
        constants,
        input_handling::{FormError, Validator},
        styling::Styling,
        widget::{
            detail::labeled,
            input::{ColorField, ColorMessage, Error, StringField},
        },
    },
};

/// The part of the app a palette color paints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Background,
    Surface,
    Hairline,
    Accent,
    Text,
    Success,
    Warning,
    Danger,
}

impl Role {
    /// Every role, in the order the editor lists them.
    pub const ALL: [Role; 8] = [
        Role::Background,
        Role::Surface,
        Role::Hairline,
        Role::Accent,
        Role::Text,
        Role::Success,
        Role::Warning,
        Role::Danger,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Role::Background => "Background",
            Role::Surface => "Surface",
            Role::Hairline => "Hairline",
            Role::Accent => "Accent",
            Role::Text => "Text",
            Role::Success => "Success",
            Role::Warning => "Warning",
            Role::Danger => "Danger",
        }
    }

    /// The line under the name, saying where the color shows.
    pub fn hint(self) -> &'static str {
        match self {
            Role::Background => "The panels behind everything",
            Role::Surface => "Cards, tiles and the frame",
            Role::Hairline => "Lines around cards and the frame",
            Role::Accent => "Highlights, and the fill of primary buttons",
            Role::Text => "Words, on every surface",
            Role::Success => "Plenty in stock; ready to make",
            Role::Warning => "Running low",
            Role::Danger => "Nearly out, errors, and Delete",
        }
    }

    /// This role's color in `colors`.
    pub fn of(self, colors: &PaletteColors) -> Rgb {
        match self {
            Role::Background => colors.background,
            Role::Surface => colors.surface,
            Role::Hairline => colors.hairline,
            Role::Accent => colors.accent,
            Role::Text => colors.text,
            Role::Success => colors.success,
            Role::Warning => colors.warning,
            Role::Danger => colors.danger,
        }
    }

    /// Where this role sits in [`ALL`](Self::ALL).
    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&role| role == self)
            .expect("every role is in ALL")
    }
}

/// A pair of colors one is read against the other, checked for contrast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Contrast {
    TextOnBackground,
    TextOnSurface,
    /// Primary buttons are lettered in the background color on the accent.
    ButtonLettering,
}

impl Contrast {
    pub const ALL: [Contrast; 3] = [
        Contrast::TextOnBackground,
        Contrast::TextOnSurface,
        Contrast::ButtonLettering,
    ];

    /// The two colors, foreground first.
    fn pair(self, colors: &PaletteColors) -> (Rgb, Rgb) {
        match self {
            Contrast::TextOnBackground => (colors.text, colors.background),
            Contrast::TextOnSurface => (colors.text, colors.surface),
            Contrast::ButtonLettering => (colors.background, colors.accent),
        }
    }

    pub fn ratio(self, colors: &PaletteColors) -> f32 {
        let (front, back) = self.pair(colors);
        contrast_ratio(front, back)
    }
}

impl Display for Contrast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pair = match self {
            Contrast::TextOnBackground => "Text on the background",
            Contrast::TextOnSurface => "Text on cards",
            Contrast::ButtonLettering => "Lettering on accent buttons",
        };
        write!(f, "{pair}")
    }
}

/// The pairs in `colors` that fall below [`MIN_CONTRAST`], with their
/// ratios, in the order of [`Contrast::ALL`].
pub fn low_contrast(colors: &PaletteColors) -> Vec<(Contrast, f32)> {
    Contrast::ALL
        .into_iter()
        .map(|pair| (pair, pair.ratio(colors)))
        .filter(|&(_, ratio)| ratio < MIN_CONTRAST)
        .collect()
}

/// An edit to one field of a [`PaletteInput`].
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteMessage {
    Name(String),
    Color(Role, ColorMessage),
}

/// A field of a [`PaletteInput`], as named in its [`FormError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteField {
    Name,
    Color(Role),
}

impl Display for PaletteField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaletteField::Name => write!(f, "Name"),
            PaletteField::Color(role) => write!(f, "{}", role.name()),
        }
    }
}

/// The form for a custom palette: its name, and a [`ColorField`] per
/// [`Role`], at most one of them open at a time.
#[derive(Debug)]
pub struct PaletteInput {
    name: StringField,
    /// One per role, in the order of [`Role::ALL`].
    colors: [ColorField; 8],
    /// Every other palette's name, built-in and custom, which this one
    /// mustn't take (ignoring case).
    taken: Vec<String>,
}

impl PaletteInput {
    /// A form filled with `name` and `colors`, whose name must differ
    /// from every name in `taken`.
    pub fn new(name: &str, colors: &PaletteColors, taken: Vec<String>) -> Self {
        let mut field = StringField::new("");
        field.set(name.to_string());
        Self {
            name: field,
            colors: Role::ALL.map(|role| ColorField::new(role.of(colors))),
            taken,
        }
    }

    pub fn update(&mut self, message: PaletteMessage) {
        match message {
            PaletteMessage::Name(text) => self.name.set(text),
            PaletteMessage::Color(role, message) => {
                // Opening one field closes the others.
                if message == ColorMessage::Toggle && !self.field(role).is_open() {
                    self.colors.iter_mut().for_each(ColorField::close);
                }
                self.colors[role.index()].update(message);
            }
        }
    }

    fn field(&self, role: Role) -> &ColorField {
        &self.colors[role.index()]
    }

    /// The name, checked: not blank, and not another palette's.
    fn name_value(&self) -> Result<String, Error> {
        let name = self.name.value()?;
        let taken = self.taken.iter().map(String::as_str);
        palette_name(&name, taken).map_err(|error| match error {
            PaletteNameError::Empty => Error::StringEmpty,
            PaletteNameError::Taken(_) => Error::NameTaken,
        })
    }

    /// Checks every field. Reading it changes nothing.
    pub fn validate(&self) -> Result<CustomPalette, FormError<PaletteField>> {
        let mut validator = Validator::default();
        let name = validator.check(PaletteField::Name, self.name_value());
        let colors = Role::ALL
            .map(|role| validator.check(PaletteField::Color(role), self.field(role).value()));
        match (name, colors) {
            (
                Some(name),
                [
                    Some(background),
                    Some(surface),
                    Some(hairline),
                    Some(accent),
                    Some(text),
                    Some(success),
                    Some(warning),
                    Some(danger),
                ],
            ) => Ok(CustomPalette {
                name,
                colors: PaletteColors {
                    background,
                    surface,
                    hairline,
                    accent,
                    text,
                    success,
                    warning,
                    danger,
                },
            }),
            _ => Err(validator.into_error()),
        }
    }

    /// The colors as they stand, each field's last valid one: what the
    /// preview shows while a code is half typed.
    pub fn draft(&self) -> PaletteColors {
        let color = |role: Role| self.field(role).color();
        PaletteColors {
            background: color(Role::Background),
            surface: color(Role::Surface),
            hairline: color(Role::Hairline),
            accent: color(Role::Accent),
            text: color(Role::Text),
            success: color(Role::Success),
            warning: color(Role::Warning),
            danger: color(Role::Danger),
        }
    }

    /// The name field, then a row per role. With `show_errors`, each
    /// field's current error is shown under it.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self, styling: &Styling, show_errors: bool) -> Element<'_, PaletteMessage> {
        let name_error = if show_errors {
            self.name_value().err()
        } else {
            None
        };
        let name = labeled(
            styling,
            "Name",
            self.name
                .view_with_error(styling, PaletteMessage::Name, name_error),
        );
        let rows = Role::ALL.map(|role| {
            self.field(role).view(
                styling,
                role.name(),
                role.hint(),
                move |message| PaletteMessage::Color(role, message),
                show_errors,
            )
        });
        column![name, column(rows).spacing(constants::SPACING / 2)]
            .spacing(constants::SPACING)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::palette_sample as sample;

    fn colors() -> PaletteColors {
        sample("Harbor").colors
    }

    fn input(name: &str, taken: &[&str]) -> PaletteInput {
        PaletteInput::new(
            name,
            &colors(),
            taken.iter().map(ToString::to_string).collect(),
        )
    }

    fn open(input: &PaletteInput) -> Vec<Role> {
        Role::ALL
            .into_iter()
            .filter(|&role| input.field(role).is_open())
            .collect()
    }

    mod roles {
        use super::*;

        #[test]
        fn read_their_own_color() {
            let colors = colors();
            let read = Role::ALL.map(|role| role.of(&colors));
            assert_eq!(
                read,
                [
                    colors.background,
                    colors.surface,
                    colors.hairline,
                    colors.accent,
                    colors.text,
                    colors.success,
                    colors.warning,
                    colors.danger,
                ]
            );
        }

        #[test]
        fn have_names_and_hints() {
            let names = Role::ALL.map(Role::name);
            assert_eq!(
                names,
                [
                    "Background",
                    "Surface",
                    "Hairline",
                    "Accent",
                    "Text",
                    "Success",
                    "Warning",
                    "Danger"
                ]
            );
            assert!(Role::ALL.iter().all(|role| !role.hint().is_empty()));
            for (index, role) in Role::ALL.into_iter().enumerate() {
                assert_eq!(role.index(), index);
            }
        }
    }

    mod contrast {
        use super::*;

        const BLACK: Rgb = Rgb::new(0, 0, 0);
        const WHITE: Rgb = Rgb::new(255, 255, 255);
        const MID: Rgb = Rgb::new(0x77, 0x77, 0x77);

        #[test]
        fn a_readable_palette_has_no_warnings() {
            assert_eq!(low_contrast(&colors()), []);
        }

        #[test]
        fn names_each_pair_that_is_hard_to_read() {
            let colors = PaletteColors {
                text: MID,
                background: WHITE,
                surface: Rgb::new(0x66, 0x66, 0x66),
                accent: WHITE,
                ..colors()
            };
            let low = low_contrast(&colors);
            let pairs: Vec<Contrast> = low.iter().map(|&(pair, _)| pair).collect();
            assert_eq!(
                pairs,
                [
                    Contrast::TextOnBackground,
                    Contrast::TextOnSurface,
                    Contrast::ButtonLettering
                ]
            );
            assert!((low[0].1 - 4.48).abs() < 0.01, "{low:?}");
            assert!((low[2].1 - 1.0).abs() < 0.001, "{low:?}");
        }

        #[test]
        fn button_lettering_is_the_background_on_the_accent() {
            let colors = PaletteColors {
                background: BLACK,
                accent: WHITE,
                text: BLACK,
                surface: BLACK,
                ..colors()
            };
            let pairs: Vec<Contrast> = low_contrast(&colors)
                .into_iter()
                .map(|(pair, _)| pair)
                .collect();
            assert_eq!(pairs, [Contrast::TextOnBackground, Contrast::TextOnSurface]);
        }

        #[test]
        fn pairs_read_as_words() {
            let words = Contrast::ALL.map(|pair| pair.to_string());
            assert_eq!(
                words,
                [
                    "Text on the background",
                    "Text on cards",
                    "Lettering on accent buttons"
                ]
            );
        }
    }

    mod form {
        use super::*;

        #[test]
        fn starts_with_its_name_and_colors() {
            let input = input("Harbor copy", &["Harbor"]);
            assert_eq!(
                input.validate(),
                Ok(CustomPalette {
                    name: "Harbor copy".to_string(),
                    colors: colors(),
                })
            );
            assert_eq!(input.draft(), colors());
            assert_eq!(open(&input), []);
        }

        #[test]
        fn edits_reach_the_right_field() {
            let mut input = input("Harbor", &[]);
            input.update(PaletteMessage::Name("  Dusk ".to_string()));
            input.update(PaletteMessage::Color(
                Role::Accent,
                ColorMessage::Hex("#FF0000".to_string()),
            ));

            let palette = input.validate().unwrap();
            assert_eq!(palette.name, "Dusk");
            assert_eq!(palette.colors.accent, Rgb::new(255, 0, 0));
            assert_eq!(
                PaletteColors {
                    accent: colors().accent,
                    ..palette.colors
                },
                colors()
            );
        }

        #[test]
        fn opening_one_color_closes_the_others() {
            let mut input = input("Harbor", &[]);
            input.update(PaletteMessage::Color(Role::Accent, ColorMessage::Toggle));
            assert_eq!(open(&input), [Role::Accent]);

            input.update(PaletteMessage::Color(Role::Text, ColorMessage::Toggle));
            assert_eq!(open(&input), [Role::Text]);

            input.update(PaletteMessage::Color(Role::Text, ColorMessage::Toggle));
            assert_eq!(open(&input), []);
        }

        #[test]
        fn a_blank_name_is_reported() {
            let input = input("   ", &[]);
            assert_eq!(
                input.validate().unwrap_err().error_for(&PaletteField::Name),
                Some(Error::StringEmpty)
            );
        }

        #[test]
        fn a_taken_name_is_reported_ignoring_case() {
            let input = input("nord", &["Navy & Champagne", "Nord"]);
            assert_eq!(
                input.validate(),
                Err(FormError(vec![(PaletteField::Name, Error::NameTaken)]))
            );
        }

        #[test]
        fn every_bad_field_is_reported() {
            let mut input = input("", &[]);
            for role in [Role::Surface, Role::Danger] {
                input.update(PaletteMessage::Color(
                    role,
                    ColorMessage::Hex("#12".to_string()),
                ));
            }
            assert_eq!(
                input.validate(),
                Err(FormError(vec![
                    (PaletteField::Name, Error::StringEmpty),
                    (PaletteField::Color(Role::Surface), Error::ColorInvalid),
                    (PaletteField::Color(Role::Danger), Error::ColorInvalid),
                ]))
            );
        }

        #[test]
        fn the_draft_keeps_the_last_valid_colors_while_a_code_is_half_typed() {
            let mut input = input("Harbor", &[]);
            input.update(PaletteMessage::Color(
                Role::Text,
                ColorMessage::Hex("#FF".to_string()),
            ));
            assert_eq!(input.draft(), colors());
        }

        #[test]
        fn fields_display_their_names() {
            assert_eq!(PaletteField::Name.to_string(), "Name");
            assert_eq!(PaletteField::Color(Role::Hairline).to_string(), "Hairline");
        }
    }
}
