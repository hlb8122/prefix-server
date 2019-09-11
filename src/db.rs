use std::sync::Arc;

use prost::Message;
use rocksdb::{Error, DB};

use crate::models::Item;

#[derive(Clone)]
pub struct KeyDB(Arc<DB>);

impl KeyDB {
    pub fn try_new(path: &str) -> Result<Self, Error> {
        DB::open_default(path).map(Arc::new).map(KeyDB)
    }

    pub fn close(self) {
        drop(self)
    }

    pub fn put(&self, hash: &[u8], item: &Item) -> Result<(), Error> {
        let mut raw_item = Vec::with_capacity(item.encoded_len());
        item.encode(&mut raw_item).unwrap(); // This is safe
        self.0.put(&hash, raw_item)
    }

    pub fn get(&self, addr: &[u8]) -> Option<Vec<Item>> {
        // This panics if stored bytes are fucked
        let items: Vec<Item> = self
            .0
            .prefix_iterator(addr)
            .take_while(|(prefix, _)| {
                &prefix[..addr.len()] == addr
            })
            .map(|(_prefix, raw_item)| {
                // This is safe as long as DB is not corrupted
                Item::decode(&raw_item[..]).unwrap()
            })
            .collect();

        if items.is_empty() {
            None
        } else {
            Some(items)
        }
    }
}
