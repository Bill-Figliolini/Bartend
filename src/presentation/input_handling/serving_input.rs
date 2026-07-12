use std::any::Any;

use crate::{
    application,
    logic::{CategoryService, ItemService, RecipeService},
    models::{Category, Ingredient, Item, ItemID, Quantity, Recipe},
    presentation::{
        input_handling::InputMessage,
        widget::input::{Input, RequiredPickInput},
    },
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
    pub fn new(ingredient: &Ingredient, item_service: &ItemService) -> Self {
        Self {
            category: (),
            quantity: (),
            ingredient: (),
        }
    }
    pub fn view(&self) -> iced::Element<'_, application::Message> {
        todo!()
    }
}
impl ServingInput {
    pub fn new(recipe_service: &RecipeService) -> Self {
        todo!();
    }
    pub fn view(
        &self,
        item_service: &ItemService,
        recipe_service: &RecipeService,
    ) -> iced::Element<'_, application::Message> {
        todo!()
    }
    pub fn update(
        &mut self,
        msg: super::InputMessage,
        item_service: &ItemService,
        category_servicce: &CategoryService,
        recipe_service: &RecipeService,
    ) {
        match msg {
            InputMessage::Recipe(id, selected_recipe) if id == *self.recipe.id() => {
                self.recipe.update(selected_recipe);
                self.ingredients.clear();
                let ingredients = recipe_service.get_ingredients(selected_recipe.id);
                for ingredient in ingredients {
                    self.ingredients.push(IngredientUseInput::new(ingredient));
                }
            }
            InputMessage::Item(id, selected_item) => {
                if let Some(input) = self
                    .ingredients
                    .iter_mut()
                    .find(|ingredient_input| *ingredient_input.ingredient.id() == id)
                {
                    input.ingredient.update(selected_item);
                }
            }
            _ => unreachable!("Invalid message passed"),
        }
    }

    pub fn output(&mut self) -> Result<application::Message, ()> {
        todo!()
    }

    pub fn clear(&mut self) {
        self.recipe.clear();
        self.ingredients.clear();
    }
}
