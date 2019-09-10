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
        item.encode(&mut raw_item).unwrap();
        self.0.put(&hash, raw_item)
    }

    pub fn get(&self, addr: &[u8]) -> Result<Option<Item>, Error> {
        // This panics if stored bytes are fucked
        self.0
            .get(&addr)
            .map(|opt_dat| opt_dat.map(|dat| Item::decode(&dat[..]).unwrap()))
    }
}
