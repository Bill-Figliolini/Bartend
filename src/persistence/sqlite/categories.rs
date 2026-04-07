use rusqlite::Connection;

pub(super) fn create_category_tables(connection: &Connection) {
    let create_category = "
        CREATE TABLE IF NOT EXISTS category(
            id INTEGER PRIMARY KEY,
            name STRING NOT NULL,
        );";
    let category_result = connection.execute(create_category, ());
    if let Err(e) = category_result {
        panic!("Category table creation failed with error: {e}");
    }
    let create_category_graph = "
        CREATE TABLE IF NOT EXISTS category_relations(
            parent_id INTEGER,
            child_id INTEGER,
            UNIQUE (parent_id, child_id)
        );";
    let graph_result = connection.execute(create_category_graph, ());
    if let Err(e) = graph_result {
        panic!("Graph table creation failed with error: {e}");
    }
}
