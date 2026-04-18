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
    fn read(id: impl ToSql, db: &Database) -> Self;
    fn update(self, db: &Database);
    fn delete(self, db: &Database);
}
