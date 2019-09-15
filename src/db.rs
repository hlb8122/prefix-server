use std::sync::Arc;

use prost::Message;
use rocksdb::{Error, DB};

use crate::models::DbItem;

#[derive(Clone, Debug)]
pub struct KeyDB(pub Arc<DB>);

impl KeyDB {
    pub fn try_new(path: &str) -> Result<Self, Error> {
        DB::open_default(path).map(Arc::new).map(KeyDB)
    }

    pub fn close(self) {
        drop(self)
    }

    pub fn put(&self, hash: &[u8], item: &DbItem) -> Result<(), Error> {
        let mut raw_item = Vec::with_capacity(item.encoded_len());
        item.encode(&mut raw_item).unwrap(); // This is safe
        self.0.put(&hash, raw_item)
    }

    pub fn prefix_iter(self, prefix: &[u8]) -> Vec<DbItem> {
        self.0
            .prefix_iterator(&prefix)
            .take_while(|(prefix, _)| prefix[..prefix.len()] == prefix[..])
            .map(|(_prefix, raw_item)| {
                // This is safe as long as DB is not corrupted
                DbItem::decode(&raw_item[..]).unwrap()
            })
            .collect()
    }
}
