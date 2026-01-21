use iced::{
    Element,
    Length::Fill,
    Theme,
    widget::{button, column, container, row},
};

use crate::presentation::{screen::overview, widget::sidebar};

pub fn run() -> iced::Result {
    //Application requires the boot component to have default implemented,
    // Which is probably a good practice as it avoids inconsistency due to partial
    // state initialization
    iced::application(move || AppState::new(), AppState::update, AppState::view).run()
}

#[derive(Debug, Default)]
enum Screen {
    #[default]
    Overview,
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
        //Not sure if I will keep the Menu Bar. It doesn't match modern styles, and I am not sure how well it would work compared to a settings screen.
        let menu_bar = container("I am a menu bar!")
            .style(container::rounded_box)
            .height(10);

        //TODO: Create a sidebar struct to handle store width and syncronize style
        let sidebar = column![
            sidebar::item("I am a sidebar!", || Message::ButtonPressed),
            sidebar::item("with a button!", || Message::ButtonPressed)
        ]
        .width(400)
        .padding(10);

        let main_screen = column!["I am the center top!", "I am the center bottom!"]
            .width(Fill)
            .spacing(10);
        //Do I really need this Container?
        container(
            column![
                menu_bar,
                row![
                    //TODO: Refactor the Column and width pattern here into a
                    sidebar,
                    main_screen
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
