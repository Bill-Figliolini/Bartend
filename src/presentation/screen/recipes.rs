use crate::{
    models::{
        category::Category,
        config::Config,
        quantity::UnitSystem,
        recipe::{Ingredient, Recipe, RecipeBody, RecipeID},
    },
    presentation::{
        Updateable, Viewable, application,
        input_handling::{InputCollection, InputMessage, recipe_input::RecipeInput},
        widget::{footer::footer, header::header, text_style::title},
    },
};
use iced::{
    Element,
    Length::Fill,
    widget::{button, column, row, table, text},
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
    SwapUnits,
    RemoveIngredient(usize),
    BeginEdit(Recipe),
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
        let header = match self.edit_state {
            EditState::Editing(_) => text("Editing: "),
            EditState::None => text("New Recipe: "),
        };
        let input = self.input.view();
        let save_button = button("Save").on_press(application::Message::Recipes(Message::Save));
        let input_row = row![input, save_button];
        let add_ingredient_button = button("Add Ingredient")
            .on_press(application::Message::Recipes(Message::AddIngredient));
        column![header, input_row, add_ingredient_button].into()
    }
    fn save(&mut self, body: RecipeBody) -> application::Command {
        match self.edit_state {
            EditState::Editing(id) => application::Command::UpdateRecipe(Recipe { id, body }),
            EditState::None => application::Command::AddRecipe(body),
        }
    }
    fn build_display_table(&self) -> Element<'_, application::Message> {
        let name_column = table::column(text("Name"), |recipe: &Recipe| text(&recipe.body.name));
        let ingredient_column =
            table::column(text("Ingredients"), |recipe: &Recipe| {
                column(recipe.body.ingredients.iter().map(|ingredient| {
                    view_ingredient(ingredient, &self.categories, &self.unit_system)
                }))
            });
        let edit_column = table::column(text("Edit"), |recipe: &Recipe| match self.edit_state {
            EditState::Editing(recipe_id) if recipe_id == recipe.id => {
                button("Cancel").on_press(application::Message::UpdateRecipes)
            }
            EditState::Editing(_) => button("Edit"),
            EditState::None => button("Edit").on_press(application::Message::Recipes(
                Message::BeginEdit(recipe.clone()),
            )),
        });
        let delete_column = table::column(text("Delete"), |recipe: &Recipe| {
            button("X").on_press(application::Message::DeleteRecipe(recipe.clone()))
        });
        let columns = vec![name_column, ingredient_column, edit_column, delete_column];
        table(columns, &self.recipes).into()
    }
}

impl Viewable<application::Message> for Recipes {
    fn view(&self) -> Element<'_, application::Message> {
        let header = header(title("Recipes"));
        let input_row = self.build_input();
        let recipe_display = self.build_display_table();
        let body = column![input_row, recipe_display];

        let unit_swap_button = iced::widget::Button::new(text(self.unit_system.to_string()))
            .on_press(application::Message::Recipes(Message::SwapUnits));
        let footer_contents = row![unit_swap_button];
        let footer_container = iced::widget::Container::new(footer_contents).align_left(Fill);
        let footer = footer(footer_container);
        column![header, body, footer].into()
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
                self.edit_state = EditState::None;
                self.input.clear();
                self.recipes = recipes;
                None
            }
            Message::RemoveIngredient(index) => {
                self.input.remove_ingredient(index);
                None
            }
            Message::BeginEdit(recipe) => {
                self.edit_state = EditState::Editing(recipe.id);
                self.input.begin_edit(&recipe.body, self.unit_system);
                None
            }
            Message::SwapUnits => {
                self.unit_system.swap();
                None
            }
        }
    }
}
fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Recipes(Message::Input(msg))
}
fn view_ingredient<'a>(
    ingredient: &'a Ingredient,
    categories: &'a [Category],
    unit_system: &'a UnitSystem,
) -> Element<'a, application::Message> {
    let category_name = categories
        .iter()
        .find(|category| category.id == ingredient.category)
        .unwrap()
        .body
        .name
        .clone();
    let category = text!("{category_name}: ");
    let quantity_value = text(ingredient.quantity.value(*unit_system));
    let quantity_unit = text(ingredient.quantity.unit(*unit_system).to_string());
    row![category, quantity_value, quantity_unit].into()
}
