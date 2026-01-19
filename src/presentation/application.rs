use iced::{
    Element,
    Length::Fill,
    Theme,
    theme::Base,
    widget::{Container, button, column, container, row, text},
};

use crate::presentation::screen::{Page, overview};

pub fn run() -> iced::Result {
    //Application requires the boot component to have default implemented,
    // Which is probably a good practice as it avoids inconsistency due to partial
    // state initialization
    iced::application(move || AppState::new(), AppState::update, AppState::view).run()
}
#[derive(Debug)]
struct AppState {
    count: u64,
    page: Page,
    theme: Theme,
}

#[derive(Debug, Clone)]
enum Message {
    ButtonPressed,
}

impl AppState {
    fn new() -> Self {
        AppState {
            count: 0,
            page: Page::Overview,
            theme: Theme::TokyoNight,
        }
    }

    //Question I still need to figure out; What should a message be in the context of this application?
    //As a personal reminder while developing. Update is for transformations over AppState
    // Given my trying to keep this program modular, this will likely be a branching function between pages' update functions.
    fn update(&mut self, message: Message) {
        match message {
            Message::ButtonPressed => {}
        }
    }

    //Personal Reminder:
    // This is for displays and views of the current app state.
    fn view(&self) -> Element<'_, Message> {
        let palette = self.theme.extended_palette();
        container(
            column![
                "I am a menu bar!",
                row![
                    column![
                        "I am a sidebar!",
                        button("with a button!")
                            .on_press(Message::ButtonPressed)
                            .style(|theme: &Theme, status| {
                                match status {
                                    button::Status::Active => button::Style::default()
                                        .with_background(palette.success.strong.color),
                                    _ => button::primary(theme, status),
                                }
                            })
                    ],
                    column!["I am the center top!", "I am the center bottom!"].spacing(10)
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
