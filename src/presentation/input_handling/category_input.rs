use crate::{
    models::{CategoryBody, Config, UnitSystem},
    presentation::{
        application::Message,
        input_handling::{Field, FormError, InputMessage, UnknownTarget, Validator, route},
        widget::input::StringField,
    },
};

#[derive(Debug)]
pub struct CategoryInput {
    name_input: StringField,
    msg: fn(InputMessage) -> Message,
    /// Set once a save has been attempted, so errors are not shown on a form
    /// the user has not finished yet.
    show_errors: bool,
}

impl CategoryInput {
    pub fn new(_config: &Config, msg: fn(InputMessage) -> Message) -> Self {
        Self {
            name_input: StringField::new("Name"),
            msg,
            show_errors: false,
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self) -> iced::Element<'_, Message> {
        self.name_input.view(
            route(self.msg, self.name_input.id(), InputMessage::String),
            self.show_errors,
        )
    }

    pub fn update(&mut self, msg: InputMessage) -> Result<(), UnknownTarget> {
        match msg {
            InputMessage::String(id, new_text) if self.name_input.id() == &id => {
                self.name_input.set(new_text);
                Ok(())
            }
            msg => Err(UnknownTarget(msg)),
        }
    }

    pub fn output(&mut self) -> Result<CategoryBody, FormError> {
        self.show_errors = true;
        let mut validator = Validator::default();
        let name = validator.check(Field::Name, self.name_input.value());
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
    #[cfg(test)]
    pub fn name_id(&self) -> iced::widget::Id {
        self.name_input.id().clone()
    }
}

#[cfg(test)]
mod tests {
    use iced::widget::Id;

    use super::*;
    use crate::presentation::{
        input_handling::{
            FormError,
            test_support::{config, msg},
        },
        widget::input::Error,
    };

    fn set_name(input: &mut CategoryInput, name: &str) {
        let id = input.name_input.id().clone();
        input
            .update(InputMessage::String(id, name.to_string()))
            .unwrap();
    }

    #[test]
    fn empty_name_reports_name_field() {
        let mut input = CategoryInput::new(&config(), msg);
        assert_eq!(
            input.output(),
            Err(FormError(vec![(Field::Name, Error::StringEmpty)]))
        );
    }

    #[test]
    fn valid_name_is_returned_and_form_cleared() {
        let mut input = CategoryInput::new(&config(), msg);
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
    fn errors_are_shown_only_after_a_save_attempt() {
        let mut input = CategoryInput::new(&config(), msg);
        assert!(!input.show_errors);
        input.output().unwrap_err();
        assert!(input.show_errors);
        input.clear();
        assert!(!input.show_errors);
    }

    #[test]
    fn successful_save_hides_errors() {
        let mut input = CategoryInput::new(&config(), msg);
        input.output().unwrap_err();
        set_name(&mut input, "Gin");
        input.output().unwrap();
        assert!(!input.show_errors);
    }

    #[test]
    fn message_for_unknown_field_is_rejected() {
        let mut input = CategoryInput::new(&config(), msg);
        let stale = InputMessage::String(Id::unique(), "Gin".to_string());
        assert!(input.update(stale).is_err());
    }
}
