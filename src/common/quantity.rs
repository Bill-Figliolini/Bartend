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
        match self {
            Quantity::Volume { quantity } => *quantity,
            Quantity::Mass { quantity } => *quantity,
            Quantity::Count { quantity, name: _ } => *quantity,
        }
    }
    fn imperial_value(&self) -> f32 {
        match self {
            Quantity::Volume { quantity } => {
                if *quantity < 15.0 {
                    //ml to tsp
                    *quantity / 4.929
                } else {
                    //ml to oz
                    *quantity / 29.57
                }
            }
            Quantity::Mass { quantity } => {
                //grams to oz
                *quantity / 28.35
            }
            Quantity::Count { quantity, name: _ } => *quantity,
        }
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
                if *quantity < 15.0 {
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
#[cfg(test)]
mod test {
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
