//! Maps the [`ThemeChoice`] saved in the config to iced [`Theme`]s,
//! defines the app's own themes (Navy & Champagne and Violet &
//! Candlelight), and turns the user's custom palettes into themes.
//!
//! A theme is only a palette: the colors. The shapes, type and ornaments
//! the app is drawn with are its [styling](super::styling), picked
//! separately.

use std::borrow::Cow;

use iced::{
    Theme,
    theme::{
        Palette,
        palette::{Extended, Pair},
    },
};

use crate::{
    models::{ConfigError, CustomPalette, DEFAULT_THEME_NAME, PaletteColors, Rgb, ThemeChoice},
    presentation::color::{to_color, to_rgb},
};

/// Midnight navy and slate, steel lines, a champagne accent, off-white
/// text, seafoam, amber and rose.
const NAVY_AND_CHAMPAGNE: PaletteColors = PaletteColors {
    background: Rgb::new(0x0F, 0x16, 0x26),
    surface: Rgb::new(0x17, 0x20, 0x36),
    hairline: Rgb::new(0x2D, 0x3A, 0x57),
    accent: Rgb::new(0xD8, 0xC0, 0x8A),
    text: Rgb::new(0xF0, 0xEB, 0xDD),
    success: Rgb::new(0x86, 0xB3, 0x9A),
    warning: Rgb::new(0xE2, 0xB6, 0x5C),
    danger: Rgb::new(0xC4, 0x6A, 0x6A),
};

/// The name of the [`violet_and_candlelight`] theme.
const VIOLET_AND_CANDLELIGHT_NAME: &str = "Violet & Candlelight";

/// Deep aubergine and plum-black, dusk-violet lines, a candle-gold
/// accent, bone text, moss, burnt orange (kept clear of the gold) and
/// crimson.
const VIOLET_AND_CANDLELIGHT: PaletteColors = PaletteColors {
    background: Rgb::new(0x1A, 0x12, 0x20),
    surface: Rgb::new(0x24, 0x18, 0x29),
    hairline: Rgb::new(0x3D, 0x2A, 0x45),
    accent: Rgb::new(0xE8, 0xB8, 0x60),
    text: Rgb::new(0xED, 0xE3, 0xD1),
    success: Rgb::new(0x8F, 0xA8, 0x7A),
    warning: Rgb::new(0xD9, 0x89, 0x4A),
    danger: Rgb::new(0xB8, 0x47, 0x5A),
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

/// The theme a palette the user made is drawn with. Built the same way as
/// the app's own themes, so it looks right under every styling.
pub fn custom_theme(palette: &CustomPalette) -> Theme {
    pinned(palette.name.clone(), palette.colors)
}

/// A theme called `name` in `colors`, with the shades the frame, cards
/// and buttons are drawn with pinned rather than derived.
fn pinned(name: impl Into<Cow<'static, str>>, colors: PaletteColors) -> Theme {
    let palette = Palette {
        background: to_color(colors.background),
        text: to_color(colors.text),
        primary: to_color(colors.accent),
        success: to_color(colors.success),
        warning: to_color(colors.warning),
        danger: to_color(colors.danger),
    };
    Theme::custom_with_fn(name, palette, move |palette| {
        let mut extended = Extended::generate(palette);
        let (surface, hairline) = (to_color(colors.surface), to_color(colors.hairline));
        // Pin the shades the frame and cards are drawn with, rather than
        // the ones iced would derive from the background.
        extended.background.weakest = Pair::new(surface, palette.text);
        extended.background.weak = Pair::new(hairline, palette.text);
        // Lettering on accent buttons in the background color.
        extended.primary.base.text = palette.background;
        extended.primary.strong.text = palette.background;
        extended
    })
}

/// The eight colors `theme` is drawn with: what a new palette copied from
/// it starts as. Exact for the app's own themes and custom ones, whose
/// surface and hairline are pinned; for iced's themes those two are the
/// shades iced derives, which is what the app draws with under them.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "New in Settings starts from it, with the palette editor"
    )
)]
pub fn colors_of(theme: &Theme) -> PaletteColors {
    let palette = theme.palette();
    let extended = theme.extended_palette();
    PaletteColors {
        background: to_rgb(palette.background),
        surface: to_rgb(extended.background.weakest.color),
        hairline: to_rgb(extended.background.weak.color),
        accent: to_rgb(palette.primary),
        text: to_rgb(palette.text),
        success: to_rgb(palette.success),
        warning: to_rgb(palette.warning),
        danger: to_rgb(palette.danger),
    }
}

/// Used whenever the saved theme can't be found.
fn fallback() -> Theme {
    navy_and_champagne()
}

/// Every theme the user can pick from: the app's own themes first
/// ([`navy_and_champagne`], then [`violet_and_candlelight`]), then the
/// user's custom palettes (only once [read in](Themes::with_custom), as
/// Settings does), then iced's built-in themes.
#[derive(Debug, Clone)]
pub struct Themes {
    options: Vec<Theme>,
    /// The custom palettes among the options, so a pick can be saved with
    /// its colors (see [`choice_for`](Self::choice_for)).
    custom: Vec<CustomPalette>,
}

impl Themes {
    /// The built-in themes only. Enough to start the app: a saved custom
    /// palette carries its own colors (see [`ThemeChoice::Custom`]).
    pub fn new() -> Self {
        Self::with_custom(&[]).0
    }

    /// The built-in themes and `palettes`, with the names of any palettes
    /// left out because a built-in theme already has that name (ignoring
    /// case), which would make the two impossible to tell apart.
    pub fn with_custom(palettes: &[CustomPalette]) -> (Self, Vec<String>) {
        let own = [navy_and_champagne(), violet_and_candlelight()];
        let built_in: Vec<String> = own
            .iter()
            .chain(Theme::ALL)
            .map(|theme| theme.to_string().to_lowercase())
            .collect();
        let (custom, clashing): (Vec<CustomPalette>, Vec<CustomPalette>) = palettes
            .iter()
            .cloned()
            .partition(|palette| !built_in.contains(&palette.name.to_lowercase()));

        let mut options = own.to_vec();
        options.extend(custom.iter().map(custom_theme));
        options.extend_from_slice(Theme::ALL);
        let clashing = clashing.into_iter().map(|palette| palette.name).collect();
        (Self { options, custom }, clashing)
    }

    /// The themes offered in the picker.
    pub fn options(&self) -> &[Theme] {
        &self.options
    }

    /// The theme `choice` names: a built-in theme under its current name or
    /// a former one (see [`RENAMED`]), or a custom palette, drawn from the
    /// colors the choice carries.
    ///
    /// # Errors
    ///
    /// [`ConfigError::UnknownTheme`] if no built-in theme has that name.
    pub fn lookup(&self, choice: &ThemeChoice) -> Result<Theme, ConfigError> {
        let name = match choice {
            ThemeChoice::BuiltIn(name) => name,
            ThemeChoice::Custom(palette) => return Ok(custom_theme(palette)),
        };
        let name = RENAMED
            .iter()
            .find(|(old, _)| old == name)
            .map_or(name.as_str(), |&(_, new)| new);
        self.options
            .iter()
            .filter(|theme| !self.is_custom(theme))
            .find(|theme| theme.to_string() == name)
            .cloned()
            .ok_or_else(|| ConfigError::UnknownTheme(choice.clone()))
    }

    /// The theme `choice` names, or the fallback if there is none.
    pub fn resolve(&self, choice: &ThemeChoice) -> Theme {
        self.lookup(choice).unwrap_or_else(|_| fallback())
    }

    /// The config value that selects `theme`: a custom palette with its
    /// colors, or a built-in theme by name.
    pub fn choice_for(&self, theme: &Theme) -> ThemeChoice {
        let name = theme.to_string();
        match self.custom.iter().find(|palette| palette.name == name) {
            Some(palette) => ThemeChoice::Custom(palette.clone()),
            None => ThemeChoice::BuiltIn(name),
        }
    }

    /// Whether `theme` is one of the custom palettes.
    fn is_custom(&self, theme: &Theme) -> bool {
        let name = theme.to_string();
        self.custom.iter().any(|palette| palette.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::palette_sample as sample;

    /// `sample(name)` in `colors`.
    fn palette(name: &str, colors: PaletteColors) -> CustomPalette {
        CustomPalette {
            colors,
            ..sample(name)
        }
    }

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

    /// Checks that `theme` is called `name`, takes its colors from
    /// `colors`, and pins its frame, card and button shades.
    fn assert_pinned(theme: &Theme, name: &str, colors: PaletteColors) {
        let palette = theme.extended_palette();
        assert_eq!(theme.to_string(), name);
        assert_eq!(theme.palette().primary, to_color(colors.accent));
        assert_eq!(theme.palette().text, to_color(colors.text));
        assert_eq!(theme.palette().success, to_color(colors.success));
        assert_eq!(theme.palette().warning, to_color(colors.warning));
        assert_eq!(theme.palette().danger, to_color(colors.danger));
        assert_eq!(palette.background.base.color, to_color(colors.background));
        assert_eq!(palette.background.weakest.color, to_color(colors.surface));
        assert_eq!(palette.background.weak.color, to_color(colors.hairline));
        assert_eq!(palette.primary.base.text, to_color(colors.background));
        assert_eq!(palette.primary.strong.text, to_color(colors.background));
        assert!(palette.is_dark);
    }

    #[test]
    fn a_custom_theme_is_named_after_its_palette_and_pins_its_shades() {
        let harbor = palette("Harbor", VIOLET_AND_CANDLELIGHT);
        assert_pinned(&custom_theme(&harbor), "Harbor", VIOLET_AND_CANDLELIGHT);
    }

    #[test]
    fn custom_palettes_come_after_the_apps_own_themes_and_before_iced() {
        let palettes = [sample("Harbor"), sample("Dusk")];
        let (themes, clashing) = Themes::with_custom(&palettes);

        let (own, rest) = themes.options().split_at(2);
        let (custom, iced) = rest.split_at(2);
        assert_eq!(own, [navy_and_champagne(), violet_and_candlelight()]);
        assert_eq!(custom, palettes.map(|palette| custom_theme(&palette)));
        assert_eq!(iced, Theme::ALL);
        assert_eq!(clashing, Vec::<String>::new());
    }

    #[test]
    fn custom_palettes_named_like_a_built_in_theme_are_left_out() {
        let (themes, clashing) =
            Themes::with_custom(&[sample("nord"), sample("Harbor"), sample("NAVY & CHAMPAGNE")]);

        assert_eq!(clashing, ["nord", "NAVY & CHAMPAGNE"]);
        assert_eq!(themes.options().len(), 2 + 1 + Theme::ALL.len());
        assert_eq!(themes.options()[2], custom_theme(&sample("Harbor")));
    }

    #[test]
    fn a_custom_choice_is_drawn_from_its_own_colors() {
        // Not among the options: the choice carries what it needs.
        let harbor = palette("Harbor", VIOLET_AND_CANDLELIGHT);
        let choice = ThemeChoice::Custom(harbor.clone());
        assert_eq!(Themes::new().lookup(&choice), Ok(custom_theme(&harbor)));
    }

    #[test]
    fn a_built_in_choice_never_finds_a_custom_palette() {
        let (themes, _) = Themes::with_custom(&[sample("Harbor")]);
        let choice = ThemeChoice::BuiltIn("Harbor".to_string());
        assert_eq!(
            themes.lookup(&choice),
            Err(ConfigError::UnknownTheme(choice))
        );
    }

    #[test]
    fn a_custom_palette_is_chosen_with_its_colors() {
        let harbor = palette("Harbor", VIOLET_AND_CANDLELIGHT);
        let (themes, _) = Themes::with_custom(std::slice::from_ref(&harbor));
        assert_eq!(
            themes.choice_for(&custom_theme(&harbor)),
            ThemeChoice::Custom(harbor)
        );
        assert_eq!(
            themes.choice_for(&Theme::Nord),
            ThemeChoice::BuiltIn("Nord".to_string())
        );
    }

    #[test]
    fn colors_of_gives_back_exactly_what_a_pinned_theme_was_made_from() {
        assert_eq!(colors_of(&navy_and_champagne()), NAVY_AND_CHAMPAGNE);
        assert_eq!(colors_of(&violet_and_candlelight()), VIOLET_AND_CANDLELIGHT);
        let harbor = palette("Harbor", VIOLET_AND_CANDLELIGHT);
        assert_eq!(colors_of(&custom_theme(&harbor)), VIOLET_AND_CANDLELIGHT);
    }

    #[test]
    fn colors_of_an_iced_theme_are_the_shades_it_is_drawn_with() {
        use crate::presentation::color::to_rgb;
        for theme in Theme::ALL {
            let colors = colors_of(theme);
            let extended = theme.extended_palette();
            // iced derives its shades, so they're rounded to the nearest byte.
            let expected = [
                extended.background.base.color,
                extended.background.weakest.color,
                extended.background.weak.color,
                theme.palette().primary,
                theme.palette().text,
            ]
            .map(to_rgb);
            let actual = [
                colors.background,
                colors.surface,
                colors.hairline,
                colors.accent,
                colors.text,
            ];
            assert_eq!(actual, expected, "{theme}");
        }
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
        let (themes, _) = Themes::with_custom(&[sample("Harbor"), sample("Dusk")]);
        for theme in themes.options() {
            assert_eq!(
                themes.lookup(&themes.choice_for(theme)).ok().as_ref(),
                Some(theme)
            );
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
