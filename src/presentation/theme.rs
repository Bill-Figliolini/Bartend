//! Maps the [`ThemeChoice`] saved in the config to iced [`Theme`]s, and
//! defines the app's own Navy & Champagne theme.
//!
//! A theme is only a palette: the colours. The shapes, type and ornaments
//! the app is drawn with are its [styling](super::styling), picked
//! separately.

use iced::{
    Color, Theme, color,
    theme::{
        Palette,
        palette::{Extended, Pair},
    },
};

use crate::models::{ConfigError, DEFAULT_THEME_NAME, ThemeChoice};

/// The Navy & Champagne theme's colours.
mod navy_and_champagne {
    use super::{Color, color};

    /// Midnight navy: the panels behind everything.
    pub const BACKGROUND: Color = color!(0x0F, 0x16, 0x26);
    /// Slate navy: the frame, cards and tiles.
    pub const SURFACE: Color = color!(0x17, 0x20, 0x36);
    /// Steel: the lines edging cards and the frame.
    pub const HAIRLINE: Color = color!(0x2D, 0x3A, 0x57);
    /// Champagne: the accent.
    pub const ACCENT: Color = color!(0xD8, 0xC0, 0x8A);
    /// Off-white.
    pub const TEXT: Color = color!(0xF0, 0xEB, 0xDD);
    /// Seafoam.
    pub const SUCCESS: Color = color!(0x86, 0xB3, 0x9A);
    /// Amber.
    pub const WARNING: Color = color!(0xE2, 0xB6, 0x5C);
    /// Rose.
    pub const DANGER: Color = color!(0xC4, 0x6A, 0x6A);
}

/// Names the app's own theme was saved under before it was renamed, and the
/// name each now goes by. Saved configs keep the old name until Settings is
/// next saved.
const RENAMED: [(&str, &str); 1] = [("Art Deco", DEFAULT_THEME_NAME)];

/// The app's own theme, and the default: navy and champagne.
pub fn navy_and_champagne() -> Theme {
    use navy_and_champagne as colours;

    let palette = Palette {
        background: colours::BACKGROUND,
        text: colours::TEXT,
        primary: colours::ACCENT,
        success: colours::SUCCESS,
        warning: colours::WARNING,
        danger: colours::DANGER,
    };
    Theme::custom_with_fn(DEFAULT_THEME_NAME, palette, |palette| {
        let mut extended = Extended::generate(palette);
        // Pin the shades the frame and cards are drawn with, rather than
        // the ones iced would derive from the background.
        extended.background.weakest = Pair::new(colours::SURFACE, colours::TEXT);
        extended.background.weak = Pair::new(colours::HAIRLINE, colours::TEXT);
        // Navy lettering on champagne buttons.
        extended.primary.base.text = colours::BACKGROUND;
        extended.primary.strong.text = colours::BACKGROUND;
        extended
    })
}

/// Used whenever the saved theme can't be found.
fn fallback() -> Theme {
    navy_and_champagne()
}

/// Every theme the user can pick from: the app's own [`navy_and_champagne`] theme
/// first, then iced's built-in themes. User-defined themes would be added
/// here as well.
#[derive(Debug, Clone)]
pub struct Themes {
    options: Vec<Theme>,
}

impl Themes {
    pub fn new() -> Self {
        let mut options = vec![navy_and_champagne()];
        options.extend_from_slice(Theme::ALL);
        Self { options }
    }

    /// The themes offered in the picker.
    pub fn options(&self) -> &[Theme] {
        &self.options
    }

    /// The theme `choice` names, under its current name or a former one
    /// (see [`RENAMED`]).
    ///
    /// # Errors
    ///
    /// [`ConfigError::UnknownTheme`] if no theme has that name.
    pub fn lookup(&self, choice: &ThemeChoice) -> Result<Theme, ConfigError> {
        let ThemeChoice::BuiltIn(name) = choice;
        let name = RENAMED
            .iter()
            .find(|(old, _)| old == name)
            .map_or(name.as_str(), |&(_, new)| new);
        self.options
            .iter()
            .find(|theme| theme.to_string() == name)
            .cloned()
            .ok_or_else(|| ConfigError::UnknownTheme(choice.clone()))
    }

    /// The theme `choice` names, or the fallback if there is none.
    pub fn resolve(&self, choice: &ThemeChoice) -> Theme {
        self.lookup(choice).unwrap_or_else(|_| fallback())
    }
}

/// The config value that selects `theme`.
pub fn choice_for(theme: &Theme) -> ThemeChoice {
    ThemeChoice::BuiltIn(theme.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offers_navy_and_champagne_first_then_every_built_in_theme() {
        let themes = Themes::new();
        let (first, rest) = themes.options().split_first().unwrap();
        assert_eq!(*first, navy_and_champagne());
        assert_eq!(rest, Theme::ALL);
    }

    #[test]
    fn theme_names_are_unique() {
        let themes = Themes::new();
        let mut names: Vec<String> = themes.options().iter().map(ToString::to_string).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), themes.options().len());
    }

    #[test]
    fn navy_and_champagne_pins_its_shades() {
        use navy_and_champagne as colours;
        let theme = navy_and_champagne();
        let palette = theme.extended_palette();
        assert_eq!(theme.to_string(), DEFAULT_THEME_NAME);
        assert_eq!(theme.palette().primary, colours::ACCENT);
        assert_eq!(palette.background.base.color, colours::BACKGROUND);
        assert_eq!(palette.background.weakest.color, colours::SURFACE);
        assert_eq!(palette.background.weak.color, colours::HAIRLINE);
        assert_eq!(palette.primary.base.text, colours::BACKGROUND);
        assert_eq!(palette.primary.strong.text, colours::BACKGROUND);
        assert!(palette.is_dark);
    }

    #[test]
    fn every_theme_round_trips_through_its_choice() {
        let themes = Themes::new();
        for theme in themes.options() {
            assert_eq!(themes.lookup(&choice_for(theme)).ok().as_ref(), Some(theme));
        }
    }

    #[test]
    fn the_default_choice_is_the_fallback_theme() {
        let themes = Themes::new();
        assert_eq!(
            themes.lookup(&ThemeChoice::default()).ok(),
            Some(fallback())
        );
    }

    #[test]
    fn an_unknown_theme_is_an_error_and_resolves_to_the_fallback() {
        let themes = Themes::new();
        let unknown = ThemeChoice::BuiltIn("Not A Theme".to_string());

        assert!(matches!(
            themes.lookup(&unknown),
            Err(ConfigError::UnknownTheme(choice)) if choice == unknown
        ));
        assert_eq!(themes.resolve(&unknown), fallback());
    }

    #[test]
    fn a_theme_saved_under_a_former_name_is_found() {
        let themes = Themes::new();
        let old = ThemeChoice::BuiltIn("Art Deco".to_string());
        assert_eq!(themes.lookup(&old).ok(), Some(navy_and_champagne()));
    }

    #[test]
    fn former_names_lead_to_offered_themes_and_are_not_reused() {
        let themes = Themes::new();
        for (old, new) in RENAMED {
            let named = |name: &str| themes.options().iter().any(|t| t.to_string() == name);
            assert!(
                named(new),
                "{old} was renamed to {new}, which isn't offered"
            );
            assert!(!named(old), "{old} is a former name and a current one");
        }
    }
}
