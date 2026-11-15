//! [`Editor`]: the add-or-edit flow that every entry screen shares.

use iced::{
    Element,
    widget::{button, column, row, text},
};

use crate::{
    models::UnitSystem,
    presentation::input_handling::{Form, FormError},
};

/// A message for an [`Editor`]. Edits are started by the screen with
/// [`Editor::toggle_edit`], since each screen finds the value to edit its
/// own way.
#[derive(Debug, Clone)]
pub enum EditorMessage<M> {
    /// An edit to one of the form's fields.
    Input(M),
    Save,
    /// Stops editing and empties the form.
    Cancel,
}

/// What a successful save asks the screen to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Saved<Id, O> {
    /// The form was not editing anything, so `O` is new.
    Add(O),
    /// `O` replaces the value with this id.
    Update(Id, O),
}

/// What [`Editor::update`] gives back: `Ok(Some(..))` when a save succeeds,
/// `Err` naming every failed field when it does not, and `Ok(None)` for any
/// other message.
pub type EditorResult<F, Id> =
    Result<Option<Saved<Id, <F as Form>::Output>>, FormError<<F as Form>::Field>>;

/// A [`Form`] plus what it is editing, if anything.
///
/// The editor decides when the form's errors are shown: from the first save
/// attempt until the form is next reset. Any reset (a successful save,
/// [`Cancel`](EditorMessage::Cancel), or [`reset`](Self::reset)) clears the
/// form and stops editing, so a save never adds a leftover copy of the value
/// that was being edited.
#[derive(Debug)]
pub struct Editor<F, Id> {
    form: F,
    editing: Option<Id>,
    show_errors: bool,
    /// What the form makes, for its header, e.g. "Item".
    noun: &'static str,
}

impl<F: Form, Id: Copy + PartialEq> Editor<F, Id> {
    pub fn new(form: F, noun: &'static str) -> Self {
        Self {
            form,
            editing: None,
            show_errors: false,
            noun,
        }
    }

    /// The id of the value being edited, if any.
    pub fn editing(&self) -> Option<Id> {
        self.editing
    }

    /// The form, for tests that need to look inside it.
    #[cfg(test)]
    pub fn form(&self) -> &F {
        &self.form
    }

    /// See [`EditorResult`] for what comes back.
    pub fn update(&mut self, msg: EditorMessage<F::Message>) -> EditorResult<F, Id> {
        match msg {
            EditorMessage::Input(msg) => {
                self.form.update(msg);
                Ok(None)
            }
            EditorMessage::Save => self.save().map(Some),
            EditorMessage::Cancel => {
                self.reset();
                Ok(None)
            }
        }
    }

    fn save(&mut self) -> Result<Saved<Id, F::Output>, FormError<F::Field>> {
        self.show_errors = true;
        let output = self.form.validate()?;
        let saved = match self.editing {
            None => Saved::Add(output),
            Some(id) => Saved::Update(id, output),
        };
        self.reset();
        Ok(saved)
    }

    /// Starts editing `id`, filling the form from `load()`. Asking for the
    /// value already being edited cancels the edit instead. While another
    /// value is being edited, nothing happens: that edit must be saved or
    /// cancelled first.
    pub fn toggle_edit(&mut self, id: Id, load: impl FnOnce() -> F::Value, units: UnitSystem) {
        match self.editing {
            None => {
                self.reset();
                self.form.fill(&load(), units);
                self.editing = Some(id);
            }
            Some(current) if current == id => self.reset(),
            Some(_) => {}
        }
    }

    /// Empties the form, stops editing, and hides errors again.
    pub fn reset(&mut self) {
        self.form.clear();
        self.editing = None;
        self.show_errors = false;
    }

    /// The form's heading: "New Item", or "Edit Item" while editing.
    pub fn title(&self) -> String {
        match self.editing {
            None => format!("New {}", self.noun),
            Some(_) => format!("Edit {}", self.noun),
        }
    }

    /// The form's fields, showing their errors once a save has been tried.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn form_view(&self) -> Element<'_, EditorMessage<F::Message>> {
        self.form.view(self.show_errors).map(EditorMessage::Input)
    }

    /// Save, plus Cancel while editing.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn buttons(&self) -> Element<'_, EditorMessage<F::Message>> {
        let mut buttons = row![button("Save").on_press(EditorMessage::Save)].spacing(10);
        if self.editing.is_some() {
            let cancel = button("Cancel")
                .style(button::secondary)
                .on_press(EditorMessage::Cancel);
            buttons = buttons.push(cancel);
        }
        buttons.into()
    }

    /// The [`title`](Self::title), then the form, then its
    /// [`buttons`](Self::buttons). Screens wanting another layout can put
    /// those parts together themselves.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view(&self) -> Element<'_, EditorMessage<F::Message>> {
        let header = text(self.title()).size(24);
        column![header, self.form_view(), self.buttons()]
            .spacing(16)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use iced::widget::text_input;

    use super::*;
    use crate::presentation::{input_handling::Validator, widget::input::Error};

    /// A one-field form whose clear keeps a `sticky` choice, like an item's
    /// unit.
    #[derive(Debug, Default)]
    struct NameForm {
        name: String,
        sticky: u8,
    }

    #[derive(Debug, Clone)]
    enum NameMessage {
        Name(String),
        Sticky(u8),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct NameField;

    impl Form for NameForm {
        type Message = NameMessage;
        type Field = NameField;
        type Output = String;
        type Value = String;

        fn update(&mut self, msg: NameMessage) {
            match msg {
                NameMessage::Name(name) => self.name = name,
                NameMessage::Sticky(sticky) => self.sticky = sticky,
            }
        }

        fn validate(&self) -> Result<String, FormError<NameField>> {
            let mut validator = Validator::default();
            let result = if self.name.is_empty() {
                Err(Error::StringEmpty)
            } else {
                Ok(self.name.clone())
            };
            validator
                .check(NameField, result)
                .ok_or_else(|| validator.into_error())
        }

        fn clear(&mut self) {
            self.name.clear();
        }

        fn fill(&mut self, value: &String, _units: UnitSystem) {
            self.name.clone_from(value);
        }

        #[cfg_attr(coverage_nightly, coverage(off))]
        fn view(&self, _show_errors: bool) -> Element<'_, NameMessage> {
            text_input("Name", &self.name)
                .on_input(NameMessage::Name)
                .into()
        }
    }

    type TestEditor = Editor<NameForm, u32>;

    fn editor() -> TestEditor {
        Editor::new(NameForm::default(), "Name")
    }

    fn type_name(editor: &mut TestEditor, name: &str) {
        let input = EditorMessage::Input(NameMessage::Name(name.to_string()));
        assert_eq!(editor.update(input), Ok(None));
    }

    fn begin(editor: &mut TestEditor, id: u32, name: &str) {
        editor.toggle_edit(id, || name.to_string(), UnitSystem::Metric);
    }

    fn save(editor: &mut TestEditor) -> EditorResult<NameForm, u32> {
        editor.update(EditorMessage::Save)
    }

    #[test]
    fn title_says_whether_the_value_is_new_or_edited() {
        let mut editor = editor();
        assert_eq!(editor.title(), "New Name");
        begin(&mut editor, 7, "Gin");
        assert_eq!(editor.title(), "Edit Name");
    }

    #[test]
    fn saving_a_new_value_adds_it_and_resets() {
        let mut editor = editor();
        type_name(&mut editor, "Gin");
        assert_eq!(save(&mut editor), Ok(Some(Saved::Add("Gin".to_string()))));
        assert_eq!(editor.form().name, "");
        assert!(!editor.show_errors);
    }

    #[test]
    fn saving_an_edit_updates_it_and_stops_editing() {
        let mut editor = editor();
        begin(&mut editor, 7, "Gin");
        assert_eq!(editor.editing(), Some(7));
        type_name(&mut editor, "Rum");
        assert_eq!(
            save(&mut editor),
            Ok(Some(Saved::Update(7, "Rum".to_string())))
        );
        assert_eq!(editor.editing(), None);
        assert_eq!(editor.form().name, "");
    }

    #[test]
    fn a_failed_save_keeps_the_form_and_shows_errors() {
        let mut editor = editor();
        editor
            .update(EditorMessage::Input(NameMessage::Sticky(3)))
            .unwrap();
        begin(&mut editor, 7, "");
        assert_eq!(
            save(&mut editor),
            Err(FormError(vec![(NameField, Error::StringEmpty)]))
        );
        assert!(editor.show_errors);
        assert_eq!(editor.editing(), Some(7), "still editing after a failure");
        assert_eq!(editor.form().sticky, 3);
    }

    #[test]
    fn errors_are_hidden_again_once_reset() {
        let mut editor = editor();
        save(&mut editor).unwrap_err();
        editor.reset();
        assert!(!editor.show_errors);
    }

    #[test]
    fn cancel_empties_the_form_so_save_cannot_add_a_copy() {
        let mut editor = editor();
        begin(&mut editor, 7, "Gin");
        assert_eq!(editor.update(EditorMessage::Cancel), Ok(None));
        assert_eq!(editor.editing(), None);
        assert_eq!(editor.form().name, "");
        save(&mut editor).unwrap_err();
    }

    #[test]
    fn beginning_the_same_edit_again_cancels_it() {
        let mut editor = editor();
        begin(&mut editor, 7, "Gin");
        editor.toggle_edit(7, || unreachable!("not loaded"), UnitSystem::Metric);
        assert_eq!(editor.editing(), None);
        assert_eq!(editor.form().name, "");
    }

    #[test]
    fn beginning_another_edit_while_editing_is_ignored() {
        let mut editor = editor();
        begin(&mut editor, 7, "Gin");
        type_name(&mut editor, "Old Tom Gin");
        editor.toggle_edit(8, || unreachable!("not loaded"), UnitSystem::Metric);
        assert_eq!(editor.editing(), Some(7));
        assert_eq!(editor.form().name, "Old Tom Gin");
    }

    #[test]
    fn beginning_an_edit_replaces_a_half_typed_new_value() {
        let mut editor = editor();
        save(&mut editor).unwrap_err();
        type_name(&mut editor, "Half");
        begin(&mut editor, 7, "Gin");
        assert_eq!(editor.form().name, "Gin");
        assert!(!editor.show_errors, "a fresh edit starts without errors");
    }

    #[test]
    fn reset_keeps_what_the_form_chooses_to_keep() {
        let mut editor = editor();
        editor
            .update(EditorMessage::Input(NameMessage::Sticky(3)))
            .unwrap();
        type_name(&mut editor, "Gin");
        editor.reset();
        assert_eq!(editor.form().sticky, 3);
        assert_eq!(editor.form().name, "");
    }
}
