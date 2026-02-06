pub mod mock_items;
mod sqlite;

pub enum PersistenceError {
    FailedToOpenFile(String),
    FailedToExecute,
}

pub trait Repository {}
