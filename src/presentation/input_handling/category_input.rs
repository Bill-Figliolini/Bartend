use crate::{
    logic::{category::CategoryBody, config::Config},
    presentation::{
        Viewable,
        application::Message,
        input_handling::{InputCollection, InputMessage},
        widget::input::{InputContents, text_input::string_input::StringInput},
    },
};

#[derive(Debug)]
pub(super) struct CategoryInput {
    name_input: StringInput<Message>,
}

impl Viewable<Message> for CategoryInput {
    fn view(&self) -> iced::Element<'_, Message> {
        self.name_input.view()
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
        }
    }
    fn update(&mut self, msg: InputMessage) {
        todo!()
    }

    fn output(&mut self) -> Result<CategoryBody, ()> {
        let name_result = self.name_input.get_output();
        todo!()
    }
}
