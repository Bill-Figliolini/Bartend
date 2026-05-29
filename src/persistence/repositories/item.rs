use crate::{
    models::{
        item::{Item, ItemBody, ItemID},
        quantity::Quantity,
    },
    persistence::{
        DBError, ItemDB,
        repositories::{ItemRepository, Repository},
    },
};

impl<'a> Repository for ItemDB<'a> {
    fn create_table(&self) -> Result<(), DBError> {
        let query = "CREATE TABLE IF NOT EXISTS items(
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    quantity REAL NOT NULL,
                    unit INTEGER NOT NULL
                );";
        self.connection.execute(query, ())?;
        Ok(())
    }
}

impl<'a> ItemRepository for ItemDB<'a> {
    fn insert(&self, item: &ItemBody) -> Result<ItemID, DBError> {
        let (quantity, unit) = item.quantity.db_format();
        self.connection.execute(
            "INSERT INTO items(name, quantity, unit) VALUES (?1, ?2, ?3)",
            (&item.name, quantity, unit),
        )?;
        Ok(ItemID(self.connection.last_insert_rowid()))
    }

    fn update(&self, item: &Item) -> Result<(), DBError> {
        let id = item.id.0;
        let (quantity, unit) = item.body.quantity.db_format();
        self.connection.execute(
            "UPDATE items SET
                name = ?2,
                quantity = ?3,
                unit = ?4
                WHERE id = ?1",
            (id, &item.body.name, quantity, unit),
        )?;
        Ok(())
    }

    fn delete(&self, item: Item) -> Result<(), DBError> {
        self.connection
            .execute("DELETE FROM items WHERE id = ?1", (item.id.0,))?;
        Ok(())
    }

    fn get_range(&self, offset: usize, quantity: usize) -> Result<Vec<Item>, DBError> {
        let query = format!("SELECT * FROM items LIMIT {quantity} OFFSET {offset}");
        let mut stmt = self
            .connection
            .prepare(&query)
            .expect("Query must be valid");
        let rows = stmt
            .query_map([], |row| {
                let id = row.get(0).unwrap();
                let name = row.get(1).unwrap();
                let quantity = Quantity::from_db(row.get(3).unwrap(), row.get(2).unwrap());
                Ok(Item {
                    id,
                    body: ItemBody { name, quantity },
                })
            })
            .unwrap();
        Ok(rows
            .into_iter()
            .fold(Vec::with_capacity(quantity), |mut acc, row| {
                match row {
                    Ok(item) => acc.push(item),
                    Err(e) => panic!("Retrieving Items failled with error: {e}"),
                }
                acc
            }))
    }
}
