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
impl InputCollection<application::Message> for ServiceInput {
    fn update(&mut self, msg: super::InputMessage) {
        match msg {
            _ => unreachable!("Invalid message passed"),
        }
    }

    fn output(&mut self) -> Result<application::Message, ()> {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }
}
