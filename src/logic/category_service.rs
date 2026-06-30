use std::collections::HashMap;

use crate::{
    logic::Service,
    models::{CategoryBody, CategoryCommand, CategoryID},
};

#[derive(Debug)]
pub(super) struct CategoryService {
    categories: HashMap<CategoryID, CategoryBody>,
    reciever: flume::Receiver<CategoryCommand>,
    sender: flume::Sender<CategoryCommand>,
}

impl CategoryService {
    pub(super) fn new() -> Self {
        let (sender, reciever) = flume::unbounded();
        Self {
            categories: HashMap::new(),
            reciever,
            sender,
        }
    }
}

impl Service<CategoryCommand> for CategoryService {
    fn get_channel(&self) -> flume::Sender<CategoryCommand> {
        self.sender.clone()
    }
}
