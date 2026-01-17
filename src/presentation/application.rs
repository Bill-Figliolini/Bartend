use iced::{
    Element,
    widget::{button, text},
};

use crate::{config::Config, presentation::page::Page};

pub fn run() -> iced::Result {
    //Application requires the boot component to have default implemented,
    // Which is probably a good practice as it avoids inconsistency due to partial
    // state initialization
    iced::application(AppState::default, update, view).run()
}
#[derive(Debug, Default)]
struct AppState {
    page: Page,
    config: Config,
}

#[derive(Debug, Clone)]
//Question I still need to figure out; What should a message be in the context of this application?
enum Message {}
//As a personal reminder while developing. Update is for transformations over AppState
// Given my trying to keep this program modular, this will likely be a branching function between pages' update functions.
fn update(state: &mut AppState, message: Message) {
    match message {}
}

//Personal Reminder:
// This is for displays and views of the current app state.
fn view(state: &AppState) -> Element<'_, Message> {
    button(text("Todo")).into()
}

#[cfg(test)]
mod test {
    use super::*;
}
