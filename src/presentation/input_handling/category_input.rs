use std::collections::HashSet;

use iced::widget::{column, row};

use crate::{
    logic::{category::CategoryBody, config::Config},
    presentation::{
        Viewable,
        application::Message,
        input_handling::{InputCollection, InputMessage},
        widget::input::{Error, Input, InputContents, text_input::string_input::StringInput},
    },
};

#[derive(Debug)]
pub struct CategoryInput {
    name_input: StringInput<Message>,
    errors: HashSet<Error>,
}

impl Viewable<Message> for CategoryInput {
    fn view(&self) -> iced::Element<'_, Message> {
        let input_row = self.name_input.view();
        column![input_row].into()
    }
}

impl InputCollection<CategoryBody> for CategoryInput {
    fn new(_config: &Config, msg: fn(InputMessage) -> Message) -> Self {
        Self {
            name_input: StringInput::new(
                move |id, str| msg(InputMessage::String(id, str)),
                "name".to_string(),
                String::new(),
            ),
            errors: HashSet::new(),
        }
    }
    fn update(&mut self, msg: InputMessage) {
        match msg {
            InputMessage::String(id, new_text) if self.name_input.id() == &id => {
                self.name_input.update(new_text)
            }
            _ => unreachable!("Category has recieved invalid messsage {msg:?}"),
        }
    }

    fn output(&mut self) -> Result<CategoryBody, ()> {
        self.errors.clear();
        let name_result = self.name_input.get_output();
        if let Err(ref e) = name_result {
            self.errors.insert(e.clone());
        }
        if self.errors.is_empty() {
            Err(())
        } else {
            Ok(CategoryBody {
                name: name_result.unwrap(),
            })
        }
    }
}
