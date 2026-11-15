use crate::{
    logic::CategoryService,
    models::{Category, CategoryBody, CategoryID, UnitSystem},
    presentation::{
        application::{self, Context},
        constants,
        input_handling::{CategoryInput, CategoryMessage, Editor, EditorMessage, Saved},
        screen::Panels,
        widget::master_list::{Entry, master_list},
    },
};
use iced::{
    Alignment, Element,
    Length::Fill,
    Task,
    widget::{Column, button, column, pick_list, row, text},
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
            Command::AddCategory(body) => {
                match ctx
                    .category_service
                    .insert(&ctx.database.category_db(), &body)
                {
                    Ok(_) => Task::done(application::Message::ReloadScreen),
                    Err(e) => Task::done(application::Message::Error(e)),
                }
            }
            Command::UpdateCategory(category) => {
                match ctx
                    .category_service
                    .update(&ctx.database.category_db(), &category)
                {
                    Ok(()) => Task::done(application::Message::ReloadScreen),
                    Err(e) => Task::done(application::Message::Error(e)),
                }
            }
            Command::AddRelation(parent, child) => {
                match ctx.category_service.add_category_relation(
                    &ctx.database.category_db(),
                    parent,
                    child,
                ) {
                    Ok(()) => Task::none(),
                    Err(e) => Task::done(application::Message::Error(e)),
                }
            }
            Command::RemoveRelation(parent, child) => {
                match ctx.category_service.remove_category_relation(
                    &ctx.database.category_db(),
                    parent,
                    child,
                ) {
                    Ok(()) => Task::none(),
                    Err(e) => Task::done(application::Message::Error(e)),
                }
            }
            Command::DeleteCategory(category) => {
                match ctx
                    .category_service
                    .delete(&ctx.database.category_db(), category)
                {
                    Ok(()) => Task::done(application::Message::ReloadScreen),
                    Err(e) => Task::done(application::Message::Error(e)),
                }
            }
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
    pub fn view(&self, category_service: &CategoryService) -> Panels<'_> {
        let entries = category_service.sorted_ids().into_iter().map(|id| Entry {
            label: category_name(category_service, id),
            selected: self.selected == Some(id),
            on_press: categories_msg(Message::Select(Some(id))),
            dimmed: false,
        });
        let new = (
            "+ New Category".to_string(),
            categories_msg(Message::Select(None)),
        );
        let master = master_list(Some(new), entries);

        let detail = match self.selected {
            Some(category) if self.editor.editing().is_none() => {
                Self::category_details(category, category_service)
            }
            _ => self.editor.view().map(editor_msg),
        };

        Panels { master, detail }
    }

    /// The read-only view of `category`: its subcategories, a picker to add
    /// more, and buttons to edit or delete it.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn category_details(
        category: CategoryID,
        service: &CategoryService,
    ) -> Element<'static, application::Message> {
        let name = text(category_name(service, category)).size(24);

        let mut children: Vec<CategoryID> = service
            .child_categories(category)
            .unwrap_or_default()
            .into_iter()
            .collect();
        children.sort_by_cached_key(|&child| category_name(service, child).to_lowercase());
        let subcategories: Element<'_, _> = if children.is_empty() {
            text("No subcategories").into()
        } else {
            Column::with_children(children.into_iter().map(|child| {
                let remove = button(text("Remove"))
                    .style(button::secondary)
                    .on_press(category_remove_msg(category, child));
                row![text(category_name(service, child)).width(Fill), remove]
                    .spacing(constants::SPACING)
                    .align_y(Alignment::Center)
                    .into()
            }))
            .spacing(constants::SPACING / 2)
            .width(400)
            .into()
        };

        let edit = button("Edit").on_press(categories_msg(Message::Edit(category)));
        let delete = button("Delete")
            .style(button::danger)
            .on_press(categories_msg(Message::DeleteCategory(category)));
        let buttons = row![edit, delete].spacing(constants::SPACING);

        column![
            name,
            text("Subcategories").size(18),
            subcategories,
            Self::relation_add_view(category, service),
            buttons
        ]
        .spacing(16)
        .into()
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
        .width(400)
        .into()
    }
}

fn category_name(service: &CategoryService, id: CategoryID) -> String {
    service.get(id).cloned().unwrap_or_default().name
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
        assert_eq!(
            world.categories.child_categories(spirit).unwrap(),
            [gin].into()
        );

        let remove = screen.update(Message::RemoveRelation(spirit, gin), &world.categories);
        assert!(apply(&mut world, remove.unwrap()).is_empty());
        assert!(
            world
                .categories
                .child_categories(spirit)
                .unwrap()
                .is_empty()
        );
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
