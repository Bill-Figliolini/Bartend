use std::vec;

use crate::{
    logic::CategoryService,
    models::{Category, CategoryBody, CategoryID, Config, UnitSystem},
    presentation::{
        application::{self, Context},
        input_handling::{CategoryInput, CategoryMessage},
        widget::{self, text_style},
    },
};
use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{button, column, container, pick_list, row, table, text},
};

#[derive(Debug)]
pub struct Categories {
    input: CategoryInput,

    edit_state: EditState,

    current_page: usize,
    stub: Option<Category>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Reload,
    Input(CategoryMessage),
    Edit(CategoryID),
    AddRelation(CategoryID, CategoryID),
    RemoveRelation(CategoryID, CategoryID),
    DeleteCategory(CategoryID),
    SelectPage(usize),
    Save,
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
#[derive(Debug)]
enum EditState {
    Editing(CategoryID),
    None,
}
impl Categories {
    pub fn new(config: &Config) -> Self {
        Self {
            input: CategoryInput::new(config),
            edit_state: EditState::None,
            current_page: 0,
            stub: None,
        }
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn build_category_entry(&self) -> Element<'_, application::Message> {
        let header = match self.edit_state {
            EditState::None => text("New Category:"),
            EditState::Editing(_) => text("Edit Category:"),
        };
        let save_button = iced::widget::Button::new("Save")
            .on_press(application::Message::Categories(Message::Save));
        let input = self
            .input
            .view()
            .map(|msg| application::Message::Categories(Message::Input(msg)));
        let body = row![input, save_button];
        column![header, body].into()
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn build_category_display(
        &self,
        category_service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let name_column = table::column(text("Name").width(200), |category: CategoryID| {
            text(
                category_service
                    .get(category)
                    .cloned()
                    .unwrap_or_default()
                    .name,
            )
        });
        let add_relation_column = table::column("Addable Relations", |category: CategoryID| {
            self.relation_add_view(category, category_service)
        });
        let list_relation_column = table::column("Current Relations", |category: CategoryID| {
            self.category_remove_view(category, category_service)
        });
        let edit_column_width = 70;
        let edit_column = table::column(
            text("Edit").width(edit_column_width).center(),
            |category: CategoryID| match self.edit_state {
                EditState::None => iced::widget::Button::new(text("Edit").center())
                    .on_press(application::Message::Categories(Message::Edit(category)))
                    .width(edit_column_width),
                EditState::Editing(category_id) if category == category_id => {
                    iced::widget::Button::new(text("Cancel").center())
                        .on_press(application::Message::Categories(Message::Edit(category_id)))
                        .width(edit_column_width)
                }
                EditState::Editing(_) => {
                    iced::widget::Button::new(text("Edit").center()).width(edit_column_width)
                }
            },
        );
        let delete_column_width = 50;
        let delete_column = table::column(
            text("Delete").width(delete_column_width).center(),
            |category: CategoryID| {
                iced::widget::Button::new(text("X").width(delete_column_width).center()).on_press(
                    application::Message::Categories(Message::DeleteCategory(category)),
                )
            },
        );
        let columns = vec![
            name_column,
            add_relation_column,
            list_relation_column,
            edit_column,
            delete_column,
        ];
        let contents = category_service.get_page(self.current_page, 15);
        table(columns, contents).into()
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self, category_service: &CategoryService) -> Element<'_, application::Message> {
        let header = widget::header::header(text_style::title("Categories"));
        let category_entry = self.build_category_entry();
        let categories = self.build_category_display(category_service);
        let mut page_controller_contents = Vec::new();
        if self.current_page != 0 {
            let previous_page = self.current_page.wrapping_sub(1);
            let previous_page_button = button("Previous")
                .on_press(application::Message::Categories(Message::SelectPage(
                    previous_page,
                )))
                .into();
            page_controller_contents.push(previous_page_button);
        }
        let space = iced::widget::space().width(Fill).into();
        page_controller_contents.push(space);
        if (self.current_page + 1) * 15 < category_service.category_count() {
            let next_page = self.current_page.wrapping_add(1);
            let next_page_button = button("Next")
                .on_press(application::Message::Categories(Message::SelectPage(
                    next_page,
                )))
                .into();
            page_controller_contents.push(next_page_button);
        }
        let page_controller = row(page_controller_contents);
        let body = container(column![category_entry, categories]).align_top(Fill);
        column![header, body, page_controller].into()
    }
    pub fn update(
        &mut self,
        message: Message,
        category_service: &CategoryService,
    ) -> Option<Command> {
        match message {
            Message::Input(msg) => {
                self.input.update(msg);
                None
            }
            Message::Edit(category) => {
                match self.edit_state {
                    EditState::None => {
                        self.edit_state = EditState::Editing(category);
                        let body = category_service.get(category).cloned().unwrap_or_default();
                        self.input.begin_edit(&body, UnitSystem::Metric);
                    }
                    EditState::Editing(category_id) if category_id == category => {
                        self.input.clear();
                        self.edit_state = EditState::None;
                    }
                    EditState::Editing(_) => unreachable!(
                        "Edit buttons to items not currently under edit should not be accesable"
                    ),
                }
                None
            }
            Message::Save => match self.input.output() {
                Ok(body) => match self.edit_state {
                    EditState::Editing(id) => Some(Command::UpdateCategory(Category { id, body })),
                    EditState::None => Some(Command::AddCategory(body)),
                },
                Err(_) => None,
            },
            Message::Reload => {
                self.input.clear();
                self.edit_state = EditState::None;
                self.current_page = self.current_page.min(
                    category_service
                        .category_count()
                        .div_ceil(15)
                        .saturating_sub(1),
                );
                None
            }
            Message::AddRelation(parent, child) => Some(Command::AddRelation(parent, child)),
            Message::RemoveRelation(parent, child) => Some(Command::RemoveRelation(parent, child)),
            Message::DeleteCategory(category) => Some(Command::DeleteCategory(category)),
            Message::SelectPage(page) => {
                self.current_page = page;
                None
            }
        }
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn relation_add_view(
        &self,
        id: CategoryID,
        service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let options = service
            .valid_relations(id)
            .into_iter()
            .fold(Vec::new(), |mut acc, id| {
                acc.push(Category {
                    id,
                    body: service.get(id).cloned().unwrap_or_default(),
                });
                acc
            });
        pick_list(options, self.stub.clone(), move |category| {
            category_add_msg(id, category.id)
        })
        .placeholder("Add a Subcategory")
        .into()
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn category_remove_view(
        &self,
        id: CategoryID,
        service: &CategoryService,
    ) -> Element<'_, application::Message> {
        let options = service
            .child_categories(id)
            .unwrap_or_default()
            .into_iter()
            .fold(Vec::new(), |mut acc, id| {
                acc.push(Category {
                    id,
                    body: service.get(id).cloned().unwrap_or_default(),
                });
                acc
            });
        pick_list(options, self.stub.clone(), move |selected| {
            category_remove_msg(id, selected.id)
        })
        .placeholder("List subcategories (click to remove)")
        .into()
    }
}
fn category_add_msg(parent: CategoryID, child: CategoryID) -> application::Message {
    application::Message::Categories(Message::AddRelation(parent, child))
}

fn category_remove_msg(parent: CategoryID, child: CategoryID) -> application::Message {
    application::Message::Categories(Message::RemoveRelation(parent, child))
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

    fn type_name(screen: &mut Categories, world: &World, name: &str) {
        let input = CategoryMessage::Name(name.to_string());
        assert!(
            screen
                .update(Message::Input(input), &world.categories)
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
        let mut screen = Categories::new(&world.config);
        type_name(&mut screen, &world, "Gin");

        let command = screen.update(Message::Save, &world.categories);
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
        let mut screen = Categories::new(&world.config);
        assert!(screen.update(Message::Save, &world.categories).is_none());
    }

    #[test]
    fn editing_updates_the_existing_category() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let mut screen = Categories::new(&world.config);

        screen.update(Message::Edit(gin), &world.categories);
        type_name(&mut screen, &world, "London Dry Gin");
        let command = screen.update(Message::Save, &world.categories);
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
        let mut screen = Categories::new(&world.config);

        screen.update(Message::Edit(gin), &world.categories);
        screen.update(Message::Edit(gin), &world.categories);

        assert!(matches!(screen.edit_state, EditState::None));
        assert!(
            screen.update(Message::Save, &world.categories).is_none(),
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
        let mut screen = Categories::new(&world.config);

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
        let mut screen = Categories::new(&world.config);

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
    fn reload_ends_editing_and_clamps_the_page() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let mut screen = Categories::new(&world.config);
        screen.update(Message::Edit(gin), &world.categories);
        screen.update(Message::SelectPage(4), &world.categories);
        assert_eq!(screen.current_page, 4);

        screen.update(Message::Reload, &world.categories);

        assert!(matches!(screen.edit_state, EditState::None));
        assert_eq!(screen.current_page, 0);
        assert!(screen.update(Message::Save, &world.categories).is_none());
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
