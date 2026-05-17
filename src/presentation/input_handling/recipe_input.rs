use crate::{
    logic::{
        category::Category,
        config::Config,
        quantity::{Unit, UnitSystem},
        recipe::RecipeBody,
    },
    presentation::{
        Viewable,
        application::Message,
        input_handling::{InputCollection, InputMessage},
        widget::input::{
            pick_input::required::RequiredPickInput,
            text_input::{number_input::NumberInput, string_input::StringInput},
        },
    },
};

#[derive(Debug)]
pub struct RecipeInput {
    name_input: StringInput<Message>,
}
#[derive(Debug)]
pub struct IngredientInput {
    category_input: RequiredPickInput<Category, Message>,
    quantity_input: NumberInput<Message>,
    unit_input: RequiredPickInput<Unit, Message>,
}

impl RecipeInput {
    pub fn new(config: &Config, msg: fn(InputMessage) -> Message) -> Self {
        todo!()
    }
}

impl InputCollection<RecipeBody> for RecipeInput {
    fn update(&mut self, msg: InputMessage) {
        todo!()
    }

    fn output(&mut self) -> Result<RecipeBody, ()> {
        Ok(todo!())
    }

    fn begin_edit(&mut self, edit: RecipeBody, unit_system: UnitSystem) {
        todo!()
    }
    fn clear(&mut self) {
        todo!()
    }
}

impl Viewable<Message> for RecipeInput {
    fn view(&self) -> iced::Element<'_, Message> {
        todo!()
    }
}
