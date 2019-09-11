use bitcoin::Block;
use futures::Stream;

use super::tx_stream::StreamError;

use crate::models::Item;

pub enum Status {
    Idle,
    Scraping(u32, u32, u32),
}

impl Default for Status {
    fn default() -> Self {
        Status::Idle
    }
}

// TODO
// pub fn scrape(
//     start: u32,
//     end: u32,
// ) -> impl Stream<Item = Vec<(Vec<u8>, Item)>, Error = StreamError> {

// }
