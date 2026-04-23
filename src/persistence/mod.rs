use std::{path::Path, sync::atomic::AtomicI64};

use rusqlite::Connection;

pub mod sqlite;

#[derive(Debug)]
pub struct Database {
    pub connection: Connection,
}

#[derive(Debug)]
pub struct MockDB {
    current_id: AtomicI64,
}
pub trait Repository {
    fn execute(&mut self, sql: &str);
    fn get_last_id(&self) -> i64;
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

impl MockDB {
    pub fn new() -> Self {
        Self {
            current_id: AtomicI64::new(0),
        }
    }
}

impl Repository for MockDB {
    fn execute(&mut self, _sql: &str) {
        return;
    }

    fn get_last_id(&self) -> i64 {
        self.current_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}
