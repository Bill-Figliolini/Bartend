use iced::widget::Id;

use crate::{
    logic::{category::Category, config::Config, quantity::Unit},
    presentation::{
        Viewable,
        application::Message,
        screen::inventory::PreCommitItem,
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
pub trait InputCollection<T>: Viewable<Message> {
    fn new(config: &Config, msg: fn(InputMessage) -> Message) -> Self;
    fn update(&mut self, msg: InputMessage);
    fn output(&mut self) -> T;
}
impl Viewable<Message> for CategoryInput {
    fn view(&self) -> iced::Element<'_, Message> {
        todo!()
    }
}

impl<T> InputCollection<T> for CategoryInput {
    fn update(&mut self, msg: InputMessage) {
        todo!()
    }

    fn new(config: &Config, msg: fn(InputMessage) -> Message) -> Self {
        todo!()
    }

    fn output(&mut self) -> T {
        todo!()
    }
}
impl Viewable<Message> for ItemInput {
    fn view(&self) -> iced::Element<'_, Message> {
        todo!()
    }
}

impl InputCollection<PreCommitItem> for ItemInput {
    fn update(&mut self, msg: InputMessage) {
        todo!()
    }

    fn new(config: &Config, msg: fn(InputMessage) -> Message) -> Self {
        todo!()
    }

    fn output(&mut self) -> PreCommitItem {
        todo!()
    }
}
impl Viewable<Message> for RecipeInput {
    fn view(&self) -> iced::Element<'_, Message> {
        todo!()
    }
}

impl<T> InputCollection<PreCommitRecipe> for RecipeInput {
    fn new(config: &Config, msg: fn(InputMessage) -> Message) -> Self {
        todo!()
    }
    fn update(&mut self, msg: InputMessage) {
        todo!()
    }
    fn output(&mut self) -> PreCommitRecipe {
        todo!()
    }
}
