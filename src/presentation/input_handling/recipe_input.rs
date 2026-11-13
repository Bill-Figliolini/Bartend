use iced::widget::{Id, button, column, row};

use crate::{
    models::{Category, Config, Ingredient, Quantity, RecipeBody, Unit, UnitSystem},
    presentation::{
        application::Message,
        input_handling::{
            Field, FormError, IngredientField, InputMessage, UnknownTarget, Validator, route,
        },
        screen::recipes,
        widget::input::{NumberField, RequiredPick, StringField},
    },
};

#[derive(Debug)]
pub struct RecipeInput {
    name_input: StringField,
    ingredient_inputs: Vec<IngredientInput>,

    categories: Vec<Category>,

    msgfn: fn(InputMessage) -> Message,
    unit_system: UnitSystem,
    /// Set once a save has been attempted; see `CategoryInput::show_errors`.
    show_errors: bool,
}
#[derive(Debug)]
pub struct IngredientInput {
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
    fn has_id(&self, id: &Id) -> bool {
        self.category.id() == id || self.quantity.id() == id || self.unit.id() == id
    }
    fn update(&mut self, msg: InputMessage) -> Result<(), UnknownTarget> {
        match msg {
            InputMessage::String(id, str) if id == *self.quantity.id() => {
                self.quantity.set(str);
            }
            InputMessage::Unit(id, unit) if id == *self.unit.id() => {
                self.unit.select(unit);
            }
            InputMessage::Category(id, category) if id == *self.category.id() => {
                self.category.select(category);
            }
            msg => return Err(UnknownTarget(msg)),
        }
        Ok(())
    }
    /// Validates every field of this row, recording failures under `index`.
    fn get_output(&self, index: usize, validator: &mut Validator) -> Option<Ingredient> {
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
    fn view(&self, msg: fn(InputMessage) -> Message, show: bool) -> iced::Element<'_, Message> {
        row![
            self.category
                .view(route(msg, self.category.id(), InputMessage::Category), show),
            self.quantity
                .view(route(msg, self.quantity.id(), InputMessage::String), show),
            self.unit
                .view(route(msg, self.unit.id(), InputMessage::Unit), show),
        ]
        .into()
    }
}
impl RecipeInput {
    pub fn new(
        config: &Config,
        msg: fn(InputMessage) -> Message,
        categories: Vec<Category>,
    ) -> Self {
        RecipeInput {
            name_input: StringField::new("Name"),
            ingredient_inputs: Vec::new(),
            categories,
            msgfn: msg,
            unit_system: config.default_units(),
            show_errors: false,
        }
    }
    pub fn add_ingredient(&mut self) {
        self.ingredient_inputs.push(IngredientInput::new(
            self.categories.clone(),
            self.unit_system,
        ));
    }
    /// Ignores an index that is out of range, e.g. from a remove button
    /// pressed twice before the view refreshed.
    pub fn remove_ingredient(&mut self, index: usize) {
        if index < self.ingredient_inputs.len() {
            self.ingredient_inputs.remove(index);
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self) -> iced::Element<'_, Message> {
        let name = self.name_input.view(
            route(self.msgfn, self.name_input.id(), InputMessage::String),
            self.show_errors,
        );
        let ingredients = column(self.ingredient_inputs.iter().enumerate().map(
            |(idx, ingredient)| {
                row![
                    ingredient.view(self.msgfn, self.show_errors),
                    button("X").on_press(Message::Recipes(recipes::Message::RemoveIngredient(idx)))
                ]
                .into()
            },
        ));
        column![name, ingredients].into()
    }
    pub fn update(&mut self, msg: InputMessage) -> Result<(), UnknownTarget> {
        match msg {
            InputMessage::String(id, str) if self.name_input.id() == &id => {
                self.name_input.set(str);
                Ok(())
            }
            msg => {
                let id = msg.id();
                match self
                    .ingredient_inputs
                    .iter_mut()
                    .find(|ingredient| ingredient.has_id(&id))
                {
                    Some(ingredient) => ingredient.update(msg),
                    None => Err(UnknownTarget(msg)),
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
            .iter()
            .enumerate()
            .map(|(index, ingredient)| ingredient.get_output(index, &mut validator))
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
    #[cfg(test)]
    pub fn name_id(&self) -> Id {
        self.name_input.id().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::CategoryID,
        presentation::{
            input_handling::test_support::{category, config, msg},
            widget::input::Error,
        },
    };

    fn recipe_input() -> RecipeInput {
        RecipeInput::new(
            &config(),
            msg,
            vec![category(1, "Gin"), category(2, "Tonic")],
        )
    }

    fn set_name(input: &mut RecipeInput, name: &str) {
        let id = input.name_input.id().clone();
        input
            .update(InputMessage::String(id, name.to_string()))
            .unwrap();
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
        input.add_ingredient();
        input.add_ingredient();
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
    fn removing_out_of_range_ingredient_is_ignored() {
        let mut input = recipe_input();
        input.add_ingredient();
        input.remove_ingredient(5);
        assert_eq!(input.ingredient_inputs.len(), 1);
        input.remove_ingredient(0);
        assert!(input.ingredient_inputs.is_empty());
    }

    #[test]
    fn message_for_removed_ingredient_is_rejected() {
        let mut input = recipe_input();
        input.add_ingredient();
        let stale_id = input.ingredient_inputs[0].quantity.id().clone();
        input.remove_ingredient(0);
        assert!(
            input
                .update(InputMessage::String(stale_id, "5".to_string()))
                .is_err()
        );
    }

    #[test]
    fn wrong_message_kind_for_name_is_rejected() {
        let mut input = recipe_input();
        let name_id = input.name_input.id().clone();
        assert!(
            input
                .update(InputMessage::Unit(name_id, Unit::Gram))
                .is_err()
        );
    }

    #[test]
    fn ingredient_rows_accept_their_own_fields() {
        let mut input = recipe_input();
        set_name(&mut input, "Bitters");
        input.add_ingredient();
        let row = &input.ingredient_inputs[0];
        let (category_id, quantity_id, unit_id) = (
            row.category.id().clone(),
            row.quantity.id().clone(),
            row.unit.id().clone(),
        );

        input
            .update(InputMessage::Category(category_id, category(2, "Tonic")))
            .unwrap();
        input
            .update(InputMessage::String(quantity_id, "2".to_string()))
            .unwrap();
        input
            .update(InputMessage::Unit(unit_id.clone(), Unit::Dash))
            .unwrap();
        assert!(
            input
                .update(InputMessage::String(unit_id, "wrong kind".to_string()))
                .is_err()
        );

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
