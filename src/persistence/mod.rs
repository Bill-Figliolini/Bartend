use rusqlite::{Connection, ToSql};

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
