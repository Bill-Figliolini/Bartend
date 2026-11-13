use std::fmt::Display;

use iced::Element;

use crate::{
    models::{CategoryBody, UnitSystem},
    presentation::{
        input_handling::{Form, FormError, Validator},
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
}

impl CategoryInput {
    pub fn new() -> Self {
        Self {
            name_input: StringField::new("Name"),
        }
    }
}

impl Form for CategoryInput {
    type Message = CategoryMessage;
    type Field = CategoryField;
    type Output = CategoryBody;
    type Value = CategoryBody;

    fn update(&mut self, msg: CategoryMessage) {
        match msg {
            CategoryMessage::Name(text) => self.name_input.set(text),
        }
    }

    fn validate(&self) -> Result<CategoryBody, FormError<CategoryField>> {
        let mut validator = Validator::default();
        let name = validator.check(CategoryField::Name, self.name_input.value());
        match name {
            Some(name) => Ok(CategoryBody { name }),
            None => Err(validator.into_error()),
        }
    }

    fn clear(&mut self) {
        self.name_input.clear();
    }

    fn fill(&mut self, value: &CategoryBody, _units: UnitSystem) {
        self.name_input.set(value.name.clone());
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn view(&self, show_errors: bool) -> Element<'_, CategoryMessage> {
        self.name_input.view(CategoryMessage::Name, show_errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::{input_handling::FormError, widget::input::Error};

    fn set_name(input: &mut CategoryInput, name: &str) {
        input.update(CategoryMessage::Name(name.to_string()));
    }

    #[test]
    fn empty_name_reports_name_field() {
        let input = CategoryInput::new();
        assert_eq!(
            input.validate(),
            Err(FormError(vec![(CategoryField::Name, Error::StringEmpty)]))
        );
    }

    #[test]
    fn valid_name_is_returned() {
        let mut input = CategoryInput::new();
        set_name(&mut input, "Gin");
        let gin = Ok(CategoryBody {
            name: "Gin".to_string(),
        });
        assert_eq!(input.validate(), gin);
        assert_eq!(input.validate(), gin, "validating changes nothing");
    }

    #[test]
    fn clear_empties_the_name() {
        let mut input = CategoryInput::new();
        set_name(&mut input, "Gin");
        input.clear();
        input.validate().unwrap_err();
    }

    #[test]
    fn editing_fills_the_name() {
        let mut input = CategoryInput::new();
        let gin = CategoryBody {
            name: "Gin".to_string(),
        };
        input.fill(&gin, UnitSystem::Metric);
        assert_eq!(input.validate(), Ok(gin));
    }

    #[test]
    fn fields_display_their_names() {
        assert_eq!(CategoryField::Name.to_string(), "Name");
    }
}
