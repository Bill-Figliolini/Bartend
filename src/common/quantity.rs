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

const IMPERIAL_CONVERSION_VOLUME: f32 = 29.57;
const IMPERIAL_CONVERSION_MASS: f32 = 28.35;
const TSP_THRESHOLD: f32 = 15.0;
const TSP_CONVERSION: f32 = 4.929;

impl Quantity {
    pub fn volume_from_metric(quantity: f32) -> Quantity {
        Self::Volume { quantity }
    }
    pub fn volume_from_imperial(quantity: f32) -> Quantity {
        Self::Volume {
            quantity: quantity * IMPERIAL_CONVERSION_VOLUME,
        }
    }
    #[must_use]
    pub const fn metric_value(&self) -> f32 {
        match self {
            Self::Volume { quantity }
            | Self::Mass { quantity }
            | Self::Count { quantity, name: _ } => *quantity,
        }
    }
    #[must_use]
    pub const fn imperial_value(&self) -> f32 {
        match self {
            Self::Volume { quantity } => {
                if *quantity < TSP_THRESHOLD {
                    //ml to tsp
                    *quantity / TSP_CONVERSION
                } else {
                    //ml to oz
                    *quantity / IMPERIAL_CONVERSION_VOLUME
                }
            }
            Self::Mass { quantity } => {
                //grams to oz
                *quantity / IMPERIAL_CONVERSION_MASS
            }
            Self::Count { quantity, name: _ } => *quantity,
        }
    }
    #[must_use]
    pub fn metric_name(&self) -> String {
        match self {
            Self::Volume { quantity: _ } => "ml".to_string(),
            Self::Mass { quantity: _ } => "grams".to_string(),
            Self::Count { quantity: _, name } => name.name(),
        }
    }
    #[must_use]
    pub fn imperial_name(&self) -> String {
        match self {
            Self::Volume { quantity } => {
                if *quantity < TSP_THRESHOLD {
                    "tsp".to_string()
                } else {
                    "oz".to_string()
                }
            }
            Self::Mass { quantity: _ } => "oz".to_string(),
            Self::Count { quantity: _, name } => name.name(),
        }
    }
    #[must_use]
    pub const fn db_compatible(&self) -> (f32, i32) {
        match self {
            Self::Volume { quantity } => (*quantity, 0),
            Self::Mass { quantity } => (*quantity, 1),
            Self::Count { quantity, name } => match name {
                crate::common::quantity::CountName::Dash => (*quantity, 2),
            },
        }
    }
}

impl PartialEq for Quantity {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Volume {
                    quantity: l_quantity,
                },
                Self::Volume {
                    quantity: r_quantity,
                },
            )
            | (
                Self::Mass {
                    quantity: l_quantity,
                },
                Self::Mass {
                    quantity: r_quantity,
                },
            ) => l_quantity == r_quantity,
            (
                Self::Count {
                    quantity: l_quantity,
                    name: l_name,
                },
                Self::Count {
                    quantity: r_quantity,
                    name: r_name,
                },
            ) => l_quantity == r_quantity && l_name == r_name,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountName {
    Dash,
}

impl CountName {
    fn name(self) -> String {
        match self {
            Self::Dash => "Dash".to_string(),
        }
    }
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
