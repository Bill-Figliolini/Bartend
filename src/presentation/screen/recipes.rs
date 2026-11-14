use crate::{
    logic::{CategoryService, RecipeService},
    models::{CategoryFilter, Config, Ingredient, Recipe, RecipeBody, RecipeID, UnitSystem},
    presentation::{
        application::{self, Context},
        constants,
        input_handling::{Editor, EditorMessage, RecipeInput, RecipeMessage, Saved},
        screen::Panels,
        widget::{
            footer::footer,
            master_list::{Entry, master_list},
        },
    },
};
use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{Column, button, column, container, row, space, text},
};
#[derive(Debug)]
pub struct Recipes {
    editor: Editor<RecipeInput, RecipeID>,
    /// The recipe shown in the detail panel; `None` shows the new-recipe
    /// form.
    selected: Option<RecipeID>,
    unit_system: UnitSystem,
}
#[derive(Debug, Clone)]
pub enum Message {
    Editor(EditorMessage<RecipeMessage>),
    SwapUnits,
    /// Shows the recipe in the detail panel, or the new-recipe form for
    /// `None`. Any edit in progress is discarded.
    Select(Option<RecipeID>),
    /// Starts editing the recipe, or cancels if it is already being edited.
    BeginEdit(Recipe),
    DeleteRecipe(RecipeID),
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
            selected: None,
            unit_system,
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(
        &self,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Panels<'_> {
        let entries = recipe_service.sorted_ids().into_iter().map(|id| Entry {
            label: recipe_service.get(id).unwrap_or_default().name,
            selected: self.selected == Some(id),
            on_press: recipes_msg(Message::Select(Some(id))),
        });
        let new = (
            "+ New Recipe".to_string(),
            recipes_msg(Message::Select(None)),
        );
        let master = master_list(Some(new), entries);

        let detail = match self.selected {
            Some(recipe) if self.editor.editing().is_none() => {
                self.recipe_details(recipe, category_service, recipe_service)
            }
            _ => self.editor.view().map(editor_msg),
        };
        let unit_swap_button = button(text(self.unit_system.to_string()))
            .style(button::secondary)
            .on_press(recipes_msg(Message::SwapUnits));
        let footer = footer(container(row![space().width(Fill), unit_swap_button]).center_y(Fill));
        let detail = column![container(detail).height(Fill), footer].into();

        Panels { master, detail }
    }

    /// The read-only view of `recipe`: its ingredients, and buttons to edit
    /// or delete it.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn recipe_details(
        &self,
        id: RecipeID,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Element<'_, application::Message> {
        let body = recipe_service.get(id).unwrap_or_default();
        let name = text(body.name.clone()).size(24);
        let ingredients: Element<'_, _> =
            if body.ingredients.is_empty() {
                text("No ingredients").into()
            } else {
                Column::with_children(body.ingredients.iter().map(|ingredient| {
                    view_ingredient(ingredient, category_service, &self.unit_system)
                }))
                .spacing(constants::SPACING / 2)
                .into()
            };
        let edit = button("Edit").on_press(recipes_msg(Message::BeginEdit(Recipe { id, body })));
        let delete = button("Delete")
            .style(button::danger)
            .on_press(recipes_msg(Message::DeleteRecipe(id)));
        let buttons = row![edit, delete].spacing(constants::SPACING);
        column![name, text("Ingredients").size(18), ingredients, buttons]
            .spacing(16)
            .into()
    }

    pub fn update(&mut self, message: Message, recipe_service: &RecipeService) -> Option<Command> {
        match message {
            Message::Select(recipe) => {
                self.editor.reset();
                self.selected = recipe;
                None
            }
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
                self.selected = self
                    .selected
                    .filter(|&recipe| recipe_service.get(recipe).is_ok());
                None
            }
            Message::DeleteRecipe(recipe) => Some(Command::Delete(recipe)),
        }
    }
}
fn recipes_msg(msg: Message) -> application::Message {
    application::Message::Recipes(msg)
}
fn editor_msg(msg: EditorMessage<RecipeMessage>) -> application::Message {
    recipes_msg(Message::Editor(msg))
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
    fn reload_ends_editing_and_keeps_a_live_selection() {
        let mut world = World::new();
        let id = world.add_recipe("Draft", &[]);
        let mut screen = Recipes::new(&world.config, &world.categories);
        let recipe = Recipe {
            id,
            body: world.recipes.get(id).unwrap(),
        };
        screen.update(Message::Select(Some(id)), &world.recipes);
        screen.update(Message::BeginEdit(recipe), &world.recipes);

        screen.update(Message::Reload, &world.recipes);

        assert_eq!(screen.editor.editing(), None);
        assert_eq!(screen.selected, Some(id));
        assert!(screen.update(SAVE, &world.recipes).is_none());
    }

    #[test]
    fn reload_after_a_delete_clears_the_selection() {
        let mut world = World::new();
        let id = world.add_recipe("Draft", &[]);
        let mut screen = Recipes::new(&world.config, &world.categories);
        screen.update(Message::Select(Some(id)), &world.recipes);

        let command = screen.update(Message::DeleteRecipe(id), &world.recipes);
        apply(&mut world, command.unwrap());
        screen.update(Message::Reload, &world.recipes);

        assert_eq!(screen.selected, None);
    }

    #[test]
    fn selecting_a_recipe_discards_an_edit_in_progress() {
        let mut world = World::new();
        let first = world.add_recipe("Draft", &[]);
        let second = world.add_recipe("Martini", &[]);
        let mut screen = Recipes::new(&world.config, &world.categories);
        let recipe = Recipe {
            id: first,
            body: world.recipes.get(first).unwrap(),
        };
        screen.update(Message::BeginEdit(recipe), &world.recipes);

        screen.update(Message::Select(Some(second)), &world.recipes);

        assert_eq!(screen.selected, Some(second));
        assert_eq!(screen.editor.editing(), None);

        screen.update(Message::Select(None), &world.recipes);
        assert_eq!(screen.selected, None);
    }
}
