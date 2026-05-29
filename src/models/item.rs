//! # Item
//! The base element of this program, which all others work over.
//! Currently has no responsibilities, due to Bartend currently being structured in a data-oriented manner.
//! Instead, they are free floating structs to be operated on by free functions in the Presentation, Logic, and Persistance modules
//!
//! ## Potential Future Changes
//! Variants for mappings of IDs to quantities and names could be useful for Recipes, in a later version.

use crate::models::quantity::Quantity;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ItemID(pub i64);

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemID,
    pub body: ItemBody,
}

#[derive(Debug, Clone)]
pub struct ItemBody {
    pub name: String,
    pub quantity: Quantity,
}

impl Item {}
