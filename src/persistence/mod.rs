use rusqlite::{Connection, ToSql};

pub mod sqlite;

#[derive(Debug)]
pub struct DB {
    pub connection: Connection,
}
pub trait DBCreate {
    fn create(db: &DB);
}
pub trait DBUnit {
    fn update(self, db: &DB);
    fn delete(self, db: &DB);
}
