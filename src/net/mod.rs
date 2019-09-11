pub mod errors;
pub mod jsonrpc_client;

use actix_web::{web, HttpResponse};
use futures::{future::err, stream, Future, Stream};
use serde_derive::Serialize;

use crate::{bitcoin::BitcoinClient, db::KeyDB};

use errors::*;

#[derive(Serialize)]
struct PrefixResponse {
    result: Vec<PrefixItem>,
}

#[derive(Serialize)]
struct PrefixItem {
    raw_tx: String,
    input_index: u32,
}

pub fn status() -> Result<HttpResponse, ServerError> {
    // TODO
    Ok(HttpResponse::Ok().finish())
}

pub fn prefix_search(
    prefix: web::Path<String>,
    db_data: web::Data<KeyDB>,
    client: web::Data<BitcoinClient>,
) -> Box<dyn Future<Item = HttpResponse, Error = ServerError>> {
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
            client.get_raw_tx(&item.tx_id).map(move |raw_tx| PrefixItem {
                raw_tx,
                input_index: item.index,
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
