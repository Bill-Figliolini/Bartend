use std::{
    convert::Infallible,
    fmt::{Debug, Display},
    marker::PhantomData,
};

use iced::{Element, widget::combo_box};

use crate::presentation::{
    styling::Styling,
    widget::{
        controls,
        input::{Error, keyboard_pick::KeyboardPick, with_error},
    },
};

/// Whether a [`PickField`] must have a selection, and how picking behaves.
pub trait Requirement<T> {
    /// What [`PickField::value`] gives back.
    type Output;
    /// [`Infallible`] when no choice is needed, so the value can be read
    /// with an irrefutable `let Ok(value) = field.value();`.
    type Error: Into<Error>;
    /// How the placeholder notes that a choice is optional, if it is: added
    /// in brackets after it, or shown alone when there is no placeholder.
    const NOTE: Option<&'static str>;
    /// Checks the current selection.
    fn apply(selection: Option<&T>) -> Result<Self::Output, Self::Error>;
    /// Updates the selection when the user picks `picked`.
    fn select(current: &mut Option<T>, picked: T);
}

/// A value must be chosen.
#[derive(Debug)]
pub struct Required;

impl<T: Clone> Requirement<T> for Required {
    type Output = T;
    type Error = Error;
    const NOTE: Option<&'static str> = None;

    fn apply(selection: Option<&T>) -> Result<T, Error> {
        selection.cloned().ok_or(Error::MustChooseValue)
    }

    fn select(current: &mut Option<T>, picked: T) {
        *current = Some(picked);
    }
}

/// Choosing nothing is valid, so the value can never fail.
#[derive(Debug)]
pub struct Optional;

impl<T: Clone + PartialEq> Requirement<T> for Optional {
    type Output = Option<T>;
    type Error = Infallible;
    const NOTE: Option<&'static str> = Some("Optional");

    fn apply(selection: Option<&T>) -> Result<Option<T>, Infallible> {
        Ok(selection.cloned())
    }

    /// Picking the current value again deselects it, which is the only way
    /// to get back to "nothing chosen".
    fn select(current: &mut Option<T>, picked: T) {
        if current.as_ref() == Some(&picked) {
            *current = None;
        } else {
            *current = Some(picked);
        }
    }
}

/// A searchable drop-down of `T`s; `Q` says whether a choice is required.
#[derive(Debug)]
pub struct PickField<T, Q> {
    options: combo_box::State<T>,
    selection: Option<T>,
    placeholder: String,
    requirement: PhantomData<Q>,
}

/// A drop-down that must have a value chosen.
pub type RequiredPick<T> = PickField<T, Required>;
/// A drop-down that may be left empty.
pub type OptionalPick<T> = PickField<T, Optional>;

impl<T, Q> PickField<T, Q>
where
    T: Clone + Display + 'static,
    Q: Requirement<T>,
{
    /// `initial` is selected to begin with; it need not be in `options`.
    pub fn new(options: Vec<T>, initial: Option<T>, placeholder: impl Into<String>) -> Self {
        let placeholder = placeholder.into();
        let placeholder = match Q::NOTE {
            None => placeholder,
            Some(note) if placeholder.is_empty() => note.to_string(),
            Some(note) => format!("{placeholder} ({note})"),
        };
        Self {
            options: combo_box::State::new(options),
            selection: initial,
            placeholder,
            requirement: PhantomData,
        }
    }

    /// Applies a pick from the drop-down, as [`Requirement::select`] says.
    pub fn select(&mut self, picked: T) {
        Q::select(&mut self.selection, picked);
    }

    pub fn clear(&mut self) {
        self.selection = None;
    }

    /// The selection, checked against `Q`. Reading it changes nothing.
    pub fn value(&self) -> Result<Q::Output, Q::Error> {
        Q::apply(self.selection.as_ref())
    }

    /// `on_select` builds the message for a pick. With `show_error`, the
    /// field's current error, if any, is shown under it. The drop-down is a
    /// Tab stop; see [`KeyboardPick`].
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view<'a, M: Clone + 'a>(
        &'a self,
        styling: &Styling,
        on_select: impl Fn(T) -> M + 'static,
        show_error: bool,
    ) -> Element<'a, M> {
        let error = if show_error {
            self.value().err().map(Into::into)
        } else {
            None
        };
        self.view_with_error(styling, on_select, error)
    }

    /// Like [`view`](Self::view), but shows `error`, for a form that checks
    /// more than the field itself can.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view_with_error<'a, M: Clone + 'a>(
        &'a self,
        styling: &Styling,
        on_select: impl Fn(T) -> M + 'static,
        error: Option<Error>,
    ) -> Element<'a, M> {
        let picker = combo_box(
            &self.options,
            &self.placeholder,
            self.selection.as_ref(),
            on_select,
        )
        .input_style(controls::text_field(styling))
        .menu_style(controls::menu(styling));
        with_error(KeyboardPick::new(picker).into(), error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spirits<Q: Requirement<String>>() -> PickField<String, Q> {
        PickField::new(vec!["Gin".to_string(), "Rum".to_string()], None, "Spirit")
    }

    #[test]
    fn required_with_nothing_selected_is_rejected() {
        assert_eq!(spirits::<Required>().value(), Err(Error::MustChooseValue));
    }

    #[test]
    fn required_returns_selection() {
        let mut field = spirits::<Required>();
        field.select("Rum".to_string());
        assert_eq!(field.value(), Ok("Rum".to_string()));
        // Picking the same value again keeps it selected.
        field.select("Rum".to_string());
        assert_eq!(field.value(), Ok("Rum".to_string()));
    }

    #[test]
    fn initial_value_is_selected() {
        let field: RequiredPick<String> = PickField::new(vec![], Some("Gin".to_string()), "");
        assert_eq!(field.value(), Ok("Gin".to_string()));
    }

    #[test]
    fn clear_removes_selection() {
        let mut field = spirits::<Required>();
        field.select("Gin".to_string());
        field.clear();
        assert_eq!(field.value(), Err(Error::MustChooseValue));
    }

    #[test]
    fn optional_with_nothing_selected_is_valid() {
        let Ok(selection) = spirits::<Optional>().value();
        assert_eq!(selection, None);
    }

    #[test]
    fn optional_choosing_the_same_value_again_deselects_it() {
        let mut field = spirits::<Optional>();
        field.select("Gin".to_string());
        assert_eq!(field.value(), Ok(Some("Gin".to_string())));
        field.select("Gin".to_string());
        assert_eq!(field.value(), Ok(None));
    }

    #[test]
    fn placeholder_marks_optional_fields() {
        assert_eq!(spirits::<Required>().placeholder, "Spirit");
        assert_eq!(spirits::<Optional>().placeholder, "Spirit (Optional)");
    }

    #[test]
    fn without_a_placeholder_only_the_optional_note_shows() {
        let required = RequiredPick::<String>::new(Vec::new(), None, "");
        let optional = OptionalPick::<String>::new(Vec::new(), None, "");
        assert_eq!(required.placeholder, "");
        assert_eq!(optional.placeholder, "Optional");
    }

    #[test]
    fn debug_lists_the_options() {
        let shown = format!("{:?}", spirits::<Required>());
        assert!(shown.contains("Gin") && shown.contains("Rum"), "{shown}");
    }
}
