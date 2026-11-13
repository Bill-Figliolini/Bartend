use iced::{
    Element,
    widget::{button, column, row},
};

use crate::{
    models::{Category, Config, Ingredient, Quantity, RecipeBody, Unit, UnitSystem},
    presentation::{
        input_handling::{
            Field, FormError, IngredientField, RowKey, Rows, Validator, discard_stale_row,
        },
        widget::input::{NumberField, RequiredPick, StringField},
    },
};

/// An edit to a [`RecipeInput`].
#[derive(Debug, Clone)]
pub enum RecipeMessage {
    Name(String),
    Row(RowKey, IngredientMessage),
    AddRow,
    RemoveRow(RowKey),
}

/// An edit to one field of an ingredient row.
#[derive(Debug, Clone)]
pub enum IngredientMessage {
    Category(Category),
    Quantity(String),
    Unit(Unit),
}

#[derive(Debug)]
pub struct RecipeInput {
    name_input: StringField,
    ingredient_inputs: Rows<IngredientInput>,

    categories: Vec<Category>,

    unit_system: UnitSystem,
    /// Set once a save has been attempted; see `CategoryInput::show_errors`.
    show_errors: bool,
}
#[derive(Debug)]
struct IngredientInput {
    category: RequiredPick<Category>,
    quantity: NumberField,
    unit: RequiredPick<Unit>,
}

impl IngredientInput {
    fn new(categories: Vec<Category>, unit_system: UnitSystem) -> Self {
        Self {
            category: RequiredPick::new(categories, None, "Category"),
            quantity: NumberField::new("Quantity"),
            unit: RequiredPick::new(Unit::get_units(), Some(unit_system.default_units()), ""),
        }
    }
    fn update(&mut self, msg: IngredientMessage) {
        match msg {
            IngredientMessage::Category(category) => self.category.select(category),
            IngredientMessage::Quantity(text) => self.quantity.set(text),
            IngredientMessage::Unit(unit) => self.unit.select(unit),
        }
    }
    /// Validates every field of this row, recording failures under `index`.
    fn output(&self, index: usize, validator: &mut Validator) -> Option<Ingredient> {
        let field = |field| Field::Ingredient { index, field };
        let category = validator.check(field(IngredientField::Category), self.category.value());
        let quantity = validator.check(field(IngredientField::Quantity), self.quantity.value());
        let unit = validator.check(field(IngredientField::Unit), self.unit.value());
        Some(Ingredient {
            category: category?.id,
            quantity: Quantity::new(quantity?, unit?),
        })
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn view(&self, show: bool) -> Element<'_, IngredientMessage> {
        row![
            self.category.view(IngredientMessage::Category, show),
            self.quantity.view(IngredientMessage::Quantity, show),
            self.unit.view(IngredientMessage::Unit, show),
        ]
        .into()
    }
}
impl RecipeInput {
    pub fn new(config: &Config, categories: Vec<Category>) -> Self {
        RecipeInput {
            name_input: StringField::new("Name"),
            ingredient_inputs: Rows::default(),
            categories,
            unit_system: config.default_units(),
            show_errors: false,
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self) -> Element<'_, RecipeMessage> {
        let name = self.name_input.view(RecipeMessage::Name, self.show_errors);
        let ingredients = column(self.ingredient_inputs.iter().map(|(key, ingredient)| {
            row![
                ingredient
                    .view(self.show_errors)
                    .map(move |msg| RecipeMessage::Row(key, msg)),
                button("X").on_press(RecipeMessage::RemoveRow(key))
            ]
            .into()
        }));
        column![name, ingredients].into()
    }
    pub fn update(&mut self, msg: RecipeMessage) {
        match msg {
            RecipeMessage::Name(text) => self.name_input.set(text),
            RecipeMessage::Row(key, msg) => match self.ingredient_inputs.get_mut(key) {
                Some(ingredient) => ingredient.update(msg),
                None => discard_stale_row(key, &msg),
            },
            RecipeMessage::AddRow => {
                self.ingredient_inputs.push(IngredientInput::new(
                    self.categories.clone(),
                    self.unit_system,
                ));
            }
            RecipeMessage::RemoveRow(key) => {
                if !self.ingredient_inputs.remove(key) {
                    discard_stale_row(key, &msg);
                }
            }
        }
    }

    /// A recipe with no ingredients is valid, so a partly written recipe can
    /// be saved and finished later.
    pub fn output(&mut self) -> Result<RecipeBody, FormError> {
        self.show_errors = true;
        let mut validator = Validator::default();
        let name = validator.check(Field::Name, self.name_input.value());
        // Collect into a Vec first so every row is validated, rather than
        // stopping at the first invalid one.
        let ingredients: Vec<Option<Ingredient>> = self
            .ingredient_inputs
            .values()
            .enumerate()
            .map(|(index, ingredient)| ingredient.output(index, &mut validator))
            .collect();
        let ingredients: Option<Vec<Ingredient>> = ingredients.into_iter().collect();
        match (name, ingredients) {
            (Some(name), Some(ingredients)) => {
                self.clear();
                Ok(RecipeBody { name, ingredients })
            }
            _ => Err(validator.into_error()),
        }
    }

    pub fn clear(&mut self) {
        self.name_input.clear();
        self.ingredient_inputs.clear();
        self.show_errors = false;
    }

    pub fn begin_edit(&mut self, recipe: &RecipeBody, unit_system: UnitSystem) {
        self.clear();
        self.name_input.set(recipe.name.clone());
        for ingredient in &recipe.ingredients {
            let mut input = IngredientInput::new(self.categories.clone(), self.unit_system);
            // A missing category leaves the picker empty; validation on save
            // then asks the user to choose one.
            if let Some(category) = self
                .categories
                .iter()
                .find(|category| category.id == ingredient.category)
            {
                input.category.select(category.clone());
            }
            input
                .quantity
                .set(ingredient.quantity.value(unit_system).to_string());
            input.unit.select(ingredient.quantity.unit(unit_system));
            self.ingredient_inputs.push(input);
        }
    }

    /// The key of the `index`th ingredient row.
    #[cfg(test)]
    pub fn row_key(&self, index: usize) -> RowKey {
        self.ingredient_inputs.key(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::CategoryID,
        presentation::{
            input_handling::test_support::{category, config},
            widget::input::Error,
        },
    };

    fn recipe_input() -> RecipeInput {
        RecipeInput::new(&config(), vec![category(1, "Gin"), category(2, "Tonic")])
    }

    fn set_name(input: &mut RecipeInput, name: &str) {
        input.update(RecipeMessage::Name(name.to_string()));
    }

    fn ingredient_error(index: usize, field: IngredientField, error: Error) -> (Field, Error) {
        (Field::Ingredient { index, field }, error)
    }

    fn gin_and_tonic() -> RecipeBody {
        RecipeBody {
            name: "Gin and Tonic".to_string(),
            ingredients: vec![
                Ingredient {
                    category: CategoryID(1),
                    quantity: Quantity::new(50.0, Unit::Milliliter),
                },
                Ingredient {
                    category: CategoryID(2),
                    quantity: Quantity::new(150.0, Unit::Milliliter),
                },
            ],
        }
    }

    #[test]
    fn recipe_without_ingredients_is_valid() {
        let mut input = recipe_input();
        set_name(&mut input, "Draft");
        assert_eq!(
            input.output(),
            Ok(RecipeBody {
                name: "Draft".to_string(),
                ingredients: Vec::new(),
            })
        );
    }

    #[test]
    fn every_invalid_ingredient_row_is_reported() {
        let mut input = recipe_input();
        input.update(RecipeMessage::AddRow);
        input.update(RecipeMessage::AddRow);
        assert_eq!(
            input.output(),
            Err(FormError(vec![
                (Field::Name, Error::StringEmpty),
                ingredient_error(0, IngredientField::Category, Error::MustChooseValue),
                ingredient_error(0, IngredientField::Quantity, Error::QuantityInvalid),
                ingredient_error(1, IngredientField::Category, Error::MustChooseValue),
                ingredient_error(1, IngredientField::Quantity, Error::QuantityInvalid),
            ]))
        );
    }

    #[test]
    fn editing_a_recipe_round_trips() {
        let mut input = recipe_input();
        input.begin_edit(&gin_and_tonic(), UnitSystem::Metric);
        assert_eq!(input.output(), Ok(gin_and_tonic()));
    }

    #[test]
    fn editing_with_missing_category_leaves_picker_empty() {
        let mut input = recipe_input();
        let mut recipe = gin_and_tonic();
        recipe.ingredients[1].category = CategoryID(99);
        input.begin_edit(&recipe, UnitSystem::Metric);

        assert_eq!(input.ingredient_inputs.len(), 2);
        assert_eq!(
            input.output(),
            Err(FormError(vec![ingredient_error(
                1,
                IngredientField::Category,
                Error::MustChooseValue
            )]))
        );
    }

    #[test]
    fn removing_a_row_twice_leaves_the_other_rows() {
        let mut input = recipe_input();
        set_name(&mut input, "Bitters");
        input.update(RecipeMessage::AddRow);
        input.update(RecipeMessage::AddRow);
        let (first, second) = (input.row_key(0), input.row_key(1));

        input.update(RecipeMessage::RemoveRow(first));
        input.update(RecipeMessage::RemoveRow(first));

        assert_eq!(input.ingredient_inputs.len(), 1);
        assert_eq!(input.row_key(0), second);
    }

    #[test]
    fn message_for_removed_row_changes_nothing() {
        let mut input = recipe_input();
        set_name(&mut input, "Bitters");
        input.update(RecipeMessage::AddRow);
        let removed = input.row_key(0);
        input.update(RecipeMessage::RemoveRow(removed));
        input.update(RecipeMessage::AddRow);

        let quantity = IngredientMessage::Quantity("5".to_string());
        input.update(RecipeMessage::Row(removed, quantity));

        // The new row did not receive the quantity meant for the old one.
        assert_eq!(
            input.output(),
            Err(FormError(vec![
                ingredient_error(0, IngredientField::Category, Error::MustChooseValue),
                ingredient_error(0, IngredientField::Quantity, Error::QuantityInvalid),
            ]))
        );
    }

    #[test]
    fn ingredient_rows_accept_their_own_fields() {
        let mut input = recipe_input();
        set_name(&mut input, "Bitters");
        input.update(RecipeMessage::AddRow);
        let key = input.row_key(0);
        let row = |msg| RecipeMessage::Row(key, msg);

        input.update(row(IngredientMessage::Category(category(2, "Tonic"))));
        input.update(row(IngredientMessage::Quantity("2".to_string())));
        input.update(row(IngredientMessage::Unit(Unit::Dash)));

        assert_eq!(
            input.output(),
            Ok(RecipeBody {
                name: "Bitters".to_string(),
                ingredients: vec![Ingredient {
                    category: CategoryID(2),
                    quantity: Quantity::new(2.0, Unit::Dash),
                }],
            })
        );
    }
}
