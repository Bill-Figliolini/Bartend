pub(super) mod categories;
pub(super) mod inventory;
pub(super) mod recipes;
pub(super) mod serving;
pub(super) mod settings;

use iced::{Element, Task};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::Config,
    presentation::{
        application::{Context, Message},
        screen::{recipes::Recipes, serving::Serving},
    },
};

//Screen holders
#[derive(Debug)]
pub enum Screen {
    Inventory(Box<inventory::Inventory>),
    Settings(settings::Settings),
    Categories(categories::Categories),
    Recipes(Recipes),
    Serving(Serving),
}

//Screen Labels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenKind {
    Inventory,
    Settings,
    Categories,
    Recipes,
    Serving,
}

//Sum of module commands
pub enum ScreenCommand {
    Inventory(inventory::Command),
    Settings(settings::Command),
    Categories(categories::Command),
    Recipes(recipes::Command),
    Serving(serving::Command),
}
impl ScreenCommand {
    pub fn apply(self, ctx: &mut Context<'_>) -> Task<Message> {
        match self {
            ScreenCommand::Inventory(cmd) => cmd.apply(ctx),
            ScreenCommand::Settings(cmd) => cmd.apply(ctx),
            ScreenCommand::Categories(cmd) => cmd.apply(ctx),
            ScreenCommand::Recipes(cmd) => cmd.apply(ctx),
            ScreenCommand::Serving(cmd) => cmd.apply(ctx),
        }
    }
}

impl Screen {
    pub fn start(config: &Config, categories: &CategoryService) -> Self {
        let inventory = inventory::Inventory::new(config, categories);
        Self::Inventory(Box::new(inventory))
    }

    pub fn kind(&self) -> ScreenKind {
        match self {
            Self::Inventory(_) => ScreenKind::Inventory,
            Self::Settings(_) => ScreenKind::Settings,
            Self::Categories(_) => ScreenKind::Categories,
            Self::Recipes(_) => ScreenKind::Recipes,
            Self::Serving(_) => ScreenKind::Serving,
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(
        &self,
        item_service: &ItemService,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Element<'_, Message> {
        match self {
            Self::Inventory(inventory) => inventory.view(item_service, category_service),
            Self::Settings(settings) => settings.view(),
            Self::Categories(categories) => categories.view(category_service),
            Self::Recipes(recipes) => recipes.view(category_service, recipe_service),
            Self::Serving(serving) => serving.view(category_service),
        }
    }

    pub fn update(
        &mut self,
        item_service: &ItemService,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
        message: Message,
    ) -> Option<ScreenCommand> {
        match (self, message) {
            (Self::Inventory(inventory), Message::Inventory(message)) => inventory
                .update(item_service, message)
                .map(ScreenCommand::Inventory),
            (Self::Settings(settings), Message::Settings(message)) => {
                settings.update(message).map(ScreenCommand::Settings)
            }
            (Self::Categories(categories), Message::Categories(message)) => categories
                .update(message, category_service)
                .map(ScreenCommand::Categories),
            (Self::Recipes(recipes), Message::Recipes(message)) => recipes
                .update(message, recipe_service)
                .map(ScreenCommand::Recipes),
            (Self::Serving(service), Message::Serving(message)) => service
                .update(item_service, category_service, message)
                .map(ScreenCommand::Serving),
            //A message meant for a screen that is no longer active (e.g. an async
            //task, like the DB file picker, resolving after the user navigated away).
            _ => None,
        }
    }

    //The message that refreshes the active screen after a Command has been applied.
    pub fn reload_message(&self, config: &Config) -> Message {
        match self {
            Screen::Inventory(_) => Message::Inventory(inventory::Message::Reload),
            Screen::Settings(_) => {
                Message::Settings(settings::Message::ResetConfig(config.clone()))
            }
            Screen::Categories(_) => Message::Categories(categories::Message::Reload),
            Screen::Recipes(_) => Message::Recipes(recipes::Message::Reload),
            Screen::Serving(_) => Message::Serving(serving::Message::Reload),
        }
    }

    pub fn inventory(config: &Config, category_service: &CategoryService) -> Self {
        let inventory = inventory::Inventory::new(config, category_service);
        Self::Inventory(Box::new(inventory))
    }

    pub fn settings(current_config: &Config) -> Self {
        Self::Settings(settings::Settings::new(current_config))
    }
    pub fn categories(config: &Config) -> Self {
        Self::Categories(categories::Categories::new(config))
    }
    pub fn recipes(config: &Config, category_service: &CategoryService) -> Self {
        Screen::Recipes(Recipes::new(config, category_service))
    }
    pub fn serving(config: &Config, recipe_service: &RecipeService) -> Self {
        Screen::Serving(Serving::new(config, recipe_service))
    }
}

/// Shared fixtures for testing screens' `update` and `Command::apply`.
#[cfg(test)]
pub(in crate::presentation) mod test_support {
    use std::{
        pin::Pin,
        task::{Context as PollContext, Poll, Waker},
    };

    use iced::Task;
    use iced_runtime::Action;
    use rusqlite::Connection;
    use tempfile::TempDir;

    use crate::{
        logic::{CategoryService, ItemService, RecipeService},
        models::{
            CategoryBody, CategoryID, Config, Ingredient, ItemBody, ItemID, Quantity, RecipeBody,
            RecipeID, Unit,
        },
        persistence::{Database, repositories::CategoryRepository},
        presentation::application::{Context, Message},
    };

    /// The app state a Command applies to: real services over an in-memory
    /// database, and a config saved in a throwaway directory.
    pub struct World {
        pub dir: TempDir,
        pub config: Config,
        pub database: Database,
        pub items: ItemService,
        pub categories: CategoryService,
        pub recipes: RecipeService,
    }

    impl World {
        pub fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let config = Config::load(
                Some(dir.path().join("config.json")),
                Some(dir.path().join("bartend.db")),
            )
            .unwrap();
            let database = Database::new(Connection::open_in_memory().unwrap()).unwrap();
            let items = ItemService::new(&database.item_db()).unwrap();
            let categories = CategoryService::new(&database.category_db()).unwrap();
            let recipes = RecipeService::new(&database.recipe_db()).unwrap();
            Self {
                dir,
                config,
                database,
                items,
                categories,
                recipes,
            }
        }

        pub fn ctx(&mut self) -> Context<'_> {
            Context {
                database: &mut self.database,
                item_service: &mut self.items,
                category_service: &mut self.categories,
                recipe_service: &mut self.recipes,
                config: &mut self.config,
            }
        }

        pub fn add_item(&mut self, name: &str, milliliters: f32) -> ItemID {
            let body = ItemBody {
                name: name.to_string(),
                quantity: Quantity::new(milliliters, Unit::Milliliter),
            };
            self.items.add(&self.database.item_db(), &body).unwrap()
        }

        pub fn add_category(&mut self, name: &str) -> CategoryID {
            let body = CategoryBody {
                name: name.to_string(),
            };
            self.categories
                .insert(&self.database.category_db(), &body)
                .unwrap()
        }

        pub fn map_item(&mut self, item: ItemID, category: CategoryID) {
            self.categories
                .add_item_mapping(&self.database.category_db(), item, category)
                .unwrap();
        }

        /// A recipe using `milliliters` of each category, in order.
        pub fn add_recipe(&mut self, name: &str, ingredients: &[(CategoryID, f32)]) -> RecipeID {
            let body = recipe_body(name, ingredients);
            self.recipes.add(&self.database.recipe_db(), body).unwrap()
        }

        /// The database's own view of a category, bypassing the service cache.
        pub fn stored_category_names(&self) -> Vec<String> {
            let mut names: Vec<String> = self
                .database
                .category_db()
                .get_all()
                .unwrap()
                .into_values()
                .map(|body| body.name)
                .collect();
            names.sort();
            names
        }
    }

    pub fn recipe_body(name: &str, ingredients: &[(CategoryID, f32)]) -> RecipeBody {
        RecipeBody {
            name: name.to_string(),
            ingredients: ingredients
                .iter()
                .map(|&(category, milliliters)| Ingredient {
                    category,
                    quantity: Quantity::new(milliliters, Unit::Milliliter),
                })
                .collect(),
        }
    }

    /// Runs a Task to completion and returns the messages it produces.
    ///
    /// Commands only return `Task::none`, `Task::done` or batches of them.
    /// iced makes every task yield once before its output, so a few `Pending`
    /// polls are expected; a task that keeps pending is a test bug.
    pub fn drain(task: Task<Message>) -> Vec<Message> {
        const MAX_PENDING: usize = 100;
        let Some(mut stream) = iced_runtime::task::into_stream(task) else {
            return Vec::new();
        };
        let mut cx = PollContext::from_waker(Waker::noop());
        let mut messages = Vec::new();
        let mut pending = 0;
        loop {
            match Pin::as_mut(&mut stream).poll_next(&mut cx) {
                Poll::Ready(Some(Action::Output(message))) => messages.push(message),
                Poll::Ready(Some(_)) => {}
                Poll::Ready(None) => return messages,
                Poll::Pending => {
                    pending += 1;
                    assert!(pending < MAX_PENDING, "task did not complete");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{test_support::World, *};
    use crate::{
        models::{ItemID, RecipeID, UnitSystem},
        presentation::screen::test_support::drain,
    };

    fn screens(world: &World) -> Vec<Screen> {
        vec![
            Screen::inventory(&world.config, &world.categories),
            Screen::settings(&world.config),
            Screen::categories(&world.config),
            Screen::recipes(&world.config, &world.categories),
            Screen::serving(&world.config, &world.recipes),
        ]
    }

    fn update(world: &World, screen: &mut Screen, message: Message) -> Option<ScreenCommand> {
        screen.update(&world.items, &world.categories, &world.recipes, message)
    }

    #[test]
    fn app_starts_on_inventory() {
        let world = World::new();
        let screen = Screen::start(&world.config, &world.categories);
        assert_eq!(screen.kind(), ScreenKind::Inventory);
    }

    #[test]
    fn each_constructor_builds_its_kind() {
        let world = World::new();
        let kinds: Vec<ScreenKind> = screens(&world).iter().map(Screen::kind).collect();
        assert_eq!(
            kinds,
            vec![
                ScreenKind::Inventory,
                ScreenKind::Settings,
                ScreenKind::Categories,
                ScreenKind::Recipes,
                ScreenKind::Serving,
            ]
        );
    }

    #[test]
    fn messages_route_to_the_active_screen() {
        let world = World::new();
        let mut screens = screens(&world);
        let messages = [
            Message::Inventory(inventory::Message::DeleteItem(ItemID(1))),
            Message::Settings(settings::Message::Save),
            Message::Categories(categories::Message::DeleteCategory(
                crate::models::CategoryID(1),
            )),
            Message::Recipes(recipes::Message::DeleteRecipe(RecipeID(1))),
        ];
        for (screen, message) in screens.iter_mut().zip(messages) {
            let command = update(&world, screen, message);
            let routed = matches!(
                (screen.kind(), &command),
                (ScreenKind::Inventory, Some(ScreenCommand::Inventory(_)))
                    | (ScreenKind::Settings, Some(ScreenCommand::Settings(_)))
                    | (ScreenKind::Categories, Some(ScreenCommand::Categories(_)))
                    | (ScreenKind::Recipes, Some(ScreenCommand::Recipes(_)))
            );
            assert!(routed, "{:?} did not produce its command", screen.kind());
        }
        // An empty serving form produces no command, but still routes.
        let serving = &mut screens[4];
        let swap = Message::Serving(serving::Message::SwapUnits);
        assert!(update(&world, serving, swap).is_none());
    }

    #[test]
    fn messages_for_another_screen_are_ignored() {
        let world = World::new();
        let mut screen = Screen::categories(&world.config);
        let stale = Message::Inventory(inventory::Message::DeleteItem(ItemID(1)));
        assert!(update(&world, &mut screen, stale).is_none());
    }

    #[test]
    fn reload_message_targets_the_active_screen() {
        let world = World::new();
        for screen in screens(&world) {
            let reload = screen.reload_message(&world.config);
            let matches_screen = matches!(
                (screen.kind(), &reload),
                (
                    ScreenKind::Inventory,
                    Message::Inventory(inventory::Message::Reload)
                ) | (
                    ScreenKind::Settings,
                    Message::Settings(settings::Message::ResetConfig(_))
                ) | (
                    ScreenKind::Categories,
                    Message::Categories(categories::Message::Reload)
                ) | (
                    ScreenKind::Recipes,
                    Message::Recipes(recipes::Message::Reload)
                ) | (
                    ScreenKind::Serving,
                    Message::Serving(serving::Message::Reload)
                )
            );
            assert!(matches_screen, "{:?} got {reload:?}", screen.kind());
        }
    }

    #[test]
    fn screen_commands_apply_their_inner_command() {
        let mut world = World::new();
        let item = world.add_item("Tanqueray", 750.0);
        let category = world.add_category("Gin");
        let recipe = world.add_recipe("Draft", &[]);
        let config = world.config.clone();
        let commands = vec![
            ScreenCommand::Inventory(inventory::Command::Delete(item)),
            ScreenCommand::Categories(categories::Command::DeleteCategory(category)),
            ScreenCommand::Recipes(recipes::Command::Delete(recipe)),
            ScreenCommand::Serving(serving::Command::UseItems(Vec::new())),
            ScreenCommand::Settings(settings::Command::UpdateConfig(config)),
        ];
        for command in commands {
            let messages = drain(command.apply(&mut world.ctx()));
            assert!(
                matches!(messages.as_slice(), [Message::ReloadScreen]),
                "{messages:?}"
            );
        }
        assert_eq!(world.items.item_count(), 0);
        assert_eq!(world.categories.category_count(), 0);
        assert_eq!(world.recipes.recipe_count(), 0);
        assert_eq!(world.config.default_units(), UnitSystem::Metric);
    }
}
