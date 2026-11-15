use crate::{
    logic::{CategoryService, IngredientStatus, ItemService, RecipeService, ingredient_statuses},
    models::{CategoryFilter, Config, Recipe, RecipeBody, RecipeID, UnitSystem},
    presentation::{
        application::{self, Context},
        constants,
        input_handling::{Editor, EditorMessage, RecipeInput, RecipeMessage, Saved},
        screen::{
            Panels,
            shared::{
                Availability, counted, detail_header, form_page_with_cards, reload_or_error,
                volume_figure,
            },
        },
        widget::{
            detail::{Tone, card, hint, stat_tile, status_link_row},
            master_list::{Entry, master_list},
        },
    },
};
use iced::{
    Element,
    Length::{Fill, Shrink},
    Task,
    widget::{Column, button, column, row, scrollable},
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
    /// Shows amounts in the given unit system, as picked in the top bar.
    SetUnits(UnitSystem),
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
            Command::Add(body) => {
                reload_or_error(ctx.recipe_service.add(&ctx.database.recipe_db(), body))
            }
            Command::Update(recipe) => {
                reload_or_error(ctx.recipe_service.update(&ctx.database.recipe_db(), recipe))
            }
            Command::Delete(recipe) => {
                reload_or_error(ctx.recipe_service.delete(&ctx.database.recipe_db(), recipe))
            }
        }
    }
}
impl Recipes {
    /// The unit system amounts are shown in.
    #[cfg(test)]
    pub fn units(&self) -> UnitSystem {
        self.unit_system
    }

    pub fn new(config: &Config, category_service: &CategoryService, units: UnitSystem) -> Self {
        let form = RecipeInput::new(config, category_service.get_all(CategoryFilter {}));
        Self {
            editor: Editor::new(form, "Recipe"),
            selected: None,
            unit_system: units,
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(
        &self,
        item_service: &ItemService,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Panels<'_> {
        let entries = recipe_service.sorted_ids().into_iter().map(|id| Entry {
            label: recipe_service.get(id).unwrap_or_default().name,
            selected: self.selected == Some(id),
            on_press: recipes_msg(Message::Select(Some(id))),
            dimmed: false,
        });
        let new = (
            "+ New Recipe".to_string(),
            recipes_msg(Message::Select(None)),
        );
        let master = master_list(Some(new), entries, "No recipes yet");

        let detail = match self.selected {
            Some(recipe) if self.editor.editing().is_none() => {
                self.recipe_details(recipe, item_service, category_service, recipe_service)
            }
            _ => self.form_view(),
        };
        Panels { master, detail }
    }

    /// The new- or edit-recipe form, laid out like the recipe's details:
    /// the heading with Save and Cancel to its right, then the name and the
    /// ingredient table in their own cards.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn form_view(&self) -> Element<'_, application::Message> {
        let note = if self.editor.editing().is_some() {
            "Change any details, then Save."
        } else {
            "Add a recipe, then Save."
        };
        form_page_with_cards(
            self.editor.title(),
            note,
            self.editor.buttons().map(editor_msg),
            self.editor.form_view().map(editor_msg),
        )
    }

    /// The read-only view of `recipe`: its name and whether it can be made
    /// from what is in stock, tiles for its volume and ingredient count,
    /// then its ingredients, each with how well stock covers it.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn recipe_details(
        &self,
        id: RecipeID,
        item_service: &ItemService,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Element<'_, application::Message> {
        let body = recipe_service.get(id).unwrap_or_default();
        let statuses = ingredient_statuses(&body, item_service, category_service);

        let serve = button("Serve")
            .on_press(application::Message::ShowServing(id))
            .into();
        let header = detail_header(
            body.name.clone(),
            Availability::of(&statuses).view(),
            Some(serve),
            recipes_msg(Message::BeginEdit(Recipe {
                id,
                body: body.clone(),
            })),
            recipes_msg(Message::DeleteRecipe(id)),
        );

        let (volume, below) = volume_figure(&body.volume_summary(), self.unit_system);
        let volume_tile = stat_tile(
            "Drink volume",
            volume,
            below.map(|below| hint(below).into()),
        );
        // The ingredients tile has nothing under its figure, so it is never
        // the taller one and takes the volume tile's height. It must come
        // after that tile to do so (see `stat_tile`).
        let ingredients_tile = stat_tile(
            "Ingredients",
            counted(body.ingredients.len(), "ingredient"),
            None,
        )
        .height(Fill);
        let tiles = row![volume_tile, ingredients_tile]
            .spacing(constants::SPACING)
            .height(Shrink);

        let ingredients = self.ingredients_card(&body, &statuses, category_service);

        scrollable(column![header, tiles, ingredients].spacing(16)).into()
    }

    /// One row per ingredient: its category, amount and stock status, each
    /// opening the category on the Categories screen when clicked.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn ingredients_card(
        &self,
        body: &RecipeBody,
        statuses: &[IngredientStatus],
        category_service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let content: Element<'_, _> = if body.ingredients.is_empty() {
            hint("No ingredients yet. Edit the recipe to add some.").into()
        } else {
            Column::with_children(body.ingredients.iter().zip(statuses).map(
                |(ingredient, &status)| {
                    status_link_row(
                        category_service.name_of(ingredient.category),
                        ingredient.quantity.readable(self.unit_system),
                        status_label(status),
                        application::Message::ShowCategory(ingredient.category),
                    )
                },
            ))
            .into()
        };
        card(Some("Ingredients"), content)
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
            Message::SetUnits(units) => {
                self.unit_system = units;
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
#[cfg(test)]
impl Recipes {
    /// The recipe shown in the detail panel.
    pub fn selected(&self) -> Option<RecipeID> {
        self.selected
    }
}

fn recipes_msg(msg: Message) -> application::Message {
    application::Message::Recipes(msg)
}
fn editor_msg(msg: EditorMessage<RecipeMessage>) -> application::Message {
    recipes_msg(Message::Editor(msg))
}
/// How an ingredient's stock status reads on its row.
fn status_label(status: IngredientStatus) -> (&'static str, Tone) {
    match status {
        IngredientStatus::InStock => ("In stock", Tone::Neutral),
        IngredientStatus::Short => ("Short", Tone::Warning),
        IngredientStatus::NoneInStock => ("None in stock", Tone::Danger),
    }
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
        let mut screen = Recipes::new(&world.config, &world.categories, UnitSystem::Metric);
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
        let mut screen = Recipes::new(&world.config, &world.categories, UnitSystem::Metric);
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
        let mut screen = Recipes::new(&world.config, &world.categories, UnitSystem::Metric);

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
        let mut screen = Recipes::new(&world.config, &world.categories, UnitSystem::Metric);
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
        let mut screen = Recipes::new(&world.config, &world.categories, UnitSystem::Metric);

        let command = screen.update(Message::DeleteRecipe(id), &world.recipes);
        assert_reloads(&apply(&mut world, command.unwrap()));

        assert_eq!(world.recipes.recipe_count(), 0);
    }

    #[test]
    fn set_units_changes_the_unit_system() {
        let world = World::new();
        let mut screen = Recipes::new(&world.config, &world.categories, UnitSystem::Metric);
        screen.update(Message::SetUnits(UnitSystem::Imperial), &world.recipes);
        assert_eq!(screen.unit_system, UnitSystem::Imperial);
    }

    #[test]
    fn reload_ends_editing_and_keeps_a_live_selection() {
        let mut world = World::new();
        let id = world.add_recipe("Draft", &[]);
        let mut screen = Recipes::new(&world.config, &world.categories, UnitSystem::Metric);
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
        let mut screen = Recipes::new(&world.config, &world.categories, UnitSystem::Metric);
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
        let mut screen = Recipes::new(&world.config, &world.categories, UnitSystem::Metric);
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

    #[test]
    fn statuses_read_as_labels_in_their_tone() {
        assert_eq!(
            status_label(IngredientStatus::InStock),
            ("In stock", Tone::Neutral)
        );
        assert_eq!(
            status_label(IngredientStatus::Short),
            ("Short", Tone::Warning)
        );
        assert_eq!(
            status_label(IngredientStatus::NoneInStock),
            ("None in stock", Tone::Danger)
        );
    }
}
