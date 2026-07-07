use rusqlite::Connection;

pub struct GraphDB<'a> {
    pub connection: &'a Connection,
}
