use std::collections::HashMap;

use crate::{
    models::{Item, ItemBody, ItemID},
    persistence::repositories::ItemRepository,
};

#[derive(Debug)]
pub struct ItemService {
    items: HashMap<ItemID, ItemBody>,
    page_size: usize,
}

impl ItemService {
    pub fn new(db: &impl ItemRepository) -> Self {
        let items = match db.get_all() {
            Ok(items) => items,
            Err(e) => panic!("Error loading Items: {e}"),
        };
        ItemService {
            items,
            page_size: 15,
        }
    }

    pub fn get_page(&self, page: usize) -> Vec<ItemID> {
        let page_start = self.page_size * page;
        self.items
            .keys()
            .copied()
            .skip(page_start)
            .take(self.page_size)
            .collect()
    }
    pub fn get_all(&self) -> Vec<ItemID> {
        self.items.keys().copied().collect()
    }

    #[must_use]
    pub fn get(&self, item: &ItemID) -> &ItemBody {
        match self.items.get(item) {
            Some(body) => body,
            None => panic!("Invalid Item in circulation!"),
        }
    }
    pub fn add(&mut self, db: &impl ItemRepository, item: &ItemBody) -> ItemID {
        let id = match db.insert(item) {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        };
        self.items.insert(id, item.clone());
        id
    }
    pub fn update(&mut self, db: &impl ItemRepository, item: Item) {
        if let Err(e) = db.update(&item) {
            panic!("{e}");
        };
        let item_location = self.items.get_mut(&item.id).unwrap();
        *item_location = item.body;
    }
    pub fn delete(&mut self, db: &impl ItemRepository, item: ItemID) {
        if let Err(e) = db.delete(item) {
            panic!("{e}");
        };
        self.items.remove(&item);
    }
}
