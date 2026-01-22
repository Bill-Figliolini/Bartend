use iced::{
    Element,
    Length::Fill,
    Task, Theme,
    widget::{column, container, row},
};

use crate::presentation::{
    screen::{
        inventory::{self, Inventory},
        overview::{self, Overview},
        recipe::{self, Recipe},
    },
    widget::sidebar,
};

pub fn run() -> iced::Result {
    //Application requires the boot component to have default implemented,
    // Which is probably a good practice as it avoids inconsistency due to partial
    // state initialization
    iced::application(AppState::new, AppState::update, AppState::view).run()
}

#[derive(Debug)]
enum Screen {
    About,
    Overview(Overview),
    Recipe(Recipe),
    Inventory(Inventory),
}

#[derive(Debug)]
struct AppState {
    screen: Screen,
    theme: Theme,
}

#[derive(Debug, Clone)]
pub enum Message {
    NoOp,
    OpenOverview,
    OpenRecipe,
    OpenInventory,
    //Messages for forwarding to relevant structs
    Overview(overview::Message),
    Recipe(recipe::Message),
    Inventory(inventory::Message),
}

#[derive(Debug, Clone)]
pub enum Event {
    None,
}

impl AppState {
    fn new() -> Self {
        AppState {
            screen: Screen::Overview(Overview::new()),
            theme: Theme::TokyoNight,
        }
    }

    //Question I still need to figure out; What should a message be in the context of this application?
    //As a personal reminder while developing. Update is for transformations over AppState
    // Given my trying to keep this program modular, this will likely be a branching function between pages' update functions.
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            //TODO: Clean up state transitions here
            Message::OpenOverview => {
                self.screen = Screen::Overview(Overview::new());
                Task::none()
            }
            Message::OpenRecipe => {
                self.screen = Screen::Recipe(Recipe::new());
                Task::none()
            }
            Message::OpenInventory => {
                self.screen = Screen::Inventory(Inventory::new());
                Task::none()
            }
            Message::NoOp => Task::none(),
        }
    }

    //Personal Reminder:
    // This is for displays and views of the current app state.
    fn view(&self) -> Element<'_, Message> {
        //Not sure if I will keep the Menu Bar. It doesn't match modern styles, and I am not sure how well it would work compared to a settings screen.
        let menu_bar = container("I am a menu bar!")
            .style(container::rounded_box)
            .width(Fill)
            .height(20);

        //TODO: Create a sidebar struct to handle store width and syncronize style
        let sidebar = column![
            sidebar::button("Overview", || Message::OpenOverview),
            sidebar::button("Inventory", || Message::OpenInventory),
            sidebar::button("Recipes", || Message::OpenRecipe)
        ]
        .width(300)
        .padding(10);

        let screen = match &self.screen {
            Screen::Overview(overview) => overview.view(),
            Screen::Recipe(recipe) => recipe.view(),
            Screen::Inventory(inventory) => inventory.view(),
            _ => todo!(),
        };
        //Do I really need this Container?
        container(
            column![
                menu_bar,
                row![sidebar, container(screen).padding(10).width(Fill)].spacing(10),
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
