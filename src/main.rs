#![windows_subsystem = "windows"]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod logic;
pub mod models;
mod persistence;
mod presentation;

use presentation::application;

fn main() -> iced::Result {
    application::run()
}
