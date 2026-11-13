use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{column, container, row, table, text},
};

use crate::{
    logic::{CategoryService, ItemService},
    models::{Category, CategoryFilter, CategoryID, Config, Item, ItemBody, ItemID, UnitSystem},
    presentation::{
        application::{self, Context},
        input_handling::{Editor, EditorMessage, ItemInput, ItemMessage, Saved},
        widget::{footer::footer, header::header, text_style::title},
    },
};

#[derive(Debug)]
pub struct Inventory {
    editor: Editor<ItemInput, ItemID>,

    current_page: usize,
    unit_system: UnitSystem,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwapUnits,
    /// Starts editing the item, which is in the given category, or cancels
    /// if it is already being edited.
    BeginEdit(ItemID, Option<Category>),
    Editor(EditorMessage<ItemMessage>),
    DeleteItem(ItemID),
    SelectPage(usize),

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
                let mut tasks = Vec::new();
                let item_id = match ctx.item_service.add(&ctx.database.item_db(), &item_body) {
                    Ok(id) => {
                        tasks.push(Task::done(application::Message::ReloadScreen));
                        id
                    }
                    Err(e) => return Task::done(application::Message::Error(e)),
                };
                if let Some(category_id) = category_id
                    && let Err(e) = ctx.category_service.add_item_mapping(
                        &ctx.database.category_db(),
                        item_id,
                        category_id,
                    )
                {
                    tasks.push(Task::done(application::Message::Error(e)));
                }
                Task::batch(tasks)
            }
            Command::Update(item, category_id) => {
                let mut tasks = Vec::new();
                let item_id = item.id;
                if let Err(e) = ctx.item_service.update(&ctx.database.item_db(), item) {
                    tasks.push(Task::done(application::Message::Error(e)));
                } else {
                    tasks.push(Task::done(application::Message::ReloadScreen));
                }
                if let Err(e) = ctx.category_service.update_item_mapping(
                    &ctx.database.category_db(),
                    item_id,
                    category_id,
                ) {
                    tasks.push(Task::done(application::Message::Error(e)));
                }
                Task::batch(tasks)
            }
            Command::Delete(id) => match ctx.item_service.delete(&ctx.database.item_db(), id) {
                Ok(()) => Task::done(application::Message::ReloadScreen),
                Err(e) => Task::done(application::Message::Error(e)),
            },
        }
    }
}

impl Inventory {
    pub fn new(config: &Config, category_service: &CategoryService) -> Self {
        let unit_system = config.default_units();

        let form = ItemInput::new(config, category_service.get_all(CategoryFilter {}));
        Self {
            editor: Editor::new(form, "Item"),
            unit_system,
            current_page: 0,
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn build_inventory_display(
        &self,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let name_column = table::column(text("Name").width(Fill), |item: ItemID| {
            text(item_service.get(item).cloned().unwrap_or_default().name)
        });
        let quantity_column = table::column(text("Quantity"), |item: ItemID| {
            let item = item_service.get(item).cloned().unwrap_or_default();
            text!(
                "{:.0} {}",
                item.quantity.value(self.unit_system),
                item.quantity.unit(self.unit_system).to_string()
            )
        });
        let category_column =
            table::column(
                text("Category").width(200),
                |item: ItemID| match category_service.item_category(item) {
                    Some(cat_id) => text(
                        category_service
                            .get(cat_id)
                            .cloned()
                            .unwrap_or_default()
                            .name,
                    ),
                    None => text!("None"),
                },
            );
        let edit_column_width = 70;
        let edit_column = table::column(
            text("Edit").width(edit_column_width).center(),
            |item: ItemID| {
                let begin = || {
                    let category = category_service.item_category(item).map(|id| Category {
                        id,
                        body: category_service.get(id).cloned().unwrap_or_default(),
                    });
                    application::Message::Inventory(Message::BeginEdit(item, category))
                };
                self.editor
                    .edit_button(item, begin, editor_msg(EditorMessage::Cancel))
                    .width(edit_column_width)
            },
        );
        let delete_column_width = 50;
        let delete_column = table::column(
            text("Delete").width(delete_column_width).center(),
            |item: ItemID| {
                iced::widget::Button::new(text("X").width(delete_column_width).center())
                    .on_press(application::Message::Inventory(Message::DeleteItem(item)))
            },
        );
        let columns = vec![
            name_column,
            quantity_column,
            category_column,
            edit_column,
            delete_column,
        ];
        let contents = item_service.get_page(self.current_page, 15);
        table(columns, contents).into()
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(
        &self,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let title_text = title("Inventory");
        let header = header(title_text);

        let entry_section = self.editor.view().map(editor_msg);

        let inventory = self.build_inventory_display(item_service, category_service);

        let body_contents = column![entry_section, inventory];
        let body = container(body_contents).align_top(Fill);

        let mut page_controller_contents = Vec::new();
        if self.current_page != 0 {
            let previous_page = self.current_page.wrapping_sub(1);
            let previous_page_button = iced::widget::button("Previous")
                .on_press(application::Message::Inventory(Message::SelectPage(
                    previous_page,
                )))
                .into();
            page_controller_contents.push(previous_page_button);
        }
        let space = iced::widget::space().width(Fill).into();
        page_controller_contents.push(space);
        if (self.current_page + 1) * 15 < item_service.item_count() {
            let next_page = self.current_page.wrapping_add(1);
            let next_page_button = iced::widget::button("Next")
                .on_press(application::Message::Inventory(Message::SelectPage(
                    next_page,
                )))
                .into();
            page_controller_contents.push(next_page_button);
        }
        let page_controller = row(page_controller_contents);
        let unit_swap_button = iced::widget::Button::new(text(self.unit_system.to_string()))
            .on_press(application::Message::Inventory(Message::SwapUnits));
        let footer_contents = row![unit_swap_button];
        let footer_container = iced::widget::Container::new(footer_contents).align_left(Fill);
        let footer = footer(footer_container);

        column![header, body, page_controller, footer].into()
    }

    pub fn update(&mut self, item_service: &ItemService, message: Message) -> Option<Command> {
        match message {
            Message::SwapUnits => {
                self.unit_system.swap();
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
                self.current_page = self
                    .current_page
                    .min(item_service.item_count().div_ceil(15).saturating_sub(1));
                None
            }
            Message::DeleteItem(item) => Some(Command::Delete(item)),
            Message::SelectPage(page) => {
                self.current_page = page;
                None
            }
        }
    }
}

fn editor_msg(msg: EditorMessage<ItemMessage>) -> application::Message {
    application::Message::Inventory(Message::Editor(msg))
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
        let mut screen = Inventory::new(&world.config, &world.categories);
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
        let mut screen = Inventory::new(&world.config, &world.categories);
        fill(&mut screen, &world, "Tanqueray", "lots");
        assert!(screen.update(&world.items, SAVE).is_none());
    }

    #[test]
    fn adding_an_item_to_a_missing_category_reports_error() {
        let mut world = World::new();
        let body = ItemBody {
            name: "Tanqueray".to_string(),
            quantity: ml(750.0),
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
        let mut screen = Inventory::new(&world.config, &world.categories);

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
        let mut screen = Inventory::new(&world.config, &world.categories);

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
        let mut screen = Inventory::new(&world.config, &world.categories);
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
        let mut screen = Inventory::new(&world.config, &world.categories);

        let command = screen.update(&world.items, Message::DeleteItem(item));
        assert_reloads(&apply(&mut world, command.unwrap()));

        assert_eq!(world.items.item_count(), 0);
    }

    #[test]
    fn swap_units_toggles_the_unit_system() {
        let world = World::new();
        let mut screen = Inventory::new(&world.config, &world.categories);
        screen.update(&world.items, Message::SwapUnits);
        assert_eq!(screen.unit_system, UnitSystem::Imperial);
        screen.update(&world.items, Message::SwapUnits);
        assert_eq!(screen.unit_system, UnitSystem::Metric);
    }

    #[test]
    fn reload_ends_editing_and_clamps_to_the_last_page() {
        let mut world = World::new();
        for i in 0..16 {
            world.add_item(&format!("Item {i}"), 1.0);
        }
        let first = world.items.get_page(0, 1)[0];
        let mut screen = Inventory::new(&world.config, &world.categories);
        screen.update(&world.items, Message::BeginEdit(first, None));
        screen.update(&world.items, Message::SelectPage(5));

        screen.update(&world.items, Message::Reload);

        assert_eq!(screen.editor.editing(), None);
        assert_eq!(screen.current_page, 1);
    }
}
