mod category_input;
mod item_input;
mod recipe_input;
mod serving_input;

use std::fmt::Display;

use crate::presentation::widget::input::Error;

/// Identifies one row of a [`Rows`] for as long as it exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowKey(u64);

/// The repeated rows of a form, such as a recipe's ingredients.
///
/// Each row gets a key that is never reused, even after `clear`, so a
/// message for a removed row cannot reach a different one. That matters
/// because a message can still be in flight for a row that was removed in
/// the same frame (e.g. an edit, or a second click on its remove button).
#[derive(Debug)]
struct Rows<T> {
    rows: Vec<(RowKey, T)>,
    next_key: u64,
}

impl<T> Default for Rows<T> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            next_key: 0,
        }
    }
}

impl<T> Rows<T> {
    fn push(&mut self, row: T) -> RowKey {
        let key = RowKey(self.next_key);
        self.next_key += 1;
        self.rows.push((key, row));
        key
    }

    /// Returns whether a row with `key` was there to remove.
    fn remove(&mut self, key: RowKey) -> bool {
        let before = self.rows.len();
        self.rows.retain(|(row_key, _)| *row_key != key);
        self.rows.len() != before
    }

    fn get_mut(&mut self, key: RowKey) -> Option<&mut T> {
        self.rows
            .iter_mut()
            .find(|(row_key, _)| *row_key == key)
            .map(|(_, row)| row)
    }

    fn iter(&self) -> impl Iterator<Item = (RowKey, &T)> {
        self.rows.iter().map(|(key, row)| (*key, row))
    }

    fn values(&self) -> impl Iterator<Item = &T> {
        self.rows.iter().map(|(_, row)| row)
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.rows.len()
    }

    fn clear(&mut self) {
        self.rows.clear();
    }

    #[cfg(test)]
    fn key(&self, index: usize) -> RowKey {
        self.rows[index].0
    }
}

/// Drops a message for a row that has since been removed; see [`Rows`].
/// Such messages are harmless. Builds with the `devtools` feature log them.
fn discard_stale_row(key: RowKey, msg: &dyn std::fmt::Debug) {
    #[cfg(feature = "devtools")]
    eprintln!("Message for a removed row {key:?}: {msg:?}");
    #[cfg(not(feature = "devtools"))]
    let _ = (key, msg);
}

/// A field of one of the input forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Name,
    Quantity,
    Unit,
    Recipe,
    Ingredient {
        index: usize,
        field: IngredientField,
    },
}

/// A field within one ingredient row of a recipe or serving form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngredientField {
    Category,
    Quantity,
    Unit,
    Item,
}

/// Every field that failed validation when a form was submitted.
///
/// The same errors are also shown inline next to each field; this value lets
/// callers know *which* fields failed rather than only that something did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormError(Vec<(Field, Error)>);

/// Collects field results while a form is validated, so that every field is
/// checked (and shows its error) instead of stopping at the first failure.
#[derive(Debug, Default)]
struct Validator(Vec<(Field, Error)>);

impl Validator {
    fn check<T>(&mut self, field: Field, result: Result<T, Error>) -> Option<T> {
        result.map_err(|error| self.0.push((field, error))).ok()
    }

    fn into_error(self) -> FormError {
        FormError(self.0)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::Name => write!(f, "Name"),
            Field::Quantity => write!(f, "Quantity"),
            Field::Unit => write!(f, "Unit"),
            Field::Recipe => write!(f, "Recipe"),
            Field::Ingredient { index, field } => write!(f, "Ingredient {} {field}", index + 1),
        }
    }
}

impl Display for IngredientField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            IngredientField::Category => "Category",
            IngredientField::Quantity => "Quantity",
            IngredientField::Unit => "Unit",
            IngredientField::Item => "Item",
        };
        write!(f, "{text}")
    }
}

impl Display for FormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, (field, error)) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{field}: {error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for FormError {}

pub use {
    category_input::{CategoryInput, CategoryMessage},
    item_input::{ItemInput, ItemMessage},
    recipe_input::{RecipeInput, RecipeMessage},
    serving_input::{ServingInput, ServingMessage},
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_keys_are_never_reused() {
        let mut rows = Rows::default();
        let a = rows.push("a");
        rows.clear();
        let b = rows.push("b");
        assert_ne!(a, b);
        assert!(rows.get_mut(a).is_none());
        assert_eq!(rows.get_mut(b), Some(&mut "b"));
    }

    #[test]
    fn removing_a_row_leaves_the_others() {
        let mut rows = Rows::default();
        let a = rows.push("a");
        let b = rows.push("b");
        assert!(rows.remove(a));
        assert!(!rows.remove(a), "a second remove finds nothing");
        assert_eq!(rows.iter().collect::<Vec<_>>(), vec![(b, &"b")]);
        assert_eq!(rows.values().collect::<Vec<_>>(), vec![&"b"]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows.key(0), b);
    }

    #[test]
    fn stale_row_messages_are_discarded() {
        let mut rows = Rows::default();
        let key = rows.push(());
        discard_stale_row(key, &"Gin");
    }

    #[test]
    fn fields_display_human_names() {
        assert_eq!(Field::Name.to_string(), "Name");
        assert_eq!(Field::Quantity.to_string(), "Quantity");
        assert_eq!(Field::Unit.to_string(), "Unit");
        assert_eq!(Field::Recipe.to_string(), "Recipe");
        let ingredient = |field| Field::Ingredient { index: 0, field };
        assert_eq!(
            ingredient(IngredientField::Category).to_string(),
            "Ingredient 1 Category"
        );
        assert_eq!(
            ingredient(IngredientField::Quantity).to_string(),
            "Ingredient 1 Quantity"
        );
        assert_eq!(
            ingredient(IngredientField::Unit).to_string(),
            "Ingredient 1 Unit"
        );
        assert_eq!(
            ingredient(IngredientField::Item).to_string(),
            "Ingredient 1 Item"
        );
    }

    #[test]
    fn form_errors_list_every_failure() {
        let error = FormError(vec![
            (Field::Name, Error::StringEmpty),
            (Field::Quantity, Error::QuantityInvalid),
        ]);
        assert_eq!(
            error.to_string(),
            format!(
                "Name: {}; Quantity: {}",
                Error::StringEmpty,
                Error::QuantityInvalid
            )
        );
    }
}

#[cfg(test)]
mod test_support {
    use crate::models::{Category, CategoryBody, CategoryID, Config};

    /// A default (metric) config, backed by a throwaway directory.
    pub(super) fn config() -> Config {
        let dir = tempfile::tempdir().unwrap();
        Config::load(
            Some(dir.path().join("config.json")),
            Some(dir.path().join("bartend.db")),
        )
        .unwrap()
    }

    pub(super) fn category(id: i64, name: &str) -> Category {
        Category {
            id: CategoryID(id),
            body: CategoryBody {
                name: name.to_string(),
            },
        }
    }
}
