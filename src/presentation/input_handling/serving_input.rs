use iced::{
    Element,
    widget::{column, row, text},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{CategoryID, Ingredient, Item, ItemID, ItemUse, Quantity, Recipe, UnitSystem},
    presentation::{
        input_handling::{
            Field, FormError, IngredientField, RowKey, Rows, Validator, discard_stale_row,
        },
        widget::input::RequiredPick,
    },
};

/// An edit to a [`ServingInput`].
#[derive(Debug, Clone)]
pub enum ServingMessage {
    /// Choosing a recipe replaces every ingredient row.
    Recipe(Recipe),
    Item(RowKey, Item),
}

#[derive(Debug)]
pub struct ServingInput {
    recipe: RequiredPick<Recipe>,
    ingredients: Rows<IngredientUseInput>,
    /// Set once a save has been attempted; see `CategoryInput::show_errors`.
    show_errors: bool,
}
impl ServingInput {
    pub fn new(recipe_service: &RecipeService) -> Self {
        Self {
            recipe: RequiredPick::new(recipe_service.get_all(), None, "Recipe"),
            ingredients: Rows::default(),
            show_errors: false,
        }
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(
        &self,
        category_service: &CategoryService,
        unit_system: UnitSystem,
    ) -> Element<'_, ServingMessage> {
        let recipe_selector = self.recipe.view(ServingMessage::Recipe, self.show_errors);
        let ingredient_use_selectors = column(self.ingredients.iter().map(|(key, ingredient)| {
            ingredient
                .view(category_service, unit_system, self.show_errors)
                .map(move |item| ServingMessage::Item(key, item))
        }));
        column![recipe_selector, ingredient_use_selectors].into()
    }
    pub fn update(
        &mut self,
        msg: ServingMessage,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) {
        match msg {
            ServingMessage::Recipe(selected_recipe) => {
                // Rows from the previous recipe are dropped; their keys are
                // never reused, so late messages for them are discarded.
                self.ingredients.clear();
                for ingredient in &selected_recipe.body.ingredients {
                    self.ingredients.push(IngredientUseInput::new(
                        ingredient,
                        item_service,
                        category_service,
                    ));
                }
                self.recipe.select(selected_recipe);
            }
            ServingMessage::Item(key, item) => match self.ingredients.get_mut(key) {
                Some(row) => row.ingredient.select(item),
                None => discard_stale_row(key, &item),
            },
        }
    }

    pub fn output(&mut self) -> Result<Vec<ItemUse>, FormError> {
        self.show_errors = true;
        let mut validator = Validator::default();
        let recipe = validator.check(Field::Recipe, self.recipe.value());
        // Collect into a Vec first so every row is validated, rather than
        // stopping at the first invalid one.
        let used_items: Vec<Option<ItemUse>> = self
            .ingredients
            .values()
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
        self.show_errors = false;
    }

    /// The key of the `index`th ingredient row.
    #[cfg(test)]
    pub fn row_key(&self, index: usize) -> RowKey {
        self.ingredients.key(index)
    }
}

#[derive(Debug)]
struct IngredientUseInput {
    category: CategoryID,
    quantity: Quantity,
    ingredient: RequiredPick<Item>,
}
impl IngredientUseInput {
    fn new(
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
            ingredient: RequiredPick::new(valid_ingredients, None, "Ingredient"),
        }
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn view(
        &self,
        category_service: &CategoryService,
        unit_system: UnitSystem,
        show_error: bool,
    ) -> Element<'_, Item> {
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
        let picker = self.ingredient.view(|item| item, show_error);
        row![category, quantity, picker].spacing(20).into()
    }
    fn output(&self, index: usize, validator: &mut Validator) -> Option<ItemUse> {
        let field = Field::Ingredient {
            index,
            field: IngredientField::Item,
        };
        let selected_item = validator.check(field, self.ingredient.value())?;
        Some(ItemUse {
            id: selected_item.id,
            quantity: self.quantity,
        })
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::{
        models::{ItemBody, RecipeBody, RecipeID, Unit},
        persistence::Database,
        presentation::widget::input::Error,
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
        let mut ingredients = Rows::default();
        ingredients.push(IngredientUseInput {
            category: CategoryID(1),
            quantity: Quantity::new(50.0, Unit::Milliliter),
            ingredient: RequiredPick::new(vec![gin()], None, "Ingredient"),
        });
        ServingInput {
            recipe: RequiredPick::new(vec![recipe()], None, "Recipe"),
            ingredients,
            show_errors: false,
        }
    }

    #[test]
    fn no_recipe_selected_is_rejected() {
        let mut input = ServingInput {
            ingredients: Rows::default(),
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
        input.update(ServingMessage::Recipe(recipe()), &items, &categories);

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
        input.recipe.select(recipe());
        let key = input.row_key(0);
        input.update(ServingMessage::Item(key, gin()), &items, &categories);

        let uses = input.output().unwrap();
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].id, ItemID(7));
        assert_eq!(uses[0].quantity, Quantity::new(50.0, Unit::Milliliter));
    }

    #[test]
    fn item_for_a_previous_recipes_row_is_discarded() {
        let (items, categories) = services();
        let mut input = serving_input();
        let old_row = input.row_key(0);
        input.update(ServingMessage::Recipe(recipe()), &items, &categories);

        input.update(ServingMessage::Item(old_row, gin()), &items, &categories);

        assert_ne!(input.row_key(0), old_row);
        assert!(input.output().is_err(), "the new row has no item chosen");
    }

    #[test]
    fn rows_offer_only_items_with_enough_stock() {
        use crate::persistence::repositories::{CategoryRepository, ItemRepository};
        let db = Database::new(Connection::open_in_memory().unwrap()).unwrap();
        let gin = db
            .category_db()
            .insert(&crate::models::CategoryBody {
                name: "Gin".to_string(),
            })
            .unwrap();
        for (name, ml) in [("Full bottle", 750.0), ("Nearly empty", 10.0)] {
            let id = db
                .item_db()
                .insert(&ItemBody {
                    name: name.to_string(),
                    quantity: Quantity::new(ml, Unit::Milliliter),
                })
                .unwrap();
            db.category_db().map_insert(id, gin).unwrap();
        }
        let items = ItemService::new(&db.item_db()).unwrap();
        let categories = CategoryService::new(&db.category_db()).unwrap();
        let mut recipes = RecipeService::new(&db.recipe_db()).unwrap();
        let shot = RecipeBody {
            name: "Gin Shot".to_string(),
            ingredients: vec![Ingredient {
                category: gin,
                quantity: Quantity::new(50.0, Unit::Milliliter),
            }],
        };
        recipes.add(&db.recipe_db(), shot).unwrap();
        let mut input = ServingInput::new(&recipes);

        let recipe = recipes.get_all().remove(0);
        input.update(ServingMessage::Recipe(recipe), &items, &categories);

        assert_eq!(input.ingredients.len(), 1);
        // The picker's options are only visible through its Debug output.
        let options = format!(
            "{:?}",
            input.ingredients.values().next().unwrap().ingredient
        );
        assert!(options.contains("Full bottle"), "{options}");
        assert!(!options.contains("Nearly empty"), "{options}");
    }
}
