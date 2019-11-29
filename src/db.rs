use std::sync::Arc;

use prost::Message;
use rocksdb::{Direction, Error, IteratorMode, DB};

use crate::models::{BlockInterval, DbItem};

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

    pub fn prefix_iter(
        self,
        start_prefix: &[u8],
        opt_interval: Option<BlockInterval>,
    ) -> Vec<DbItem> {
        let unfiltered = self
            .0
            .iterator(IteratorMode::From(&start_prefix, Direction::Forward))
            .take_while(|(prefix, _)| prefix[..start_prefix.len()] == start_prefix[..])
            .map(|(_prefix, raw_item)| {
                // This is safe as long as DB is not corrupted
                DbItem::decode(&raw_item[..]).unwrap()
            });
        // Filter
        match opt_interval {
            Some(interval) => match interval.end {
                0 => unfiltered
                    .filter(|db_item| db_item.block_height >= interval.start)
                    .collect(),
                y => unfiltered
                    .filter(|db_item| {
                        db_item.block_height >= interval.start && db_item.block_height <= y
                    })
                    .collect(),
            },
            None => unfiltered
                .filter(|db_item| db_item.block_height == 0)
                .collect(),
        }
    }
}
