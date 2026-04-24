use std::path::Path;

use rusqlite::{Connection, ToSql};

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
    pub fn execute(&self, stmt: &str) {
        if let Err(e) = self.connection.execute(stmt, ()) {
            panic!("DB Error:\r\n sql: {stmt}\r\nerror: {e}");
        }
    }

    pub fn bulk_execute(&self, stmts: &[String]) {
        let stmt = stmts.join(";\n");
        if let Err(e) = self.connection.execute_batch(&stmt) {
            panic!("DB Error:\r\n sql: {stmt}\r\nerror: {e}");
        }
    }
    pub fn get_last_id(&self) -> i64 {
        self.connection.last_insert_rowid()
    }

    fn sanitize(input: String) -> String {
        match input.to_sql() {
            Ok(output) => output,
            Err(e) => panic!("Error Sanitizing SQL query: {input}\nError: {e}"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::common::{item::Item, quantity::Quantity};

    use super::*;
    use rusqlite::ToSql;
    #[test]
    fn to_sql_works_separately_or_on_whole() {
        let id = 0;
        let name = "Laird's";
        let quantity = Quantity::Volume { quantity: 750.0 };
        let (quantity, unit) = quantity.db_format();
        let query_format = "INSERT INTO items(name, quantity, unit) VALUES ({}, {}, {})";

        let query_whole: String = format!(
            "INSERT INTO items(name, quantity, unit) VALUES ({}, {}, {})",
            name.clone(),
            quantity,
            unit
        )
        .to_sql()
        .unwrap()
        .into();
        let name = name.to_sql().unwrap();
        let query_part = format!(
            "INSERT INTO items(name, quantity, unit) VALUES ({}, {}, {})",
            name, quantity, unit
        );
        assert_eq!(query_whole, query_part);
    }
}
