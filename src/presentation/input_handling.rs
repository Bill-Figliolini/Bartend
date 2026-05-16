use iced::widget::Id;

use crate::{
    logic::{
        Editable,
        category::{Category, CategoryBody},
        config::Config,
        item::ItemBody,
        quantity::{Quantity, Unit},
        recipe::RecipeBody,
    },
    presentation::{
        Viewable,
        application::Message,
        widget::input::{
            pick_input::{OptionalPickInput, RequiredPickInput},
            string_input::{NumberInput, StringInput},
        },
    },
};
#[derive(Clone, Debug)]
pub enum InputMessage {
    String(Id, String),
    Unit(Id, Unit),
    Category(Id, Category),
    OptionalCategory(Id, Option<Category>),
}
#[derive(Debug)]
pub(super) struct ItemInput {
    name_input: StringInput<Message>,
    quantity_input: NumberInput<Message>,
    unit_input: RequiredPickInput<Unit, Message>,
    category_input: OptionalPickInput<Category, Message>,
}
#[derive(Debug)]
pub(super) struct IngredientInput {
    category_input: RequiredPickInput<Category, Message>,
    quantity_input: NumberInput<Message>,
    unit_input: RequiredPickInput<Unit, Message>,
}
#[derive(Debug)]
pub(super) struct CategoryInput {
    name_input: StringInput<Message>,
}
#[derive(Debug)]
pub(super) struct RecipeInput {
    name_input: StringInput<Message>,
}
pub trait InputCollection<T>: Viewable<Message>
where
    T: Editable,
{
    fn new(config: &Config, msg: fn(InputMessage) -> Message) -> Self;
    fn update(&mut self, msg: InputMessage);
    fn output(&mut self) -> T;
}
impl Viewable<Message> for CategoryInput {
    fn view(&self) -> iced::Element<'_, Message> {
        self.name_input.view()
    }
}

impl InputCollection<CategoryBody> for CategoryInput {
    fn new(_config: &Config, msg: fn(InputMessage) -> Message) -> Self {
        Self {
            name_input: StringInput::new(
                move |id, str| msg(InputMessage::String(id, str)),
                "name".to_string(),
                String::new(),
            ),
        }
    }
    fn update(&mut self, msg: InputMessage) {
        todo!()
    }

    fn output(&mut self) -> CategoryBody {
        todo!()
    }
}

impl Viewable<Message> for ItemInput {
    fn view(&self) -> iced::Element<'_, Message> {
        todo!()
    }
}
impl InputCollection<ItemBody> for ItemInput {
    fn new(config: &Config, msg: fn(InputMessage) -> Message) -> Self {
        todo!()
    }
    fn update(&mut self, msg: InputMessage) {
        todo!()
    }
    fn output(&mut self) -> ItemBody {
        todo!()
    }
}

impl Viewable<Message> for RecipeInput {
    fn view(&self) -> iced::Element<'_, Message> {
        todo!()
    }
}

impl InputCollection<RecipeBody> for RecipeInput {
    fn new(config: &Config, msg: fn(InputMessage) -> Message) -> Self {
        todo!()
    }
    fn update(&mut self, msg: InputMessage) {
        todo!()
    }

    fn output(&mut self) -> RecipeBody {
        todo!()
    }
}
