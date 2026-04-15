use rusqlite::Connection;

pub mod sqlite;

#[derive(Debug)]
pub struct DB {
    connection: Connection,
}
trait Persistable {
    fn create(&self, db: &DB);
    fn read(db: &DB) -> Self;
    fn update(&self, db: &DB);
    fn delete(self, db: &DB);
}
