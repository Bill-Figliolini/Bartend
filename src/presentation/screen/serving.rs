use crate::{
    logic::RecipeService,
    models::{Config, UnitSystem},
    presentation::{application, input_handling::ServingInput},
};

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {}

#[derive(Debug)]
pub(in crate::presentation) struct Serving {
    input: ServingInput,

    unit_system: UnitSystem,
}

impl Serving {
    pub fn new(config: &Config, recipe_service: &RecipeService) -> Self {
        let input = ServingInput::new();
        let unit_system = config.default_units();
        Self { input, unit_system }
    }
    pub fn view(&self) -> iced::Element<'_, application::Message> {
        todo!()
    }
    pub fn update(&mut self, message: Message) -> Option<application::Command> {
        todo!()
    }
}
