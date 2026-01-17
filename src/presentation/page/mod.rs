use iced::overlay::Element;

mod overview;

//Page Ideas:
// Interfaces will need to be compliant with being called by view and update in main.

#[derive(Debug, Default)]
pub enum Page {
    #[default]
    Overview,
}

trait View {
    fn view(&self) -> Element<'_>;
}
