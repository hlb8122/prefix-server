use std::fmt;

use bitcoin::{
    consensus::encode::{self, Decodable, Encodable},
    util::psbt::serialize::Deserialize,
    Block, Transaction,
};
use bitcoin_hashes::{sha256::Hash as Sha256, Hash};
use bitcoin_zmq::{errors::SubscriptionError, Topic, ZMQSubscriber};
use futures::{stream, Future, Stream};

use crate::{bitcoin::BitcoinClient, models::DbItem, net::jsonrpc_client::ClientError};

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

pub fn get_item_stream(
    zmq_sub: ZMQSubscriber,
    client: BitcoinClient,
) -> impl Stream<Item = Vec<(Vec<u8>, DbItem)>, Error = StreamError> {
    // Get stream of transactions from rawtx zmq
    let zmq_tx_stream = zmq_sub.subscribe(Topic::RawTx).then(move |raw_tx| {
        Transaction::deserialize(&raw_tx.unwrap())
            .map_err(StreamError::Deserialization)
            .map(|tx| (0, tx))
    });

    // Get stream of block hashes via hashblock zmq
    let block_tx_stream = zmq_sub
        .subscribe(Topic::HashBlock)
        .then(move |hash_block| {
            let hash_block = hash_block.unwrap();
            let fut_block = client.get_raw_block(&hash_block);
            let fut_number = client.get_block_number(&hash_block);
            fut_number.join(fut_block)
        })
        .map_err(StreamError::Client)
        .and_then(move |(block_height, block_raw)| {
            let block =
                Block::consensus_decode(&block_raw[..]).map_err(StreamError::Deserialization)?;
            let tx_iter = block.txdata.into_iter().map(move |tx| (block_height, tx));
            Ok(stream::iter_ok(tx_iter))
        })
        .flatten();

    let tx_stream = zmq_tx_stream.select(block_tx_stream);
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
