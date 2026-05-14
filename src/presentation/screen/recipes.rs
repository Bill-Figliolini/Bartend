use std::collections::HashSet;

use iced::{
    Element,
    widget::{Id, button, column, row, text},
};

use crate::{
    logic::{category::Category, config::Config, quantity::UnitSystem},
    presentation::{
        Updateable, Viewable, application,
        widget::{
            header::header,
            input::{Error, Input, string_input::StringInput},
            text_style::title,
        },
    },
};

#[derive(Debug)]
pub struct Recipes {
    input_name: StringInput,
    input_ingredients: Vec<IngredientRow>,
    unit_system: UnitSystem,

    errors: HashSet<Error>,

    categories: Vec<Category>,
}
#[derive(Debug)]
struct IngredientRow {
    input_name: String,
    input_quantity: String,
}
impl Recipes {
    pub fn new(config: &Config) -> Self {
        let unit_system = config.default_units();
        Self {
            input_name: StringInput::new(
                |id, str: String| application::Message::Recipes(Message::NameUpdate(id, str)),
                String::new(),
            ),
            input_ingredients: Vec::new(),
            unit_system,
            errors: HashSet::new(),
            categories: Vec::new(),
        }
    }
    fn build_input(&self) -> Element<'_, application::Message> {
        let name_input = self.input_name.view();
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
    NameUpdate(Id, String),
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
            Message::NameUpdate(_id, _name) => None,
            Message::Save => None,
            Message::InitializeCategoryList(categories) => {
                self.categories = categories;
                None
            }
        }
    }
}
