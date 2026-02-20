pub mod common;
mod logic;
mod persistence;
mod presentation;

use presentation::application;

fn main() -> iced::Result {
    application::run()
}
