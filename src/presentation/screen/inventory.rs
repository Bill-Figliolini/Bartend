use iced::{
    Element,
    Length::{Fill, Shrink},
    Task,
    widget::{Column, Container, column, row, scrollable},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService, RecipeUse, StockLevel, recipes_using},
    models::{Category, CategoryFilter, CategoryID, Config, Item, ItemBody, ItemID, UnitSystem},
    presentation::{
        application::{self, Context},
        constants,
        input_handling::{Editor, EditorMessage, ItemInput, ItemMessage, Saved},
        screen::{
            Panels,
            shared::{
                breadcrumb_label, breadcrumb_line, counted, detail_header, form_page,
                reload_or_error, report_error,
            },
        },
        widget::{
            detail::{card, gauge, hint, link_row, stat_tile},
            master_list::{Entry, master_list},
        },
    },
};

#[derive(Debug)]
pub struct Inventory {
    editor: Editor<ItemInput, ItemID>,
    /// The item shown in the detail panel; `None` shows the new-item form.
    selected: Option<ItemID>,
    unit_system: UnitSystem,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Shows amounts in the given unit system, as picked in the top bar.
    SetUnits(UnitSystem),
    /// Shows the item in the detail panel, or the new-item form for `None`.
    /// Any edit in progress is discarded.
    Select(Option<ItemID>),
    /// Starts editing the item, which is in the given category, or cancels
    /// if it is already being edited.
    BeginEdit(ItemID, Option<Category>),
    Editor(EditorMessage<ItemMessage>),
    DeleteItem(ItemID),

    //Variants for Application's use
    Reload,
}

pub enum Command {
    Add(ItemBody, Option<CategoryID>),
    Update(Item, Option<CategoryID>),
    Delete(ItemID),
}
impl Command {
    pub fn apply(self, ctx: &mut Context<'_>) -> Task<application::Message> {
        match self {
            Command::Add(item_body, category_id) => {
                let item_id = match ctx.item_service.add(&ctx.database.item_db(), &item_body) {
                    Ok(id) => id,
                    Err(e) => return Task::done(application::Message::Error(e)),
                };
                let mapping = category_id.map_or(Ok(()), |category_id| {
                    ctx.category_service.add_item_mapping(
                        &ctx.database.category_db(),
                        item_id,
                        category_id,
                    )
                });
                Task::batch([
                    Task::done(application::Message::ReloadScreen),
                    report_error(mapping),
                ])
            }
            Command::Update(item, category_id) => {
                let item_id = item.id;
                let update = ctx.item_service.update(&ctx.database.item_db(), item);
                let mapping = ctx.category_service.update_item_mapping(
                    &ctx.database.category_db(),
                    item_id,
                    category_id,
                );
                Task::batch([reload_or_error(update), report_error(mapping)])
            }
            Command::Delete(id) => {
                reload_or_error(ctx.item_service.delete(&ctx.database.item_db(), id))
            }
        }
    }
}

impl Inventory {
    /// The unit system amounts are shown in.
    #[cfg(test)]
    pub fn units(&self) -> UnitSystem {
        self.unit_system
    }

    pub fn new(config: &Config, category_service: &CategoryService, units: UnitSystem) -> Self {
        let form = ItemInput::new(config, category_service.get_all(CategoryFilter {}));
        Self {
            editor: Editor::new(form, "Item"),
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
        let entries = item_service.sorted_ids().into_iter().map(|id| Entry {
            label: item_service.name_of(id),
            selected: self.selected == Some(id),
            on_press: inventory_msg(Message::Select(Some(id))),
            dimmed: false,
        });
        let new = (
            "+ New Item".to_string(),
            inventory_msg(Message::Select(None)),
        );
        let master = master_list(Some(new), entries, "No items yet");

        let detail = match self.selected {
            Some(item) if self.editor.editing().is_none() => {
                self.item_details(item, item_service, category_service, recipe_service)
            }
            _ => self.form_view(),
        };
        Panels { master, detail }
    }

    /// The read-only view of `item`: its name and category, tiles for its
    /// stock and how many recipes use it, then a list of those recipes.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn item_details(
        &self,
        item: ItemID,
        item_service: &ItemService,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Element<'_, application::Message> {
        let body = item_service.get(item).cloned().unwrap_or_default();
        let category = category_service.item_category(item).map(|id| Category {
            id,
            body: category_service.get(id).cloned().unwrap_or_default(),
        });
        let uses = recipes_using(item, item_service, category_service, recipe_service);

        let breadcrumb = category
            .as_ref()
            .and_then(|category| breadcrumb_label(category.id, category_service));
        // The stock tile always has a line under its figure, so it is the
        // taller one and sets the row's height.
        let tiles = row![self.stock_tile(&body), uses_tile(&uses).height(Fill)]
            .spacing(constants::SPACING)
            .height(Shrink);
        let used_in = self.used_in_card(&uses, category.is_some());
        let header = header(item, body.name, breadcrumb, category);

        scrollable(column![header, tiles, used_in].spacing(16)).into()
    }

    /// The new- or edit-item form, laid out like the item's details: the
    /// heading with Save and Cancel to its right, then the fields in a card.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn form_view(&self) -> Element<'_, application::Message> {
        let note = if self.editor.editing().is_some() {
            "Change any details, then Save."
        } else {
            "Add something you stock, then Save."
        };
        form_page(
            self.editor.title(),
            note,
            self.editor.buttons().map(editor_msg),
            self.editor.form_view().map(editor_msg),
        )
    }

    /// How much is in stock, with a gauge against the full size if the
    /// item has one.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn stock_tile(&self, body: &ItemBody) -> Element<'_, application::Message> {
        let units = self.unit_system;
        let gauge_parts = body
            .fill_fraction()
            .zip(StockLevel::of(body))
            .zip(body.full_size);
        let below: Element<'_, _> = match gauge_parts {
            Some(((fraction, level), full_size)) => {
                let full = hint(format!("of {} full size", full_size.readable(units)));
                column![gauge(fraction, level), full].spacing(4).into()
            }
            None => hint("Add a full size to see how full it is").into(),
        };
        stat_tile("In stock", body.quantity.readable(units), Some(below)).into()
    }

    /// The recipes the item can go into, each opening on the Recipes
    /// screen when clicked.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn used_in_card(
        &self,
        uses: &[RecipeUse],
        has_category: bool,
    ) -> Element<'_, application::Message> {
        let content: Element<'_, _> = if uses.is_empty() {
            let note = if has_category {
                "No recipe calls for this item's category yet."
            } else {
                "Give this item a category to use it in recipes."
            };
            hint(note).into()
        } else {
            Column::with_children(uses.iter().map(|usage| {
                link_row(
                    usage.name.clone(),
                    usage.amount.readable(self.unit_system),
                    application::Message::ShowRecipe(usage.recipe),
                )
            }))
            .into()
        };
        card(Some("Used in"), content)
    }

    pub fn update(&mut self, item_service: &ItemService, message: Message) -> Option<Command> {
        match message {
            Message::SetUnits(units) => {
                self.unit_system = units;
                None
            }
            Message::Select(item) => {
                self.editor.reset();
                self.selected = item;
                None
            }
            Message::Editor(msg) => match self.editor.update(msg) {
                Ok(Some(Saved::Add((body, category)))) => {
                    Some(Command::Add(body, category.map(|category| category.id)))
                }
                Ok(Some(Saved::Update(id, (body, category)))) => Some(Command::Update(
                    Item { id, body },
                    category.map(|category| category.id),
                )),
                Ok(None) | Err(_) => None,
            },
            Message::BeginEdit(item, category) => {
                let load = || {
                    (
                        item_service.get(item).cloned().unwrap_or_default(),
                        category,
                    )
                };
                self.editor.toggle_edit(item, load, self.unit_system);
                None
            }
            Message::Reload => {
                self.editor.reset();
                self.selected = self.selected.filter(|&item| item_service.get(item).is_ok());
                None
            }
            Message::DeleteItem(item) => Some(Command::Delete(item)),
        }
    }
}

/// The item's name and category on the left; Edit and Delete on the right.
#[cfg_attr(coverage_nightly, coverage(off))]
fn header<'a>(
    item: ItemID,
    name: String,
    breadcrumb: Option<(String, usize)>,
    category: Option<Category>,
) -> Element<'a, application::Message> {
    detail_header(
        name,
        breadcrumb_line(breadcrumb, "No category"),
        None,
        inventory_msg(Message::BeginEdit(item, category)),
        inventory_msg(Message::DeleteItem(item)),
    )
}

/// How many recipes the item can go into.
#[cfg_attr(coverage_nightly, coverage(off))]
fn uses_tile<'a>(uses: &[RecipeUse]) -> Container<'a, application::Message> {
    stat_tile("Used in", counted(uses.len(), "recipe"), None)
}

fn inventory_msg(msg: Message) -> application::Message {
    application::Message::Inventory(msg)
}

fn editor_msg(msg: EditorMessage<ItemMessage>) -> application::Message {
    inventory_msg(Message::Editor(msg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        logic::LogicError,
        models::{BartendError, Quantity, Unit},
        persistence::DBError,
        presentation::screen::test_support::{World, drain},
    };

    const SAVE: Message = Message::Editor(EditorMessage::Save);

    fn send(screen: &mut Inventory, world: &World, input: ItemMessage) {
        let input = Message::Editor(EditorMessage::Input(input));
        assert!(screen.update(&world.items, input).is_none());
    }

    fn fill(screen: &mut Inventory, world: &World, name: &str, quantity: &str) {
        send(screen, world, ItemMessage::Name(name.to_string()));
        send(screen, world, ItemMessage::Quantity(quantity.to_string()));
    }

    fn category(world: &World, id: CategoryID) -> Category {
        Category {
            id,
            body: world.categories.get(id).unwrap().clone(),
        }
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

    fn ml(quantity: f32) -> Quantity {
        Quantity::new(quantity, Unit::Milliliter)
    }

    #[test]
    fn saving_a_new_item_adds_it_with_its_category() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let mut screen = Inventory::new(&world.config, &world.categories, UnitSystem::Metric);
        fill(&mut screen, &world, "Tanqueray", "750");
        send(
            &mut screen,
            &world,
            ItemMessage::Category(category(&world, gin)),
        );

        let command = screen.update(&world.items, SAVE);
        let Some(command @ Command::Add(_, Some(_))) = command else {
            panic!("expected Add with a category");
        };
        assert_reloads(&apply(&mut world, command));

        let item = world.items.get_page(0, 1)[0];
        assert_eq!(world.items.get(item).unwrap().name, "Tanqueray");
        assert_eq!(world.items.get(item).unwrap().quantity, ml(750.0));
        assert_eq!(world.categories.item_category(item), Some(gin));
    }

    #[test]
    fn saving_an_invalid_form_does_nothing() {
        let world = World::new();
        let mut screen = Inventory::new(&world.config, &world.categories, UnitSystem::Metric);
        fill(&mut screen, &world, "Tanqueray", "lots");
        assert!(screen.update(&world.items, SAVE).is_none());
    }

    #[test]
    fn adding_an_item_to_a_missing_category_reports_error() {
        let mut world = World::new();
        let body = ItemBody {
            name: "Tanqueray".to_string(),
            quantity: ml(750.0),
            full_size: None,
        };

        let messages = apply(&mut world, Command::Add(body, Some(CategoryID(99))));

        // The item itself is still saved, so the screen reloads as well.
        assert_eq!(messages.len(), 2, "{messages:?}");
        assert!(
            messages
                .iter()
                .any(|m| matches!(m, application::Message::ReloadScreen))
        );
        assert!(messages.iter().any(|m| matches!(
            m,
            application::Message::Error(BartendError::Db(DBError::RestrictViolation))
        )));
        assert_eq!(world.items.item_count(), 1);
    }

    #[test]
    fn editing_updates_the_item_and_moves_its_category() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let vodka = world.add_category("Vodka");
        let item = world.add_item("Tanqueray", 750.0);
        world.map_item(item, gin);
        let mut screen = Inventory::new(&world.config, &world.categories, UnitSystem::Metric);

        let edit = Message::BeginEdit(item, Some(category(&world, gin)));
        assert!(screen.update(&world.items, edit).is_none());
        fill(&mut screen, &world, "Tito's", "500");
        send(
            &mut screen,
            &world,
            ItemMessage::Category(category(&world, vodka)),
        );
        let command = screen.update(&world.items, SAVE);
        let Some(command @ Command::Update(Item { id, .. }, Some(_))) = command else {
            panic!("expected Update with a category");
        };
        assert_eq!(id, item);
        assert_reloads(&apply(&mut world, command));

        assert_eq!(world.items.get(item).unwrap().name, "Tito's");
        assert_eq!(world.items.get(item).unwrap().quantity, ml(500.0));
        assert_eq!(world.categories.item_category(item), Some(vodka));
        assert!(world.categories.get_items(gin).is_empty());
    }

    #[test]
    fn editing_can_remove_the_category() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let item = world.add_item("Tanqueray", 750.0);
        world.map_item(item, gin);
        let body = world.items.get(item).unwrap().clone();

        let messages = apply(&mut world, Command::Update(Item { id: item, body }, None));

        assert_reloads(&messages);
        assert_eq!(world.categories.item_category(item), None);
    }

    #[test]
    fn begin_edit_on_the_same_item_cancels_and_others_are_ignored() {
        let mut world = World::new();
        let first = world.add_item("Tanqueray", 750.0);
        let second = world.add_item("Beefeater", 750.0);
        let mut screen = Inventory::new(&world.config, &world.categories, UnitSystem::Metric);

        screen.update(&world.items, Message::BeginEdit(first, None));
        screen.update(&world.items, Message::BeginEdit(second, None));
        assert_eq!(screen.editor.editing(), Some(first));

        screen.update(&world.items, Message::BeginEdit(first, None));
        assert_eq!(screen.editor.editing(), None);
        assert!(screen.update(&world.items, SAVE).is_none());
    }

    #[test]
    fn cancel_edit_returns_to_an_empty_form() {
        let mut world = World::new();
        let item = world.add_item("Tanqueray", 750.0);
        let mut screen = Inventory::new(&world.config, &world.categories, UnitSystem::Metric);
        screen.update(&world.items, Message::BeginEdit(item, None));

        let cancel = Message::Editor(EditorMessage::Cancel);
        screen.update(&world.items, cancel);

        assert_eq!(screen.editor.editing(), None);
        assert!(
            screen.update(&world.items, SAVE).is_none(),
            "saving after a cancel must not add a copy of the item"
        );
    }

    #[test]
    fn updating_a_missing_item_reports_error() {
        let mut world = World::new();
        let ghost = Item {
            id: ItemID(99),
            body: ItemBody::default(),
        };
        let messages = apply(&mut world, Command::Update(ghost, None));
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
    fn deleting_an_item_removes_it() {
        let mut world = World::new();
        let item = world.add_item("Tanqueray", 750.0);
        let mut screen = Inventory::new(&world.config, &world.categories, UnitSystem::Metric);

        let command = screen.update(&world.items, Message::DeleteItem(item));
        assert_reloads(&apply(&mut world, command.unwrap()));

        assert_eq!(world.items.item_count(), 0);
    }

    #[test]
    fn set_units_changes_the_unit_system() {
        let world = World::new();
        let mut screen = Inventory::new(&world.config, &world.categories, UnitSystem::Metric);
        screen.update(&world.items, Message::SetUnits(UnitSystem::Imperial));
        assert_eq!(screen.unit_system, UnitSystem::Imperial);
        screen.update(&world.items, Message::SetUnits(UnitSystem::Metric));
        assert_eq!(screen.unit_system, UnitSystem::Metric);
    }

    #[test]
    fn reload_ends_editing_and_keeps_a_live_selection() {
        let mut world = World::new();
        let item = world.add_item("Tanqueray", 750.0);
        let mut screen = Inventory::new(&world.config, &world.categories, UnitSystem::Metric);
        screen.update(&world.items, Message::Select(Some(item)));
        screen.update(&world.items, Message::BeginEdit(item, None));

        screen.update(&world.items, Message::Reload);

        assert_eq!(screen.editor.editing(), None);
        assert_eq!(screen.selected, Some(item));
    }

    #[test]
    fn reload_after_a_delete_clears_the_selection() {
        let mut world = World::new();
        let item = world.add_item("Tanqueray", 750.0);
        let mut screen = Inventory::new(&world.config, &world.categories, UnitSystem::Metric);
        screen.update(&world.items, Message::Select(Some(item)));

        let command = screen.update(&world.items, Message::DeleteItem(item));
        apply(&mut world, command.unwrap());
        screen.update(&world.items, Message::Reload);

        assert_eq!(screen.selected, None);
    }

    #[test]
    fn selecting_an_item_discards_an_edit_in_progress() {
        let mut world = World::new();
        let first = world.add_item("Tanqueray", 750.0);
        let second = world.add_item("Beefeater", 750.0);
        let mut screen = Inventory::new(&world.config, &world.categories, UnitSystem::Metric);
        screen.update(&world.items, Message::BeginEdit(first, None));

        screen.update(&world.items, Message::Select(Some(second)));

        assert_eq!(screen.selected, Some(second));
        assert_eq!(screen.editor.editing(), None);
        assert!(
            screen.update(&world.items, SAVE).is_none(),
            "the discarded edit must not be saved"
        );
    }

    #[test]
    fn selecting_new_clears_the_selection() {
        let mut world = World::new();
        let item = world.add_item("Tanqueray", 750.0);
        let mut screen = Inventory::new(&world.config, &world.categories, UnitSystem::Metric);
        screen.update(&world.items, Message::Select(Some(item)));

        screen.update(&world.items, Message::Select(None));

        assert_eq!(screen.selected, None);
    }
}
