use crate::{
    logic::{CategoryService, ItemCounts, RecipeService, recipes_for_category},
    models::{Category, CategoryBody, CategoryID, UnitSystem},
    presentation::{
        application::{self, Context},
        constants,
        input_handling::{CategoryInput, CategoryMessage, Editor, EditorMessage, Saved},
        screen::{
            Panels,
            shared::{
                breadcrumb_label, breadcrumb_line, counted, detail_header, form_page,
                reload_or_error, report_error,
            },
        },
        widget::{
            controls,
            detail::{card, hint, link_row, link_row_with_action, row_danger_button, stat_tile},
            master_list::{Entry, master_list},
        },
    },
};
use iced::{
    Element,
    Length::{Fill, Shrink},
    Task,
    widget::{Column, column, pick_list, row},
};

#[derive(Debug)]
pub struct Categories {
    editor: Editor<CategoryInput, CategoryID>,
    /// The category shown in the detail panel; `None` shows the
    /// new-category form.
    selected: Option<CategoryID>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Reload,
    /// Shows the category in the detail panel, or the new-category form for
    /// `None`. Any edit in progress is discarded.
    Select(Option<CategoryID>),
    Editor(EditorMessage<CategoryMessage>),
    /// Starts editing the category, or cancels if it is already being
    /// edited.
    Edit(CategoryID),
    AddRelation(CategoryID, CategoryID),
    RemoveRelation(CategoryID, CategoryID),
    DeleteCategory(CategoryID),
}

pub enum Command {
    AddCategory(CategoryBody),
    UpdateCategory(Category),
    AddRelation(CategoryID, CategoryID),
    RemoveRelation(CategoryID, CategoryID),
    DeleteCategory(CategoryID),
}
impl Command {
    pub fn apply(self, ctx: &mut Context<'_>) -> Task<application::Message> {
        match self {
            Command::AddCategory(body) => reload_or_error(
                ctx.category_service
                    .insert(&ctx.database.category_db(), &body),
            ),
            Command::UpdateCategory(category) => reload_or_error(
                ctx.category_service
                    .update(&ctx.database.category_db(), &category),
            ),
            Command::AddRelation(parent, child) => {
                report_error(ctx.category_service.add_category_relation(
                    &ctx.database.category_db(),
                    parent,
                    child,
                ))
            }
            Command::RemoveRelation(parent, child) => {
                report_error(ctx.category_service.remove_category_relation(
                    &ctx.database.category_db(),
                    parent,
                    child,
                ))
            }
            Command::DeleteCategory(category) => reload_or_error(
                ctx.category_service
                    .delete(&ctx.database.category_db(), category),
            ),
        }
    }
}
impl Categories {
    pub fn new() -> Self {
        Self {
            editor: Editor::new(CategoryInput::new(), "Category"),
            selected: None,
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(
        &self,
        category_service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Panels<'_> {
        let entries = category_service.sorted_ids().into_iter().map(|id| Entry {
            label: category_service.name_of(id),
            selected: self.selected == Some(id),
            on_press: categories_msg(Message::Select(Some(id))),
            dimmed: false,
        });
        let new = (
            "+ New Category".to_string(),
            categories_msg(Message::Select(None)),
        );
        let master = master_list(Some(new), entries, "No categories yet");

        let detail = match self.selected {
            Some(category) if self.editor.editing().is_none() => {
                Self::category_details(category, category_service, recipe_service)
            }
            _ => self.form_view(),
        };

        Panels { master, detail }
    }

    /// The new- or edit-category form, laid out like the category's
    /// details: the heading with Save and Cancel to its right, then the
    /// name in a card.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn form_view(&self) -> Element<'_, application::Message> {
        let note = if self.editor.editing().is_some() {
            "Change the name, then Save."
        } else {
            "Add a category, then Save."
        };
        form_page(
            self.editor.title(),
            note,
            self.editor.buttons().map(editor_msg),
            self.editor.form_view().map(editor_msg),
        )
    }

    /// The read-only view of `category`: its name and where it sits,
    /// tiles counting its items and the recipes it can go into, then its
    /// subcategories (which can be added and removed here) and the
    /// categories it is part of.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn category_details(
        category: CategoryID,
        service: &CategoryService,
        recipe_service: &RecipeService,
    ) -> Element<'static, application::Message> {
        let parents = service.parents_by_name(category);
        // A top-level category's breadcrumb is just its own name.
        let breadcrumb = if parents.is_empty() {
            None
        } else {
            breadcrumb_label(category, service)
        };
        let header = detail_header(
            service.name_of(category),
            breadcrumb_line(breadcrumb, "Top-level category"),
            None,
            categories_msg(Message::Edit(category)),
            categories_msg(Message::DeleteCategory(category)),
        );

        let counts = service.item_counts(category);
        let items_tile = stat_tile(
            "Items",
            counted(counts.total, "item"),
            Some(hint(item_split(counts)).into()),
        );
        let recipes = recipes_for_category(category, service, recipe_service);
        // The items tile always has a line under its figure, so it is the
        // taller one and sets the row's height.
        let uses_tile = stat_tile("Used in", counted(recipes.len(), "recipe"), None).height(Fill);
        let tiles = row![items_tile, uses_tile]
            .spacing(constants::SPACING)
            .height(Shrink);

        let subcategories = Self::subcategories_card(category, service);
        let part_of = Self::part_of_card(&parents, service);

        controls::scrolling(column![header, tiles, subcategories, part_of].spacing(16)).into()
    }

    /// `category`'s subcategories, each opening when clicked and with a
    /// button to remove it, then a picker to add another.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn subcategories_card(
        category: CategoryID,
        service: &CategoryService,
    ) -> Element<'static, application::Message> {
        let children = service.children_by_name(category);
        let list: Element<'_, _> = if children.is_empty() {
            hint("No subcategories yet.").into()
        } else {
            Column::with_children(children.into_iter().map(|child| {
                link_row_with_action(
                    service.name_of(child),
                    "",
                    categories_msg(Message::Select(Some(child))),
                    row_danger_button("Remove", category_remove_msg(category, child)),
                )
            }))
            .into()
        };
        let content =
            column![list, Self::relation_add_view(category, service)].spacing(constants::SPACING);
        card(Some("Subcategories"), content)
    }

    /// The categories directly above this one, each opening when clicked.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn part_of_card(
        parents: &[CategoryID],
        service: &CategoryService,
    ) -> Element<'static, application::Message> {
        let content: Element<'_, _> = if parents.is_empty() {
            hint("Not part of any other category.").into()
        } else {
            Column::with_children(parents.iter().map(|&parent| {
                link_row(
                    service.name_of(parent),
                    "",
                    categories_msg(Message::Select(Some(parent))),
                )
            }))
            .into()
        };
        card(Some("Part of"), content)
    }

    pub fn update(
        &mut self,
        message: Message,
        category_service: &CategoryService,
    ) -> Option<Command> {
        match message {
            Message::Select(category) => {
                self.editor.reset();
                self.selected = category;
                None
            }
            Message::Editor(msg) => match self.editor.update(msg) {
                Ok(Some(Saved::Add(body))) => Some(Command::AddCategory(body)),
                Ok(Some(Saved::Update(id, body))) => {
                    Some(Command::UpdateCategory(Category { id, body }))
                }
                Ok(None) | Err(_) => None,
            },
            Message::Edit(category) => {
                let load = || category_service.get(category).cloned().unwrap_or_default();
                // Categories have no quantities, so the unit system is unused.
                self.editor.toggle_edit(category, load, UnitSystem::Metric);
                None
            }
            Message::Reload => {
                self.editor.reset();
                self.selected = self
                    .selected
                    .filter(|&category| category_service.get(category).is_ok());
                None
            }
            Message::AddRelation(parent, child) => Some(Command::AddRelation(parent, child)),
            Message::RemoveRelation(parent, child) => Some(Command::RemoveRelation(parent, child)),
            Message::DeleteCategory(category) => Some(Command::DeleteCategory(category)),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn relation_add_view(
        id: CategoryID,
        service: &CategoryService,
    ) -> Element<'static, application::Message> {
        let mut options: Vec<Category> = service
            .valid_relations(id)
            .into_iter()
            .map(|id| Category {
                id,
                body: service.get(id).cloned().unwrap_or_default(),
            })
            .collect();
        options.sort_by_cached_key(|category| category.body.name.to_lowercase());
        pick_list(options, None::<Category>, move |category| {
            category_add_msg(id, category.id)
        })
        .placeholder("Add a subcategory")
        .width(Fill)
        .style(controls::picker)
        .menu_style(controls::menu)
        .into()
    }
}

/// Where a category's items are filed, for under its item count.
fn item_split(counts: ItemCounts) -> String {
    let via = counts.via_subcategories();
    match (counts.direct, via) {
        (0, 0) => "None here or in its subcategories".to_string(),
        (_, 0) => "All filed here directly".to_string(),
        (0, _) => "All from subcategories".to_string(),
        (direct, via) => format!("{direct} directly, {via} from subcategories"),
    }
}

fn categories_msg(msg: Message) -> application::Message {
    application::Message::Categories(msg)
}

fn editor_msg(msg: EditorMessage<CategoryMessage>) -> application::Message {
    categories_msg(Message::Editor(msg))
}

fn category_add_msg(parent: CategoryID, child: CategoryID) -> application::Message {
    categories_msg(Message::AddRelation(parent, child))
}

fn category_remove_msg(parent: CategoryID, child: CategoryID) -> application::Message {
    categories_msg(Message::RemoveRelation(parent, child))
}

#[cfg(test)]
impl Categories {
    /// The category shown in the detail panel.
    pub fn selected(&self) -> Option<CategoryID> {
        self.selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        logic::LogicError,
        models::BartendError,
        persistence::DBError,
        presentation::screen::test_support::{World, drain},
    };

    const SAVE: Message = Message::Editor(EditorMessage::Save);

    fn type_name(screen: &mut Categories, world: &World, name: &str) {
        let input = CategoryMessage::Name(name.to_string());
        assert!(
            screen
                .update(
                    Message::Editor(EditorMessage::Input(input)),
                    &world.categories
                )
                .is_none()
        );
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
    fn saving_a_new_category_adds_it() {
        let mut world = World::new();
        let mut screen = Categories::new();
        type_name(&mut screen, &world, "Gin");

        let command = screen.update(SAVE, &world.categories);
        let Some(command @ Command::AddCategory(_)) = command else {
            panic!("expected AddCategory");
        };
        assert_reloads(&apply(&mut world, command));

        assert_eq!(world.categories.category_count(), 1);
        assert_eq!(world.stored_category_names(), vec!["Gin"]);
    }

    #[test]
    fn saving_an_empty_form_does_nothing() {
        let world = World::new();
        let mut screen = Categories::new();
        assert!(screen.update(SAVE, &world.categories).is_none());
    }

    #[test]
    fn editing_updates_the_existing_category() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let mut screen = Categories::new();

        screen.update(Message::Edit(gin), &world.categories);
        type_name(&mut screen, &world, "London Dry Gin");
        let command = screen.update(SAVE, &world.categories);
        let Some(command @ Command::UpdateCategory(Category { id, .. })) = command else {
            panic!("expected UpdateCategory");
        };
        assert_eq!(id, gin);
        assert_reloads(&apply(&mut world, command));

        assert_eq!(world.categories.get(gin).unwrap().name, "London Dry Gin");
        assert_eq!(world.stored_category_names(), vec!["London Dry Gin"]);
    }

    #[test]
    fn editing_the_same_category_again_cancels() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let mut screen = Categories::new();

        screen.update(Message::Edit(gin), &world.categories);
        screen.update(Message::Edit(gin), &world.categories);

        assert_eq!(screen.editor.editing(), None);
        assert!(
            screen.update(SAVE, &world.categories).is_none(),
            "cancelling should clear the form"
        );
    }

    #[test]
    fn updating_a_missing_category_reports_error() {
        let mut world = World::new();
        let missing = Category {
            id: CategoryID(99),
            body: CategoryBody {
                name: "Ghost".to_string(),
            },
        };
        let messages = apply(&mut world, Command::UpdateCategory(missing));
        assert!(
            matches!(
                messages.as_slice(),
                [application::Message::Error(BartendError::Logic(
                    LogicError::InvalidCategory(CategoryID(99))
                ))]
            ),
            "{messages:?}"
        );
    }

    #[test]
    fn relation_commands_add_and_remove_edges() {
        let mut world = World::new();
        let spirit = world.add_category("Spirit");
        let gin = world.add_category("Gin");
        let mut screen = Categories::new();

        let add = screen.update(Message::AddRelation(spirit, gin), &world.categories);
        assert!(apply(&mut world, add.unwrap()).is_empty());
        assert_eq!(world.categories.children_by_name(spirit), [gin]);

        let remove = screen.update(Message::RemoveRelation(spirit, gin), &world.categories);
        assert!(apply(&mut world, remove.unwrap()).is_empty());
        assert_eq!(world.categories.children_by_name(spirit), []);
    }

    #[test]
    fn cyclic_relation_reports_error() {
        let mut world = World::new();
        let spirit = world.add_category("Spirit");
        let gin = world.add_category("Gin");
        assert!(apply(&mut world, Command::AddRelation(spirit, gin)).is_empty());

        let messages = apply(&mut world, Command::AddRelation(gin, spirit));
        assert!(
            matches!(
                messages.as_slice(),
                [application::Message::Error(BartendError::Logic(
                    LogicError::InvalidCategoryRelation { .. }
                ))]
            ),
            "{messages:?}"
        );
    }

    #[test]
    fn removing_a_relation_from_a_missing_category_is_harmless() {
        let mut world = World::new();
        let messages = apply(
            &mut world,
            Command::RemoveRelation(CategoryID(1), CategoryID(2)),
        );
        assert!(messages.is_empty(), "{messages:?}");
    }

    #[test]
    fn deleting_a_category_removes_it() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let mut screen = Categories::new();

        let command = screen.update(Message::DeleteCategory(gin), &world.categories);
        assert_reloads(&apply(&mut world, command.unwrap()));

        assert_eq!(world.categories.category_count(), 0);
        assert_eq!(world.stored_category_names(), Vec::<String>::new());
    }

    #[test]
    fn deleting_a_category_used_by_a_recipe_reports_error() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        world.add_recipe("Gin Shot", &[(gin, 50.0)]);

        let messages = apply(&mut world, Command::DeleteCategory(gin));
        assert!(
            matches!(
                messages.as_slice(),
                [application::Message::Error(BartendError::Db(
                    DBError::RestrictViolation
                ))]
            ),
            "{messages:?}"
        );
        assert_eq!(world.categories.category_count(), 1);
    }

    #[test]
    fn reload_ends_editing_and_keeps_a_live_selection() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let mut screen = Categories::new();
        screen.update(Message::Select(Some(gin)), &world.categories);
        screen.update(Message::Edit(gin), &world.categories);

        screen.update(Message::Reload, &world.categories);

        assert_eq!(screen.editor.editing(), None);
        assert_eq!(screen.selected, Some(gin));
        assert!(screen.update(SAVE, &world.categories).is_none());
    }

    #[test]
    fn reload_after_a_delete_clears_the_selection() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let mut screen = Categories::new();
        screen.update(Message::Select(Some(gin)), &world.categories);

        let command = screen.update(Message::DeleteCategory(gin), &world.categories);
        apply(&mut world, command.unwrap());
        screen.update(Message::Reload, &world.categories);

        assert_eq!(screen.selected, None);
    }

    #[test]
    fn selecting_a_category_discards_an_edit_in_progress() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let vodka = world.add_category("Vodka");
        let mut screen = Categories::new();
        screen.update(Message::Edit(gin), &world.categories);

        screen.update(Message::Select(Some(vodka)), &world.categories);

        assert_eq!(screen.selected, Some(vodka));
        assert_eq!(screen.editor.editing(), None);

        screen.update(Message::Select(None), &world.categories);
        assert_eq!(screen.selected, None);
    }

    #[test]
    fn item_split_says_where_items_are_filed() {
        let split = |direct, total| item_split(ItemCounts { direct, total });
        assert_eq!(split(0, 0), "None here or in its subcategories");
        assert_eq!(split(2, 2), "All filed here directly");
        assert_eq!(split(0, 3), "All from subcategories");
        assert_eq!(split(2, 5), "2 directly, 3 from subcategories");
    }

    #[test]
    fn message_helpers_wrap_category_messages() {
        let (a, b) = (CategoryID(1), CategoryID(2));
        assert!(matches!(
            category_add_msg(a, b),
            application::Message::Categories(Message::AddRelation(p, c)) if p == a && c == b
        ));
        assert!(matches!(
            category_remove_msg(a, b),
            application::Message::Categories(Message::RemoveRelation(p, c)) if p == a && c == b
        ));
    }
}
