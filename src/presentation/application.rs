use iced::{
    Element,
    widget::{button, text},
};

use crate::config::Config;

pub fn run(config: Config) -> iced::Result {
    iced::run(update, view)
}
#[derive(Debug, Default)]
struct AppState {
    counter: i64,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

fn update(state: &mut AppState, message: Message) {
    match message {
        Message::Increment => state.counter += 1,
    }
}

fn view(state: &AppState) -> Element<'_, Message> {
    button(text(state.counter))
        .on_press(Message::Increment)
        .into()
}


#[cfg(test)]
mod test {
    use super::*;
}