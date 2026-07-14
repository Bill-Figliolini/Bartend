use iced::widget::{Container, button, column, row, text};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{Config, UnitSystem},
    presentation::{
        application,
        input_handling::{InputMessage, ServingInput},
        widget::footer::footer,
    },
};

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {
    Reload,
    SwapUnits,
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
    pub fn view(
        &self,
        item_service: &ItemService,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> iced::Element<'_, application::Message> {
        let body = column![
            self.input
                .view(item_service, category_service, recipe_service)
        ];
        let unit_swap_button = button(text(self.unit_system.to_string()))
            .on_press(application::Message::Serving(Message::SwapUnits));
        let footer_content = row![unit_swap_button];
        let footer_container = Container::new(footer_content);
        let footer = footer(footer_container);

        column![body, footer].into()
    }
    pub fn update(
        &mut self,
        item_service: &ItemService,
        category_service: &CategoryService,
        message: Message,
    ) -> Option<application::Command> {
        match message {
            Message::Reload => todo!(),
            Message::SwapUnits => {
                self.unit_system.swap();
                None
            }
            Message::Input(input_message) => {
                self.input
                    .update(input_message, item_service, category_service);
                None
            }
        }
    }
}

fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Serving(Message::Input(msg))
}
