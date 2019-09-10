pub mod errors;
pub mod jsonrpc_client;

use actix_web::{web, HttpResponse};
use futures::{future::err, Future};
use serde_derive::Serialize;

use crate::{bitcoin::BitcoinClient, db::KeyDB};

use errors::*;

#[derive(Serialize)]
struct PrefixResponse {
    raw_tx: String,
    input_index: u32,
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
    let item = match db_data.get(&raw_prefix) {
        Some(some) => some,
        None => return Box::new(err(ServerError::PrefixNotFound)),
    };

    // Get tx from bitcoind
    let fut_tx = client.get_raw_tx(&item.tx_id);

    // Create response
    Box::new(fut_tx.map(move |raw_tx| {
        HttpResponse::Ok().json(PrefixResponse {
            raw_tx,
            input_index: item.index
        })
    }).map_err(|e| e.into()))
}
