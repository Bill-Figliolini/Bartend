mod config;
mod presentation;

use presentation::application;

fn main() -> iced::Result {
    let config = config::build_config();

    application::run(config)
}
