use crate::{logic::LogicError, persistence::DBError};
#[derive(Debug)]
pub enum BartendError {
    LogicError(LogicError),
    DBError(DBError),
}

impl From<DBError> for BartendError {
    fn from(value: DBError) -> Self {
        BartendError::DBError(value)
    }
}
impl From<LogicError> for BartendError {
    fn from(value: LogicError) -> Self {
        BartendError::LogicError(value)
    }
}
