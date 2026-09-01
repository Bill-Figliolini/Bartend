use crate::{
    logic::{CategoryService, RecipeService},
    models::{CategoryFilter, Config, Ingredient, Recipe, RecipeBody, RecipeID, UnitSystem},
    presentation::{
        application::{self, Context},
        input_handling::{InputMessage, RecipeInput},
        widget::{footer::footer, header::header, text_style::title},
    },
};
use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{button, column, container, row, table, text},
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

    current_page: usize,
    unit_system: UnitSystem,
}
#[derive(Debug, Clone)]
pub enum Message {
    Input(InputMessage),
    Save,
    AddIngredient,
    SwapUnits,
    RemoveIngredient(usize),
    BeginEdit(Recipe),
    EndEdit,
    DeleteRecipe(RecipeID),
    SelectPage(usize),
    Reload,
}

pub enum Command {
    AddRecipe(RecipeBody),
    UpdateRecipe(Recipe),
    DeleteRecipe(RecipeID),
}
impl Command {
    pub fn apply(self, ctx: &mut Context<'_>) -> Task<application::Message> {
        match self {
            Command::AddRecipe(body) => {
                match ctx.recipe_service.add(&ctx.database.recipe_db(), body) {
                    Ok(_) => Task::done(application::Message::ReloadScreen),
                    Err(e) => Task::done(application::Message::Error(e)),
                }
            }
            Command::UpdateRecipe(recipe) => {
                match ctx.recipe_service.update(&ctx.database.recipe_db(), recipe) {
                    Ok(()) => Task::done(application::Message::ReloadScreen),
                    Err(e) => Task::done(application::Message::Error(e)),
                }
            }
            Command::DeleteRecipe(recipe) => {
                match ctx.recipe_service.delete(&ctx.database.recipe_db(), recipe) {
                    Ok(()) => Task::done(application::Message::ReloadScreen),
                    Err(e) => Task::done(application::Message::Error(e)),
                }
            }
        }
    }
}
impl Recipes {
    pub fn new(config: &Config, category_service: &CategoryService) -> Self {
        let unit_system = config.default_units();
        Self {
            input: RecipeInput::new(
                config,
                input_msg,
                category_service.get_all(CategoryFilter {}),
            ),
            edit_state: EditState::None,

            current_page: 0,
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
    fn save(&mut self, body: RecipeBody) -> Command {
        match self.edit_state {
            EditState::Editing(id) => Command::UpdateRecipe(Recipe { id, body }),
            EditState::None => Command::AddRecipe(body),
        }
    }
    fn build_display_table(
        &self,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Element<'_, application::Message> {
        let name_column = table::column(text("Name"), |recipe: RecipeID| {
            text(recipe_service.get(recipe).unwrap_or_default().name)
        });
        let ingredient_column = table::column(text("Ingredients"), |recipe: RecipeID| {
            column(
                recipe_service
                    .get(recipe)
                    .unwrap_or_default()
                    .ingredients
                    .iter()
                    .map(|ingredient| {
                        let ingredient = ingredient.clone();
                        view_ingredient(ingredient, category_service, &self.unit_system)
                    }),
            )
        });
        let edit_column = table::column(text("Edit"), |recipe: RecipeID| match self.edit_state {
            EditState::Editing(recipe_id) if recipe_id == recipe => {
                button("Cancel").on_press(application::Message::Recipes(Message::EndEdit))
            }
            EditState::Editing(_) => button("Edit"),
            EditState::None => {
                button("Edit").on_press(application::Message::Recipes(Message::BeginEdit(Recipe {
                    id: recipe,
                    body: recipe_service.get(recipe).unwrap_or_default(),
                })))
            }
        });
        let delete_column = table::column(text("Delete"), |recipe: RecipeID| {
            button("X").on_press(application::Message::Recipes(Message::DeleteRecipe(recipe)))
        });
        let columns = vec![name_column, ingredient_column, edit_column, delete_column];
        let contents = recipe_service.get_page(self.current_page, 15);
        table(columns, contents).into()
    }
    pub fn view(
        &self,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Element<'_, application::Message> {
        let header = header(title("Recipes"));
        let input_row = self.build_input();
        let recipe_display = self.build_display_table(category_service, recipe_service);
        let body = container(column![input_row, recipe_display]).align_top(Fill);

        let mut page_controller_contents = Vec::new();
        if self.current_page != 0 {
            let previous_page = self.current_page.wrapping_sub(1);
            let previous_page_button = button("Previous")
                .on_press(application::Message::Recipes(Message::SelectPage(
                    previous_page,
                )))
                .into();
            page_controller_contents.push(previous_page_button);
        }
        let space = iced::widget::space().width(Fill).into();
        page_controller_contents.push(space);
        if (self.current_page + 1) * 15 < recipe_service.recipe_count() {
            let next_page = self.current_page.wrapping_add(1);
            let next_page_button = button("Next")
                .on_press(application::Message::Recipes(Message::SelectPage(
                    next_page,
                )))
                .into();
            page_controller_contents.push(next_page_button);
        }
        let page_controller = row(page_controller_contents);
        let unit_swap_button = iced::widget::Button::new(text(self.unit_system.to_string()))
            .on_press(application::Message::Recipes(Message::SwapUnits));
        let footer_contents = row![unit_swap_button];
        let footer_container = iced::widget::Container::new(footer_contents).align_left(Fill);
        let footer = footer(footer_container);
        column![header, body, page_controller, footer].into()
    }
    pub fn update(&mut self, message: Message, recipe_service: &RecipeService) -> Option<Command> {
        match message {
            Message::Input(msg) => {
                self.input.update(msg);
                None
            }
            Message::Save => match self.input.output() {
                Ok(body) => {
                    self.input.clear();
                    Some(self.save(body))
                }
                Err(()) => None,
            },
            Message::AddIngredient => {
                self.input.add_ingredient();
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
            Message::EndEdit => {
                self.edit_state = EditState::None;
                None
            }
            Message::Reload => {
                self.input.clear();
                self.edit_state = EditState::None;
                self.current_page = self
                    .current_page
                    .min((recipe_service.recipe_count() / 15).saturating_sub(1));
                None
            }
            Message::DeleteRecipe(recipe) => Some(Command::DeleteRecipe(recipe)),
            Message::SelectPage(page) => {
                self.current_page = page;
                None
            }
        }
    }
}
fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Recipes(Message::Input(msg))
}
fn view_ingredient<'a>(
    ingredient: Ingredient,
    category_service: &CategoryService,
    unit_system: &'a UnitSystem,
) -> Element<'a, application::Message> {
    let category_name = category_service
        .get(&ingredient.category)
        .cloned()
        .unwrap_or_default()
        .name;
    let category = text!("{category_name}: ");
    let quantity_value = text(ingredient.quantity.value(*unit_system));
    let quantity_unit = text(ingredient.quantity.unit(*unit_system).to_string());
    row![category, quantity_value, quantity_unit].into()
}
