use std::fmt::Display;

use iced::Element;

use crate::{
    models::{CategoryBody, Config, UnitSystem},
    presentation::{
        input_handling::{FormError, Validator},
        widget::input::StringField,
    },
};

/// An edit to one field of a [`CategoryInput`].
#[derive(Debug, Clone)]
pub enum CategoryMessage {
    Name(String),
}

/// A field of a [`CategoryInput`], as named in its [`FormError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryField {
    Name,
}

impl Display for CategoryField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CategoryField::Name => write!(f, "Name"),
        }
    }
}

/// The form for adding a category or renaming one.
#[derive(Debug)]
pub struct CategoryInput {
    name_input: StringField,
    /// Whether fields show their errors; see the [module docs](super).
    show_errors: bool,
}

impl CategoryInput {
    pub fn new(_config: &Config) -> Self {
        Self {
            name_input: StringField::new("Name"),
            show_errors: false,
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self) -> Element<'_, CategoryMessage> {
        self.name_input
            .view(CategoryMessage::Name, self.show_errors)
    }

    pub fn update(&mut self, msg: CategoryMessage) {
        match msg {
            CategoryMessage::Name(text) => self.name_input.set(text),
        }
    }

    pub fn output(&mut self) -> Result<CategoryBody, FormError<CategoryField>> {
        self.show_errors = true;
        let mut validator = Validator::default();
        let name = validator.check(CategoryField::Name, self.name_input.value());
        match name {
            Some(name) => {
                self.clear();
                Ok(CategoryBody { name })
            }
            None => Err(validator.into_error()),
        }
    }
    pub fn clear(&mut self) {
        self.name_input.clear();
        self.show_errors = false;
    }
    pub fn begin_edit(&mut self, edit: &CategoryBody, _unit_system: UnitSystem) {
        self.clear();
        self.name_input.set(edit.name.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::{
        input_handling::{FormError, test_support::config},
        widget::input::Error,
    };

    fn set_name(input: &mut CategoryInput, name: &str) {
        input.update(CategoryMessage::Name(name.to_string()));
    }

    #[test]
    fn empty_name_reports_name_field() {
        let mut input = CategoryInput::new(&config());
        assert_eq!(
            input.output(),
            Err(FormError(vec![(CategoryField::Name, Error::StringEmpty)]))
        );
    }

    #[test]
    fn valid_name_is_returned_and_form_cleared() {
        let mut input = CategoryInput::new(&config());
        set_name(&mut input, "Gin");
        assert_eq!(
            input.output(),
            Ok(CategoryBody {
                name: "Gin".to_string()
            })
        );
        assert!(input.output().is_err(), "form should be empty after a save");
    }

    #[test]
    fn fields_display_their_names() {
        assert_eq!(CategoryField::Name.to_string(), "Name");
    }

    #[test]
    fn errors_are_shown_only_after_a_save_attempt() {
        let mut input = CategoryInput::new(&config());
        assert!(!input.show_errors);
        input.output().unwrap_err();
        assert!(input.show_errors);
        input.clear();
        assert!(!input.show_errors);
    }

    #[test]
    fn successful_save_hides_errors() {
        let mut input = CategoryInput::new(&config());
        input.output().unwrap_err();
        set_name(&mut input, "Gin");
        input.output().unwrap();
        assert!(!input.show_errors);
    }
}
