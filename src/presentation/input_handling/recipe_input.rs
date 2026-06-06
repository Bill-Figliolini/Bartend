use iced::widget::{Id, button, column, row};

use crate::{
    models::{Category, Config, Ingredient, Quantity, RecipeBody, Unit, UnitSystem},
    presentation::{
        Viewable,
        application::Message,
        input_handling::{EditableCollection, InputCollection, InputMessage},
        screen::recipes,
        widget::input::{Input, InputContents, NumberInput, RequiredPickInput, StringInput},
    },
};

#[derive(Debug)]
pub struct RecipeInput {
    name_input: StringInput<Message>,
    ingredient_inputs: Vec<IngredientInput>,

    categories: Vec<Category>,

    msgfn: fn(InputMessage) -> Message,
    unit_system: UnitSystem,
}
#[derive(Debug)]
pub struct IngredientInput {
    category_input: RequiredPickInput<Category, Message>,
    quantity_input: NumberInput<Message>,
    unit_input: RequiredPickInput<Unit, Message>,
}

impl IngredientInput {
    fn new(
        msg: fn(InputMessage) -> Message,
        categories: Vec<Category>,
        unit_system: UnitSystem,
    ) -> Self {
        let category_input = RequiredPickInput::new(
            move |id, category| msg(InputMessage::Category(id, category)),
            categories,
            None,
        );
        let quantity_input = NumberInput::new(
            move |id, str| msg(InputMessage::String(id, str)),
            "Quantity".to_string(),
            String::new(),
        );
        let unit_input = RequiredPickInput::new(
            move |id, unit| msg(InputMessage::Unit(id, unit)),
            Unit::get_units(),
            Some(unit_system.default_units()),
        );
        Self {
            category_input,
            quantity_input,
            unit_input,
        }
    }
    fn has_id(&self, id: &Id) -> bool {
        self.category_input.id() == id
            || self.quantity_input.id() == id
            || self.unit_input.id() == id
    }
    fn update(&mut self, msg: InputMessage) {
        match msg {
            InputMessage::String(id, str) if id == *self.quantity_input.id() => {
                self.quantity_input.update(str)
            }
            InputMessage::Unit(id, unit) if id == *self.unit_input.id() => {
                self.unit_input.update(unit)
            }
            InputMessage::Category(id, category) if id == *self.category_input.id() => {
                self.category_input.update(category)
            }
            _ => unreachable!(),
        }
    }
    fn has_error(&self) -> bool {
        self.category_input.has_error() || self.quantity_input.has_error()
    }
    fn get_output(&mut self) -> Result<Ingredient, ()> {
        let category_result = self.category_input.get_output();
        let quantity_result = self.quantity_input.get_output();
        if !self.has_error() {
            let quantity = Quantity::new(
                quantity_result.unwrap(),
                self.unit_input.get_output().unwrap(),
            );
            let category = category_result.unwrap().id;
            Ok(Ingredient { category, quantity })
        } else {
            Err(())
        }
    }
}

impl Viewable<Message> for IngredientInput {
    fn view(&self) -> iced::Element<'_, Message> {
        row![
            self.category_input.view(),
            self.quantity_input.view(),
            self.unit_input.view()
        ]
        .into()
    }
}
impl RecipeInput {
    pub fn new(
        config: &Config,
        msg: fn(InputMessage) -> Message,
        categories: Vec<Category>,
    ) -> Self {
        RecipeInput {
            name_input: StringInput::new(
                move |id, str| msg(InputMessage::String(id, str)),
                "Name".to_string(),
                String::new(),
            ),
            ingredient_inputs: Vec::new(),
            categories,
            msgfn: msg,
            unit_system: config.default_units(),
        }
    }
    pub fn add_ingredient(&mut self) {
        self.ingredient_inputs.push(IngredientInput::new(
            self.msgfn,
            self.categories.clone(),
            self.unit_system,
        ));
    }
    pub fn remove_ingredient(&mut self, index: usize) {
        self.ingredient_inputs.remove(index);
    }
}

impl InputCollection<RecipeBody> for RecipeInput {
    fn update(&mut self, msg: InputMessage) {
        let id = msg.id();
        if self.name_input.id() == &id {
            if let InputMessage::String(_, str) = msg {
                self.name_input.update(str);
            } else {
                unreachable!()
            }
        } else {
            match self
                .ingredient_inputs
                .iter_mut()
                .find(|ingredient| ingredient.has_id(&id))
            {
                Some(ingredient) => ingredient.update(msg),
                None => unreachable!(),
            }
        }
    }

    fn output(&mut self) -> Result<RecipeBody, ()> {
        let name_result = self.name_input.get_output();
        let ingredient_results: Vec<Result<Ingredient, ()>> = self
            .ingredient_inputs
            .iter_mut()
            .map(|ingredient_input| ingredient_input.get_output())
            .collect();
        if !self.name_input.has_error()
            && self
                .ingredient_inputs
                .iter()
                .all(|ingredient| !ingredient.has_error())
        {
            let name = name_result.unwrap();
            let ingredients = ingredient_results
                .into_iter()
                .map(|ingredient| ingredient.unwrap())
                .collect();
            Ok(RecipeBody { name, ingredients })
        } else {
            Err(())
        }
    }
    fn clear(&mut self) {
        self.name_input.clear();
        self.ingredient_inputs.clear();
    }
}

impl EditableCollection<RecipeBody> for RecipeInput {
    fn begin_edit(&mut self, recipe: &RecipeBody, unit_system: UnitSystem) {
        self.clear();
        self.name_input.update(recipe.name.clone());
        for ingredient in &recipe.ingredients {
            self.add_ingredient();
            let last_input = self.ingredient_inputs.last_mut().unwrap();
            let category = self
                .categories
                .iter()
                .find(|category| category.id == ingredient.category)
                .expect("Category must be valid");
            last_input.category_input.update(category.clone());
            last_input
                .quantity_input
                .update(ingredient.quantity.value(unit_system).to_string());
            last_input
                .unit_input
                .update(ingredient.quantity.unit(unit_system));
        }
    }
}

impl Viewable<Message> for RecipeInput {
    fn view(&self) -> iced::Element<'_, Message> {
        let name = self.name_input.view();
        let ingredients = column(self.ingredient_inputs.iter().enumerate().map(
            |(idx, ingredient)| {
                row![
                    ingredient.view(),
                    button("X").on_press(Message::Recipes(recipes::Message::RemoveIngredient(idx)))
                ]
                .into()
            },
        ));
        column![name, ingredients].into()
    }
}
