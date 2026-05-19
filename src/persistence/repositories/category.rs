use crate::{
    logic::category::{Category, CategoryBody, CategoryID},
    persistence::{
        DBError,
        repositories::{CategoryDB, CategoryRepository},
    },
};

impl<'a> CategoryRepository for CategoryDB<'a> {
    fn create_table(&self) -> Result<(), DBError> {
        let query = "CREATE TABLE IF NOT EXISTS category(
                id INTEGER PRIMARY KEY,
                name STRING NOT NULL
            );";
        self.connection.execute(query, ())?;
        Ok(())
    }
    fn insert(&self, body: &CategoryBody) -> Result<CategoryID, DBError> {
        self.connection
            .execute("INSERT INTO category(name) VALUES(?1)", (&body.name,))?;
        Ok(CategoryID(self.connection.last_insert_rowid()))
    }
    fn update(&self, category: &Category) -> Result<(), DBError> {
        self.connection.execute(
            "
                UPDATE category SET
                name = ?2
                WHERE id = ?1
            ",
            (category.id.0, &category.body.name),
        )?;
        Ok(())
    }
    fn delete(&self, category: Category) -> Result<(), DBError> {
        self.connection
            .execute("DELETE FROM category WHERE id=?1", (&category.id,))?;
        Ok(())
    }
    fn get_range(&self, offset: usize, quantity: usize) -> Result<Vec<Category>, DBError> {
        let query = format!("SELECT * FROM category LIMIT {quantity} OFFSET {offset}");
        let mut stmt = self
            .connection
            .prepare(&query)
            .expect("Query must be valid");
        let rows = stmt.query_map([], |row| {
            let id = row.get(0).unwrap();
            let name = row.get(1).unwrap();
            Ok(Category {
                id,
                body: CategoryBody { name },
            })
        })?;
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
