use iced::{
    Element,
    widget::{button, column, row},
};

use crate::{
    logic::{
        category::Category,
        config::Config,
        recipe::{RecipeBody, RecipeID},
    },
    presentation::{
        Updateable, Viewable, application,
        input_handling::{InputCollection, InputMessage, recipe_input::RecipeInput},
        widget::{header::header, text_style::title},
    },
};
#[derive(Debug)]
enum EditState {
    Editing(RecipeID),
    None,
}

#[derive(Debug)]
pub struct Recipes {
    input: RecipeInput,
    edit_state: EditState,
}

impl Recipes {
    pub fn new(config: &Config, categories: Vec<Category>) -> Self {
        let unit_system = config.default_units();
        Self {
            input: RecipeInput::new(config, input_msg, categories),
            edit_state: EditState::None,
        }
    }
    fn build_input(&self) -> Element<'_, application::Message> {
        let name_input = self.input.view();
        let save_button = button("Save").on_press(application::Message::Recipes(Message::Save));
        let input_row = row![name_input, save_button];
        let add_ingredient_button = button("Add Ingredient")
            .on_press(application::Message::Recipes(Message::AddIngredient));
        column![input_row, add_ingredient_button].into()
    }
    fn save(&mut self, body: RecipeBody) -> application::Command {
        application::Command::AddRecipe(body)
    }
}
#[derive(Debug, Clone)]
pub enum Message {
    Input(InputMessage),
    Save,
    AddIngredient,
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
            Message::Input(msg) => {
                self.input.update(msg);
                None
            }
            Message::Save => match self.input.output() {
                Ok(body) => Some(self.save(body)),
                Err(()) => None,
            },
            Message::AddIngredient => {
                self.input.add_ingredient();
                None
            }
        }
    }
}
fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Recipes(Message::Input(msg))
}
