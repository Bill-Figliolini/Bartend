use std::fmt::Display;

use iced::{
    Element,
    widget::{button, column, row, text},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService, totals_by_key},
    models::{
        CategoryID, Ingredient, Item, ItemID, ItemUse, Quantity, Recipe, RecipeID, UnitSystem,
    },
    presentation::{
        input_handling::{FormError, RowKey, Rows, Validator, discard_stale_row},
        widget::input::{Error, PositiveField, RequiredPick},
    },
};

/// Which way the servings stepper moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Up,
    Down,
}

/// One pour of one ingredient: the ingredient row's key, then the pour's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PourKey {
    pub ingredient: RowKey,
    pub pour: RowKey,
}

/// An edit to a [`ServingInput`].
#[derive(Debug, Clone)]
pub enum ServingMessage {
    /// Choosing a recipe replaces every ingredient row and goes back to one
    /// serving.
    Recipe(Recipe),
    Servings(String),
    StepServings(Step),
    Item(PourKey, Item),
    Amount(PourKey, String),
    /// Splits the ingredient across another item. Amounts entered for it
    /// are in the given unit system.
    AddPour(RowKey, UnitSystem),
    RemovePour(PourKey),
}

/// A field of a [`ServingInput`], as named in its [`FormError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServingField {
    Recipe,
    Servings,
    /// An ingredient whose pours do not add up to what it calls for.
    Ingredient(RowKey),
    /// The item chosen for a pour.
    PourItem(PourKey),
    /// The amount of a pour, entered once an ingredient has several.
    PourAmount(PourKey),
}

/// Rows are named without a position; see `RecipeField`'s `Display`.
impl Display for ServingField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServingField::Recipe => write!(f, "Recipe"),
            ServingField::Servings => write!(f, "Servings"),
            ServingField::Ingredient(_) => write!(f, "Ingredient"),
            ServingField::PourItem(_) => write!(f, "Ingredient Item"),
            ServingField::PourAmount(_) => write!(f, "Ingredient Amount"),
        }
    }
}

/// The form for serving a recipe: how many servings, and which items in
/// stock supply each of its ingredients.
///
/// An ingredient normally comes from one item (a *pour*) that supplies all
/// of it. It can be split across several items, such as finishing one
/// bottle and opening another; each pour then has its own amount, and the
/// amounts must add up to what the ingredient calls for.
///
/// Usage is checked against stock across the whole form, so picking one
/// bottle for two ingredients cannot take more than it holds.
#[derive(Debug)]
pub struct ServingInput {
    recipe: Option<Recipe>,
    servings: PositiveField,
    ingredients: Rows<IngredientUseInput>,
    /// Whether fields show their errors; see the [module docs](super).
    show_errors: bool,
}

impl Default for ServingInput {
    fn default() -> Self {
        Self::new()
    }
}

impl ServingInput {
    pub fn new() -> Self {
        let mut servings = PositiveField::new("Servings");
        servings.set("1".to_string());
        Self {
            recipe: None,
            servings,
            ingredients: Rows::default(),
            show_errors: false,
        }
    }

    /// The chosen recipe's id, if one is chosen.
    pub fn recipe_id(&self) -> Option<RecipeID> {
        self.recipe.as_ref().map(|recipe| recipe.id)
    }

    /// Whether stepping servings down would still leave more than zero.
    pub fn can_step_down(&self) -> bool {
        self.servings.value().is_ok_and(|servings| servings > 1.0)
    }

    /// The servings stepper, then a row per ingredient with a picker per
    /// pour. The recipe itself is chosen from the screen's master list.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(
        &self,
        category_service: &CategoryService,
        unit_system: UnitSystem,
    ) -> Element<'_, ServingMessage> {
        let show = self.show_errors;
        let step_down = self
            .can_step_down()
            .then_some(ServingMessage::StepServings(Step::Down));
        let servings = row![
            text("Servings"),
            button("-").on_press_maybe(step_down),
            self.servings.view(ServingMessage::Servings, show),
            button("+").on_press(ServingMessage::StepServings(Step::Up)),
        ]
        .spacing(10);
        let ingredients =
            column(self.ingredients.iter().map(|(key, ingredient)| {
                ingredient.view(key, category_service, unit_system, show)
            }))
            .spacing(10);
        column![servings, ingredients].spacing(16).into()
    }

    pub fn update(
        &mut self,
        msg: ServingMessage,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) {
        match msg {
            ServingMessage::Recipe(recipe) => {
                // Rows from the previous recipe are dropped; their keys are
                // never reused, so late messages for them are discarded.
                self.clear();
                self.build_rows(&recipe, item_service, category_service);
                self.recipe = Some(recipe);
            }
            ServingMessage::Servings(text) => self.servings.set(text),
            ServingMessage::StepServings(step) => self.step_servings(step),
            ServingMessage::Item(key, item) => match self.pour_mut(key) {
                Some(pour) => pour.item.select(item),
                None => discard_stale_row(key.pour, &item),
            },
            ServingMessage::Amount(key, amount) => match self.pour_mut(key) {
                Some(pour) => pour.amount.set(amount),
                None => discard_stale_row(key.pour, &amount),
            },
            ServingMessage::AddPour(key, units) => {
                let servings = self.servings.value().ok();
                match self.ingredients.get_mut(key) {
                    Some(ingredient) => ingredient.add_pour(servings, units),
                    None => discard_stale_row(key, &units),
                }
            }
            ServingMessage::RemovePour(key) => {
                let removed = self
                    .ingredients
                    .get_mut(key.ingredient)
                    .is_some_and(|ingredient| ingredient.remove_pour(key.pour));
                if !removed {
                    discard_stale_row(key.pour, &key);
                }
            }
        }
    }

    /// Brings the form up to date with the services, such as after a
    /// serving has used up stock. The recipe, servings, and every pick that
    /// is still in stock are kept, with stock levels refreshed. If the
    /// recipe's ingredients have changed its rows are rebuilt, and if it has
    /// been deleted the form is cleared.
    pub fn refresh(
        &mut self,
        recipe_service: &RecipeService,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) {
        let Some(id) = self.recipe_id() else {
            return;
        };
        let Ok(body) = recipe_service.get(id) else {
            self.clear();
            return;
        };
        let recipe = Recipe { id, body };
        let unchanged = self
            .recipe
            .as_ref()
            .is_some_and(|old| old.body.ingredients == recipe.body.ingredients);
        if unchanged {
            for (_, ingredient) in self.ingredients.iter_mut() {
                ingredient.restock(item_service, category_service);
            }
        } else {
            self.ingredients.clear();
            self.build_rows(&recipe, item_service, category_service);
        }
        // Picks up a rename, even when the ingredients are unchanged.
        self.recipe = Some(recipe);
    }

    /// Checks every field and the stock they draw on. Reading it changes
    /// nothing. Gives how much of each item the serving uses; an item used
    /// by several pours appears once per pour.
    pub fn validate(&self) -> Result<Vec<ItemUse>, FormError<ServingField>> {
        let mut validator = Validator::default();
        let recipe = validator.check(
            ServingField::Recipe,
            self.recipe.as_ref().ok_or(Error::MustChooseValue),
        );
        let servings = validator.check(ServingField::Servings, self.servings.value());
        let mut pours = Vec::new();
        let mut complete = true;
        for (key, ingredient) in self.ingredients.iter() {
            match ingredient.validate(key, servings, &mut validator) {
                Some(ingredient_pours) => pours.extend(ingredient_pours),
                None => complete = false,
            }
        }
        let overdrawn = overdrawn_items(&pours);
        for pour in pours
            .iter()
            .filter(|pour| overdrawn.contains(&pour.usage.id))
        {
            let field = ServingField::PourItem(pour.key);
            validator.check(field, Err::<(), _>(Error::NotEnoughStock));
        }
        match recipe {
            Some(_) if complete && overdrawn.is_empty() => {
                Ok(pours.into_iter().map(|pour| pour.usage).collect())
            }
            _ => Err(validator.into_error()),
        }
    }

    /// [`validate`](Self::validate)s, and from then on shows each field's
    /// errors until a serving succeeds. The form is kept either way, so the
    /// same picks can be served again.
    pub fn output(&mut self) -> Result<Vec<ItemUse>, FormError<ServingField>> {
        let result = self.validate();
        self.show_errors = result.is_err();
        result
    }

    pub fn clear(&mut self) {
        // The rows are cleared rather than replaced, so their keys keep
        // counting up and are never reused.
        self.ingredients.clear();
        let ingredients = std::mem::take(&mut self.ingredients);
        *self = Self {
            ingredients,
            ..Self::new()
        };
    }

    fn build_rows(
        &mut self,
        recipe: &Recipe,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) {
        for ingredient in &recipe.body.ingredients {
            let row = IngredientUseInput::new(ingredient, item_service, category_service);
            self.ingredients.push(row);
        }
    }

    fn step_servings(&mut self, step: Step) {
        let next = match (step, self.servings.value()) {
            (Step::Up, Ok(servings)) => servings + 1.0,
            (Step::Up, Err(_)) => 1.0,
            (Step::Down, Ok(servings)) if servings > 1.0 => servings - 1.0,
            (Step::Down, _) => return,
        };
        self.servings.set(format_number(next));
    }

    fn pour_mut(&mut self, key: PourKey) -> Option<&mut PourInput> {
        self.ingredients
            .get_mut(key.ingredient)?
            .pours
            .get_mut(key.pour)
    }

    /// The key of the `pour`th pour of the `ingredient`th ingredient row.
    #[cfg(test)]
    pub fn pour_key(&self, ingredient: usize, pour: usize) -> PourKey {
        let (ingredient, row) = self.ingredients.iter().nth(ingredient).unwrap();
        PourKey {
            ingredient,
            pour: row.pours.key(pour),
        }
    }
}

/// A validated pour: what it uses, and the stock of its item when the
/// item was offered.
#[derive(Debug)]
struct CheckedPour {
    key: PourKey,
    usage: ItemUse,
    stock: Quantity,
}

/// The items that `pours`, taken together, use more of than they have.
fn overdrawn_items(pours: &[CheckedPour]) -> Vec<ItemID> {
    let uses = pours
        .iter()
        .map(|pour| (pour.usage.id, pour.usage.quantity));
    let Ok(totals) = totals_by_key(uses) else {
        // Items are only offered for ingredients of their own kind of
        // quantity, so this cannot happen; if it did, `use_items` would
        // still refuse the serving.
        return Vec::new();
    };
    totals
        .into_iter()
        .filter(|&(id, total)| {
            pours
                .iter()
                .find(|pour| pour.usage.id == id)
                .is_some_and(|pour| pour.stock.covers(total) != Some(true))
        })
        .map(|(id, _)| id)
        .collect()
}

/// How many decimal places amounts are filled in with.
const DECIMALS: f32 = 100.0;

/// `number` to at most two decimal places, without trailing zeros.
fn format_number(number: f32) -> String {
    let rounded = (number * DECIMALS).round() / DECIMALS;
    format!("{rounded}")
}

/// [`format_number`], but rounded down, for an amount that must not exceed
/// `number`.
fn format_number_down(number: f32) -> String {
    let rounded = (number * DECIMALS).floor() / DECIMALS;
    format!("{rounded}")
}

/// Whether `total` is `required`, give or take `slack`.
fn within(total: Quantity, required: Quantity, slack: Quantity) -> bool {
    let over = total.saturating_sub(required);
    let under = required.saturating_sub(total);
    [over, under]
        .into_iter()
        .all(|gap| gap.is_some_and(|gap| slack.covers(gap) == Some(true)))
}

/// One ingredient of the chosen recipe, and the pours that supply it.
#[derive(Debug)]
struct IngredientUseInput {
    category: CategoryID,
    per_serving: Quantity,
    /// The items each pour offers: those satisfying the category that have
    /// some stock, of the same kind of quantity as the ingredient, by name.
    /// As of when the row was built or last restocked.
    candidates: Vec<Item>,
    /// Never empty.
    pours: Rows<PourInput>,
}

impl IngredientUseInput {
    /// One pour, already picked if only one item can supply it.
    fn new(
        ingredient: &Ingredient,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) -> Self {
        let candidates = candidates(ingredient, item_service, category_service);
        let mut pours = Rows::default();
        pours.push(PourInput::new(
            &candidates,
            only(&candidates),
            UnitSystem::Metric,
        ));
        Self {
            category: ingredient.category,
            per_serving: ingredient.quantity,
            candidates,
            pours,
        }
    }

    /// Refreshes the items offered. Picks that are still in stock are kept,
    /// with their new stock level; others are cleared.
    fn restock(&mut self, item_service: &ItemService, category_service: &CategoryService) {
        let ingredient = Ingredient {
            category: self.category,
            quantity: self.per_serving,
        };
        self.candidates = candidates(&ingredient, item_service, category_service);
        let single = self.pours.len() == 1;
        for (_, pour) in self.pours.iter_mut() {
            pour.restock(&self.candidates, single);
        }
    }

    /// Adds an empty pour. Its amount starts at whatever the other pours
    /// leave short of `servings`' worth. When this splits a single pour,
    /// that pour is given an amount first: all its item can supply, up to
    /// what is needed.
    fn add_pour(&mut self, servings: Option<f32>, units: UnitSystem) {
        let required = servings.map(|servings| self.per_serving * servings);
        let mut pour = PourInput::new(&self.candidates, None, units);
        if let Some(required) = required {
            let single = self.pours.len() == 1;
            if let Some((_, first)) = self.pours.iter_mut().next().filter(|_| single) {
                let stock = first.item.value().ok().map(|item| item.body.quantity);
                // Rounded down when finishing the item, so as not to ask
                // for a hair more than it holds.
                let amount = match stock {
                    Some(stock) if stock.covers(required) == Some(false) => {
                        format_number_down(stock.value(units))
                    }
                    _ => format_number(required.value(units)),
                };
                first.units = units;
                first.amount.set(amount);
            }
            let poured = self
                .pours
                .iter()
                .filter_map(|(_, pour)| pour.amount(self.per_serving).ok())
                .try_fold(self.per_serving * 0.0, Quantity::checked_add);
            let rest = poured.and_then(|poured| required.saturating_sub(poured));
            if let Some(rest) = rest.filter(|rest| !rest.is_empty()) {
                pour.amount.set(format_number(rest.value(units)));
            }
        }
        self.pours.push(pour);
    }

    /// Removes a pour, unless it is the last one. Returns whether it did.
    fn remove_pour(&mut self, key: RowKey) -> bool {
        self.pours.len() > 1 && self.pours.remove(key)
    }

    /// Validates each pour, recording failures under `key`. With a single
    /// pour, it takes the whole of `servings`' worth; with several, their
    /// amounts must add up to it, give or take the rounding of amounts
    /// filled in to two decimal places. Without valid `servings` no amount
    /// is known, but the items are still checked.
    fn validate(
        &self,
        key: RowKey,
        servings: Option<f32>,
        validator: &mut Validator<ServingField>,
    ) -> Option<Vec<CheckedPour>> {
        let required = servings.map(|servings| self.per_serving * servings);
        let single = self.pours.len() == 1;
        let mut pours = Vec::new();
        let mut amounts = Vec::new();
        let mut slack = self.per_serving * 0.0;
        for (pour_key, pour) in self.pours.iter() {
            let key = PourKey {
                ingredient: key,
                pour: pour_key,
            };
            let item = validator.check(ServingField::PourItem(key), pour.item.value());
            let amount = if single {
                required
            } else {
                let amount = pour.amount(self.per_serving);
                validator.check(ServingField::PourAmount(key), amount)
            };
            amounts.push(amount);
            let rounding = Quantity::new(0.5 / DECIMALS, self.per_serving.unit(pour.units));
            slack = slack.checked_add(rounding).unwrap_or(slack);
            if let (Some(item), Some(quantity)) = (item, amount) {
                let usage = ItemUse {
                    id: item.id,
                    quantity,
                };
                let stock = item.body.quantity;
                pours.push(CheckedPour { key, usage, stock });
            }
        }
        let poured: Option<Vec<Quantity>> = amounts.into_iter().collect();
        if let (false, Some(required), Some(poured)) = (single, required, poured) {
            let total = poured
                .into_iter()
                .try_fold(self.per_serving * 0.0, Quantity::checked_add);
            if !total.is_some_and(|total| within(total, required, slack)) {
                validator.check(
                    ServingField::Ingredient(key),
                    Err::<(), _>(Error::PoursDontAddUp),
                );
                return None;
            }
        }
        (pours.len() == self.pours.len() && required.is_some()).then_some(pours)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn view(
        &self,
        key: RowKey,
        category_service: &CategoryService,
        unit_system: UnitSystem,
        show: bool,
    ) -> Element<'_, ServingMessage> {
        let category = category_service
            .get(self.category)
            .map_or_else(|_| String::new(), |category| category.name.clone());
        let header = row![text(category), text(self.per_serving.readable(unit_system))].spacing(20);
        let split = self.pours.len() > 1;
        let pours = column(self.pours.iter().map(|(pour_key, pour)| {
            let key = PourKey {
                ingredient: key,
                pour: pour_key,
            };
            let picker = pour
                .item
                .view(move |item| ServingMessage::Item(key, item), show);
            if split {
                let unit = self.per_serving.unit(pour.units).to_string();
                row![
                    picker,
                    pour.amount
                        .view(move |amount| ServingMessage::Amount(key, amount), show),
                    text(unit),
                    button("Remove").on_press(ServingMessage::RemovePour(key)),
                ]
                .spacing(10)
                .into()
            } else {
                picker
            }
        }))
        .spacing(5);
        let add = button("+ Add bottle").on_press(ServingMessage::AddPour(key, unit_system));
        column![header, pours, add].spacing(5).into()
    }
}

/// The items offered for `ingredient`; see [`IngredientUseInput::candidates`].
fn candidates(
    ingredient: &Ingredient,
    item_service: &ItemService,
    category_service: &CategoryService,
) -> Vec<Item> {
    let mut candidates: Vec<Item> = category_service
        .satisfying_items(ingredient.category)
        .into_iter()
        .filter_map(|id| {
            let body = item_service.get(id).ok()?.clone();
            Some(Item { id, body })
        })
        .filter(|item| {
            let stock = item.body.quantity;
            stock.same_kind(ingredient.quantity) && !stock.is_empty()
        })
        .collect();
    candidates.sort_by_cached_key(|item| (item.body.name.to_lowercase(), item.id.0));
    candidates
}

/// The one candidate, if there is exactly one.
fn only(candidates: &[Item]) -> Option<Item> {
    match candidates {
        [only] => Some(only.clone()),
        _ => None,
    }
}

/// One item supplying (part of) an ingredient.
#[derive(Debug)]
struct PourInput {
    item: RequiredPick<Item>,
    /// Only read once the ingredient has several pours; a single pour
    /// takes all the ingredient needs.
    amount: PositiveField,
    /// What `amount` is entered in. Fixed when the pour is split, so that
    /// toggling the screen's units does not change what was typed.
    units: UnitSystem,
}

impl PourInput {
    fn new(candidates: &[Item], initial: Option<Item>, units: UnitSystem) -> Self {
        Self {
            item: RequiredPick::new(candidates.to_vec(), initial, "Item"),
            amount: PositiveField::new("Amount"),
            units,
        }
    }

    /// Offers `candidates` instead. The pick is kept, with its new stock
    /// level, if it is still among them. A lone pour with nothing picked
    /// takes the only candidate, as a new row would.
    fn restock(&mut self, candidates: &[Item], lone: bool) {
        let kept = self.item.value().ok().and_then(|picked| {
            candidates
                .iter()
                .find(|candidate| candidate.id == picked.id)
                .cloned()
        });
        let picked = match kept {
            Some(kept) => Some(kept),
            None if lone => only(candidates),
            None => None,
        };
        self.item = RequiredPick::new(candidates.to_vec(), picked, "Item");
    }

    /// The amount entered, as the same kind of quantity as `like`.
    fn amount(&self, like: Quantity) -> Result<Quantity, Error> {
        let unit = like.unit(self.units);
        self.amount
            .value()
            .map(|amount| Quantity::new(amount, unit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{ItemBody, Unit},
        presentation::screen::test_support::World,
    };

    fn ml(amount: f32) -> Quantity {
        Quantity::new(amount, Unit::Milliliter)
    }

    fn item(world: &World, id: ItemID) -> Item {
        let body = world.items.get(id).unwrap().clone();
        Item { id, body }
    }

    fn recipe(world: &World, id: RecipeID) -> Recipe {
        let body = world.recipes.get(id).unwrap();
        Recipe { id, body }
    }

    fn send(input: &mut ServingInput, world: &World, msg: ServingMessage) {
        input.update(msg, &world.items, &world.categories);
    }

    fn choose(input: &mut ServingInput, world: &World, id: RecipeID) {
        send(input, world, ServingMessage::Recipe(recipe(world, id)));
    }

    fn pick(input: &mut ServingInput, world: &World, key: PourKey, id: ItemID) {
        send(input, world, ServingMessage::Item(key, item(world, id)));
    }

    fn servings(input: &mut ServingInput, world: &World, text: &str) {
        send(input, world, ServingMessage::Servings(text.to_string()));
    }

    fn refresh(input: &mut ServingInput, world: &World) {
        input.refresh(&world.recipes, &world.items, &world.categories);
    }

    /// The names of the items the `ingredient`th row offers, in order.
    fn offered(input: &ServingInput, ingredient: usize) -> Vec<String> {
        let (_, row) = input.ingredients.iter().nth(ingredient).unwrap();
        row.candidates
            .iter()
            .map(|item| item.body.name.clone())
            .collect()
    }

    /// The item picked for a pour, if any.
    fn picked(input: &ServingInput, key: PourKey) -> Option<Item> {
        let (_, row) = input
            .ingredients
            .iter()
            .find(|(ingredient, _)| *ingredient == key.ingredient)
            .unwrap();
        let (_, pour) = row
            .pours
            .iter()
            .find(|(pour, _)| *pour == key.pour)
            .unwrap();
        pour.item.value().ok()
    }

    fn uses(input: &ServingInput) -> Vec<(ItemID, Quantity)> {
        input
            .validate()
            .unwrap()
            .into_iter()
            .map(|usage| (usage.id, usage.quantity))
            .collect()
    }

    fn errors(input: &ServingInput) -> Vec<(ServingField, Error)> {
        input.validate().unwrap_err().0
    }

    /// Gin stocked as a full bottle and a nearly empty one, and a recipe
    /// for 30 ml of it.
    struct GinBar {
        world: World,
        gin: CategoryID,
        full: ItemID,
        dregs: ItemID,
        shot: RecipeID,
    }

    fn gin_bar() -> GinBar {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let full = world.add_item("Full", 750.0);
        let dregs = world.add_item("Dregs", 20.0);
        world.map_item(full, gin);
        world.map_item(dregs, gin);
        let shot = world.add_recipe("Gin Shot", &[(gin, 30.0)]);
        GinBar {
            world,
            gin,
            full,
            dregs,
            shot,
        }
    }

    #[test]
    fn rows_offer_items_with_some_stock_of_the_same_kind() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        for (name, ml) in [("Tanqueray", 750.0), ("Dregs", 5.0), ("Empty", 0.0)] {
            let id = world.add_item(name, ml);
            world.map_item(id, gin);
        }
        let by_mass = ItemBody {
            name: "Gin Jelly".to_string(),
            quantity: Quantity::new(100.0, Unit::Gram),
        };
        let jelly = world
            .items
            .add(&world.database.item_db(), &by_mass)
            .unwrap();
        world.map_item(jelly, gin);
        let shot = world.add_recipe("Gin Shot", &[(gin, 50.0)]);
        let mut input = ServingInput::new();

        choose(&mut input, &world, shot);

        assert_eq!(offered(&input, 0), vec!["Dregs", "Tanqueray"]);
    }

    #[test]
    fn the_only_item_on_offer_is_picked_for_you() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let campari = world.add_category("Campari");
        let tanqueray = world.add_item("Tanqueray", 750.0);
        world.map_item(tanqueray, gin);
        for name in ["Campari", "Campari (new)"] {
            let id = world.add_item(name, 750.0);
            world.map_item(id, campari);
        }
        let recipe = world.add_recipe("Half Negroni", &[(gin, 30.0), (campari, 30.0)]);
        let mut input = ServingInput::new();

        choose(&mut input, &world, recipe);

        assert_eq!(
            picked(&input, input.pour_key(0, 0)),
            Some(item(&world, tanqueray))
        );
        assert_eq!(picked(&input, input.pour_key(1, 0)), None);
    }

    #[test]
    fn no_recipe_chosen_is_rejected() {
        let input = ServingInput::new();
        assert_eq!(
            errors(&input),
            vec![(ServingField::Recipe, Error::MustChooseValue)]
        );
    }

    #[test]
    fn each_pour_needs_an_item() {
        let GinBar { world, shot, .. } = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &world, shot);

        let pour = input.pour_key(0, 0);
        assert_eq!(
            errors(&input),
            vec![(ServingField::PourItem(pour), Error::MustChooseValue)]
        );
    }

    #[test]
    fn one_serving_uses_what_the_recipe_calls_for() {
        let GinBar {
            world, full, shot, ..
        } = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &world, shot);
        let key = input.pour_key(0, 0);
        pick(&mut input, &world, key, full);

        assert_eq!(uses(&input), vec![(full, ml(30.0))]);
        assert_eq!(input.recipe_id(), Some(shot));
    }

    #[test]
    fn servings_scale_every_amount() {
        let GinBar {
            world, full, shot, ..
        } = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &world, shot);
        let key = input.pour_key(0, 0);
        pick(&mut input, &world, key, full);

        servings(&mut input, &world, "1.5");

        assert_eq!(uses(&input), vec![(full, ml(45.0))]);
    }

    #[test]
    fn servings_must_be_above_zero_and_picks_survive_until_fixed() {
        let GinBar {
            world, full, shot, ..
        } = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &world, shot);
        let pour = input.pour_key(0, 0);
        pick(&mut input, &world, pour, full);

        for text in ["0", "-1", "abc", ""] {
            servings(&mut input, &world, text);
            assert_eq!(
                errors(&input),
                vec![(ServingField::Servings, Error::NumberNotPositive)],
                "{text:?}"
            );
        }
        servings(&mut input, &world, "2");
        assert_eq!(uses(&input), vec![(full, ml(60.0))]);
    }

    #[test]
    fn stepping_servings_never_reaches_zero() {
        let world = World::new();
        let mut input = ServingInput::new();
        let step = |input: &mut ServingInput, step| {
            send(input, &world, ServingMessage::StepServings(step));
            input.servings.value().map(f32::to_bits)
        };

        assert!(!input.can_step_down());
        assert_eq!(step(&mut input, Step::Down), Ok(1f32.to_bits()));
        assert_eq!(step(&mut input, Step::Up), Ok(2f32.to_bits()));
        assert!(input.can_step_down());
        assert_eq!(step(&mut input, Step::Down), Ok(1f32.to_bits()));

        servings(&mut input, &world, "1.5");
        assert!(input.can_step_down());
        assert_eq!(step(&mut input, Step::Down), Ok(0.5f32.to_bits()));
        assert!(!input.can_step_down());
        assert_eq!(step(&mut input, Step::Up), Ok(1.5f32.to_bits()));

        servings(&mut input, &world, "abc");
        assert!(!input.can_step_down());
        assert_eq!(step(&mut input, Step::Up), Ok(1f32.to_bits()));
    }

    #[test]
    fn stepped_servings_are_shown_to_two_places() {
        assert_eq!(format_number(2.0), "2");
        assert_eq!(format_number(1.5), "1.5");
        assert_eq!(format_number(1.0 / 3.0), "0.33");
    }

    /// The gin bar with 30 ml served from the dregs, then split onto the
    /// full bottle.
    fn split_shot() -> (GinBar, ServingInput) {
        let bar = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &bar.world, bar.shot);
        let key = input.pour_key(0, 0);
        pick(&mut input, &bar.world, key, bar.dregs);
        let row = input.pour_key(0, 0).ingredient;
        send(
            &mut input,
            &bar.world,
            ServingMessage::AddPour(row, UnitSystem::Metric),
        );
        let key = input.pour_key(0, 1);
        pick(&mut input, &bar.world, key, bar.full);
        (bar, input)
    }

    #[test]
    fn splitting_finishes_the_first_item_and_takes_the_rest_from_the_next() {
        let (bar, input) = split_shot();
        assert_eq!(
            uses(&input),
            vec![(bar.dregs, ml(20.0)), (bar.full, ml(10.0))]
        );
    }

    #[test]
    fn split_amounts_must_add_up() {
        let (bar, mut input) = split_shot();
        let second = input.pour_key(0, 1);
        send(
            &mut input,
            &bar.world,
            ServingMessage::Amount(second, "5".to_string()),
        );

        assert_eq!(
            errors(&input),
            vec![(
                ServingField::Ingredient(second.ingredient),
                Error::PoursDontAddUp
            )]
        );
    }

    #[test]
    fn split_amounts_must_be_above_zero() {
        let (bar, mut input) = split_shot();
        let second = input.pour_key(0, 1);
        send(
            &mut input,
            &bar.world,
            ServingMessage::Amount(second, "0".to_string()),
        );

        assert_eq!(
            errors(&input),
            vec![(ServingField::PourAmount(second), Error::NumberNotPositive)]
        );
    }

    #[test]
    fn split_amounts_stay_put_when_servings_change() {
        let (bar, mut input) = split_shot();
        servings(&mut input, &bar.world, "2");
        assert_eq!(
            errors(&input),
            vec![(
                ServingField::Ingredient(input.pour_key(0, 0).ingredient),
                Error::PoursDontAddUp
            )]
        );
    }

    #[test]
    fn split_amounts_are_entered_in_the_units_chosen_when_splitting() {
        let GinBar {
            mut world,
            gin,
            full,
            ..
        } = gin_bar();
        let ounce = world.add_recipe("Ounce of Gin", &[(gin, 29.57)]);
        let mut input = ServingInput::new();
        choose(&mut input, &world, ounce);
        let row = input.pour_key(0, 0).ingredient;
        send(
            &mut input,
            &world,
            ServingMessage::AddPour(row, UnitSystem::Imperial),
        );
        let (first, second) = (input.pour_key(0, 0), input.pour_key(0, 1));
        pick(&mut input, &world, first, full);
        pick(&mut input, &world, second, full);
        // Shown in ml, these would be 1 ml in all, not 1 fl oz.
        send(
            &mut input,
            &world,
            ServingMessage::Amount(first, "0.5".to_string()),
        );
        send(
            &mut input,
            &world,
            ServingMessage::Amount(second, "0.5".to_string()),
        );

        let half_ounce = Quantity::new(0.5, Unit::FluidOunce);
        assert_eq!(uses(&input), vec![(full, half_ounce), (full, half_ounce)]);
    }

    #[test]
    fn filled_in_amounts_add_up_despite_rounding() {
        let GinBar {
            world,
            full,
            dregs,
            shot,
            ..
        } = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &world, shot);
        let key = input.pour_key(0, 0);
        pick(&mut input, &world, key, dregs);
        let row = input.pour_key(0, 0).ingredient;
        // 30 ml is 1.0145 fl oz and the 20 ml left is 0.6764 fl oz; filled
        // in to two places, neither split adds up exactly.
        send(
            &mut input,
            &world,
            ServingMessage::AddPour(row, UnitSystem::Imperial),
        );
        let key = input.pour_key(0, 1);
        pick(&mut input, &world, key, full);

        let (_, row) = input.ingredients.iter().next().unwrap();
        let amounts: Vec<&str> = row
            .pours
            .iter()
            .map(|(_, pour)| pour.amount.text())
            .collect();
        assert_eq!(amounts, vec!["0.67", "0.34"], "the first rounded down");
        let used = uses(&input);
        assert_eq!(used.len(), 2);
        assert_eq!(
            item(&world, dregs).body.quantity.covers(used[0].1),
            Some(true)
        );
    }

    #[test]
    fn removing_a_split_pour_gives_the_whole_amount_back_to_one_item() {
        let (bar, mut input) = split_shot();
        let first = input.pour_key(0, 0);
        send(&mut input, &bar.world, ServingMessage::RemovePour(first));

        assert_eq!(uses(&input), vec![(bar.full, ml(30.0))]);

        let last = input.pour_key(0, 0);
        send(&mut input, &bar.world, ServingMessage::RemovePour(last));
        assert_eq!(input.pour_key(0, 0), last, "the last pour stays");
    }

    #[test]
    fn one_item_cannot_be_overdrawn_across_ingredients() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let bottle = world.add_item("Tanqueray", 100.0);
        world.map_item(bottle, gin);
        let double = world.add_recipe("Double Gin", &[(gin, 60.0), (gin, 60.0)]);
        let mut input = ServingInput::new();
        choose(&mut input, &world, double);
        let (first, second) = (input.pour_key(0, 0), input.pour_key(1, 0));

        assert_eq!(
            errors(&input),
            vec![
                (ServingField::PourItem(first), Error::NotEnoughStock),
                (ServingField::PourItem(second), Error::NotEnoughStock),
            ]
        );

        servings(&mut input, &world, "0.5");
        assert_eq!(uses(&input), vec![(bottle, ml(30.0)), (bottle, ml(30.0))]);
    }

    #[test]
    fn output_keeps_the_form_and_shows_errors_only_after_a_failure() {
        let GinBar {
            world, full, shot, ..
        } = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &world, shot);

        input.output().unwrap_err();
        assert!(input.show_errors);

        let key = input.pour_key(0, 0);

        pick(&mut input, &world, key, full);
        assert_eq!(input.output().map(|uses| uses.len()), Ok(1));
        assert!(!input.show_errors);
        assert_eq!(uses(&input), vec![(full, ml(30.0))], "ready to serve again");
    }

    #[test]
    fn refreshing_keeps_picks_with_their_new_stock() {
        let GinBar {
            mut world,
            full,
            shot,
            ..
        } = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &world, shot);
        let pour = input.pour_key(0, 0);
        pick(&mut input, &world, pour, full);
        servings(&mut input, &world, "2");

        world
            .items
            .use_items(
                &world.database.item_db(),
                vec![ItemUse {
                    id: full,
                    quantity: ml(700.0),
                }],
            )
            .unwrap();
        refresh(&mut input, &world);

        assert_eq!(input.pour_key(0, 0), pour, "same rows");
        let kept = picked(&input, pour).unwrap();
        assert_eq!(kept.body.quantity, ml(50.0));
        assert_eq!(
            errors(&input),
            vec![(ServingField::PourItem(pour), Error::NotEnoughStock)],
            "two servings now need more than is left"
        );
    }

    #[test]
    fn refreshing_drops_picks_that_ran_out() {
        let GinBar {
            mut world,
            dregs,
            shot,
            ..
        } = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &world, shot);
        let pour = input.pour_key(0, 0);
        pick(&mut input, &world, pour, dregs);

        let used = ItemUse {
            id: dregs,
            quantity: ml(20.0),
        };
        world
            .items
            .use_items(&world.database.item_db(), vec![used])
            .unwrap();
        refresh(&mut input, &world);

        assert_eq!(offered(&input, 0), vec!["Full"]);
        let full = picked(&input, pour).map(|item| item.body.name);
        assert_eq!(full.as_deref(), Some("Full"), "the only one left is picked");
    }

    #[test]
    fn refreshing_rebuilds_rows_when_the_recipe_changed() {
        let GinBar {
            mut world,
            full,
            shot,
            ..
        } = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &world, shot);
        let old_pour = input.pour_key(0, 0);
        pick(&mut input, &world, old_pour, full);

        let mut changed = recipe(&world, shot);
        changed.body.name = "Big Gin Shot".to_string();
        changed.body.ingredients[0].quantity = ml(50.0);
        world
            .recipes
            .update(&world.database.recipe_db(), changed)
            .unwrap();
        refresh(&mut input, &world);

        assert_ne!(input.pour_key(0, 0), old_pour);
        assert_eq!(input.recipe.as_ref().unwrap().body.name, "Big Gin Shot");
        assert_eq!(
            errors(&input),
            vec![(
                ServingField::PourItem(input.pour_key(0, 0)),
                Error::MustChooseValue
            )]
        );
    }

    #[test]
    fn refreshing_picks_up_a_rename_without_rebuilding() {
        let GinBar {
            mut world, shot, ..
        } = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &world, shot);
        let pour = input.pour_key(0, 0);

        let mut renamed = recipe(&world, shot);
        renamed.body.name = "Gin Neat".to_string();
        world
            .recipes
            .update(&world.database.recipe_db(), renamed)
            .unwrap();
        refresh(&mut input, &world);

        assert_eq!(input.pour_key(0, 0), pour);
        assert_eq!(input.recipe.as_ref().unwrap().body.name, "Gin Neat");
    }

    #[test]
    fn refreshing_clears_a_deleted_recipe_and_ignores_an_empty_form() {
        let GinBar {
            mut world, shot, ..
        } = gin_bar();
        let mut input = ServingInput::new();
        refresh(&mut input, &world);
        assert_eq!(input.recipe_id(), None);

        choose(&mut input, &world, shot);
        servings(&mut input, &world, "3");
        world
            .recipes
            .delete(&world.database.recipe_db(), shot)
            .unwrap();
        refresh(&mut input, &world);

        assert_eq!(input.recipe_id(), None);
        assert_eq!(input.ingredients.len(), 0);
        assert_eq!(input.servings.value().map(f32::to_bits), Ok(1f32.to_bits()));
    }

    #[test]
    fn messages_for_a_previous_recipes_rows_are_discarded() {
        let GinBar {
            world, full, shot, ..
        } = gin_bar();
        let mut input = ServingInput::new();
        choose(&mut input, &world, shot);
        let old = input.pour_key(0, 0);
        choose(&mut input, &world, shot);

        pick(&mut input, &world, old, full);
        send(
            &mut input,
            &world,
            ServingMessage::Amount(old, "5".to_string()),
        );
        send(
            &mut input,
            &world,
            ServingMessage::AddPour(old.ingredient, UnitSystem::Metric),
        );
        send(&mut input, &world, ServingMessage::RemovePour(old));

        let new = input.pour_key(0, 0);
        assert_ne!(new, old);
        assert_eq!(
            errors(&input),
            vec![(ServingField::PourItem(new), Error::MustChooseValue)]
        );
    }

    #[test]
    fn choosing_a_recipe_resets_servings() {
        let GinBar { world, shot, .. } = gin_bar();
        let mut input = ServingInput::new();
        servings(&mut input, &world, "4");
        choose(&mut input, &world, shot);
        assert_eq!(input.servings.value().map(f32::to_bits), Ok(1f32.to_bits()));
    }

    #[test]
    fn fields_display_their_names() {
        let key = RowKey(0);
        let pour = PourKey {
            ingredient: key,
            pour: key,
        };
        let names = [
            (ServingField::Recipe, "Recipe"),
            (ServingField::Servings, "Servings"),
            (ServingField::Ingredient(key), "Ingredient"),
            (ServingField::PourItem(pour), "Ingredient Item"),
            (ServingField::PourAmount(pour), "Ingredient Amount"),
        ];
        for (field, name) in names {
            assert_eq!(field.to_string(), name);
        }
    }
}
