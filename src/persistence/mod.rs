use std::path::Path;

use rusqlite::Connection;

#[derive(Debug)]
pub struct Database {
    pub connection: Connection,
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let connection = match Connection::open(path) {
            Ok(connection) => connection,
            Err(e) => {
                panic!("DB could not be opened! {e}")
            }
        };
        let db = Self { connection };
        db
    }
}
