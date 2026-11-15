//! Maps the [`ThemeChoice`] saved in the config to iced [`Theme`]s.

use iced::Theme;

use crate::models::{ConfigError, ThemeChoice};

/// Used whenever the saved theme can't be found.
const FALLBACK: Theme = Theme::KanagawaWave;

/// Every theme the user can pick from. Holds only iced's built-in themes for
/// now; user-defined themes would be added here as well.
#[derive(Debug, Clone)]
pub struct Themes {
    options: Vec<Theme>,
}

impl Themes {
    pub fn new() -> Self {
        Self {
            options: Theme::ALL.to_vec(),
        }
    }

    /// The themes offered in the picker.
    pub fn options(&self) -> &[Theme] {
        &self.options
    }

    /// The theme `choice` names.
    ///
    /// # Errors
    ///
    /// [`ConfigError::UnknownTheme`] if no theme has that name.
    pub fn lookup(&self, choice: &ThemeChoice) -> Result<Theme, ConfigError> {
        let ThemeChoice::BuiltIn(name) = choice;
        self.options
            .iter()
            .find(|theme| theme.to_string() == *name)
            .cloned()
            .ok_or_else(|| ConfigError::UnknownTheme(choice.clone()))
    }

    /// The theme `choice` names, or the fallback if there is none.
    pub fn resolve(&self, choice: &ThemeChoice) -> Theme {
        self.lookup(choice).unwrap_or(FALLBACK)
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
    fn offers_every_built_in_theme() {
        assert_eq!(Themes::new().options(), Theme::ALL);
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
        assert_eq!(themes.lookup(&ThemeChoice::default()).ok(), Some(FALLBACK));
    }

    #[test]
    fn an_unknown_theme_is_an_error_and_resolves_to_the_fallback() {
        let themes = Themes::new();
        let unknown = ThemeChoice::BuiltIn("Not A Theme".to_string());

        assert!(matches!(
            themes.lookup(&unknown),
            Err(ConfigError::UnknownTheme(choice)) if choice == unknown
        ));
        assert_eq!(themes.resolve(&unknown), FALLBACK);
    }
}
