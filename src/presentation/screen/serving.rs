use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{Config, UnitSystem},
    presentation::{
        application,
        input_handling::{InputMessage, ServingInput},
    },
};

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {
    Reload,
    Input(InputMessage),
}

#[derive(Debug)]
pub(in crate::presentation) struct Serving {
    input: ServingInput,

    unit_system: UnitSystem,
}

impl Serving {
    pub fn new(config: &Config, recipe_service: &RecipeService) -> Self {
        let input = ServingInput::new(input_msg, recipe_service);
        let unit_system = config.default_units();
        Self { input, unit_system }
    }
    pub fn view(&self) -> iced::Element<'_, application::Message> {
        todo!()
    }
    pub fn update(
        &mut self,
        item_service: &ItemService,
        category_service: &CategoryService,
        message: Message,
    ) -> Option<application::Command> {
        todo!()
    }
}

fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Serving(Message::Input(msg))
}
