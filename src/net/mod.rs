pub mod errors;
pub mod jsonrpc_client;

use futures::{future::err, stream, Future, Stream};
use serde_derive::Serialize;

use crate::{bitcoin::BitcoinClient, db::KeyDB, SETTINGS};

use errors::*;

type ItemStream = Box<dyn Stream<Item = Item, Error = tower_grpc::Status> + Send>;

pub fn prefix_search(
    prefix: Vec<u8>,
    db_data: web::Data<KeyDB>,
    client: web::Data<BitcoinClient>,
) -> ItemStream {
    if prefix.len() < 2 * SETTINGS.min_prefix {
        return Box::new(err(ServerError::PrefixTooShort));
    }

    let raw_prefix = match hex::decode(&*prefix) {
        Ok(ok) => ok,
        Err(e) => return Box::new(err(e.into())),
    };

    // Get from db
    let items = match db_data.get(&raw_prefix) {
        Some(some) => some,
        None => return Box::new(err(ServerError::PrefixNotFound)),
    };

    // Get tx from bitcoind
    let fut_txs = stream::iter_ok(items)
        .and_then(move |item| {
            client
                .get_raw_tx(&item.tx_id)
                .map(move |raw_tx| PrefixItem {
                    raw_tx,
                    input_index: item.input_index,
                })
        })
        .collect();

    // Create response
    Box::new(
        fut_txs
            .map(move |prefix_items| {
                HttpResponse::Ok().json(PrefixResponse {
                    result: prefix_items,
                })
            })
            .map_err(|e| e.into()),
    )
}
