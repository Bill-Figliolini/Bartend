use rusqlite::Connection;

use crate::persistence::repositories::Repository;

pub struct GraphDB<'a> {
    pub connection: &'a Connection,
}
