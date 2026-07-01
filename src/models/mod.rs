mod category;
mod config;
mod item;
mod quantity;
mod recipe;

use flume::{Receiver, Sender};
pub use {
    category::{Category, CategoryBody, CategoryCommand, CategoryError, CategoryID},
    config::{Config, ConfigError, EditableConfig},
    item::{Item, ItemBody, ItemCommand, ItemID},
    quantity::{CountName, Quantity, Unit, UnitSystem},
    recipe::{Ingredient, Recipe, RecipeBody, RecipeID},
};

#[derive(Debug, Clone)]
pub struct Channel<Send, Recieve> {
    send: Sender<Send>,
    recieve: Receiver<Recieve>,
}
