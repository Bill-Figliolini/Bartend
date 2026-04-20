use std::path::Path;

use rusqlite::Connection;

pub mod sqlite;

#[derive(Debug)]
pub struct Database {
    pub connection: Connection,
}
pub trait DBCreate {
    fn create(db: &Database);
}
pub trait DBUnit {
    fn update(self, db: &Database);
    fn delete(self, db: &Database);
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
