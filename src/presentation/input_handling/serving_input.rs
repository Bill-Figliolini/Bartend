use iced::widget::{column, row, text};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{CategoryID, Ingredient, Item, ItemID, ItemUse, Quantity, Recipe, UnitSystem},
    presentation::{
        Viewable,
        application::Message,
        input_handling::InputMessage,
        widget::input::{Input, InputContents, RequiredPickInput},
    },
};

#[derive(Debug)]
pub struct ServingInput {
    recipe: RequiredPickInput<Recipe, Message>,
    ingredients: Vec<IngredientUseInput>,
    msg: fn(InputMessage) -> Message,
}
impl ServingInput {
    pub fn new(msg: fn(InputMessage) -> Message, recipe_service: &RecipeService) -> Self {
        let recipe = RequiredPickInput::new(
            move |id, recipe| msg(InputMessage::Recipe(id, recipe)),
            recipe_service.get_all(),
            None,
            "Recipe".to_string(),
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
        category_service: &CategoryService,
        unit_system: UnitSystem,
    ) -> iced::Element<'_, Message> {
        let recipe_selector = self.recipe.view();
        let ingredient_use_selectors = column(
            self.ingredients
                .iter()
                .map(|ingredient| ingredient.view(category_service, unit_system)),
        );
        column![recipe_selector, ingredient_use_selectors].into()
    }
    pub fn update(
        &mut self,
        msg: InputMessage,
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

    pub fn output(&mut self) -> Result<Vec<ItemUse>, ()> {
        let ingredient_count = self.ingredients.len();
        let used_ingredients = self
            .ingredients
            .iter_mut()
            .fold(
                Vec::with_capacity(ingredient_count),
                |mut acc, ingredient| {
                    let item_used = ingredient.output();
                    acc.push(item_used);
                    acc
                },
            )
            .into_iter()
            .collect::<Result<Vec<ItemUse>, ()>>()?;

        self.clear();
        Ok(used_ingredients)
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
    ingredient: RequiredPickInput<Item, Message>,
}
impl IngredientUseInput {
    pub fn new(
        msg: fn(InputMessage) -> Message,
        ingredient: &Ingredient,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) -> Self {
        let valid_ingredient_ids: Vec<ItemID> = category_service
            .satisfying_items(&ingredient.category)
            .into_iter()
            .collect();
        let valid_ingredients = valid_ingredient_ids
            .into_iter()
            .map(|id| Item {
                id,
                body: item_service.get(&id).unwrap().clone(),
            })
            .filter(|item| item.body.quantity >= ingredient.quantity)
            .collect();
        Self {
            category: ingredient.category,
            quantity: ingredient.quantity,
            ingredient: RequiredPickInput::new(
                move |id, item| msg(InputMessage::Item(id, item)),
                valid_ingredients,
                None,
                "Ingredient".to_string(),
            ),
        }
    }
    pub fn view(
        &self,
        category_service: &CategoryService,
        unit_system: UnitSystem,
    ) -> iced::Element<'_, Message> {
        let category = text!(
            "{}: ID: {}",
            category_service.get(&self.category).unwrap().name.clone(),
            self.category.0
        );
        let quantity = text(self.quantity.readable(unit_system));
        row![category, quantity, self.ingredient.view()]
            .spacing(20)
            .into()
    }
    fn output(&mut self) -> Result<ItemUse, ()> {
        let selected_item = self.ingredient.get_output()?;
        Ok(ItemUse {
            id: selected_item.id,
            quantity: self.quantity,
        })
    }
}
