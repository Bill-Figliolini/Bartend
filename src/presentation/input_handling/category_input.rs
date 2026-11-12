use crate::{
    models::{CategoryBody, Config, UnitSystem},
    presentation::{
        Viewable,
        application::Message,
        input_handling::{Field, FormError, InputMessage, UnknownTarget, Validator},
        widget::input::{Input, InputContents, StringInput},
    },
};

#[derive(Debug)]
pub struct CategoryInput {
    name_input: StringInput<Message>,
}

impl CategoryInput {
    pub fn new(_config: &Config, msg: fn(InputMessage) -> Message) -> Self {
        Self {
            name_input: StringInput::new(
                move |id, str| msg(InputMessage::String(id, str)),
                "Name".to_string(),
                String::new(),
            ),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self) -> iced::Element<'_, Message> {
        self.name_input.view()
    }

    pub fn update(&mut self, msg: InputMessage) -> Result<(), UnknownTarget> {
        match msg {
            InputMessage::String(id, new_text) if self.name_input.id() == &id => {
                self.name_input.update(new_text);
                Ok(())
            }
            msg => Err(UnknownTarget(msg)),
        }
    }

    pub fn output(&mut self) -> Result<CategoryBody, FormError> {
        let mut validator = Validator::default();
        let name = validator.check(Field::Name, self.name_input.get_output());
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
    }
    pub fn begin_edit(&mut self, edit: &CategoryBody, _unit_system: UnitSystem) {
        self.clear();
        self.name_input.update(edit.name.clone());
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
    fn message_for_unknown_field_is_rejected() {
        let mut input = CategoryInput::new(&config(), msg);
        let stale = InputMessage::String(Id::unique(), "Gin".to_string());
        assert!(input.update(stale).is_err());
    }
}
