use crate::{
    logic::{category::Category, config::Config, item::ItemBody, quantity::Unit},
    presentation::{
        Viewable,
        application::Message,
        input_handling::{InputCollection, InputMessage},
        widget::input::{
            pick_input::{OptionalPickInput, RequiredPickInput},
            text_input::{number_input::NumberInput, string_input::StringInput},
        },
    },
};

#[derive(Debug)]
pub(super) struct ItemInput {
    name_input: StringInput<Message>,
    quantity_input: NumberInput<Message>,
    unit_input: RequiredPickInput<Unit, Message>,
    category_input: OptionalPickInput<Category, Message>,
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
    fn output(&mut self) -> Result<ItemBody, ()> {
        Ok(todo!())
    }
}
