use std::collections::HashMap;

use flume::Sender;

use crate::models::{CategoryCommand, ItemBody, ItemID};

#[derive(Debug)]
pub(super) struct ItemService {
    items: HashMap<ItemID, ItemBody>,
    categories_service: Sender<CategoryCommand>,
}
