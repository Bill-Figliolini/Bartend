use crate::persistence::DBError;
#[derive(Debug)]
pub enum BartendError {
    LogicError(),
    DBError(DBError),
}

impl From<DBError> for BartendError {
    fn from(value: DBError) -> Self {
        BartendError::DBError(value)
    }
}
