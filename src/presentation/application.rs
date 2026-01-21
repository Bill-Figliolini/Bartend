use iced::{
    Element,
    Length::Fill,
    Theme,
    widget::{button, column, container, row},
};

use crate::presentation::{
    screen::{Screen, overview},
    widget::sidebar,
};

pub fn run() -> iced::Result {
    //Application requires the boot component to have default implemented,
    // Which is probably a good practice as it avoids inconsistency due to partial
    // state initialization
    iced::application(move || AppState::new(), AppState::update, AppState::view).run()
}
#[derive(Debug)]
struct AppState {
    count: u64,
    page: Screen,
    theme: Theme,
}

#[derive(Debug, Clone)]
pub enum Message {
    ButtonPressed,
}

impl AppState {
    fn new() -> Self {
        AppState {
            count: 0,
            page: Screen::Overview,
            theme: Theme::TokyoNight,
        }
    }

    //Question I still need to figure out; What should a message be in the context of this application?
    //As a personal reminder while developing. Update is for transformations over AppState
    // Given my trying to keep this program modular, this will likely be a branching function between pages' update functions.
    fn update(&mut self, message: Message) {
        match message {
            _ => {}
        }
    }

    //Personal Reminder:
    // This is for displays and views of the current app state.
    fn view(&self) -> Element<'_, Message> {
        let palette = self.theme.extended_palette();
        container(
            column![
                container("I am a menu bar!")
                    .style(container::rounded_box)
                    .height(10),
                row![
                    column![
                        sidebar::item("I am a sidebar!", || Message::ButtonPressed),
                        sidebar::item("with a button!", || Message::ButtonPressed)
                    ]
                    .width(400)
                    .padding(10),
                    column!["I am the center top!", "I am the center bottom!"]
                        .width(Fill)
                        .spacing(10)
                ]
                .spacing(10),
            ]
            .spacing(10),
        )
        .height(Fill)
        .width(Fill)
        .into()
    }
}
#[cfg(test)]
mod test {
    use super::*;
}
