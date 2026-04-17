use rusqlite::{Connection, ToSql};

pub mod sqlite;

#[derive(Debug)]
pub struct DB {
    pub connection: Connection,
}
pub trait DBStore {
    fn create(db: &DB);
    fn read_all(db: &DB) -> Vec<Self>
    where
        Self: Sized;
    fn update(&self, db: &DB);
    fn delete(self, db: &DB);
}
