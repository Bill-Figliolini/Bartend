use crate::{
    application,
    models::{Category, Item, ItemID, Quantity, Recipe},
    presentation::{
        Viewable,
        input_handling::{EditableCollection, InputCollection},
        widget::input::RequiredPickInput,
    },
};

struct ServiceInput {
    recipe: RequiredPickInput<Recipe, application::Message>,
    ingredients: Vec<IngredientUseInput>,
}
struct IngredientUseInput {
    category: Category,
    quantity: Quantity,
    ingredient: RequiredPickInput<Item, application::Message>,
}
impl Viewable<application::Message> for IngredientUseInput {
    fn view(&self) -> iced::Element<'_, application::Message> {
        todo!()
    }
}
impl Viewable<application::Message> for ServiceInput {
    fn view(&self) -> iced::Element<'_, application::Message> {
        todo!()
    }
}
impl InputCollection for ServiceInput {
    fn update(&mut self, msg: super::InputMessage) {
        todo!()
    }
    fn output(&mut self) -> Result<T, ()> {
        todo!()
    }
    fn clear(&mut self) {
        todo!()
    }
}
impl EditableCollection for ServiceInput {
    fn begin_edit(&mut self, edit: &T, unit_system: crate::models::UnitSystem) {
        todo!()
    }
}
