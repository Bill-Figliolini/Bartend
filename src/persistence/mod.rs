use rusqlite::Connection;

pub mod sqlite;

#[derive(Debug)]
pub struct DB {
    pub connection: Connection,
}
pub trait DBStore {
    fn create(db: &DB);
    fn read(db: &DB) -> Self;
    fn input(db: &DB, input: impl IntoIterator) -> Self;
    fn update(&self, db: &DB);
    fn delete(self, db: &DB);
}
