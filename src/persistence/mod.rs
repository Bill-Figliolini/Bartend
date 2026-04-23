use std::path::Path;

use rusqlite::Connection;

pub mod sqlite;

#[derive(Debug)]
pub struct Database {
    pub connection: Connection,
}
pub trait Repository {
    fn execute(&mut self, sql: &str);
    fn get_last_id(&self) -> i64;
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

impl Repository for Database {
    fn execute(&mut self, sql: &str) {
        if let Err(e) = self.connection.execute(sql, ()) {
            panic!("DB Error:\r\n sql: {sql}\r\nerror: {e}");
        }
    }

    fn get_last_id(&self) -> i64 {
        self.connection.last_insert_rowid()
    }
}
