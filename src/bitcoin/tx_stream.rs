use std::fmt;

use bitcoin::{
    consensus::encode::{self, Encodable},
    util::psbt::serialize::Deserialize,
    Transaction,
};
use bitcoin_hashes::{sha256::Hash as Sha256, Hash};
use bitcoin_zmq::{errors::SubscriptionError, Topic, ZMQSubscriber};
use futures::{Future, Stream};

use crate::{models::Item, net::jsonrpc_client::ClientError};

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

pub fn get_item_stream(
    node_addr: &str,
) -> (
    impl Stream<Item = Vec<(Vec<u8>, Item)>, Error = StreamError>,
    impl Future<Item = (), Error = StreamError> + Send + Sized,
) {
    let (tx_stream, connection) = get_tx_stream(node_addr);
    let item_stream = tx_stream.map(move |tx| {
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
                    Item {
                        index: index as u32,
                        tx_id,
                    },
                )
            })
            .collect()
    });

    (item_stream, connection)
}
