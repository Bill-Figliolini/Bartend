use crate::{
    application,
    logic::{CategoryService, ItemService, RecipeService},
    models::{CategoryID, Ingredient, Item, ItemID, Quantity, Recipe},
    presentation::{
        application::Message,
        input_handling::InputMessage,
        widget::input::{Input, RequiredPickInput},
    },
};

#[derive(Debug)]
pub struct ServingInput {
    recipe: RequiredPickInput<Recipe, application::Message>,
    ingredients: Vec<IngredientUseInput>,
    msg: fn(InputMessage) -> Message,
}
impl ServingInput {
    pub fn new(msg: fn(InputMessage) -> Message, recipe_service: &RecipeService) -> Self {
        let recipe = RequiredPickInput::new(
            move |id, recipe| msg(InputMessage::Recipe(id, recipe)),
            recipe_service.get_all(),
            None,
        );
        let ingredients = Vec::new();
        Self {
            recipe,
            ingredients,
            msg,
        }
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
        category_service: &CategoryService,
    ) {
        match msg {
            InputMessage::Recipe(id, selected_recipe) if id == *self.recipe.id() => {
                self.recipe.update(selected_recipe.clone());
                self.ingredients.clear();
                let ingredients = selected_recipe.body.ingredients.clone();
                for ingredient in ingredients {
                    self.ingredients.push(IngredientUseInput::new(
                        self.msg,
                        &ingredient,
                        item_service,
                        category_service,
                    ));
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

#[derive(Debug)]
struct IngredientUseInput {
    category: CategoryID,
    quantity: Quantity,
    ingredient: RequiredPickInput<Item, application::Message>,
}
impl IngredientUseInput {
    pub fn new(
        msg: fn(InputMessage) -> Message,
        ingredient: &Ingredient,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) -> Self {
        let valid_ingredient_ids: Vec<ItemID> = item_service
            .get_all()
            .into_iter()
            .filter(|item| category_service.item_satisifies_category(item, &ingredient.category))
            .collect();
        let valid_ingredients = valid_ingredient_ids
            .into_iter()
            .map(|id| Item {
                id,
                body: item_service.get(&id).clone(),
            })
            .collect();
        Self {
            category: ingredient.category,
            quantity: ingredient.quantity,
            ingredient: RequiredPickInput::new(
                move |id, item| msg(InputMessage::Item(id, item)),
                valid_ingredients,
                None,
            ),
        }
    }
    pub fn view(&self) -> iced::Element<'_, application::Message> {
        todo!()
    }
}
