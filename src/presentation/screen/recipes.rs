use iced::{
    Element,
    widget::{button, column, container, row, rule, space, text},
};

use crate::{
    logic::{
        category::Category,
        config::Config,
        quantity::UnitSystem,
        recipe::{Ingredient, Recipe, RecipeBody, RecipeID},
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

    recipes: Vec<Recipe>,
    categories: Vec<Category>,
    unit_system: UnitSystem,
}
#[derive(Debug, Clone)]
pub enum Message {
    Input(InputMessage),
    Update(Vec<Recipe>),
    Save,
    AddIngredient,
    RemoveIngredient(usize),
}
impl Recipes {
    pub fn new(config: &Config, categories: Vec<Category>, recipes: Vec<Recipe>) -> Self {
        let unit_system = config.default_units();
        Self {
            input: RecipeInput::new(config, input_msg, categories.clone()),
            edit_state: EditState::None,

            recipes,
            categories,
            unit_system,
        }
    }
    fn build_input(&self) -> Element<'_, application::Message> {
        let input = self.input.view();
        let save_button = button("Save").on_press(application::Message::Recipes(Message::Save));
        let input_row = row![input, save_button];
        let add_ingredient_button = button("Add Ingredient")
            .on_press(application::Message::Recipes(Message::AddIngredient));
        column![input_row, add_ingredient_button].into()
    }
    fn build_display(&self) -> Element<'_, application::Message> {
        column(self.recipes.iter().map(|recipe| self.recipe_view(recipe))).into()
    }
    fn save(&mut self, body: RecipeBody) -> application::Command {
        application::Command::AddRecipe(body)
    }
    fn recipe_view(&self, recipe: &Recipe) -> Element<'_, application::Message> {
        let name = column![text(recipe.body.name.clone())];
        let ingredients = column(
            recipe
                .body
                .ingredients
                .iter()
                .map(|ingredient| self.ingredient_view(ingredient)),
        )
        .spacing(2);
        let body = row![name, space().width(5), ingredients];
        column![body, rule::horizontal(3)].into()
    }
    fn ingredient_view(&self, ingredient: &Ingredient) -> Element<'_, application::Message> {
        let category_name = self
            .categories
            .iter()
            .find(|category| category.id == ingredient.category)
            .unwrap()
            .body
            .name
            .clone();
        let category = text!("{category_name}: ");
        let quantity_value = text(ingredient.quantity.value(self.unit_system));
        let quantity_unit = text(ingredient.quantity.unit(self.unit_system).to_string());
        row![category, quantity_value, quantity_unit].into()
    }
}

impl Viewable<application::Message> for Recipes {
    fn view(&self) -> Element<'_, application::Message> {
        let header = header(title("Recipes"));
        let input_row = self.build_input();
        let recipe_display = self.build_display();
        let body = column![input_row, recipe_display];
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
            Message::Update(recipes) => {
                self.recipes = recipes;
                None
            }
            Message::RemoveIngredient(index) => {
                self.input.remove_ingredient(index);
                None
            }
        }
    }
}
fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Recipes(Message::Input(msg))
}
