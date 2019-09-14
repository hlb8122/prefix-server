use std::fmt;

use bitcoin::{
    consensus::encode::{self, Encodable},
    util::psbt::serialize::Deserialize,
    Transaction,
};
use bitcoin_hashes::{sha256::Hash as Sha256, Hash};
use bitcoin_zmq::{errors::SubscriptionError, Topic, ZMQSubscriber};
use futures::{
    future::{self, Either},
    stream, Future, Stream,
};

use crate::{
    models::{DbItem, Item},
    net::jsonrpc_client::ClientError,
};

#[derive(Debug)]
pub enum StreamError {
    Subscription(SubscriptionError),
    Deserialization(encode::Error),
    Client(ClientError),
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StreamError::Subscription(err) => err.fmt(f),
            StreamError::Deserialization(err) => err.fmt(f),
            StreamError::Client(err) => err.fmt(f),
        }
    }
}

impl From<SubscriptionError> for StreamError {
    fn from(err: SubscriptionError) -> StreamError {
        StreamError::Subscription(err)
    }
}

fn get_tx_stream(
    node_addr: &str,
) -> (
    impl Stream<Item = Transaction, Error = StreamError>,
    impl Future<Item = (), Error = StreamError> + Send + Sized,
) {
    let (stream, broker) = ZMQSubscriber::single_stream(node_addr, Topic::RawTx, 256);
    let stream = stream
        .map_err(|_| unreachable!()) // TODO: Double check that this is safe
        .and_then(move |raw_tx| {
            Transaction::deserialize(&raw_tx).map_err(StreamError::Deserialization)
        });

    (stream, broker.map_err(StreamError::Subscription))
}

pub fn get_db_item_stream(
    zmq_sub: ZMQSubscriber,
) -> (impl Stream<Item = Vec<(Vec<u8>, DbItem)>, Error = StreamError>) {
    // Get stream of transactions from rawtx zmq
    let tx_stream = zmq_sub.subscribe(Topic::RawTx).then(move |raw_tx| {
        Transaction::deserialize(&raw_tx.unwrap())
            .map_err(StreamError::Deserialization)
            .map(|tx| (0, tx))
    });

    // Get stream of block hashes via hashblock zmq
    // let block_stream = zmq_sub.subscribe(Topic::HashBlock).then(move |block_tx| {

    // });

    tx_stream.map(move |(block_height, tx)| {
        // TODO: The memory layout all berked up here
        let mut tx_id: [u8; 32] = tx.txid().into_inner();
        // Note reversal here
        tx_id.reverse();
        let tx_id = tx_id.to_vec();

        tx.input
            .iter()
            .map(move |input| {
                let mut raw = Vec::with_capacity(128);
                // This is safe presuming bitcoind doesn't return malformed inputs
                input.consensus_encode(&mut raw).unwrap();
                Sha256::hash(&raw).to_vec()
            })
            .enumerate()
            .map(move |(index, input_hash)| {
                let tx_id = tx_id.clone(); // TODO: Remove this clone?
                (
                    input_hash,
                    DbItem {
                        input_index: index as u32,
                        tx_id,
                        block_height,
                    },
                )
            })
            .collect()
    })
}

// pub fn scrape(
//     zmq_sub: ZMQSubscriber,
//     start: u32,
//     opt_end: Option<u32>,
// ) -> impl Stream<Item = Vec<(Vec<u8>, Item)>, Error = StreamError> {
//     let fut_end = match opt_end {
//         Some(end) => Either::A(future::ok(end)),
//         None => Either::B(),
//     };
//     let fut_scrape = stream::iter_ok(start..end).and_then(|block_height| {});
// }
