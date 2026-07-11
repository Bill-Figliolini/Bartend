use crate::{
    application,
    models::{Category, Item, ItemID, Quantity, Recipe},
    presentation::widget::input::RequiredPickInput,
};

#[derive(Debug)]
pub struct ServingInput {
    recipe: RequiredPickInput<Recipe, application::Message>,
    ingredients: Vec<IngredientUseInput>,
}
#[derive(Debug)]
struct IngredientUseInput {
    category: Category,
    quantity: Quantity,
    ingredient: RequiredPickInput<Item, application::Message>,
}
impl IngredientUseInput {
    pub fn view(&self) -> iced::Element<'_, application::Message> {
        todo!()
    }
}
impl ServingInput {
    pub fn new() -> Self {
        todo!();
    }
    pub fn view(&self) -> iced::Element<'_, application::Message> {
        todo!()
    }
    pub fn update(&mut self, msg: super::InputMessage) {
        match msg {
            _ => unreachable!("Invalid message passed"),
        }
    }

    pub fn output(&mut self) -> Result<application::Message, ()> {
        todo!()
    }

    pub fn clear(&mut self) {
        todo!()
    }
}
