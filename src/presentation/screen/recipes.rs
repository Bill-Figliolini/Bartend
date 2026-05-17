use std::collections::HashSet;

use iced::{
    Element,
    widget::{button, column, row, text},
};

use crate::{
    logic::{category::Category, config::Config, quantity::UnitSystem},
    presentation::{
        Updateable, Viewable, application,
        input_handling::{InputCollection, InputMessage, recipe_input::RecipeInput},
        widget::{header::header, input::Error, text_style::title},
    },
};

#[derive(Debug)]
pub struct Recipes {
    input: RecipeInput,
    unit_system: UnitSystem,

    errors: HashSet<Error>,

    categories: Vec<Category>,
}

impl Recipes {
    pub fn new(config: &Config) -> Self {
        let unit_system = config.default_units();
        Self {
            input: RecipeInput::new(config, input_msg),
            unit_system,
            errors: HashSet::new(),
            categories: Vec::new(),
        }
    }
    fn build_input(&self) -> Element<'_, application::Message> {
        let name_input = self.input.view();
        let save_button = button("Save").on_press(application::Message::Recipes(Message::Save));
        let input_row = row![name_input, save_button];

        let error_row = row(self
            .errors
            .iter()
            .map(|error| text!("{} ", error.to_string()).into()));

        column![input_row, error_row].into()
    }
    fn save(&mut self, name: String) -> application::Command {
        self.errors.clear();

        application::Command::AddRecipe(name, Vec::new())
    }
}
#[derive(Debug, Clone)]
pub enum Message {
    Input(InputMessage),
    Save,

    InitializeCategoryList(Vec<Category>),
}
impl Viewable<application::Message> for Recipes {
    fn view(&self) -> Element<'_, application::Message> {
        let header = header(title("Recipes"));
        let input_row = self.build_input();
        let body = column![input_row];
        column![header, body].into()
    }
}
impl Updateable<Message> for Recipes {
    fn update(&mut self, message: Message) -> Option<application::Command> {
        match message {
            Message::Input(msg) => None,
            Message::Save => None,
            Message::InitializeCategoryList(categories) => {
                self.categories = categories;
                None
            }
        }
    }
}
fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Recipes(Message::Input(msg))
}
