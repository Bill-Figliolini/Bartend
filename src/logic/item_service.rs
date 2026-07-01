use std::collections::HashMap;

use crate::models::{ItemBody, ItemID};

#[derive(Debug)]
pub(super) struct ItemService {
    items: HashMap<ItemID, ItemBody>,
}
