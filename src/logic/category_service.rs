use std::collections::HashMap;

use crate::models::{CategoryBody, CategoryID};

#[derive(Debug)]
pub(super) struct CategoryService {
    categories: HashMap<CategoryID, CategoryBody>,
}
