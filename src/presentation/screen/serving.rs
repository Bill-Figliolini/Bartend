use iced::{
    Task,
    widget::{Container, button, column, row, text},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{Config, ItemUse, UnitSystem},
    presentation::{
        application::{self, Context},
        input_handling::{InputMessage, ServingInput},
        widget::footer::footer,
    },
};

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {
    Reload,
    SwapUnits,
    Input(InputMessage),
    Save,
}

pub(in crate::presentation) enum Command {
    UseItems(Vec<ItemUse>),
}
impl Command {
    pub fn apply(self, ctx: &mut Context) -> Task<application::Message> {
        match self {
            Command::UseItems(items) => {
                ctx.item_service
                    .use_items(&ctx.database.item_db(), items)
                    .unwrap();
                Task::none()
            }
        }
    }
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
        category_service: &CategoryService,
    ) -> iced::Element<'_, application::Message> {
        let save_button = button("Save").on_press(application::Message::Serving(Message::Save));
        let body = row![
            self.input.view(category_service, self.unit_system),
            save_button
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
    ) -> Option<Command> {
        match message {
            Message::Reload => {
                self.input.clear();
                None
            }
            Message::SwapUnits => {
                self.unit_system.swap();
                None
            }
            Message::Input(input_message) => {
                self.input
                    .update(input_message, item_service, category_service);
                None
            }
            Message::Save => {
                if let Ok(used_items) = self.input.output() {
                    Some(Command::UseItems(used_items))
                } else {
                    None
                }
            }
        }
    }
}

fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Serving(Message::Input(msg))
}
