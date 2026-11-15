use std::fmt::Display;

use crate::models::{category::CategoryID, quantity::Quantity};

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct RecipeID(pub i64);

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: RecipeID,
    pub body: RecipeBody,
}
#[derive(Debug, Clone, PartialEq)]
pub struct RecipeBody {
    pub name: String,
    pub ingredients: Vec<Ingredient>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Ingredient {
    pub category: CategoryID,
    pub quantity: Quantity,
}

/// What a recipe's ingredients add up to: the total volume, and the
/// amounts that cannot join it (masses, dashes) totalled by kind.
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeSummary {
    /// The volume ingredients summed, or `None` if there are none.
    pub volume: Option<Quantity>,
    /// One total per other kind of quantity, in order of first appearance.
    pub others: Vec<Quantity>,
}

impl RecipeBody {
    /// The recipe's total volume, with its other kinds of ingredient
    /// totalled separately.
    #[must_use]
    pub fn volume_summary(&self) -> VolumeSummary {
        let mut volume: Option<Quantity> = None;
        let mut others: Vec<Quantity> = Vec::new();
        for ingredient in &self.ingredients {
            let quantity = ingredient.quantity;
            // Each total only ever gains quantities of its own kind, so the
            // sums always exist.
            let add = |total: Quantity| total.checked_add(quantity).unwrap_or(total);
            if matches!(quantity, Quantity::Volume { .. }) {
                volume = Some(volume.map_or(quantity, add));
            } else if let Some(total) = others.iter_mut().find(|total| total.same_kind(quantity)) {
                *total = add(*total);
            } else {
                others.push(quantity);
            }
        }
        VolumeSummary { volume, others }
    }
}

impl VolumeSummary {
    /// The summary for `servings` of the recipe: every total multiplied by
    /// it.
    #[must_use]
    pub fn scaled(&self, servings: f32) -> Self {
        Self {
            volume: self.volume.map(|volume| volume * servings),
            others: self.others.iter().map(|&other| other * servings).collect(),
        }
    }
}

impl Display for Recipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body.name)
    }
}

impl PartialEq for Recipe {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for RecipeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for RecipeBody {
    fn default() -> Self {
        Self {
            name: "Invalid Recipe".to_string(),
            ingredients: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipes_display_their_name_and_compare_by_id() {
        let negroni = Recipe {
            id: RecipeID(1),
            body: RecipeBody {
                name: "Negroni".to_string(),
                ingredients: Vec::new(),
            },
        };
        let placeholder = Recipe {
            id: RecipeID(1),
            body: RecipeBody::default(),
        };
        assert_eq!(negroni.to_string(), "Negroni");
        assert_eq!(negroni.id.to_string(), "1");
        assert_eq!(negroni, placeholder);
        assert_eq!(placeholder.body.name, "Invalid Recipe");
    }

    fn recipe_of(quantities: &[Quantity]) -> RecipeBody {
        RecipeBody {
            name: "Test".to_string(),
            ingredients: quantities
                .iter()
                .map(|&quantity| Ingredient {
                    category: CategoryID(1),
                    quantity,
                })
                .collect(),
        }
    }

    #[test]
    fn volume_summary_sums_volumes_and_sets_other_kinds_aside() {
        use crate::models::Unit;
        let recipe = recipe_of(&[
            Quantity::new(30.0, Unit::Milliliter),
            Quantity::new(1.0, Unit::Dash),
            Quantity::new(1.0, Unit::FluidOunce),
            Quantity::new(5.0, Unit::Gram),
            Quantity::new(2.0, Unit::Dash),
        ]);
        let summary = recipe.volume_summary();
        assert!(
            summary
                .volume
                .is_some_and(|volume| volume.approx_eq(Quantity::new(59.57, Unit::Milliliter))),
            "{summary:?}"
        );
        assert_eq!(
            summary.others,
            vec![
                Quantity::new(3.0, Unit::Dash),
                Quantity::new(5.0, Unit::Gram)
            ]
        );
    }

    #[test]
    fn scaling_a_volume_summary_multiplies_every_total() {
        use crate::models::Unit;
        let summary = recipe_of(&[
            Quantity::new(30.0, Unit::Milliliter),
            Quantity::new(2.0, Unit::Dash),
        ])
        .volume_summary();
        assert_eq!(
            summary.scaled(1.5),
            VolumeSummary {
                volume: Some(Quantity::new(45.0, Unit::Milliliter)),
                others: vec![Quantity::new(3.0, Unit::Dash)],
            }
        );
        let empty = recipe_of(&[]).volume_summary();
        assert_eq!(empty.scaled(2.0), empty);
    }

    #[test]
    fn volume_summary_of_no_volumes_has_no_volume() {
        use crate::models::Unit;
        let dashes = recipe_of(&[Quantity::new(2.0, Unit::Dash)]).volume_summary();
        assert_eq!(dashes.volume, None);
        assert_eq!(dashes.others, vec![Quantity::new(2.0, Unit::Dash)]);

        let empty = recipe_of(&[]).volume_summary();
        assert_eq!(
            empty,
            VolumeSummary {
                volume: None,
                others: Vec::new()
            }
        );
    }
}
