use crate::{
    logic::{CategoryService, RecipeService},
    models::{CategoryFilter, Config, Ingredient, Recipe, RecipeBody, RecipeID, UnitSystem},
    presentation::{
        application::{self, Context},
        input_handling::{Editor, EditorMessage, RecipeInput, RecipeMessage, Saved},
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
pub struct Recipes {
    editor: Editor<RecipeInput, RecipeID>,

    current_page: usize,
    unit_system: UnitSystem,
}
#[derive(Debug, Clone)]
pub enum Message {
    Editor(EditorMessage<RecipeMessage>),
    SwapUnits,
    /// Starts editing the recipe, or cancels if it is already being edited.
    BeginEdit(Recipe),
    DeleteRecipe(RecipeID),
    SelectPage(usize),
    Reload,
}

pub enum Command {
    Add(RecipeBody),
    Update(Recipe),
    Delete(RecipeID),
}
impl Command {
    pub fn apply(self, ctx: &mut Context<'_>) -> Task<application::Message> {
        match self {
            Command::Add(body) => match ctx.recipe_service.add(&ctx.database.recipe_db(), body) {
                Ok(_) => Task::done(application::Message::ReloadScreen),
                Err(e) => Task::done(application::Message::Error(e)),
            },
            Command::Update(recipe) => {
                match ctx.recipe_service.update(&ctx.database.recipe_db(), recipe) {
                    Ok(()) => Task::done(application::Message::ReloadScreen),
                    Err(e) => Task::done(application::Message::Error(e)),
                }
            }
            Command::Delete(recipe) => {
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
        let form = RecipeInput::new(config, category_service.get_all(CategoryFilter {}));
        Self {
            editor: Editor::new(form, "Recipe"),
            current_page: 0,
            unit_system,
        }
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
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
                        view_ingredient(ingredient, category_service, &self.unit_system)
                    }),
            )
        });
        let edit_column = table::column(text("Edit"), |recipe: RecipeID| {
            let begin = || {
                application::Message::Recipes(Message::BeginEdit(Recipe {
                    id: recipe,
                    body: recipe_service.get(recipe).unwrap_or_default(),
                }))
            };
            self.editor
                .edit_button(recipe, begin, editor_msg(EditorMessage::Cancel))
        });
        let delete_column = table::column(text("Delete"), |recipe: RecipeID| {
            button("X").on_press(application::Message::Recipes(Message::DeleteRecipe(recipe)))
        });
        let columns = vec![name_column, ingredient_column, edit_column, delete_column];
        let contents = recipe_service.get_page(self.current_page, 15);
        table(columns, contents).into()
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(
        &self,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Element<'_, application::Message> {
        let header = header(title("Recipes"));
        let input_row = self.editor.view().map(editor_msg);
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
            Message::Editor(msg) => match self.editor.update(msg) {
                Ok(Some(Saved::Add(body))) => Some(Command::Add(body)),
                Ok(Some(Saved::Update(id, body))) => Some(Command::Update(Recipe { id, body })),
                Ok(None) | Err(_) => None,
            },
            Message::BeginEdit(Recipe { id, body }) => {
                self.editor.toggle_edit(id, || body, self.unit_system);
                None
            }
            Message::SwapUnits => {
                self.unit_system.swap();
                None
            }
            Message::Reload => {
                self.editor.reset();
                self.current_page = self
                    .current_page
                    .min(recipe_service.recipe_count().div_ceil(15).saturating_sub(1));
                None
            }
            Message::DeleteRecipe(recipe) => Some(Command::Delete(recipe)),
            Message::SelectPage(page) => {
                self.current_page = page;
                None
            }
        }
    }
}
fn editor_msg(msg: EditorMessage<RecipeMessage>) -> application::Message {
    application::Message::Recipes(Message::Editor(msg))
}
#[cfg_attr(coverage_nightly, coverage(off))]
fn view_ingredient<'a>(
    ingredient: &Ingredient,
    category_service: &CategoryService,
    unit_system: &'a UnitSystem,
) -> Element<'a, application::Message> {
    let category_name = category_service
        .get(ingredient.category)
        .cloned()
        .unwrap_or_default()
        .name;
    let category = text!("{category_name}: ");
    let quantity_value = text(ingredient.quantity.value(*unit_system));
    let quantity_unit = text(ingredient.quantity.unit(*unit_system).to_string());
    row![category, quantity_value, quantity_unit].into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        logic::LogicError,
        models::{BartendError, CategoryID},
        persistence::DBError,
        presentation::screen::test_support::{World, drain, recipe_body},
    };

    const SAVE: Message = Message::Editor(EditorMessage::Save);

    fn input(msg: RecipeMessage) -> Message {
        Message::Editor(EditorMessage::Input(msg))
    }

    fn type_name(screen: &mut Recipes, world: &World, name: &str) {
        let name = RecipeMessage::Name(name.to_string());
        assert!(screen.update(input(name), &world.recipes).is_none());
    }

    fn apply(world: &mut World, command: Command) -> Vec<application::Message> {
        drain(command.apply(&mut world.ctx()))
    }

    fn assert_reloads(messages: &[application::Message]) {
        assert!(
            matches!(messages, [application::Message::ReloadScreen]),
            "{messages:?}"
        );
    }

    #[test]
    fn saving_a_named_recipe_adds_it() {
        let mut world = World::new();
        let mut screen = Recipes::new(&world.config, &world.categories);
        type_name(&mut screen, &world, "Draft");

        let command = screen.update(SAVE, &world.recipes);
        let Some(command @ Command::Add(_)) = command else {
            panic!("expected Add");
        };
        assert_reloads(&apply(&mut world, command));

        let recipe = world.recipes.get_page(0, 1)[0];
        assert_eq!(
            world.recipes.get(recipe).unwrap(),
            recipe_body("Draft", &[])
        );
    }

    #[test]
    fn an_incomplete_ingredient_blocks_saving_until_removed() {
        let world = World::new();
        let mut screen = Recipes::new(&world.config, &world.categories);
        type_name(&mut screen, &world, "Draft");

        let add = input(RecipeMessage::AddRow);
        screen.update(add, &world.recipes);
        assert!(screen.update(SAVE, &world.recipes).is_none());

        let remove = RecipeMessage::RemoveRow(screen.editor.form().row_key(0));
        screen.update(input(remove), &world.recipes);
        assert!(matches!(
            screen.update(SAVE, &world.recipes),
            Some(Command::Add(_))
        ));
    }

    #[test]
    fn editing_updates_the_existing_recipe() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let id = world.add_recipe("Gin Shot", &[(gin, 50.0)]);
        let mut screen = Recipes::new(&world.config, &world.categories);

        let recipe = Recipe {
            id,
            body: world.recipes.get(id).unwrap(),
        };
        screen.update(Message::BeginEdit(recipe), &world.recipes);
        type_name(&mut screen, &world, "Double Gin Shot");
        let command = screen.update(SAVE, &world.recipes);
        let Some(command @ Command::Update(Recipe { id: edited, .. })) = command else {
            panic!("expected Update");
        };
        assert_eq!(edited, id);
        assert_reloads(&apply(&mut world, command));

        assert_eq!(
            world.recipes.get(id).unwrap(),
            recipe_body("Double Gin Shot", &[(gin, 50.0)])
        );
    }

    #[test]
    fn cancelling_an_edit_empties_the_form() {
        let mut world = World::new();
        let id = world.add_recipe("Draft", &[]);
        let mut screen = Recipes::new(&world.config, &world.categories);
        let recipe = Recipe {
            id,
            body: world.recipes.get(id).unwrap(),
        };
        screen.update(Message::BeginEdit(recipe), &world.recipes);

        screen.update(Message::Editor(EditorMessage::Cancel), &world.recipes);

        assert_eq!(screen.editor.editing(), None);
        assert!(
            screen.update(SAVE, &world.recipes).is_none(),
            "saving after a cancel must not add a copy of the recipe"
        );
    }

    #[test]
    fn updating_a_missing_recipe_reports_error() {
        let mut world = World::new();
        let ghost = Recipe {
            id: RecipeID(99),
            body: recipe_body("Ghost", &[]),
        };
        let messages = apply(&mut world, Command::Update(ghost));
        assert!(
            matches!(
                messages.as_slice(),
                [application::Message::Error(BartendError::Logic(
                    LogicError::InvalidRecipe(RecipeID(99))
                ))]
            ),
            "{messages:?}"
        );
    }

    #[test]
    fn adding_a_recipe_with_a_missing_category_reports_error() {
        let mut world = World::new();
        let body = recipe_body("Ghost Shot", &[(CategoryID(99), 50.0)]);

        let messages = apply(&mut world, Command::Add(body));

        assert!(
            matches!(
                messages.as_slice(),
                [application::Message::Error(BartendError::Db(
                    DBError::RestrictViolation
                ))]
            ),
            "{messages:?}"
        );
        assert_eq!(world.recipes.recipe_count(), 0);
    }

    #[test]
    fn deleting_a_recipe_removes_it() {
        let mut world = World::new();
        let id = world.add_recipe("Draft", &[]);
        let mut screen = Recipes::new(&world.config, &world.categories);

        let command = screen.update(Message::DeleteRecipe(id), &world.recipes);
        assert_reloads(&apply(&mut world, command.unwrap()));

        assert_eq!(world.recipes.recipe_count(), 0);
    }

    #[test]
    fn swap_units_toggles_the_unit_system() {
        let world = World::new();
        let mut screen = Recipes::new(&world.config, &world.categories);
        screen.update(Message::SwapUnits, &world.recipes);
        assert_eq!(screen.unit_system, UnitSystem::Imperial);
    }

    #[test]
    fn reload_ends_editing_and_clamps_the_page() {
        let mut world = World::new();
        let id = world.add_recipe("Draft", &[]);
        let mut screen = Recipes::new(&world.config, &world.categories);
        let recipe = Recipe {
            id,
            body: world.recipes.get(id).unwrap(),
        };
        screen.update(Message::BeginEdit(recipe), &world.recipes);
        screen.update(Message::SelectPage(3), &world.recipes);

        screen.update(Message::Reload, &world.recipes);

        assert_eq!(screen.editor.editing(), None);
        assert_eq!(screen.current_page, 0);
        assert!(screen.update(SAVE, &world.recipes).is_none());
    }
}
