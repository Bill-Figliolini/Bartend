use std::path::Path;

use iced::Length::Fill;

pub mod sqlite;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ItemID(i64);

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemID,
    pub name: String,
    pub quantity: f32,
}

//Quantity use metric as a basis,
// as it is higher resolution than imperial.
// Volume is always in ml
// Mass is always in grams
// Count are unitless.
#[derive(Debug, Clone, Copy)]
pub enum Quantity {
    Volume { quantity: f32 },
    Mass { quantity: f32 },
    Count { quantity: f32, name: CountName },
}
impl Quantity {
    fn metric_value(&self) -> f32 {
        todo!()
    }
    fn imperial_value(&self) -> f32 {
        todo!()
    }
    fn metric_name(&self) -> String {
        match self {
            Quantity::Volume { quantity: _ } => "ml".to_string(),
            Quantity::Mass { quantity: _ } => "grams".to_string(),
            Quantity::Count { quantity: _, name } => name.name(),
        }
    }
    fn imperial_name(&self) -> String {
        match self {
            Quantity::Volume { quantity } => {
                if *quantity % 5.0 == 0.0 && *quantity < 15.0 {
                    "tsp".to_string()
                } else {
                    "oz".to_string()
                }
            }
            Quantity::Mass { quantity: _ } => "oz".to_string(),
            Quantity::Count { quantity: _, name } => name.name(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CountName {
    Dash,
}
impl CountName {
    fn name(&self) -> String {
        match self {
            CountName::Dash => "Dash".to_string(),
        }
    }
}

pub trait Repository {
    fn new(file: impl AsRef<Path>) -> Self;
    fn add_item(&self, name: &str, quantity: f32) -> ItemID;
    fn get_item(&self, id: ItemID) -> Option<Item>;
    fn update_item(&self, item: Item);
    fn delete_item(&self, id: ItemID);
    fn get_all_items(&self) -> Vec<Item>;
}

#[cfg(test)]
mod test {
    use super::*;
    mod measure {
        use super::*;
        mod count {
            use super::*;
            #[test]
            fn metric_quantity_does_not_alter_counts() {
                let quantity = 2.0;
                let count = Quantity::Count {
                    quantity,
                    name: CountName::Dash,
                };

                let count_as_metric = count.metric_value();

                assert_eq!(quantity, count_as_metric);
            }
            #[test]
            fn imperial_quantity_does_not_alter_counts() {
                let quantity = 2.0;
                let count = Quantity::Count {
                    quantity,
                    name: CountName::Dash,
                };

                let count_as_imperial = count.imperial_value();

                assert_eq!(quantity, count_as_imperial);
            }
        }
        mod volume {}
        mod mass {}
    }
}
