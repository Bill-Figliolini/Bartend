//! Which recipes an inventory item, or anything in a category, can go into.

use crate::{
    logic::{CategoryService, ItemService, RecipeService},
    models::{CategoryID, ItemID, Quantity, RecipeID},
};

/// A recipe that `item` can be poured into, from [`recipes_using`].
#[derive(Debug, Clone, PartialEq)]
pub struct RecipeUse {
    pub recipe: RecipeID,
    pub name: String,
    /// How much of the item one serving would take, if it supplied every
    /// ingredient it could.
    pub amount: Quantity,
}

/// The recipes that call for a category `item` belongs to (directly or
/// through a parent category), ordered by name.
///
/// Ingredients asked for in a different kind of quantity than the item is
/// stocked in (e.g. by mass rather than volume) are left out, since serving
/// cannot use the item for them. Empty for an unknown item.
pub fn recipes_using(
    item: ItemID,
    item_service: &ItemService,
    category_service: &CategoryService,
    recipe_service: &RecipeService,
) -> Vec<RecipeUse> {
    let Ok(stock) = item_service.get(item).map(|body| body.quantity) else {
        return Vec::new();
    };
    let categories = category_service.categories_satisfied_by(item);
    recipe_service
        .sorted_ids()
        .into_iter()
        .filter_map(|recipe| {
            let body = recipe_service.get(recipe).ok()?;
            let amount = body
                .ingredients
                .iter()
                .filter(|ingredient| categories.contains(&ingredient.category))
                .map(|ingredient| ingredient.quantity)
                .filter(|quantity| quantity.same_kind(stock))
                .reduce(|total, quantity| total.checked_add(quantity).unwrap_or(total))?;
            Some(RecipeUse {
                recipe,
                name: body.name,
                amount,
            })
        })
        .collect()
}

/// The recipes something filed under `category` could go into: those
/// calling for `category` itself or any category above it, ordered by name.
/// Empty for an unknown category.
pub fn recipes_for_category(
    category: CategoryID,
    category_service: &CategoryService,
    recipe_service: &RecipeService,
) -> Vec<RecipeID> {
    let fills = category_service.self_and_ancestors(category);
    if fills.is_empty() {
        return Vec::new();
    }
    recipe_service
        .sorted_ids()
        .into_iter()
        .filter(|&recipe| {
            recipe_service.get(recipe).is_ok_and(|body| {
                body.ingredients
                    .iter()
                    .any(|ingredient| fills.contains(&ingredient.category))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::{
        models::{CategoryBody, Ingredient, ItemBody, RecipeBody, Unit},
        persistence::Database,
    };

    fn ml(amount: f32) -> Quantity {
        Quantity::new(amount, Unit::Milliliter)
    }

    /// Real services over an in-memory database.
    struct Bar {
        db: Database,
        items: ItemService,
        categories: CategoryService,
        recipes: RecipeService,
    }
    impl Bar {
        fn new() -> Self {
            let db = Database::new(Connection::open_in_memory().unwrap()).unwrap();
            let items = ItemService::new(&db.item_db()).unwrap();
            let categories = CategoryService::new(&db.category_db()).unwrap();
            let recipes = RecipeService::new(&db.recipe_db()).unwrap();
            Self {
                db,
                items,
                categories,
                recipes,
            }
        }
        fn category(&mut self, name: &str) -> CategoryID {
            let body = CategoryBody {
                name: name.to_string(),
            };
            self.categories
                .insert(&self.db.category_db(), &body)
                .unwrap()
        }
        fn relate(&mut self, parent: CategoryID, child: CategoryID) {
            self.categories
                .add_category_relation(&self.db.category_db(), parent, child)
                .unwrap();
        }
        fn item(&mut self, category: Option<CategoryID>, quantity: Quantity) -> ItemID {
            let body = ItemBody {
                name: "bottle".to_string(),
                quantity,
                full_size: None,
            };
            let id = self.items.add(&self.db.item_db(), &body).unwrap();
            if let Some(category) = category {
                self.categories
                    .add_item_mapping(&self.db.category_db(), id, category)
                    .unwrap();
            }
            id
        }
        fn recipe(&mut self, name: &str, ingredients: &[(CategoryID, Quantity)]) -> RecipeID {
            let body = RecipeBody {
                name: name.to_string(),
                ingredients: ingredients
                    .iter()
                    .map(|&(category, quantity)| Ingredient { category, quantity })
                    .collect(),
            };
            self.recipes.add(&self.db.recipe_db(), body).unwrap()
        }
        fn uses(&self, item: ItemID) -> Vec<(String, Quantity)> {
            recipes_using(item, &self.items, &self.categories, &self.recipes)
                .into_iter()
                .map(|usage| (usage.name, usage.amount))
                .collect()
        }
    }

    #[test]
    fn lists_recipes_calling_for_the_items_category_or_its_parents_by_name() {
        let mut bar = Bar::new();
        let spirit = bar.category("Spirit");
        let gin = bar.category("Gin");
        let vodka = bar.category("Vodka");
        bar.relate(spirit, gin);
        bar.relate(spirit, vodka);
        let tanqueray = bar.item(Some(gin), ml(750.0));
        bar.recipe("Negroni", &[(gin, ml(30.0))]);
        bar.recipe("Any Shot", &[(spirit, ml(45.0))]);
        bar.recipe("Vodka Shot", &[(vodka, ml(45.0))]);

        assert_eq!(
            bar.uses(tanqueray),
            vec![
                ("Any Shot".to_string(), ml(45.0)),
                ("Negroni".to_string(), ml(30.0)),
            ]
        );
    }

    #[test]
    fn amounts_add_up_over_every_ingredient_the_item_fits() {
        let mut bar = Bar::new();
        let gin = bar.category("Gin");
        let tanqueray = bar.item(Some(gin), ml(750.0));
        bar.recipe("Double Gin", &[(gin, ml(30.0)), (gin, ml(20.0))]);

        assert_eq!(
            bar.uses(tanqueray),
            vec![("Double Gin".to_string(), ml(50.0))]
        );
    }

    #[test]
    fn ingredients_of_another_kind_are_left_out() {
        let mut bar = Bar::new();
        let sugar = bar.category("Sugar");
        let syrup = bar.item(Some(sugar), ml(500.0));
        bar.recipe("Sugar Cube", &[(sugar, Quantity::new(4.0, Unit::Gram))]);
        bar.recipe(
            "Both",
            &[(sugar, Quantity::new(4.0, Unit::Gram)), (sugar, ml(10.0))],
        );

        assert_eq!(bar.uses(syrup), vec![("Both".to_string(), ml(10.0))]);
    }

    #[test]
    fn a_category_is_used_by_recipes_calling_for_it_or_its_parents() {
        let mut bar = Bar::new();
        let spirit = bar.category("Spirit");
        let gin = bar.category("Gin");
        let old_tom = bar.category("Old Tom");
        let vodka = bar.category("Vodka");
        bar.relate(spirit, gin);
        bar.relate(gin, old_tom);
        bar.relate(spirit, vodka);
        let negroni = bar.recipe("Negroni", &[(gin, ml(30.0))]);
        let any_shot = bar.recipe("Any Shot", &[(spirit, ml(45.0))]);
        let martinez = bar.recipe("Martinez", &[(old_tom, ml(45.0)), (gin, ml(5.0))]);
        bar.recipe("Vodka Shot", &[(vodka, ml(45.0))]);
        bar.recipe("Water", &[]);

        let for_category = |category| recipes_for_category(category, &bar.categories, &bar.recipes);
        assert_eq!(for_category(gin), vec![any_shot, martinez, negroni]);
        assert_eq!(for_category(old_tom), vec![any_shot, martinez, negroni]);
        assert_eq!(for_category(spirit), vec![any_shot]);
        assert_eq!(for_category(CategoryID(999)), Vec::new());
    }

    #[test]
    fn items_without_a_category_or_unknown_items_have_no_uses() {
        let mut bar = Bar::new();
        let gin = bar.category("Gin");
        bar.recipe("Negroni", &[(gin, ml(30.0))]);
        let loose = bar.item(None, ml(750.0));

        assert_eq!(bar.uses(loose), Vec::new());
        assert_eq!(bar.uses(ItemID(999)), Vec::new());
    }
}
