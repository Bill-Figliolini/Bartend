use iced::{
    Alignment::Center,
    Element,
    Length::{Fill, Shrink},
    Task,
    widget::{button, column, container, row, scrollable, space, text},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService, can_make, totals_by_key},
    models::{
        CategoryID, Config, ItemUse, Quantity, Recipe, RecipeBody, UnitSystem, format_number,
    },
    presentation::{
        application::{self, Context},
        constants,
        input_handling::{ServingInput, ServingMessage},
        screen::{
            Panels,
            shared::{Availability, page_header, volume_figure},
        },
        widget::{
            detail::{Tone, card, control_tile, hint, stat_tile, toned_card},
            footer::footer,
            master_list::{Entry, master_list},
        },
    },
};

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {
    Reload,
    SwapUnits,
    Input(ServingMessage),
    Save,
    /// A serving has been taken out of stock.
    Served(Served),
    DismissNotice,
}

pub(in crate::presentation) enum Command {
    /// Takes `uses` out of stock, then reports `served`.
    UseItems { uses: Vec<ItemUse>, served: Served },
    /// Opens the category on the Categories screen.
    ShowCategory(CategoryID),
}
impl Command {
    pub fn apply(self, ctx: &mut Context<'_>) -> Task<application::Message> {
        match self {
            Command::UseItems { uses, served } => {
                match ctx.item_service.use_items(&ctx.database.item_db(), uses) {
                    Ok(()) => Task::done(serving_msg(Message::Served(served)))
                        .chain(Task::done(application::Message::ReloadScreen)),
                    Err(e) => Task::done(application::Message::Error(e)),
                }
            }
            Command::ShowCategory(id) => Task::done(application::Message::ShowCategory(id)),
        }
    }
}

/// What a serving used, for the notice shown once it is done.
#[derive(Debug, Clone, Default, PartialEq)]
pub(in crate::presentation) struct Served {
    recipe: String,
    servings: f32,
    /// Each item's name, and how much was used of it in total.
    used: Vec<(String, Quantity)>,
}

impl Served {
    fn new(recipe: &str, servings: f32, item_uses: &[ItemUse], item_service: &ItemService) -> Self {
        let by_item = item_uses.iter().map(|usage| (usage.id, usage.quantity));
        // The form only offers an item for ingredients of its own kind of
        // quantity, so the totals always add up.
        let used = totals_by_key(by_item)
            .unwrap_or_default()
            .into_iter()
            .map(|(id, quantity)| (item_service.name_of(id), quantity))
            .collect();
        Self {
            recipe: recipe.to_string(),
            servings,
            used,
        }
    }

    /// E.g. "Served Negroni ×2: 60 ml Tanqueray, 60 ml Campari".
    fn describe(&self, units: UnitSystem) -> String {
        let times = if (self.servings - 1.0).abs() < f32::EPSILON {
            String::new()
        } else {
            format!(" ×{}", format_number(self.servings))
        };
        let used: Vec<String> = self
            .used
            .iter()
            .map(|(name, quantity)| format!("{} {name}", quantity.readable(units)))
            .collect();
        if used.is_empty() {
            format!("Served {}{times}", self.recipe)
        } else {
            format!("Served {}{times}: {}", self.recipe, used.join(", "))
        }
    }
}

#[derive(Debug)]
pub(in crate::presentation) struct Serving {
    input: ServingInput,
    /// What the last serving used, until it is dismissed or another recipe
    /// is chosen.
    notice: Option<Served>,
}

impl Serving {
    pub fn new(config: &Config) -> Self {
        Self {
            input: ServingInput::new(config.default_units()),
            notice: None,
        }
    }
    /// The recipes on the left, faded if there is not enough in stock to
    /// make one. Choosing one fills the right with its name, whether it can
    /// be made, and Serve; then tiles for the number of servings and the
    /// drink volume, and a table of its ingredients with an item picker for
    /// each.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(
        &self,
        item_service: &ItemService,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Panels<'_> {
        let chosen = self.input.recipe_id();
        let entries = recipe_service.sorted_ids().into_iter().map(|id| {
            let recipe = Recipe {
                id,
                body: recipe_service.get(id).unwrap_or_default(),
            };
            Entry {
                label: recipe.body.name.clone(),
                selected: chosen == Some(id),
                dimmed: !can_make(&recipe.body, item_service, category_service),
                on_press: serving_msg(Message::Input(ServingMessage::Recipe(recipe))),
            }
        });
        let master = master_list(None, entries);

        let detail = match self.input.recipe() {
            None => Self::empty_view(),
            Some(recipe) => self.recipe_view(recipe, category_service),
        };

        let unit_swap_button = button(text(self.input.units().to_string()))
            .style(button::secondary)
            .on_press(serving_msg(Message::SwapUnits));
        let footer = footer(
            container(row![space().width(Fill), unit_swap_button].spacing(constants::SPACING))
                .center_y(Fill),
        );
        let detail = column![container(detail).height(Fill), footer].into();

        Panels { master, detail }
    }

    /// What the panel shows before a recipe is chosen.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn empty_view<'a>() -> Element<'a, application::Message> {
        let note = hint("Choose a recipe from the list. Faded recipes are missing ingredients.");
        page_header("Serve a drink".to_string(), note.into(), row![].into())
    }

    /// The chosen recipe: its header with Serve, any notice of what was
    /// just served, the servings and drink volume tiles, then the pours in
    /// a card.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn recipe_view<'a>(
        &'a self,
        recipe: &'a Recipe,
        category_service: &CategoryService,
    ) -> Element<'a, application::Message> {
        let save = self.input.servable().then(|| serving_msg(Message::Save));
        let serve = button("Serve").on_press_maybe(save);
        let view_recipe = button("View recipe")
            .style(button::secondary)
            .on_press(application::Message::ShowRecipe(recipe.id));
        let header = page_header(
            recipe.body.name.clone(),
            availability(&recipe.body, self.input.ready()).view(),
            row![serve, view_recipe].spacing(constants::SPACING).into(),
        );

        let notice = self.notice.as_ref().map(|served| self.notice_view(served));

        // A fixed-width tile with a filling height is laid out after the
        // others, so it takes the volume tile's height wherever it sits
        // (see `stat_tile`). The volume tile always has a line under its
        // figure while the servings show an error, so it is never the
        // shorter one.
        let servings_tile = control_tile("Servings", self.input.servings_view().map(input_msg))
            .width(SERVINGS_TILE_WIDTH)
            .height(Fill);
        let (volume, below) = drink_volume(&recipe.body, self.input.servings(), self.input.units());
        let volume_tile = stat_tile(
            "Drink volume",
            volume,
            below.map(|below| hint(below).into()),
        );
        let tiles = row![servings_tile, volume_tile]
            .spacing(constants::SPACING)
            .height(Shrink);

        let pours = card(
            Some("Pours"),
            self.input.table_view(category_service).map(input_msg),
        );

        let content = column![header]
            .push(notice)
            .push(tiles)
            .push(pours)
            .spacing(16);
        scrollable(content).into()
    }

    /// What the last serving used, in a success card with Dismiss.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn notice_view(&self, served: &Served) -> Element<'_, application::Message> {
        let dismiss = button("Dismiss")
            .style(button::secondary)
            .on_press(serving_msg(Message::DismissNotice));
        let message = text(served.describe(self.input.units()));
        toned_card(
            Tone::Success,
            row![message, space().width(Fill), dismiss]
                .spacing(constants::SPACING)
                .align_y(Center),
        )
    }

    pub fn update(
        &mut self,
        item_service: &ItemService,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
        message: Message,
    ) -> Option<Command> {
        match message {
            // Keeps the form, so the same picks can be served again.
            Message::Reload => {
                self.input
                    .refresh(recipe_service, item_service, category_service);
                None
            }
            Message::SwapUnits => {
                self.input.swap_units();
                None
            }
            Message::Input(ServingMessage::ShowCategory(id)) => Some(Command::ShowCategory(id)),
            Message::Input(input_message) => {
                if matches!(input_message, ServingMessage::Recipe(_)) {
                    self.notice = None;
                }
                self.input
                    .update(input_message, item_service, category_service);
                None
            }
            Message::Save => {
                // A failed serving must not leave the last one's notice up.
                self.notice = None;
                let uses = self.input.output().ok()?;
                let recipe = self.input.recipe_name().unwrap_or_default();
                let servings = self.input.servings().unwrap_or(1.0);
                let served = Served::new(recipe, servings, &uses, item_service);
                Some(Command::UseItems { uses, served })
            }
            Message::Served(served) => {
                self.notice = Some(served);
                None
            }
            Message::DismissNotice => {
                self.notice = None;
                None
            }
        }
    }
}

/// Width of the servings tile: room for the stepper and its error.
const SERVINGS_TILE_WIDTH: f32 = 240.0;

/// Whether `recipe` can be made: `ready` when its ingredients are all in
/// stock for the servings chosen (see [`ServingInput::ready`]).
fn availability(recipe: &RecipeBody, ready: bool) -> Availability {
    if recipe.ingredients.is_empty() {
        Availability::Draft
    } else if ready {
        Availability::Ready
    } else {
        Availability::Missing
    }
}

/// The drink volume tile's figure and the line under it: the recipe's
/// volume for `servings`, or a dash and a note while they are not a valid
/// number.
fn drink_volume(
    recipe: &RecipeBody,
    servings: Option<f32>,
    units: UnitSystem,
) -> (String, Option<String>) {
    match servings {
        Some(servings) => volume_figure(&recipe.volume_summary().scaled(servings), units),
        None => (
            "—".to_string(),
            Some("Enter a number of servings above 0".to_string()),
        ),
    }
}

fn serving_msg(msg: Message) -> application::Message {
    application::Message::Serving(msg)
}

fn input_msg(msg: ServingMessage) -> application::Message {
    serving_msg(Message::Input(msg))
}

#[cfg(test)]
impl Serving {
    /// The recipe chosen to serve.
    pub fn chosen_recipe(&self) -> Option<crate::models::RecipeID> {
        self.input.recipe_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        logic::LogicError,
        models::{BartendError, Item, ItemID, Quantity, Recipe, RecipeID, Unit},
        presentation::screen::test_support::{World, drain},
    };

    /// A world with one gin, and a recipe calling for 50ml of it.
    fn gin_world() -> (World, ItemID, RecipeID) {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let tanqueray = world.add_item("Tanqueray", 750.0);
        world.map_item(tanqueray, gin);
        let shot = world.add_recipe("Gin Shot", &[(gin, 50.0)]);
        (world, tanqueray, shot)
    }

    fn ml(amount: f32) -> Quantity {
        Quantity::new(amount, Unit::Milliliter)
    }

    fn update(screen: &mut Serving, world: &World, message: Message) -> Option<Command> {
        screen.update(&world.items, &world.categories, &world.recipes, message)
    }

    fn send(screen: &mut Serving, world: &World, input: ServingMessage) {
        assert!(update(screen, world, Message::Input(input)).is_none());
    }

    fn select_recipe(screen: &mut Serving, world: &World, id: RecipeID) {
        let recipe = Recipe {
            id,
            body: world.recipes.get(id).unwrap(),
        };
        send(screen, world, ServingMessage::Recipe(recipe));
    }

    fn select_item(screen: &mut Serving, world: &World, index: usize, id: ItemID) {
        let item = Item {
            id,
            body: world.items.get(id).unwrap().clone(),
        };
        let key = screen.input.pour_key(index, 0);
        send(screen, world, ServingMessage::Item(key, item));
    }

    fn save(screen: &mut Serving, world: &World) -> Option<Command> {
        update(screen, world, Message::Save)
    }

    /// Serves the chosen recipe, and hands the resulting messages back to
    /// the screen, as the app would.
    fn serve(screen: &mut Serving, world: &mut World) {
        let command = save(screen, world).expect("serving should be complete");
        for message in drain(command.apply(&mut world.ctx())) {
            let message = match message {
                application::Message::Serving(message) => message,
                application::Message::ReloadScreen => Message::Reload,
                other => panic!("unexpected {other:?}"),
            };
            assert!(update(screen, world, message).is_none());
        }
    }

    fn served(recipe: &str, servings: f32, used: &[(&str, Quantity)]) -> Served {
        Served {
            recipe: recipe.to_string(),
            servings,
            used: used
                .iter()
                .map(|&(name, quantity)| (name.to_string(), quantity))
                .collect(),
        }
    }

    #[test]
    fn serving_a_recipe_uses_up_its_items_then_reports_it() {
        let (mut world, tanqueray, shot) = gin_world();
        let mut screen = Serving::new(&world.config);
        select_recipe(&mut screen, &world, shot);

        let command = save(&mut screen, &world).expect("the only gin is picked for you");
        let messages = drain(command.apply(&mut world.ctx()));

        let expected = served("Gin Shot", 1.0, &[("Tanqueray", ml(50.0))]);
        assert!(
            matches!(
                messages.as_slice(),
                [
                    application::Message::Serving(Message::Served(served)),
                    application::Message::ReloadScreen
                ] if *served == expected
            ),
            "{messages:?}"
        );
        assert_eq!(world.items.get(tanqueray).unwrap().quantity, ml(700.0));
    }

    #[test]
    fn saving_an_incomplete_serving_does_nothing() {
        let (mut world, tanqueray, shot) = gin_world();
        // A second gin, so neither is picked automatically.
        let gin = world.categories.item_category(tanqueray).unwrap();
        let beefeater = world.add_item("Beefeater", 750.0);
        world.map_item(beefeater, gin);
        let mut screen = Serving::new(&world.config);
        assert!(save(&mut screen, &world).is_none());

        select_recipe(&mut screen, &world, shot);
        assert!(save(&mut screen, &world).is_none());

        select_item(&mut screen, &world, 0, beefeater);
        assert!(save(&mut screen, &world).is_some());
    }

    #[test]
    fn using_an_unknown_item_reports_error() {
        let mut world = World::new();
        let ghost = ItemUse {
            id: ItemID(99),
            quantity: ml(50.0),
        };
        let command = Command::UseItems {
            uses: vec![ghost],
            served: Served::default(),
        };
        let messages = drain(command.apply(&mut world.ctx()));
        assert!(
            matches!(
                messages.as_slice(),
                [application::Message::Error(BartendError::Logic(
                    LogicError::InvalidItem(ItemID(99))
                ))]
            ),
            "{messages:?}"
        );
    }

    #[test]
    fn after_serving_the_form_is_kept_for_another_round() {
        let (mut world, tanqueray, shot) = gin_world();
        let mut screen = Serving::new(&world.config);
        select_recipe(&mut screen, &world, shot);
        select_item(&mut screen, &world, 0, tanqueray);

        for left in [700.0, 650.0] {
            serve(&mut screen, &mut world);
            assert_eq!(world.items.get(tanqueray).unwrap().quantity, ml(left));
        }
        assert_eq!(screen.input.recipe_id(), Some(shot));
        assert!(screen.notice.is_some());
    }

    #[test]
    fn the_notice_lasts_until_dismissed_another_recipe_is_chosen_or_serving_again() {
        let (mut world, _, shot) = gin_world();
        let mut screen = Serving::new(&world.config);
        select_recipe(&mut screen, &world, shot);

        serve(&mut screen, &mut world);
        let expected = served("Gin Shot", 1.0, &[("Tanqueray", ml(50.0))]);
        assert_eq!(screen.notice, Some(expected));
        update(&mut screen, &world, Message::DismissNotice);
        assert_eq!(screen.notice, None);

        serve(&mut screen, &mut world);
        select_recipe(&mut screen, &world, shot);
        assert_eq!(screen.notice, None);

        serve(&mut screen, &mut world);
        send(
            &mut screen,
            &world,
            ServingMessage::Servings("0".to_string()),
        );
        assert!(save(&mut screen, &world).is_none());
        assert_eq!(screen.notice, None, "a failed serving clears it too");
    }

    #[test]
    fn the_notice_totals_each_item_and_shows_the_servings() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let tanqueray = world.add_item("Tanqueray", 750.0);
        world.map_item(tanqueray, gin);
        let uses = [
            ItemUse {
                id: tanqueray,
                quantity: ml(45.0),
            },
            ItemUse {
                id: tanqueray,
                quantity: ml(30.0),
            },
        ];

        let notice = Served::new("Double Gin", 1.5, &uses, &world.items);

        assert_eq!(
            notice,
            served("Double Gin", 1.5, &[("Tanqueray", ml(75.0))])
        );
        assert_eq!(
            notice.describe(UnitSystem::Metric),
            "Served Double Gin ×1.5: 75 ml Tanqueray"
        );
    }

    #[test]
    fn notices_read_naturally() {
        let one = served(
            "Negroni",
            1.0,
            &[("Tanqueray", ml(30.0)), ("Campari", ml(29.57))],
        );
        assert_eq!(
            one.describe(UnitSystem::Metric),
            "Served Negroni: 30 ml Tanqueray, 29.57 ml Campari"
        );
        assert_eq!(
            one.describe(UnitSystem::Imperial),
            "Served Negroni: 1.01 fl oz Tanqueray, 1 fl oz Campari"
        );
        assert_eq!(
            served("Draft", 2.0, &[]).describe(UnitSystem::Metric),
            "Served Draft ×2"
        );
    }

    #[test]
    fn a_category_link_opens_the_category_and_leaves_the_form_alone() {
        let (mut world, _, shot) = gin_world();
        let mut screen = Serving::new(&world.config);
        select_recipe(&mut screen, &world, shot);
        let gin = world.recipes.get(shot).unwrap().ingredients[0].category;

        let link = Message::Input(ServingMessage::ShowCategory(gin));
        let command = update(&mut screen, &world, link).expect("the link opens something");
        let messages = drain(command.apply(&mut world.ctx()));

        assert!(
            matches!(
                messages.as_slice(),
                [application::Message::ShowCategory(id)] if *id == gin
            ),
            "{messages:?}"
        );
        assert_eq!(screen.chosen_recipe(), Some(shot));
    }

    #[test]
    fn a_recipe_is_a_draft_without_ingredients_else_ready_or_missing() {
        let (world, _, shot) = gin_world();
        let body = world.recipes.get(shot).unwrap();
        assert_eq!(availability(&body, true), Availability::Ready);
        assert_eq!(availability(&body, false), Availability::Missing);
        let draft = RecipeBody {
            name: "Draft".to_string(),
            ingredients: Vec::new(),
        };
        assert_eq!(availability(&draft, true), Availability::Draft);
    }

    #[test]
    fn drink_volume_scales_with_the_servings() {
        let (world, _, shot) = gin_world();
        let body = world.recipes.get(shot).unwrap();
        assert_eq!(
            drink_volume(&body, Some(1.0), UnitSystem::Metric),
            ("50 ml".to_string(), None)
        );
        assert_eq!(
            drink_volume(&body, Some(2.5), UnitSystem::Metric),
            ("125 ml".to_string(), None)
        );
        let (figure, below) = drink_volume(&body, None, UnitSystem::Metric);
        assert_eq!(figure, "—");
        assert!(
            below.is_some(),
            "explains the dash, and keeps the tile tall"
        );
    }

    #[test]
    fn swap_units_toggles_the_unit_system() {
        let world = World::new();
        let mut screen = Serving::new(&world.config);
        update(&mut screen, &world, Message::SwapUnits);
        assert_eq!(screen.input.units(), UnitSystem::Imperial);
    }
}
