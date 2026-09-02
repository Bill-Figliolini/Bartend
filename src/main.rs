#![windows_subsystem = "windows"]

mod logic;
pub mod models;
mod persistence;
mod presentation;

use presentation::application;

fn main() -> iced::Result {
    application::run()
}
