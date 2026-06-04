use crate::{
    application,
    models::{Category, Item, ItemID, Recipe},
    presentation::widget::input::RequiredPickInput,
};

struct ServiceInput {
    recipe: RequiredPickInput<Recipe, application::Message>,
    ingredients: Vec<IngredientUseInput>,
}
struct IngredientUseInput {
    category: Category,
}
