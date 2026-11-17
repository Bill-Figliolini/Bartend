use std::fmt::Display;

use iced::{
    Alignment::Center,
    Element,
    Length::FillPortion,
    Theme,
    widget::{Column, button, column, container, row, space, text},
};

use crate::{
    logic::{CategoryService, ItemService, RecipeService, StockLevel, totals_by_key},
    models::{
        CategoryID, DECIMAL_SCALE, Ingredient, Item, ItemID, ItemUse, Quantity, Recipe, RecipeID,
        UnitSystem, format_number, format_number_down,
    },
    presentation::{
        constants,
        input_handling::{FormError, RowKey, Rows, Validator, discard_stale_row},
        styling::Styling,
        widget::{
            detail::{Tone, column_heading, hint, row_danger_button, toned_text},
            input::{Error, PositiveField, RequiredPick, error_text},
            styled,
        },
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
    /// Splits the ingredient across another item.
    AddPour(RowKey),
    RemovePour(PourKey),
    /// Asks to open the category on the Categories screen. The form leaves
    /// this to the screen showing it.
    ShowCategory(CategoryID),
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

/// What is left of an item once the form's pours are taken from it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Remaining {
    Left(Quantity),
    /// The pours need this much more than the item has.
    Short(Quantity),
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
    /// What amounts and stock are shown in, and what new split amounts are
    /// entered in.
    units: UnitSystem,
    /// Whether fields show their errors; see the [module docs](super).
    show_errors: bool,
}

impl ServingInput {
    pub fn new(units: UnitSystem) -> Self {
        let mut servings = PositiveField::new("Servings");
        servings.set("1".to_string());
        Self {
            recipe: None,
            servings,
            ingredients: Rows::default(),
            units,
            show_errors: false,
        }
    }

    /// The chosen recipe, as of the last refresh.
    pub fn recipe(&self) -> Option<&Recipe> {
        self.recipe.as_ref()
    }

    /// The chosen recipe's id, if one is chosen.
    pub fn recipe_id(&self) -> Option<RecipeID> {
        self.recipe.as_ref().map(|recipe| recipe.id)
    }

    /// The chosen recipe's name, as of the last refresh.
    pub fn recipe_name(&self) -> Option<&str> {
        self.recipe.as_ref().map(|recipe| recipe.body.name.as_str())
    }

    /// The number of servings, if it is valid.
    pub fn servings(&self) -> Option<f32> {
        self.servings.value().ok()
    }

    pub fn units(&self) -> UnitSystem {
        self.units
    }

    /// Shows amounts and stock in `units`. Amounts already typed into
    /// split pours keep the units they were typed in.
    pub fn set_units(&mut self, units: UnitSystem) {
        if self.units == units {
            return;
        }
        self.units = units;
        for (_, ingredient) in self.ingredients.iter_mut() {
            ingredient.reoffer(units);
        }
    }

    /// Whether stepping servings down would still leave more than zero.
    pub fn can_step_down(&self) -> bool {
        self.servings().is_some_and(|servings| servings > 1.0)
    }

    /// Whether every ingredient has enough in stock, counting all the
    /// items it could come from. Without valid servings this is unknown,
    /// and taken as yes so that serving reports the problem.
    ///
    /// Each ingredient is checked on its own; two that draw on the same
    /// bottle can still turn out short when served.
    pub fn servable(&self) -> bool {
        self.servings().is_none() || self.ready()
    }

    /// Whether every ingredient has enough in stock for the servings
    /// entered, or for one serving while they are not a valid number; see
    /// [`Self::servable`].
    pub fn ready(&self) -> bool {
        let servings = self.servings().unwrap_or(1.0);
        self.ingredients
            .iter()
            .all(|(_, ingredient)| ingredient.in_stock(servings))
    }

    /// What is left of the item picked for `key` once every pour in the
    /// form that is known in full is taken from it; `None` if this pour has
    /// no item, or its amount is not known.
    pub fn remaining(&self, key: PourKey) -> Option<Remaining> {
        let ingredient = self.ingredients.get(key.ingredient)?;
        let pour = ingredient.pours.get(key.pour)?;
        let offer = pour.item.value().ok()?;
        ingredient.pour_amount(pour, self.servings())?;
        let used = self
            .used_by_item()
            .into_iter()
            .find(|(id, _)| *id == offer.item.id)
            .map(|(_, used)| used)?;
        let stock = offer.item.body.quantity;
        match stock.covers(used) {
            Some(false) => used.saturating_sub(stock).map(Remaining::Short),
            _ => stock.saturating_sub(used).map(Remaining::Left),
        }
    }

    /// The servings stepper: −, the number, +. It has no label of its own,
    /// so it is shown under one.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn servings_view(&self, styling: &Styling) -> Element<'_, ServingMessage> {
        let errors = self.errors();
        let error = errors
            .as_ref()
            .and_then(|errors| errors.error_for(&ServingField::Servings));
        let step_down = self
            .can_step_down()
            .then_some(ServingMessage::StepServings(Step::Down));
        // The error goes under the whole stepper rather than the narrow
        // field, so it has room and the buttons stay level with the field.
        let field = container(self.servings.view_with_error(
            styling,
            ServingMessage::Servings,
            None,
        ))
        .width(80);
        let stepper = row![
            button(text("-").center())
                .width(32)
                .style(styled::secondary(styling))
                .on_press_maybe(step_down),
            field,
            button(text("+").center())
                .width(32)
                .style(styled::secondary(styling))
                .on_press(ServingMessage::StepServings(Step::Up)),
        ]
        .spacing(constants::SPACING / 2)
        .align_y(Center);
        column![stepper]
            .push(error.map(error_text))
            .spacing(4)
            .into()
    }

    /// A row per pour of each ingredient, under headings: the ingredient
    /// (a link to its category), the amount it needs for this many
    /// servings, the item to pour from, and what that item has left
    /// afterwards.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn table_view(
        &self,
        styling: &Styling,
        category_service: &CategoryService,
    ) -> Element<'_, ServingMessage> {
        if self.ingredients.is_empty() {
            return hint("No ingredients to pour. Edit the recipe to add some.").into();
        }
        let headings = table_row(
            ["Ingredient", "Amount", "Use item", "Left after"]
                .map(|label| column_heading(styling, label).into()),
        );
        let errors = self.errors();
        let ingredients = self.ingredients.iter().map(|(key, ingredient)| {
            let name = category_service.name_of(ingredient.category);
            ingredient.view(styling, key, name, self, errors.as_ref())
        });
        column![
            headings,
            Column::with_children(ingredients).spacing(constants::SPACING)
        ]
        .spacing(constants::SPACING)
        .into()
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
            ServingMessage::Item(key, item) => {
                let units = self.units;
                match self.pour_mut(key) {
                    Some(pour) => pour.item.select(Offer::new(item, units)),
                    None => discard_stale_row(key.pour, &item),
                }
            }
            ServingMessage::Amount(key, amount) => match self.pour_mut(key) {
                Some(pour) => pour.amount.set(amount),
                None => discard_stale_row(key.pour, &amount),
            },
            ServingMessage::AddPour(key) => {
                let (servings, units) = (self.servings(), self.units);
                match self.ingredients.get_mut(key) {
                    Some(ingredient) => ingredient.add_pour(servings, units),
                    None => discard_stale_row(key, &key),
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
            ServingMessage::ShowCategory(_) => {}
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
            let units = self.units;
            for (_, ingredient) in self.ingredients.iter_mut() {
                ingredient.restock(item_service, category_service, units);
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

    /// Empties the form, keeping its units.
    pub fn clear(&mut self) {
        // The rows are cleared rather than replaced, so their keys keep
        // counting up and are never reused.
        self.ingredients.clear();
        let ingredients = std::mem::take(&mut self.ingredients);
        *self = Self {
            ingredients,
            ..Self::new(self.units)
        };
    }

    /// The errors to show, if they are being shown.
    fn errors(&self) -> Option<FormError<ServingField>> {
        if self.show_errors {
            self.validate().err()
        } else {
            None
        }
    }

    /// How much of each item the pours known in full use, in total.
    fn used_by_item(&self) -> Vec<(ItemID, Quantity)> {
        let servings = self.servings();
        let uses = self.ingredients.iter().flat_map(|(_, ingredient)| {
            ingredient.pours.iter().filter_map(move |(_, pour)| {
                let item = pour.item.value().ok()?.item.id;
                Some((item, ingredient.pour_amount(pour, servings)?))
            })
        });
        // See `overdrawn_items` for why the kinds always match.
        totals_by_key(uses).unwrap_or_default()
    }

    fn build_rows(
        &mut self,
        recipe: &Recipe,
        item_service: &ItemService,
        category_service: &CategoryService,
    ) {
        for ingredient in &recipe.body.ingredients {
            let row =
                IngredientUseInput::new(ingredient, item_service, category_service, self.units);
            self.ingredients.push(row);
        }
    }

    fn step_servings(&mut self, step: Step) {
        let next = match (step, self.servings()) {
            (Step::Up, Some(servings)) => servings + 1.0,
            (Step::Up, None) => 1.0,
            (Step::Down, Some(servings)) if servings > 1.0 => servings - 1.0,
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

/// The four cells of a [`ServingInput::table_view`] row, in proportional
/// columns so that every row lines up.
#[cfg_attr(coverage_nightly, coverage(off))]
fn table_row(cells: [Element<'_, ServingMessage>; 4]) -> Element<'_, ServingMessage> {
    const PORTIONS: [u16; 4] = [3, 2, 6, 2];
    let cells = cells
        .into_iter()
        .zip(PORTIONS)
        .map(|(cell, portion)| container(cell).width(FillPortion(portion)).into());
    iced::widget::Row::with_children(cells)
        .spacing(constants::SPACING)
        .align_y(Center)
        .into()
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

/// Whether `total` is `required`, give or take `slack`.
fn within(total: Quantity, required: Quantity, slack: Quantity) -> bool {
    let over = total.saturating_sub(required);
    let under = required.saturating_sub(total);
    [over, under]
        .into_iter()
        .all(|gap| gap.is_some_and(|gap| slack.covers(gap) == Some(true)))
}

/// An item on offer for a pour, labelled with its stock.
#[derive(Debug, Clone)]
struct Offer {
    item: Item,
    label: String,
}

impl Offer {
    fn new(item: Item, units: UnitSystem) -> Self {
        let stock = item.body.quantity.readable_down(units);
        let label = format!("{} ({stock})", item.body.name);
        Self { item, label }
    }
}

impl Display for Offer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
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
        units: UnitSystem,
    ) -> Self {
        let candidates = candidates(ingredient, item_service, category_service);
        let mut pours = Rows::default();
        pours.push(PourInput::new(&candidates, only(&candidates), units));
        Self {
            category: ingredient.category,
            per_serving: ingredient.quantity,
            candidates,
            pours,
        }
    }

    /// Refreshes the items offered. Picks that are still in stock are kept,
    /// with their new stock level. A split pour whose item has run out is
    /// dropped, leaving the rest of the ingredient to the others (and all of
    /// it, once one pour is left); the last pour is only cleared.
    fn restock(
        &mut self,
        item_service: &ItemService,
        category_service: &CategoryService,
        units: UnitSystem,
    ) {
        let ingredient = Ingredient {
            category: self.category,
            quantity: self.per_serving,
        };
        self.candidates = candidates(&ingredient, item_service, category_service);
        let ran_out: Vec<RowKey> = self
            .pours
            .iter()
            .filter(|(_, pour)| {
                pour.item.value().is_ok_and(|offer| {
                    !self
                        .candidates
                        .iter()
                        .any(|candidate| candidate.id == offer.item.id)
                })
            })
            .map(|(key, _)| key)
            .collect();
        for key in ran_out {
            self.remove_pour(key);
        }
        self.reoffer(units);
    }

    /// Offers the candidates again, labelled in `units`.
    fn reoffer(&mut self, units: UnitSystem) {
        let lone = self.pours.len() == 1;
        for (_, pour) in self.pours.iter_mut() {
            pour.reoffer(&self.candidates, units, lone);
        }
    }

    /// What `servings` of the ingredient need.
    fn required(&self, servings: f32) -> Quantity {
        self.per_serving * servings
    }

    /// Whether its candidates hold enough, between them, for `servings`.
    fn in_stock(&self, servings: f32) -> bool {
        self.stock().covers(self.required(servings)) == Some(true)
    }

    /// The candidates' stock, in total.
    fn stock(&self) -> Quantity {
        self.candidates
            .iter()
            .map(|item| item.body.quantity)
            .fold(self.per_serving * 0.0, |total, stock| {
                total.checked_add(stock).unwrap_or(total)
            })
    }

    /// What `pour` takes, if known: everything needed for `servings` when
    /// it is the only pour, or the amount entered for it otherwise.
    fn pour_amount(&self, pour: &PourInput, servings: Option<f32>) -> Option<Quantity> {
        if self.pours.len() == 1 {
            servings.map(|servings| self.required(servings))
        } else {
            pour.amount(self.per_serving).ok()
        }
    }

    /// Adds an empty pour. Its amount starts at whatever the other pours
    /// leave short of `servings`' worth. When this splits a single pour,
    /// that pour is given an amount first: all its item can supply, up to
    /// what is needed.
    fn add_pour(&mut self, servings: Option<f32>, units: UnitSystem) {
        let required = servings.map(|servings| self.required(servings));
        let mut pour = PourInput::new(&self.candidates, None, units);
        if let Some(required) = required {
            let single = self.pours.len() == 1;
            if let Some((_, first)) = self.pours.iter_mut().next().filter(|_| single) {
                let stock = first
                    .item
                    .value()
                    .ok()
                    .map(|offer| offer.item.body.quantity);
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
        let required = servings.map(|servings| self.required(servings));
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
            let rounding = Quantity::new(0.5 / DECIMAL_SCALE, self.per_serving.unit(pour.units));
            slack = slack.checked_add(rounding).unwrap_or(slack);
            if let (Some(offer), Some(quantity)) = (item, amount) {
                let usage = ItemUse {
                    id: offer.item.id,
                    quantity,
                };
                let stock = offer.item.body.quantity;
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

    /// The ingredient's rows of the table: one per pour, then a button to
    /// split it across another item, and any error for the ingredient as a
    /// whole. If its items cannot supply enough between them, a single row
    /// says so instead.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn view<'a>(
        &'a self,
        styling: &Styling,
        key: RowKey,
        name: String,
        form: &'a ServingInput,
        errors: Option<&FormError<ServingField>>,
    ) -> Element<'a, ServingMessage> {
        let units = form.units;
        let servings = form.servings();
        let error_for = |field| errors.and_then(|errors| errors.error_for(&field));
        let amount = servings.map_or_else(
            || "—".to_string(),
            |servings| self.required(servings).readable(units),
        );
        let link = button(text(name))
            .style(button::text)
            .padding(0)
            .on_press(ServingMessage::ShowCategory(self.category));
        if servings.is_some_and(|servings| !self.in_stock(servings)) {
            let (shortage, tone) = self.shortage(units);
            return table_row([
                link.into(),
                text(amount).into(),
                text(shortage)
                    .style(move |theme: &Theme| toned_text(theme, tone))
                    .into(),
                text("—").into(),
            ]);
        }

        let split = self.pours.len() > 1;
        let mut rows = Column::new().spacing(constants::SPACING / 2);
        let mut labels = Some((link, amount));
        for (pour_key, pour) in self.pours.iter() {
            let pour_key = PourKey {
                ingredient: key,
                pour: pour_key,
            };
            let picker = pour.item.view_with_error(
                styling,
                move |offer: Offer| ServingMessage::Item(pour_key, offer.item),
                error_for(ServingField::PourItem(pour_key)),
            );
            let item_cell: Element<'_, _> = if split {
                let amount = pour.amount.view_with_error(
                    styling,
                    move |amount| ServingMessage::Amount(pour_key, amount),
                    error_for(ServingField::PourAmount(pour_key)),
                );
                let unit = self.per_serving.unit(pour.units).to_string();
                let remove =
                    row_danger_button(styling, "Remove", ServingMessage::RemovePour(pour_key));
                row![
                    container(picker).width(FillPortion(3)),
                    container(amount).width(FillPortion(1)),
                    text(unit),
                    remove,
                ]
                .spacing(constants::SPACING / 2)
                .align_y(Center)
                .into()
            } else {
                picker
            };
            let left = pour.left_after_view(form.remaining(pour_key), units);
            let (name, amount): (Element<'_, _>, Element<'_, _>) = match labels.take() {
                Some((link, amount)) => (link.into(), text(amount).into()),
                None => (space().into(), space().into()),
            };
            rows = rows.push(table_row([name, amount, item_cell, left]));
        }

        // Another bottle only helps while some candidate is not yet poured.
        let add = (self.candidates.len() > self.pours.len()).then(|| {
            styled::small_button(styling, "+ Add bottle")
                .style(styled::secondary(styling))
                .on_press(ServingMessage::AddPour(key))
        });
        let error = error_for(ServingField::Ingredient(key)).map(error_text);
        if add.is_some() || error.is_some() {
            let notes = row![]
                .push(add)
                .push(error)
                .spacing(constants::SPACING)
                .align_y(Center);
            rows = rows.push(table_row([
                space().into(),
                space().into(),
                notes.into(),
                space().into(),
            ]));
        }
        rows.into()
    }

    /// Why the ingredient can't be poured, and how bad that is: there is
    /// none of it at all, or only so much.
    fn shortage(&self, units: UnitSystem) -> (String, Tone) {
        if self.candidates.is_empty() {
            ("None in stock".to_string(), Tone::Danger)
        } else {
            let stock = self.stock().readable_down(units);
            (format!("Only {stock} in stock"), Tone::Warning)
        }
    }
}

/// How what is `left` of an item after the serving reads: in the warning
/// or danger colour once it is low against the item's `full_size`, as on
/// the Inventory gauge, or plain without one.
fn remainder_tone(left: Quantity, full_size: Option<Quantity>) -> Option<Tone> {
    match StockLevel::of_remainder(left, full_size?)? {
        StockLevel::Healthy => None,
        StockLevel::Low => Some(Tone::Warning),
        StockLevel::Critical => Some(Tone::Danger),
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
    item: RequiredPick<Offer>,
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
            item: offers(candidates, initial, units),
            amount: PositiveField::new("Amount"),
            units,
        }
    }

    /// Offers `candidates`, labelled in `units`. The pick is kept, with its
    /// new stock level, if it is still among them. A `lone` pour with
    /// nothing picked takes the only candidate, as a new row would.
    fn reoffer(&mut self, candidates: &[Item], units: UnitSystem, lone: bool) {
        let kept = self.item.value().ok().and_then(|picked| {
            candidates
                .iter()
                .find(|candidate| candidate.id == picked.item.id)
                .cloned()
        });
        let picked = match kept {
            Some(kept) => Some(kept),
            None if lone => only(candidates),
            None => None,
        };
        self.item = offers(candidates, picked, units);
    }

    /// The Left after cell: what is `remaining` of this pour's item, toned
    /// once it runs low against the item's full size, or how far short it
    /// falls.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn left_after_view<'a>(
        &self,
        remaining: Option<Remaining>,
        units: UnitSystem,
    ) -> Element<'a, ServingMessage> {
        match remaining {
            Some(Remaining::Left(left)) => {
                let full_size = self
                    .item
                    .value()
                    .ok()
                    .and_then(|offer| offer.item.body.full_size);
                let shown = text(left.readable_down(units));
                match remainder_tone(left, full_size) {
                    Some(tone) => shown
                        .style(move |theme: &Theme| toned_text(theme, tone))
                        .into(),
                    None => shown.into(),
                }
            }
            Some(Remaining::Short(short)) => text(format!("short {}", short.readable(units)))
                .style(text::danger)
                .into(),
            None => text("—").into(),
        }
    }

    /// The amount entered, as the same kind of quantity as `like`.
    fn amount(&self, like: Quantity) -> Result<Quantity, Error> {
        let unit = like.unit(self.units);
        self.amount
            .value()
            .map(|amount| Quantity::new(amount, unit))
    }
}

/// A picker offering `candidates`, labelled in `units`, with `picked`
/// chosen.
fn offers(candidates: &[Item], picked: Option<Item>, units: UnitSystem) -> RequiredPick<Offer> {
    let offers = candidates
        .iter()
        .map(|item| Offer::new(item.clone(), units))
        .collect();
    let picked = picked.map(|item| Offer::new(item, units));
    RequiredPick::new(offers, picked, "Choose an item")
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
        pour.item.value().ok().map(|offer| offer.item)
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
            full_size: None,
        };
        let jelly = world
            .items
            .add(&world.database.item_db(), &by_mass)
            .unwrap();
        world.map_item(jelly, gin);
        let shot = world.add_recipe("Gin Shot", &[(gin, 50.0)]);
        let mut input = ServingInput::new(UnitSystem::Metric);

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
        let mut input = ServingInput::new(UnitSystem::Metric);

        choose(&mut input, &world, recipe);

        assert_eq!(
            picked(&input, input.pour_key(0, 0)),
            Some(item(&world, tanqueray))
        );
        assert_eq!(picked(&input, input.pour_key(1, 0)), None);
    }

    #[test]
    fn no_recipe_chosen_is_rejected() {
        let input = ServingInput::new(UnitSystem::Metric);
        assert_eq!(
            errors(&input),
            vec![(ServingField::Recipe, Error::MustChooseValue)]
        );
    }

    #[test]
    fn each_pour_needs_an_item() {
        let GinBar { world, shot, .. } = gin_bar();
        let mut input = ServingInput::new(UnitSystem::Metric);
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
        let mut input = ServingInput::new(UnitSystem::Metric);
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
        let mut input = ServingInput::new(UnitSystem::Metric);
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
        let mut input = ServingInput::new(UnitSystem::Metric);
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
        let mut input = ServingInput::new(UnitSystem::Metric);
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

    /// The gin bar with 30 ml served from the dregs, then split onto the
    /// full bottle.
    fn split_shot() -> (GinBar, ServingInput) {
        let bar = gin_bar();
        let mut input = ServingInput::new(UnitSystem::Metric);
        choose(&mut input, &bar.world, bar.shot);
        let key = input.pour_key(0, 0);
        pick(&mut input, &bar.world, key, bar.dregs);
        let row = input.pour_key(0, 0).ingredient;
        send(&mut input, &bar.world, ServingMessage::AddPour(row));
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
        let mut input = ServingInput::new(UnitSystem::Metric);
        choose(&mut input, &world, ounce);
        let row = input.pour_key(0, 0).ingredient;
        input.set_units(UnitSystem::Imperial);
        send(&mut input, &world, ServingMessage::AddPour(row));
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
        let mut input = ServingInput::new(UnitSystem::Metric);
        choose(&mut input, &world, shot);
        let key = input.pour_key(0, 0);
        pick(&mut input, &world, key, dregs);
        let row = input.pour_key(0, 0).ingredient;
        // 30 ml is 1.0145 fl oz and the 20 ml left is 0.6764 fl oz; filled
        // in to two places, neither split adds up exactly.
        input.set_units(UnitSystem::Imperial);
        send(&mut input, &world, ServingMessage::AddPour(row));
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
        let mut input = ServingInput::new(UnitSystem::Metric);
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
        let mut input = ServingInput::new(UnitSystem::Metric);
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
        let mut input = ServingInput::new(UnitSystem::Metric);
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
        let mut input = ServingInput::new(UnitSystem::Metric);
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
    fn refreshing_drops_a_split_pour_whose_item_ran_out() {
        let (mut bar, mut input) = split_shot();
        let served = input.validate().unwrap();
        bar.world
            .items
            .use_items(&bar.world.database.item_db(), served)
            .unwrap();

        refresh(&mut input, &bar.world);

        let (_, row) = input.ingredients.iter().next().unwrap();
        assert_eq!(row.pours.len(), 1, "the dregs are finished");
        assert_eq!(
            uses(&input),
            vec![(bar.full, ml(30.0))],
            "all from the full one"
        );
    }

    #[test]
    fn refreshing_rebuilds_rows_when_the_recipe_changed() {
        let GinBar {
            mut world,
            full,
            shot,
            ..
        } = gin_bar();
        let mut input = ServingInput::new(UnitSystem::Metric);
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
        let mut input = ServingInput::new(UnitSystem::Metric);
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
        let mut input = ServingInput::new(UnitSystem::Metric);
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
        let mut input = ServingInput::new(UnitSystem::Metric);
        choose(&mut input, &world, shot);
        let old = input.pour_key(0, 0);
        choose(&mut input, &world, shot);

        pick(&mut input, &world, old, full);
        send(
            &mut input,
            &world,
            ServingMessage::Amount(old, "5".to_string()),
        );
        send(&mut input, &world, ServingMessage::AddPour(old.ingredient));
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
        let mut input = ServingInput::new(UnitSystem::Metric);
        servings(&mut input, &world, "4");
        choose(&mut input, &world, shot);
        assert_eq!(input.servings.value().map(f32::to_bits), Ok(1f32.to_bits()));
    }

    /// The labels the `ingredient`th row's first pour offers.
    fn labels(input: &ServingInput, ingredient: usize) -> Vec<String> {
        let (_, row) = input.ingredients.iter().nth(ingredient).unwrap();
        row.candidates
            .iter()
            .map(|item| Offer::new(item.clone(), input.units()).to_string())
            .collect()
    }

    #[test]
    fn items_are_labelled_with_their_stock_rounded_down_in_the_current_units() {
        let GinBar {
            world, full, shot, ..
        } = gin_bar();
        let mut input = ServingInput::new(UnitSystem::Metric);
        choose(&mut input, &world, shot);
        let pour = input.pour_key(0, 0);
        pick(&mut input, &world, pour, full);
        assert_eq!(labels(&input, 0), vec!["Dregs (20 ml)", "Full (750 ml)"]);

        input.set_units(UnitSystem::Imperial);

        // 750 ml is 25.3635 fl oz.
        assert_eq!(input.units(), UnitSystem::Imperial);
        assert_eq!(
            labels(&input, 0),
            vec!["Dregs (0.67 fl oz)", "Full (25.36 fl oz)"]
        );
        let (_, row) = input.ingredients.iter().next().unwrap();
        let (_, first) = row.pours.iter().next().unwrap();
        let shown = first.item.value().unwrap().to_string();
        assert_eq!(
            shown, "Full (25.36 fl oz)",
            "the pick is relabelled, not lost"
        );
    }

    #[test]
    fn servable_only_while_each_ingredient_has_enough_in_stock() {
        let GinBar {
            mut world,
            gin,
            shot,
            ..
        } = gin_bar();
        let mut input = ServingInput::new(UnitSystem::Metric);
        choose(&mut input, &world, shot);
        assert!(input.servable(), "770 ml for 30 ml");

        servings(&mut input, &world, "26");
        assert!(!input.servable(), "770 ml for 780 ml");

        servings(&mut input, &world, "abc");
        assert!(input.servable(), "left to serving to report");
        assert!(input.ready(), "judged for one serving meanwhile");

        let vermouth = world.add_category("Vermouth");
        let martini = world.add_recipe("Martini", &[(gin, 60.0), (vermouth, 10.0)]);
        choose(&mut input, &world, martini);
        assert!(!input.servable(), "no vermouth at all");
    }

    #[test]
    fn ready_judges_the_servings_entered_or_else_one() {
        let GinBar {
            mut world, shot, ..
        } = gin_bar();
        let mut input = ServingInput::new(UnitSystem::Metric);
        choose(&mut input, &world, shot);
        assert!(input.ready());

        servings(&mut input, &world, "26");
        assert!(!input.ready(), "770 ml for 780 ml");

        servings(&mut input, &world, "0");
        assert!(input.ready(), "30 ml for one serving");

        let gin = world.add_category("Rare Gin");
        let big = world.add_recipe("Big Shot", &[(gin, 30.0)]);
        choose(&mut input, &world, big);
        servings(&mut input, &world, "x");
        assert!(!input.ready(), "none in stock even for one");
    }

    #[test]
    fn a_shortage_says_how_much_there_is_and_how_bad_it_is() {
        let GinBar {
            mut world, shot, ..
        } = gin_bar();
        let mut input = ServingInput::new(UnitSystem::Metric);
        choose(&mut input, &world, shot);
        let (_, gin) = input.ingredients.iter().next().unwrap();
        assert_eq!(
            gin.shortage(UnitSystem::Metric),
            ("Only 770 ml in stock".to_string(), Tone::Warning)
        );
        assert_eq!(
            gin.shortage(UnitSystem::Imperial).0,
            "Only 26.03 fl oz in stock",
            "rounded down, in the current units"
        );

        let rare = world.add_category("Rare Gin");
        let none = world.add_recipe("Rare Shot", &[(rare, 30.0)]);
        choose(&mut input, &world, none);
        let (_, rare) = input.ingredients.iter().next().unwrap();
        assert_eq!(
            rare.shortage(UnitSystem::Metric),
            ("None in stock".to_string(), Tone::Danger)
        );
    }

    #[test]
    fn what_is_left_is_toned_by_its_share_of_the_full_size() {
        let full = Some(ml(1000.0));
        assert_eq!(remainder_tone(ml(250.0), full), None);
        assert_eq!(remainder_tone(ml(249.0), full), Some(Tone::Warning));
        assert_eq!(remainder_tone(ml(99.0), full), Some(Tone::Danger));
        assert_eq!(remainder_tone(ml(0.0), None), None, "no full size");
    }

    #[test]
    fn a_category_link_leaves_the_form_as_it_was() {
        let GinBar {
            world,
            gin,
            full,
            shot,
            ..
        } = gin_bar();
        let mut input = ServingInput::new(UnitSystem::Metric);
        choose(&mut input, &world, shot);
        let key = input.pour_key(0, 0);
        pick(&mut input, &world, key, full);
        let before = uses(&input);

        send(&mut input, &world, ServingMessage::ShowCategory(gin));

        assert_eq!(input.recipe().map(|recipe| recipe.id), Some(shot));
        assert_eq!(uses(&input), before);
    }

    #[test]
    fn remaining_counts_every_pour_from_the_same_item() {
        let mut world = World::new();
        let gin = world.add_category("Gin");
        let bottle = world.add_item("Tanqueray", 100.0);
        world.map_item(bottle, gin);
        let double = world.add_recipe("Double Gin", &[(gin, 40.0), (gin, 40.0)]);
        let mut input = ServingInput::new(UnitSystem::Metric);
        choose(&mut input, &world, double);
        let (first, second) = (input.pour_key(0, 0), input.pour_key(1, 0));

        assert_eq!(input.remaining(first), Some(Remaining::Left(ml(20.0))));
        assert_eq!(input.remaining(second), Some(Remaining::Left(ml(20.0))));

        servings(&mut input, &world, "1.5");
        assert_eq!(input.remaining(first), Some(Remaining::Short(ml(20.0))));

        servings(&mut input, &world, "abc");
        assert_eq!(input.remaining(first), None, "amount unknown");
    }

    #[test]
    fn remaining_is_unknown_without_an_item_or_amount() {
        let (bar, mut input) = split_shot();
        let (first, second) = (input.pour_key(0, 0), input.pour_key(0, 1));
        assert_eq!(input.remaining(first), Some(Remaining::Left(ml(0.0))));
        assert_eq!(input.remaining(second), Some(Remaining::Left(ml(740.0))));

        send(
            &mut input,
            &bar.world,
            ServingMessage::Amount(second, String::new()),
        );
        assert_eq!(input.remaining(second), None);

        choose(&mut input, &bar.world, bar.shot);
        assert_eq!(input.remaining(input.pour_key(0, 0)), None, "no item");
        assert_eq!(input.recipe_name(), Some("Gin Shot"));
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
