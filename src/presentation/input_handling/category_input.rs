use std::collections::HashSet;

use iced::widget::{Id, column, row, text};

use crate::{
    logic::CategoryService,
    models::{CategoryBody, CategoryID, Config, UnitSystem},
    presentation::{
        Viewable,
        application::Message,
        input_handling::InputMessage,
        widget::input::{Error, Input, InputContents, StringInput},
    },
};

#[derive(Debug)]
pub struct CategoryInput {
    name_input: StringInput<Message>,
    errors: HashSet<Error>,
}

impl CategoryInput {
    pub fn new(_config: &Config, msg: fn(InputMessage) -> Message) -> Self {
        Self {
            name_input: StringInput::new(
                move |id, str| msg(InputMessage::String(id, str)),
                "Name".to_string(),
                String::new(),
            ),
            errors: HashSet::new(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let input_row = self.name_input.view();
        let error_row = row(self.errors.iter().map(|e| text(e.to_string()).into()));
        column![input_row, error_row].into()
    }

    pub fn update(&mut self, msg: InputMessage) {
        match msg {
            InputMessage::String(id, new_text) if self.name_input.id() == &id => {
                self.name_input.update(new_text)
            }
            _ => unreachable!("Category has recieved invalid messsage {msg:?}"),
        }
    }

    pub fn output(&mut self) -> Result<CategoryBody, ()> {
        let name_result = self.name_input.get_output();
        if !self.name_input.has_error() {
            self.clear();
            Ok(CategoryBody {
                name: name_result.unwrap(),
            })
        } else {
            Err(())
        }
    }
    pub fn clear(&mut self) {
        self.name_input.clear();
    }
    pub fn begin_edit(&mut self, edit: &CategoryBody, _unit_system: UnitSystem) {
        self.clear();
        self.name_input.update(edit.name.clone());
    }
}

pub struct CategoryRelationInput {}
impl CategoryRelationInput {
    pub fn new<F: Fn(Id, InputMessage) -> Message + 'static>(
        id: &CategoryID,
        service: &CategoryService,
        message: F,
    ) -> Self {
        let choices: Vec<CategoryID> = service.valid_relations(id).iter().copied().collect();
        let already_selected = &service.child_categories(id);
        Self {}
    }
}
