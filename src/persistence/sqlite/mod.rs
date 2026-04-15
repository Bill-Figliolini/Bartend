mod categories;
mod item;

use rusqlite::{self, Connection};
use std::path::Path;

use crate::persistence::{
    DB,
    sqlite::{categories::create_category_tables, item::create_item_table},
};

//TODO: Idea For later revision: Invert control, pass the DB into a trait-implemmented function on the members of Common
impl DB {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let connection = match Connection::open(path) {
            Ok(connection) => connection,
            Err(e) => {
                panic!("DB could not be opened! {e}")
            }
        };

        Self::create_tables(&connection);

        Self { connection }
    }
    fn create_tables(connection: &Connection) {
        create_item_table(connection);
        create_category_tables(connection);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::TempDir;
    mod table_creation {
        use super::*;
        #[test]
        fn items() {
            let dir = TempDir::new().unwrap();
            let file = dir.path().join("bartend.db");
            let db = DB::new(file);
            let items_name = "items";
            let columns = vec!["id", "name", "quantity"];

            assert!(db.connection.table_exists(None, items_name).unwrap());
            for column in columns {
                assert!(
                    db.connection
                        .column_exists(None, items_name, &column)
                        .unwrap()
                )
            }
        }
    }
}
