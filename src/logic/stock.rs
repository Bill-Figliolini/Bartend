//! Stock checks shared by serving and inventory: totalling usage, deciding
//! whether a recipe can be made from what is in stock, and how full an item
//! is.

use crate::{
    logic::{CategoryService, ItemService},
    models::{CategoryID, ItemBody, Quantity, RecipeBody},
};

/// Below this share of its full size, an item is [`StockLevel::Low`].
const LOW_BELOW: f32 = 0.25;
/// Below this share of its full size, an item is [`StockLevel::Critical`].
const CRITICAL_BELOW: f32 = 0.10;

/// How much of an item is left, compared with its full size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StockLevel {
    /// At least a quarter left.
    Healthy,
    /// Under a quarter left.
    Low,
    /// Under a tenth left.
    Critical,
}

impl StockLevel {
    /// The level `item` is at, or `None` if it has no
    /// [fill fraction](ItemBody::fill_fraction) to judge by.
    #[must_use]
    pub fn of(item: &ItemBody) -> Option<Self> {
        let fraction = item.fill_fraction()?;
        Some(if fraction < CRITICAL_BELOW {
            Self::Critical
        } else if fraction < LOW_BELOW {
            Self::Low
        } else {
            Self::Healthy
        })
    }
}

/// The total quantity for each distinct key, in order of first appearance.
///
/// # Errors
/// The first key whose quantities are of different kinds (e.g. a volume and
/// a mass), since those cannot be added.
pub fn totals_by_key<K: PartialEq>(
    quantities: impl IntoIterator<Item = (K, Quantity)>,
) -> Result<Vec<(K, Quantity)>, K> {
    let mut totals: Vec<(K, Quantity)> = Vec::new();
    for (key, quantity) in quantities {
        match totals.iter_mut().find(|(seen, _)| *seen == key) {
            Some((_, total)) => match total.checked_add(quantity) {
                Some(sum) => *total = sum,
                None => return Err(key),
            },
            None => totals.push((key, quantity)),
        }
    }
    Ok(totals)
}

/// Whether there is enough stock to make one serving of `recipe`.
///
/// This is an estimate for flagging recipes, not a guarantee: each category
/// is checked against its own stock, so two categories that share items
/// (such as Gin and its child London Dry Gin) can both count the same
/// bottle. Items stocked in a different kind of quantity than the
/// ingredient (e.g. by mass rather than volume) never count towards it.
pub fn can_make(
    recipe: &RecipeBody,
    item_service: &ItemService,
    category_service: &CategoryService,
) -> bool {
    let ingredients = recipe
        .ingredients
        .iter()
        .map(|ingredient| (ingredient.category, ingredient.quantity));
    let Ok(needed) = totals_by_key(ingredients) else {
        // One category is asked for in two kinds of quantity; no single
        // stock total can be compared against both.
        return false;
    };
    needed.iter().all(|(category, needed)| {
        category_stock(*category, *needed, item_service, category_service).covers(*needed)
            == Some(true)
    })
}

/// The total stock satisfying `category`, counting only items of the same
/// kind of quantity as `like`.
fn category_stock(
    category: CategoryID,
    like: Quantity,
    item_service: &ItemService,
    category_service: &CategoryService,
) -> Quantity {
    category_service
        .satisfying_items(category)
        .into_iter()
        .filter_map(|id| item_service.get(id).ok())
        .fold(like * 0.0, |total, item| {
            total.checked_add(item.quantity).unwrap_or(total)
        })
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::{
        models::{CategoryBody, Ingredient, ItemBody, ItemID, Unit},
        persistence::Database,
    };

    fn ml(amount: f32) -> Quantity {
        Quantity::new(amount, Unit::Milliliter)
    }

    /// Real services over an in-memory database, with helpers to stock it.
    struct Bar {
        db: Database,
        items: ItemService,
        categories: CategoryService,
    }
    impl Bar {
        fn new() -> Self {
            let db = Database::new(Connection::open_in_memory().unwrap()).unwrap();
            let items = ItemService::new(&db.item_db()).unwrap();
            let categories = CategoryService::new(&db.category_db()).unwrap();
            Self {
                db,
                items,
                categories,
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
        fn stock(&mut self, category: CategoryID, quantity: Quantity) -> ItemID {
            let body = ItemBody {
                name: "bottle".to_string(),
                quantity,
                full_size: None,
            };
            let id = self.items.add(&self.db.item_db(), &body).unwrap();
            self.categories
                .add_item_mapping(&self.db.category_db(), id, category)
                .unwrap();
            id
        }
        fn can_make(&self, ingredients: &[(CategoryID, Quantity)]) -> bool {
            let recipe = RecipeBody {
                name: "Test".to_string(),
                ingredients: ingredients
                    .iter()
                    .map(|&(category, quantity)| Ingredient { category, quantity })
                    .collect(),
            };
            can_make(&recipe, &self.items, &self.categories)
        }
    }

    fn bottle_at(stock: f32) -> ItemBody {
        ItemBody {
            name: "Tanqueray".to_string(),
            quantity: ml(stock),
            full_size: Some(ml(1000.0)),
        }
    }

    #[test]
    fn stock_level_follows_the_share_left() {
        let cases = [
            (1000.0, StockLevel::Healthy),
            (250.0, StockLevel::Healthy),
            (249.0, StockLevel::Low),
            (100.0, StockLevel::Low),
            (99.0, StockLevel::Critical),
            (0.0, StockLevel::Critical),
        ];
        for (stock, expected) in cases {
            assert_eq!(
                StockLevel::of(&bottle_at(stock)),
                Some(expected),
                "{stock} ml"
            );
        }
    }

    #[test]
    fn no_stock_level_without_a_full_size() {
        let mut item = bottle_at(10.0);
        item.full_size = None;
        assert_eq!(StockLevel::of(&item), None);
    }

    #[test]
    fn totals_sum_each_key_in_first_seen_order() {
        let totals = totals_by_key([('b', ml(10.0)), ('a', ml(5.0)), ('b', ml(15.0))]);
        assert_eq!(totals, Ok(vec![('b', ml(25.0)), ('a', ml(5.0))]));
    }

    #[test]
    fn totals_reject_mixed_kinds_under_one_key() {
        let mixed = [
            ('a', ml(10.0)),
            ('b', ml(1.0)),
            ('b', Quantity::new(1.0, Unit::Gram)),
        ];
        assert_eq!(totals_by_key(mixed), Err('b'));
    }

    #[test]
    fn makeable_when_stock_covers_every_ingredient() {
        let mut bar = Bar::new();
        let gin = bar.category("Gin");
        let campari = bar.category("Campari");
        bar.stock(gin, ml(50.0));
        bar.stock(campari, ml(30.0));

        assert!(bar.can_make(&[(gin, ml(50.0)), (campari, ml(30.0))]));
        assert!(!bar.can_make(&[(gin, ml(50.0)), (campari, ml(31.0))]));
    }

    #[test]
    fn stock_is_pooled_across_bottles_and_ingredients_of_a_category() {
        let mut bar = Bar::new();
        let gin = bar.category("Gin");
        bar.stock(gin, ml(40.0));
        bar.stock(gin, ml(30.0));

        assert!(bar.can_make(&[(gin, ml(70.0))]), "split across bottles");
        assert!(bar.can_make(&[(gin, ml(35.0)), (gin, ml(35.0))]));
        assert!(!bar.can_make(&[(gin, ml(40.0)), (gin, ml(40.0))]));
    }

    #[test]
    fn child_category_stock_satisfies_the_parent() {
        let mut bar = Bar::new();
        let gin = bar.category("Gin");
        let london_dry = bar.category("London Dry Gin");
        bar.categories
            .add_category_relation(&bar.db.category_db(), gin, london_dry)
            .unwrap();
        bar.stock(london_dry, ml(50.0));

        assert!(bar.can_make(&[(gin, ml(50.0))]));
    }

    #[test]
    fn items_of_another_kind_do_not_count() {
        let mut bar = Bar::new();
        let sugar = bar.category("Sugar");
        bar.stock(sugar, Quantity::new(500.0, Unit::Gram));

        assert!(!bar.can_make(&[(sugar, ml(10.0))]));
        assert!(bar.can_make(&[(sugar, Quantity::new(10.0, Unit::Gram))]));
    }

    #[test]
    fn mixed_kinds_for_one_category_cannot_be_made() {
        let mut bar = Bar::new();
        let bitters = bar.category("Bitters");
        bar.stock(bitters, ml(100.0));

        assert!(!bar.can_make(&[
            (bitters, ml(1.0)),
            (bitters, Quantity::new(2.0, Unit::Dash))
        ]));
    }

    #[test]
    fn a_recipe_without_ingredients_can_always_be_made() {
        let bar = Bar::new();
        assert!(bar.can_make(&[]));
    }
}
