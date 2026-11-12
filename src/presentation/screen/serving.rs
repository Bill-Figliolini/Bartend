use iced::{
    Task,
    widget::{Container, button, column, row, text},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{Config, ItemUse, UnitSystem},
    presentation::{
        application::{self, Context},
        input_handling::{InputMessage, ServingInput, UnknownTarget},
        widget::footer::footer,
    },
};

#[derive(Debug, Clone)]
pub(in crate::presentation) enum Message {
    Reload,
    SwapUnits,
    Input(InputMessage),
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
    pub fn new(config: &Config, recipe_service: &RecipeService) -> Self {
        let input = ServingInput::new(input_msg, recipe_service);
        let unit_system = config.default_units();
        Self { input, unit_system }
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(
        &self,
        category_service: &CategoryService,
    ) -> iced::Element<'_, application::Message> {
        let save_button = button("Save").on_press(application::Message::Serving(Message::Save));
        let body = row![
            self.input.view(category_service, self.unit_system),
            save_button
        ];
        let unit_swap_button = button(text(self.unit_system.to_string()))
            .on_press(application::Message::Serving(Message::SwapUnits));
        let footer_content = row![unit_swap_button];
        let footer_container = Container::new(footer_content);
        let footer = footer(footer_container);

        column![body, footer].into()
    }
    pub fn update(
        &mut self,
        item_service: &ItemService,
        category_service: &CategoryService,
        message: Message,
    ) -> Option<Command> {
        match message {
            Message::Reload => {
                self.input.clear();
                None
            }
            Message::SwapUnits => {
                self.unit_system.swap();
                None
            }
            Message::Input(input_message) => {
                self.input
                    .update(input_message, item_service, category_service)
                    .unwrap_or_else(UnknownTarget::discard);
                None
            }
            Message::Save => {
                if let Ok(used_items) = self.input.output() {
                    Some(Command::UseItems(used_items))
                } else {
                    None
                }
            }
        }
    }
}

fn input_msg(msg: InputMessage) -> application::Message {
    application::Message::Serving(Message::Input(msg))
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

    fn send(screen: &mut Serving, world: &World, input: InputMessage) {
        let message = Message::Input(input);
        assert!(
            screen
                .update(&world.items, &world.categories, message)
                .is_none()
        );
    }

    fn select_recipe(screen: &mut Serving, world: &World, id: RecipeID) {
        let recipe = Recipe {
            id,
            body: world.recipes.get(id).unwrap(),
        };
        let recipe_id = screen.input.recipe_id();
        send(screen, world, InputMessage::Recipe(recipe_id, recipe));
    }

    fn select_item(screen: &mut Serving, world: &World, index: usize, id: ItemID) {
        let item = Item {
            id,
            body: world.items.get(id).unwrap().clone(),
        };
        let item_id = screen.input.item_id(index);
        send(screen, world, InputMessage::Item(item_id, item));
    }

    fn save(screen: &mut Serving, world: &World) -> Option<Command> {
        screen.update(&world.items, &world.categories, Message::Save)
    }

    #[test]
    fn serving_a_recipe_uses_up_its_items() {
        let (mut world, tanqueray, shot) = gin_world();
        let mut screen = Serving::new(&world.config, &world.recipes);
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
    fn saving_without_choosing_items_does_nothing() {
        let (world, _, shot) = gin_world();
        let mut screen = Serving::new(&world.config, &world.recipes);
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
    fn reload_clears_the_form() {
        let (world, tanqueray, shot) = gin_world();
        let mut screen = Serving::new(&world.config, &world.recipes);
        select_recipe(&mut screen, &world, shot);
        select_item(&mut screen, &world, 0, tanqueray);

        screen.update(&world.items, &world.categories, Message::Reload);

        assert!(save(&mut screen, &world).is_none());
    }

    #[test]
    fn swap_units_toggles_the_unit_system() {
        let world = World::new();
        let mut screen = Serving::new(&world.config, &world.recipes);
        screen.update(&world.items, &world.categories, Message::SwapUnits);
        assert_eq!(screen.unit_system, UnitSystem::Imperial);
    }

    #[test]
    fn stale_input_is_discarded() {
        let (world, tanqueray, _) = gin_world();
        let mut screen = Serving::new(&world.config, &world.recipes);
        let item = Item {
            id: tanqueray,
            body: world.items.get(tanqueray).unwrap().clone(),
        };
        send(
            &mut screen,
            &world,
            InputMessage::Item(iced::widget::Id::unique(), item),
        );
    }
}
