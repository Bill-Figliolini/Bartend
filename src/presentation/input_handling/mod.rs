//! The forms in each screen's entry area: one per screen, built from the
//! fields in [`widget::input`](crate::presentation::widget::input).
//!
//! The add-or-edit forms (items, categories and recipes) implement [`Form`]
//! and are driven by an [`Editor`]:
//!
//! - **Messages.** Each form has its own message enum (e.g. [`ItemMessage`])
//!   with a variant per field, so an edit can only reach the field it was
//!   meant for. The screen wraps it as [`EditorMessage::Input`], alongside
//!   `Save` and `Cancel`, and hands it to [`Editor::update`].
//! - **Rows.** Repeated groups of fields, such as a recipe's ingredients,
//!   are kept in [`Rows`], and their messages carry the row's [`RowKey`].
//!   Keys are never reused, so a message for a row that is gone is dropped
//!   rather than landing on another row.
//! - **Saving.** [`Form::validate`] checks every field, not just up to the
//!   first failure, and changes nothing. It returns the finished value or a
//!   [`FormError`] naming each failed field by the form's own field enum,
//!   e.g. `ItemField::Quantity` or, for a row,
//!   `RecipeField::Ingredient(key, IngredientField::Quantity)`. On success
//!   the editor clears the form and says whether to add or update.
//! - **Showing errors.** The editor shows no errors until the first save
//!   attempt, so a form the user has only just started is not covered in
//!   complaints. After it, each error is worked out afresh on every view, so
//!   it disappears as soon as the field is fixed. Any reset hides them again.
//! - **Editing.** [`Editor::toggle_edit`] clears the form and fills it from
//!   an existing value with [`Form::fill`]; cancelling clears it again, so a
//!   later save cannot add a copy of the value that was being edited.
//!
//! [`ServingInput`] is the exception: it has no edit mode and needs the
//! services to update and draw itself, so it keeps its own `show_errors`
//! flag and `output`, which validates and clears in one step.
//!
//! To add a field: add it to the form struct, a variant to the message enum
//! and to `update` (the compiler points out that match), then to `view`;
//! and if the field can fail, a variant to the field enum and a check in
//! `validate`.

mod category_input;
mod editor;
mod item_input;
mod recipe_input;
mod serving_input;

use std::fmt::{Debug, Display};

use iced::Element;

use crate::{models::UnitSystem, presentation::widget::input::Error};

/// A form that adds or edits one value, driven by an [`Editor`].
///
/// A form only holds what the user entered and checks it. When errors are
/// shown, when the form is cleared, and whether a save adds or updates are
/// all up to the editor that owns it.
pub trait Form {
    /// An edit to one of the form's fields.
    type Message: Clone + Debug + 'static;
    /// Names a field in a [`FormError`].
    type Field;
    /// What a successful save produces.
    type Output;
    /// What an edit fills the form from.
    type Value;

    fn update(&mut self, msg: Self::Message);
    /// Checks every field. Reading it changes nothing.
    fn validate(&self) -> Result<Self::Output, FormError<Self::Field>>;
    /// Empties the fields. A form may keep a choice that is handy to carry
    /// over to the next entry, such as a unit.
    fn clear(&mut self);
    /// Fills a cleared form from `value`, showing quantities in `units`.
    fn fill(&mut self, value: &Self::Value, units: UnitSystem);
    /// With `show_errors`, each field's current error is shown under it.
    fn view(&self, show_errors: bool) -> Element<'_, Self::Message>;
}

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
    /// Adds `row` at the end, under a fresh key.
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

    /// The rows in display order, with their keys.
    fn iter(&self) -> impl Iterator<Item = (RowKey, &T)> {
        self.rows.iter().map(|(key, row)| (*key, row))
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.rows.len()
    }

    /// Removes every row. Keys keep counting up from where they were.
    fn clear(&mut self) {
        self.rows.clear();
    }

    /// The key of the `index`th row.
    #[cfg(test)]
    fn key(&self, index: usize) -> RowKey {
        self.rows[index].0
    }
}

/// Drops a message for a row that has since been removed; see [`Rows`].
/// Such messages are harmless. Builds with the `devtools` feature log them.
fn discard_stale_row(key: RowKey, msg: &dyn Debug) {
    #[cfg(feature = "devtools")]
    eprintln!("Message for a removed row {key:?}: {msg:?}");
    #[cfg(not(feature = "devtools"))]
    let _ = (key, msg);
}

/// Every field that failed validation when a form was saved, keyed by that
/// form's own field type `F` (e.g. `ItemField`), in the order the fields
/// were checked.
///
/// The same errors are also shown next to each field; this value lets
/// callers know *which* fields failed rather than only that something did.
/// Fields in a row carry the row's [`RowKey`], so each one is told apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormError<F>(Vec<(F, Error)>);

impl<F: PartialEq> FormError<F> {
    /// Why `field` failed, if it did.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "no screen reports field errors yet")
    )]
    pub fn error_for(&self, field: &F) -> Option<Error> {
        self.0
            .iter()
            .find(|(failed, _)| failed == field)
            .map(|(_, error)| *error)
    }
}

impl<F: Display> Display for FormError<F> {
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

impl<F: Debug + Display> std::error::Error for FormError<F> {}

/// Collects field results while a form is validated, so that every field is
/// checked instead of stopping at the first failure.
#[derive(Debug)]
struct Validator<F>(Vec<(F, Error)>);

impl<F> Default for Validator<F> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<F> Validator<F> {
    /// Passes on a field's value, or records its error under `field`.
    fn check<T>(&mut self, field: F, result: Result<T, Error>) -> Option<T> {
        result.map_err(|error| self.0.push((field, error))).ok()
    }

    fn into_error(self) -> FormError<F> {
        FormError(self.0)
    }
}

pub use {
    category_input::{CategoryInput, CategoryMessage},
    editor::{Editor, EditorMessage, Saved},
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
        assert_eq!(rows.len(), 1);
        assert_eq!(rows.key(0), b);
    }

    #[test]
    fn stale_row_messages_are_discarded() {
        let mut rows = Rows::default();
        let key = rows.push(());
        discard_stale_row(key, &"Gin");
    }

    #[derive(Debug, PartialEq)]
    enum TestField {
        Name,
        Quantity,
        Row(RowKey),
    }

    impl Display for TestField {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }

    #[test]
    fn form_errors_list_every_failure() {
        let error = FormError(vec![
            (TestField::Name, Error::StringEmpty),
            (TestField::Quantity, Error::QuantityInvalid),
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

    #[test]
    fn error_for_finds_only_the_failed_field() {
        let mut rows = Rows::default();
        let (a, b) = (rows.push(()), rows.push(()));
        let mut validator = Validator::default();
        validator.check(TestField::Name, Ok::<_, Error>(()));
        validator.check(TestField::Row(a), Ok::<_, Error>(()));
        validator.check(TestField::Row(b), Err::<(), _>(Error::QuantityInvalid));
        let error = validator.into_error();

        assert_eq!(error.error_for(&TestField::Name), None);
        assert_eq!(error.error_for(&TestField::Row(a)), None);
        assert_eq!(
            error.error_for(&TestField::Row(b)),
            Some(Error::QuantityInvalid)
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
