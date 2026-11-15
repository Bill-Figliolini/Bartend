//! # Item
//! The base element of this program, which all others work over.
//! Currently has no responsibilities, due to Bartend currently being structured in a data-oriented manner.
//! Instead, they are free floating structs to be operated on by free functions in the Presentation, Logic, and Persistance modules
//!
//! ## Potential Future Changes
//! Variants for mappings of IDs to quantities and names could be useful for Recipes, in a later version.

use std::fmt::Display;

use crate::models::quantity::Quantity;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ItemID(pub i64);

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemID,
    pub body: ItemBody,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemBody {
    pub name: String,
    /// How much is in stock.
    pub quantity: Quantity,
    /// How much a full bottle (or bag, etc.) holds, if known; stock is
    /// shown as a share of it. See [`ItemBody::full_size_fits`].
    pub full_size: Option<Quantity>,
}

#[derive(Debug, Clone, Copy)]
pub struct ItemUse {
    pub id: ItemID,
    pub quantity: Quantity,
}

impl Item {}

impl ItemBody {
    /// Whether the full size, if any, can be compared with the stock: a
    /// positive amount of the same kind (volume, mass, or count) as it.
    #[must_use]
    pub fn full_size_fits(&self) -> bool {
        self.full_size
            .is_none_or(|full| self.quantity.fraction_of(full).is_some())
    }

    /// How full the item is, from 0 (empty) to 1 (full), or `None` without a
    /// [fitting](Self::full_size_fits) full size. More than a full size in
    /// stock still counts as full.
    #[must_use]
    pub fn fill_fraction(&self) -> Option<f32> {
        let fraction = self.quantity.fraction_of(self.full_size?)?;
        Some(fraction.clamp(0.0, 1.0))
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body.name)
    }
}

impl Display for ItemID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ItemBody {
    fn default() -> Self {
        Self {
            name: "Invalid Item".to_string(),
            quantity: Quantity::Volume { quantity: 0.0 },
            full_size: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Unit;

    #[test]
    fn items_display_their_name_and_compare_by_id() {
        let gin = Item {
            id: ItemID(1),
            body: ItemBody {
                name: "Tanqueray".to_string(),
                quantity: Quantity::Volume { quantity: 750.0 },
                full_size: None,
            },
        };
        let placeholder = Item {
            id: ItemID(1),
            body: ItemBody::default(),
        };
        assert_eq!(gin.to_string(), "Tanqueray");
        assert_eq!(gin.id.to_string(), "1");
        assert_eq!(gin, placeholder);
        assert_eq!(placeholder.body.name, "Invalid Item");
    }

    fn bottle(stock: Quantity, full_size: Option<Quantity>) -> ItemBody {
        ItemBody {
            name: "Tanqueray".to_string(),
            quantity: stock,
            full_size,
        }
    }

    fn ml(amount: f32) -> Quantity {
        Quantity::new(amount, Unit::Milliliter)
    }

    #[test]
    fn fill_fraction_is_stock_over_full_size() {
        let fraction = bottle(ml(150.0), Some(ml(750.0))).fill_fraction();
        assert_eq!(fraction.map(f32::to_bits), Some(0.2_f32.to_bits()));
        let empty = bottle(ml(0.0), Some(ml(750.0))).fill_fraction();
        assert_eq!(empty.map(f32::to_bits), Some(0.0_f32.to_bits()));
    }

    #[test]
    fn more_than_a_full_size_counts_as_full() {
        let fraction = bottle(ml(1500.0), Some(ml(750.0))).fill_fraction();
        assert_eq!(fraction.map(f32::to_bits), Some(1.0_f32.to_bits()));
    }

    #[test]
    fn no_fill_fraction_without_a_fitting_full_size() {
        let unset = bottle(ml(750.0), None);
        assert!(unset.full_size_fits());
        assert_eq!(unset.fill_fraction(), None);

        let by_mass = bottle(ml(750.0), Some(Quantity::new(750.0, Unit::Gram)));
        let zero = bottle(ml(750.0), Some(ml(0.0)));
        for unfit in [by_mass, zero] {
            assert!(!unfit.full_size_fits(), "{unfit:?}");
            assert_eq!(unfit.fill_fraction(), None, "{unfit:?}");
        }
        assert!(bottle(ml(10.0), Some(ml(750.0))).full_size_fits());
    }
}
