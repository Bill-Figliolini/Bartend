//! Maps the [`ThemeChoice`] saved in the config to iced [`Theme`]s, and
//! defines the app's own themes: Navy & Champagne and Violet &
//! Candlelight.
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

/// The colours of one of the app's own themes.
#[derive(Debug, Clone, Copy)]
struct Colours {
    /// The panels behind everything.
    background: Color,
    /// The frame, cards and tiles.
    surface: Color,
    /// The lines edging cards and the frame.
    hairline: Color,
    /// The accent, and the fill of primary buttons.
    accent: Color,
    text: Color,
    success: Color,
    warning: Color,
    danger: Color,
}

/// Midnight navy and slate, steel lines, a champagne accent, off-white
/// text, seafoam, amber and rose.
const NAVY_AND_CHAMPAGNE: Colours = Colours {
    background: color!(0x0F, 0x16, 0x26),
    surface: color!(0x17, 0x20, 0x36),
    hairline: color!(0x2D, 0x3A, 0x57),
    accent: color!(0xD8, 0xC0, 0x8A),
    text: color!(0xF0, 0xEB, 0xDD),
    success: color!(0x86, 0xB3, 0x9A),
    warning: color!(0xE2, 0xB6, 0x5C),
    danger: color!(0xC4, 0x6A, 0x6A),
};

/// The name of the [`violet_and_candlelight`] theme.
const VIOLET_AND_CANDLELIGHT_NAME: &str = "Violet & Candlelight";

/// Deep aubergine and plum-black, dusk-violet lines, a candle-gold
/// accent, bone text, moss, burnt orange (kept clear of the gold) and
/// crimson.
const VIOLET_AND_CANDLELIGHT: Colours = Colours {
    background: color!(0x1A, 0x12, 0x20),
    surface: color!(0x24, 0x18, 0x29),
    hairline: color!(0x3D, 0x2A, 0x45),
    accent: color!(0xE8, 0xB8, 0x60),
    text: color!(0xED, 0xE3, 0xD1),
    success: color!(0x8F, 0xA8, 0x7A),
    warning: color!(0xD9, 0x89, 0x4A),
    danger: color!(0xB8, 0x47, 0x5A),
};

/// Names the app's own theme was saved under before it was renamed, and the
/// name each now goes by. Saved configs keep the old name until Settings is
/// next saved.
const RENAMED: [(&str, &str); 1] = [("Art Deco", DEFAULT_THEME_NAME)];

/// The app's own theme, and the default: champagne on midnight navy.
pub fn navy_and_champagne() -> Theme {
    pinned(DEFAULT_THEME_NAME, NAVY_AND_CHAMPAGNE)
}

/// Candle gold on deep aubergine, made to go with the Gothic styling.
pub fn violet_and_candlelight() -> Theme {
    pinned(VIOLET_AND_CANDLELIGHT_NAME, VIOLET_AND_CANDLELIGHT)
}

/// A theme called `name` in `colours`, with the shades the frame, cards
/// and buttons are drawn with pinned rather than derived.
fn pinned(name: &'static str, colours: Colours) -> Theme {
    let palette = Palette {
        background: colours.background,
        text: colours.text,
        primary: colours.accent,
        success: colours.success,
        warning: colours.warning,
        danger: colours.danger,
    };
    Theme::custom_with_fn(name, palette, move |palette| {
        let mut extended = Extended::generate(palette);
        // Pin the shades the frame and cards are drawn with, rather than
        // the ones iced would derive from the background.
        extended.background.weakest = Pair::new(colours.surface, colours.text);
        extended.background.weak = Pair::new(colours.hairline, colours.text);
        // Lettering on accent buttons in the background colour.
        extended.primary.base.text = colours.background;
        extended.primary.strong.text = colours.background;
        extended
    })
}

/// Used whenever the saved theme can't be found.
fn fallback() -> Theme {
    navy_and_champagne()
}

/// Every theme the user can pick from: the app's own themes first
/// ([`navy_and_champagne`], then [`violet_and_candlelight`]), then iced's
/// built-in themes. User-defined themes would be added here as well.
#[derive(Debug, Clone)]
pub struct Themes {
    options: Vec<Theme>,
}

impl Themes {
    pub fn new() -> Self {
        let mut options = vec![navy_and_champagne(), violet_and_candlelight()];
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
    fn offers_the_apps_own_themes_first_then_every_built_in_theme() {
        let themes = Themes::new();
        let (own, rest) = themes.options().split_at(2);
        assert_eq!(own, [navy_and_champagne(), violet_and_candlelight()]);
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

    /// Checks that `theme` is called `name`, takes its colours from
    /// `colours`, and pins its frame, card and button shades.
    fn assert_pinned(theme: &Theme, name: &str, colours: Colours) {
        let palette = theme.extended_palette();
        assert_eq!(theme.to_string(), name);
        assert_eq!(theme.palette().primary, colours.accent);
        assert_eq!(theme.palette().text, colours.text);
        assert_eq!(theme.palette().success, colours.success);
        assert_eq!(theme.palette().warning, colours.warning);
        assert_eq!(theme.palette().danger, colours.danger);
        assert_eq!(palette.background.base.color, colours.background);
        assert_eq!(palette.background.weakest.color, colours.surface);
        assert_eq!(palette.background.weak.color, colours.hairline);
        assert_eq!(palette.primary.base.text, colours.background);
        assert_eq!(palette.primary.strong.text, colours.background);
        assert!(palette.is_dark);
    }

    #[test]
    fn navy_and_champagne_pins_its_shades() {
        assert_pinned(
            &navy_and_champagne(),
            DEFAULT_THEME_NAME,
            NAVY_AND_CHAMPAGNE,
        );
    }

    #[test]
    fn violet_and_candlelight_pins_its_shades() {
        assert_pinned(
            &violet_and_candlelight(),
            VIOLET_AND_CANDLELIGHT_NAME,
            VIOLET_AND_CANDLELIGHT,
        );
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
