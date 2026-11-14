use iced::{
    Length::Fill,
    Task,
    widget::{button, column, container, row, space, text},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{Config, ItemUse, Recipe, UnitSystem},
    presentation::{
        application::{self, Context},
        constants,
        input_handling::{ServingInput, ServingMessage},
        screen::Panels,
        widget::{
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
}

pub(in crate::presentation) enum Command {
    UseItems(Vec<ItemUse>),
}
impl Command {
    pub fn apply(self, ctx: &mut Context<'_>) -> Task<application::Message> {
        match self {
            Command::UseItems(items) => {
                match ctx.item_service.use_items(&ctx.database.item_db(), items) {
                    Ok(()) => Task::done(application::Message::ReloadScreen),
                    Err(e) => Task::done(application::Message::Error(e)),
                }
            }
        }
    }
}

#[derive(Debug)]
pub(in crate::presentation) struct Serving {
    input: ServingInput,

    unit_system: UnitSystem,
}

impl Serving {
    pub fn new(config: &Config) -> Self {
        let input = ServingInput::new();
        let unit_system = config.default_units();
        Self { input, unit_system }
    }
    /// The recipes on the left; choosing one fills the right with an item
    /// picker for each of its ingredients.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(
        &self,
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
                on_press: serving_msg(Message::Input(ServingMessage::Recipe(recipe))),
            }
        });
        let master = master_list(None, entries);

        let unit_swap_button = button(text(self.unit_system.to_string()))
            .style(button::secondary)
            .on_press(serving_msg(Message::SwapUnits));
        let detail = match chosen {
            None => column![text("Choose a recipe from the list to serve it.")],
            Some(id) => {
                let name = text(recipe_service.get(id).unwrap_or_default().name).size(24);
                let pickers = self
                    .input
                    .view(category_service, self.unit_system)
                    .map(|msg| serving_msg(Message::Input(msg)));
                let save_button = button("Serve").on_press(serving_msg(Message::Save));
                column![name, pickers, save_button].spacing(16)
            }
        };
        let footer = footer(
            container(row![space().width(Fill), unit_swap_button].spacing(constants::SPACING))
                .center_y(Fill),
        );
        let detail = column![container(detail).height(Fill), footer].into();

        Panels { master, detail }
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
                self.unit_system.swap();
                None
            }
            Message::Input(input_message) => {
                self.input
                    .update(input_message, item_service, category_service);
                None
            }
            Message::Save => self.input.output().ok().map(Command::UseItems),
        }
    }
}

fn serving_msg(msg: Message) -> application::Message {
    application::Message::Serving(msg)
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

    #[test]
    fn serving_a_recipe_uses_up_its_items() {
        let (mut world, tanqueray, shot) = gin_world();
        let mut screen = Serving::new(&world.config);
        select_recipe(&mut screen, &world, shot);
        select_item(&mut screen, &world, 0, tanqueray);

        let command = save(&mut screen, &world).expect("serving should be complete");
        let messages = drain(command.apply(&mut world.ctx()));

        assert!(
            matches!(messages.as_slice(), [application::Message::ReloadScreen]),
            "{messages:?}"
        );
        assert_eq!(
            world.items.get(tanqueray).unwrap().quantity,
            Quantity::new(700.0, Unit::Milliliter)
        );
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
    }

    #[test]
    fn using_an_unknown_item_reports_error() {
        let mut world = World::new();
        let ghost = ItemUse {
            id: ItemID(99),
            quantity: Quantity::new(50.0, Unit::Milliliter),
        };
        let messages = drain(Command::UseItems(vec![ghost]).apply(&mut world.ctx()));
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
    fn after_serving_the_reload_keeps_the_picks_for_another_round() {
        let (mut world, tanqueray, shot) = gin_world();
        let mut screen = Serving::new(&world.config);
        select_recipe(&mut screen, &world, shot);
        select_item(&mut screen, &world, 0, tanqueray);

        for left in [700.0, 650.0] {
            let command = save(&mut screen, &world).expect("serving should be complete");
            drain(command.apply(&mut world.ctx()));
            assert!(update(&mut screen, &world, Message::Reload).is_none());
            assert_eq!(
                world.items.get(tanqueray).unwrap().quantity,
                Quantity::new(left, Unit::Milliliter)
            );
        }
        assert_eq!(screen.input.recipe_id(), Some(shot));
    }

    #[test]
    fn swap_units_toggles_the_unit_system() {
        let world = World::new();
        let mut screen = Serving::new(&world.config);
        update(&mut screen, &world, Message::SwapUnits);
        assert_eq!(screen.unit_system, UnitSystem::Imperial);
    }
}
