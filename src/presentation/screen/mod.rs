use iced::overlay::Element;

//Screens are self contained and will contain their own state. Transitions between them, as of current design,
// Will be handled by the main application.
pub(super) mod overview;

#[derive(Debug, Default)]
pub enum Screen {
    #[default]
    Overview,
}
