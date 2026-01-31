mod common;
mod item;

/// Defines potential manners in which the quantity of an ingredient can be defined.
/// Mass and Volume are handled by uom measures
/// Count is an i32 that can be multiplied into a float and interpreted by the user
pub enum Quantity {
    Mass(u32),
    Volume(u32),
    Count(u32),
}

enum Error {
    MismatchedUnits,
}

///Boundary with presentation module.
#[derive(Debug)]
pub struct BarCollection {}

impl BarCollection {
    pub fn new() -> Self {
        Self {}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
}
