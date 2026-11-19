use crate::persistence::DBVersion;

pub(super) struct Migration {
    pub version: u32,
    pub sql: &'static str,
}

pub(super) const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: V1,
    },
    Migration {
        version: 2,
        sql: V2,
    },
];
#[expect(
    clippy::cast_possible_wrap,
    reason = "the migration count is tiny, and `i64::try_from` is not usable in a const"
)]
pub(super) const LATEST: DBVersion = DBVersion(MIGRATIONS.len() as i64);
pub(super) const VERSION_PRAGMA: &str = "user_version";

const V1: &str = "
                CREATE TABLE IF NOT EXISTS items(
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    quantity REAL NOT NULL,
                    unit INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS category(
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS graph(
                    parent_id INTEGER,
                    child_id INTEGER,
                    FOREIGN KEY (parent_id) REFERENCES category(id) ON DELETE CASCADE,
                    FOREIGN KEY (child_id) REFERENCES category(id) ON DELETE CASCADE,
                UNIQUE (parent_id, child_id));

                CREATE TABLE IF NOT EXISTS category_item(
                    category_id INTEGER,
                   item_id INTEGER,
                   FOREIGN KEY (category_id) REFERENCES category(id) ON DELETE CASCADE,
                   FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
                UNIQUE(category_id, item_id));

                CREATE TABLE IF NOT EXISTS recipes(
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS ingredients(
                    recipe_id INTEGER,
                    ingredient_index INTEGER,
                    category_id INTEGER,
                    quantity REAL NOT NULL,
                    unit INTEGER NOT NULL,
                    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
                    FOREIGN KEY (category_id) REFERENCES category(id) ON DELETE RESTRICT,
                UNIQUE(recipe_id, ingredient_index));
            ";

/// An optional full size per item, stored like its quantity: an amount, and
/// the kind of unit it is in. Both are NULL when there is none.
const V2: &str = "
                ALTER TABLE items ADD COLUMN full_size REAL;
                ALTER TABLE items ADD COLUMN full_size_unit INTEGER;
            ";
#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::persistence::{
        DBError, DBVersion, Database, LATEST, MIGRATIONS, VERSION_PRAGMA,
        repositories::ItemRepository,
    };

    #[test]
    fn migration_versions_are_sequential() {
        for (v, m) in (1..=LATEST.0).zip(MIGRATIONS.iter()) {
            assert_eq!(v, i64::from(m.version));
        }
    }

    #[test]
    fn tables_exist() {
        let connection = Connection::open_in_memory().unwrap();
        let table_names = vec![
            "items",
            "category",
            "graph",
            "category_item",
            "recipes",
            "ingredients",
        ];

        let db = Database::new(connection).unwrap();

        for table in table_names {
            assert!(db.connection.table_exists(None, table).unwrap());
        }
    }
    #[test]
    fn upgrading_from_v1_keeps_items_without_a_full_size() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(super::V1).unwrap();
        connection.pragma_update(None, VERSION_PRAGMA, 1).unwrap();
        connection
            .execute(
                "INSERT INTO items(name, quantity, unit) VALUES ('Tanqueray', 750.0, 0)",
                [],
            )
            .unwrap();

        let db = Database::new(connection).unwrap();

        let items = db.item_db().get_all().unwrap();
        let body = items.values().next().unwrap();
        assert_eq!(body.name, "Tanqueray");
        assert_eq!(body.full_size, None);
    }

    #[test]
    fn rejects_used_db_file() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute("CREATE TABLE test(id PRIMARY KEY);", [])
            .unwrap();

        let db_result = Database::new(connection);

        assert_eq!(db_result.unwrap_err(), DBError::NotABartendDB);
    }
    #[test]
    fn rejects_next_version() {
        let connection = Connection::open_in_memory().unwrap();
        let next_version = DBVersion(LATEST.0 + 1);
        connection
            .pragma_update(None, VERSION_PRAGMA, &next_version)
            .unwrap();

        let db_result = Database::new(connection);

        assert_eq!(
            db_result.unwrap_err(),
            DBError::FutureSchema {
                found: next_version,
                supported: LATEST
            }
        );
    }
}
