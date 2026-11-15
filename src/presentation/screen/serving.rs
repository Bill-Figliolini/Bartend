use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Task,
    widget::{button, column, container, row, space, text},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService, totals_by_key},
    models::{Config, ItemUse, Quantity, Recipe, UnitSystem},
    presentation::{
        application::{self, Context},
        constants,
        input_handling::{ServingInput, ServingMessage, format_number, show_quantity},
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
    /// A serving has been taken out of stock.
    Served(Served),
    DismissNotice,
}

pub(in crate::presentation) enum Command {
    /// Takes `uses` out of stock, then reports `served`.
    UseItems { uses: Vec<ItemUse>, served: Served },
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
            .map(|(id, quantity)| {
                let name = item_service
                    .get(id)
                    .map_or_else(|_| String::new(), |item| item.name.clone());
                (name, quantity)
            })
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
            .map(|(name, quantity)| format!("{} {name}", show_quantity(*quantity, units)))
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
    /// The recipes on the left; choosing one fills the right with the
    /// number of servings, and a table of its ingredients with an item
    /// picker for each.
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

        let form: Element<'_, _> = match self.input.recipe_name() {
            None => text("Choose a recipe from the list to serve it.").into(),
            Some(name) => {
                let header = row![
                    text(name).size(24),
                    space().width(Fill),
                    self.input.servings_view().map(input_msg),
                ]
                .spacing(constants::SPACING)
                .align_y(Center);
                let table = self.input.table_view(category_service).map(input_msg);
                let save = self.input.servable().then(|| serving_msg(Message::Save));
                let serve = row![space().width(Fill), button("Serve").on_press_maybe(save)];
                column![header, table, serve].spacing(16).into()
            }
        };
        let notice = self.notice.as_ref().map(|served| self.notice_view(served));
        let detail = column![form, notice].spacing(16);

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

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn notice_view(&self, served: &Served) -> Element<'_, application::Message> {
        let dismiss = button("Dismiss")
            .style(button::text)
            .on_press(serving_msg(Message::DismissNotice));
        let message = text(served.describe(self.input.units())).style(text::success);
        container(row![message, space().width(Fill), dismiss].align_y(Center))
            .padding(constants::PADDING)
            .width(Fill)
            .style(container::rounded_box)
            .into()
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

fn serving_msg(msg: Message) -> application::Message {
    application::Message::Serving(msg)
}

fn input_msg(msg: ServingMessage) -> application::Message {
    serving_msg(Message::Input(msg))
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
    fn swap_units_toggles_the_unit_system() {
        let world = World::new();
        let mut screen = Serving::new(&world.config);
        update(&mut screen, &world, Message::SwapUnits);
        assert_eq!(screen.input.units(), UnitSystem::Imperial);
    }
}
