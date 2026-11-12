use iced::widget::{column, row, text};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{CategoryID, Ingredient, Item, ItemID, ItemUse, Quantity, Recipe, UnitSystem},
    presentation::{
        Viewable,
        application::Message,
        input_handling::{
            Field, FormError, IngredientField, InputMessage, UnknownTarget, Validator,
        },
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
    ) -> Result<(), UnknownTarget> {
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
                Ok(())
            }
            InputMessage::Item(id, selected_item) => match self
                .ingredients
                .iter_mut()
                .find(|ingredient_input| *ingredient_input.ingredient.id() == id)
            {
                Some(input) => {
                    input.ingredient.update(selected_item);
                    Ok(())
                }
                None => Err(UnknownTarget(InputMessage::Item(id, selected_item))),
            },
            msg => Err(UnknownTarget(msg)),
        }
    }

    pub fn output(&mut self) -> Result<Vec<ItemUse>, FormError> {
        let mut validator = Validator::default();
        let recipe = validator.check(Field::Recipe, self.recipe.get_output());
        // Collect into a Vec first so every row is validated, rather than
        // stopping at the first invalid one.
        let used_items: Vec<Option<ItemUse>> = self
            .ingredients
            .iter_mut()
            .enumerate()
            .map(|(index, ingredient)| ingredient.output(index, &mut validator))
            .collect();
        let used_items: Option<Vec<ItemUse>> = used_items.into_iter().collect();
        match (recipe, used_items) {
            (Some(_), Some(used_items)) => {
                self.clear();
                Ok(used_items)
            }
            _ => Err(validator.into_error()),
        }
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
            .satisfying_items(ingredient.category)
            .into_iter()
            .collect();
        let valid_ingredients = valid_ingredient_ids
            .into_iter()
            .map(|id| Item {
                id,
                body: item_service.get(id).cloned().unwrap_or_default(),
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
            category_service
                .get(self.category)
                .cloned()
                .unwrap_or_default()
                .name,
            self.category.0
        );
        let quantity = text(self.quantity.readable(unit_system));
        row![category, quantity, self.ingredient.view()]
            .spacing(20)
            .into()
    }
    fn output(&mut self, index: usize, validator: &mut Validator) -> Option<ItemUse> {
        let field = Field::Ingredient {
            index,
            field: IngredientField::Item,
        };
        let selected_item = validator.check(field, self.ingredient.get_output())?;
        Some(ItemUse {
            id: selected_item.id,
            quantity: self.quantity,
        })
    }
}

#[cfg(test)]
mod tests {
    use iced::widget::Id;
    use rusqlite::Connection;

    use super::*;
    use crate::{
        models::{ItemBody, RecipeBody, RecipeID, Unit},
        persistence::Database,
        presentation::{input_handling::test_support::msg, widget::input::Error},
    };

    /// Real services over an empty in-memory database.
    fn services() -> (ItemService, CategoryService) {
        let db = Database::new(Connection::open_in_memory().unwrap()).unwrap();
        (
            ItemService::new(&db.item_db()).unwrap(),
            CategoryService::new(&db.category_db()).unwrap(),
        )
    }

    fn recipe() -> Recipe {
        Recipe {
            id: RecipeID(1),
            body: RecipeBody {
                name: "Gin Shot".to_string(),
                ingredients: vec![Ingredient {
                    category: CategoryID(1),
                    quantity: Quantity::new(50.0, Unit::Milliliter),
                }],
            },
        }
    }

    fn gin() -> Item {
        Item {
            id: ItemID(7),
            body: ItemBody {
                name: "Tanqueray".to_string(),
                quantity: Quantity::new(750.0, Unit::Milliliter),
            },
        }
    }

    /// A serving form with `recipe` offered, and one ingredient row that
    /// offers `gin`.
    fn serving_input() -> ServingInput {
        ServingInput {
            recipe: RequiredPickInput::new(
                move |id, recipe| msg(InputMessage::Recipe(id, recipe)),
                vec![recipe()],
                None,
                "Recipe".to_string(),
            ),
            ingredients: vec![IngredientUseInput {
                category: CategoryID(1),
                quantity: Quantity::new(50.0, Unit::Milliliter),
                ingredient: RequiredPickInput::new(
                    move |id, item| msg(InputMessage::Item(id, item)),
                    vec![gin()],
                    None,
                    "Ingredient".to_string(),
                ),
            }],
            msg,
        }
    }

    #[test]
    fn no_recipe_selected_is_rejected() {
        let mut input = ServingInput {
            ingredients: Vec::new(),
            ..serving_input()
        };
        assert_eq!(
            input.output().map(|uses| uses.len()),
            Err(FormError(vec![(Field::Recipe, Error::MustChooseValue)]))
        );
    }

    #[test]
    fn selecting_recipe_requires_an_item_per_ingredient() {
        let (items, categories) = services();
        let mut input = serving_input();
        let recipe_id = input.recipe.id().clone();
        input
            .update(
                InputMessage::Recipe(recipe_id, recipe()),
                &items,
                &categories,
            )
            .unwrap();

        assert_eq!(input.ingredients.len(), 1);
        assert_eq!(
            input.output().map(|uses| uses.len()),
            Err(FormError(vec![(
                Field::Ingredient {
                    index: 0,
                    field: IngredientField::Item,
                },
                Error::MustChooseValue,
            )]))
        );
    }

    #[test]
    fn complete_serving_returns_item_uses() {
        let (items, categories) = services();
        let mut input = serving_input();
        input.recipe.update(recipe());
        let item_id = input.ingredients[0].ingredient.id().clone();
        input
            .update(InputMessage::Item(item_id, gin()), &items, &categories)
            .unwrap();

        let uses = input.output().unwrap();
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].id, ItemID(7));
        assert_eq!(uses[0].quantity, Quantity::new(50.0, Unit::Milliliter));
    }

    #[test]
    fn message_for_unknown_field_is_rejected() {
        let (items, categories) = services();
        let mut input = serving_input();
        let stale = InputMessage::Item(Id::unique(), gin());
        assert!(input.update(stale, &items, &categories).is_err());
        let wrong_kind = InputMessage::String(Id::unique(), String::new());
        assert!(input.update(wrong_kind, &items, &categories).is_err());
    }
}
