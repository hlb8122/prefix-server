use bitcoin::{consensus::encode, util::psbt::serialize::Deserialize, Transaction};
use bitcoin_zmq::{errors::SubscriptionError, Topic, ZMQSubscriber};
use futures::{Future, Stream};

#[derive(Debug)]
pub enum StreamError {
    Subscription(SubscriptionError),
    Deserialization(encode::Error),
}

impl From<SubscriptionError> for StreamError {
    fn from(err: SubscriptionError) -> StreamError {
        StreamError::Subscription(err)
    }
}

pub fn get_tx_stream(
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
