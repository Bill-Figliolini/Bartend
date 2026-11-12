mod category_input;
mod item_input;
mod recipe_input;
mod serving_input;

use std::fmt::Display;

use iced::widget::Id;

use crate::{
    models::{Category, Item, Recipe, Unit},
    presentation::widget::input::Error,
};

#[derive(Clone, Debug)]
pub enum InputMessage {
    String(Id, String),
    Unit(Id, Unit),
    Category(Id, Category),
    Recipe(Id, Recipe),
    Item(Id, Item),
}

impl InputMessage {
    fn id(&self) -> Id {
        match self {
            InputMessage::String(id, _)
            | InputMessage::Unit(id, _)
            | InputMessage::Category(id, _)
            | InputMessage::Recipe(id, _)
            | InputMessage::Item(id, _) => id.clone(),
        }
    }
}

/// An input message sent to a form that has no field with its id (or whose
/// field with that id takes a different kind of message).
///
/// Field ids are unique per widget, so this happens when a message arrives
/// from a widget that has since been removed or rebuilt, for example an
/// ingredient row deleted in the same frame. Such messages are harmless.
#[derive(Debug, Clone)]
pub struct UnknownTarget(InputMessage);

impl UnknownTarget {
    /// Drops the stale message. Builds with the `devtools` feature log it.
    pub fn discard(self) {
        #[cfg(feature = "devtools")]
        eprintln!("{self}");
        #[cfg(not(feature = "devtools"))]
        let _ = self;
    }
}

impl Display for UnknownTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Input message for a field not in this form: {:?}",
            self.0
        )
    }
}

impl std::error::Error for UnknownTarget {}

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
    category_input::CategoryInput, item_input::ItemInput, recipe_input::RecipeInput,
    serving_input::ServingInput,
};

#[cfg(test)]
mod tests {
    use iced::widget::Id;

    use super::*;
    use crate::models::{CategoryBody, CategoryID, ItemBody, ItemID, RecipeBody, RecipeID};

    #[test]
    fn every_message_reports_its_field_id() {
        let id = Id::unique();
        let messages = [
            InputMessage::String(id.clone(), String::new()),
            InputMessage::Unit(id.clone(), Unit::Gram),
            InputMessage::Category(
                id.clone(),
                Category {
                    id: CategoryID(1),
                    body: CategoryBody::default(),
                },
            ),
            InputMessage::Recipe(
                id.clone(),
                Recipe {
                    id: RecipeID(1),
                    body: RecipeBody::default(),
                },
            ),
            InputMessage::Item(
                id.clone(),
                Item {
                    id: ItemID(1),
                    body: ItemBody::default(),
                },
            ),
        ];
        for message in messages {
            assert_eq!(message.id(), id);
        }
    }

    #[test]
    fn unknown_targets_name_the_stale_message() {
        let stale = UnknownTarget(InputMessage::String(Id::unique(), "Gin".to_string()));
        assert!(stale.to_string().contains("\"Gin\""));
        stale.discard();
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
    use super::InputMessage;
    use crate::{
        models::{Category, CategoryBody, CategoryID, Config},
        presentation::application::Message,
    };

    /// Forms need a message constructor; tests never dispatch messages.
    pub(super) fn msg(_: InputMessage) -> Message {
        Message::NoOp
    }

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
