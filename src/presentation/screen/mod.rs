use iced::overlay::Element;

pub(super) mod overview;

//Page Ideas:
// Interfaces will need to be compliant with being called by view and update in main.

#[derive(Debug, Default)]
pub enum Screen {
    #[default]
    Overview,
}
