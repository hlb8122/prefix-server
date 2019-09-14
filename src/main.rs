#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

pub mod bitcoin;
pub mod db;
pub mod net;
pub mod settings;

use std::sync::RwLock;

use bitcoin_zmq::ZMQSubscriber;
use env_logger::Env;
use futures::{Future, Stream, FutureResult};
use lazy_static::lazy_static;
use log::{error, info};
use tokio::net::TcpListener;
use tower_grpc::{Request, Response};
use tower_hyper::server::{Http, Server};

use crate::{
    models::server,
    bitcoin::{streams, BitcoinClient},
    db::KeyDB,
    models::{DbItem, ServerStatus},
    // net::prefix_search,
    settings::Settings,
};

pub mod models {
    include!(concat!(env!("OUT_DIR"), "/models.rs"));
}

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().expect("couldn't load config");
    pub static ref STATUS: RwLock<ServerStatus> = RwLock::new(ServerStatus::default());
}

fn insertion_loop(
    item_stream: impl Stream<Item = Vec<(Vec<u8>, DbItem)>, Error = streams::StreamError>,
    key_db: KeyDB,
) -> impl Future<Item = (), Error = ()> {
    item_stream
        .for_each(move |pairs: Vec<(Vec<u8>, DbItem)>| {
            // TODO: Batch insert
            pairs.iter().for_each(|(input_hash, item)| {
                if let Err(e) = key_db.put(input_hash, item) {
                    error!("{}", e);
                }
            });
            Ok(())
        })
        .map_err(|e| {
            // TODO: Better error handling
            error!("ZMQ stream failed {}", e);
        })
}

#[derive(Clone)]
struct PublicService {
    key_db: KeyDB,
    bitcoin_client: BitcoinClient
}

impl server::PublicService for PublicService {
    type PrefixSearchStream = Box<dyn Stream<Item = Item, Error = tower_grpc::Status> + Send>;
    type PrefixSearchFuture = FutureResult<Response<Self::PrefixSearchStream>, tower_grpc::Status>;

    fn prefix_search(&mut self, request: Request<Streaming<Item>>) -> Self::PrefixSearchFuture {
        
    }
}

fn main() {
    // Setup logging
    env_logger::from_env(Env::default().default_filter_or("actix_web=info,prefix-server=info"))
        .init();
    info!("starting server @ {}", SETTINGS.bind);

    // Open DB
    let key_db = KeyDB::try_new(&SETTINGS.db_path).unwrap(); // Unrecoverable

    // Setup Bitcoin client
    let bitcoin_client = BitcoinClient::new(
        format!("http://{}:{}", SETTINGS.node_ip.clone(), SETTINGS.rpc_port),
        SETTINGS.rpc_username.clone(),
        SETTINGS.rpc_password.clone(),
    );

    // Setup insertion loop
    let zmq_addr = format!("tcp://{}:{}", SETTINGS.node_ip, SETTINGS.zmq_port);
    let (zmq_subscriber, broker) = ZMQSubscriber::new(&zmq_addr, 1024);
    let item_stream = streams::get_db_item_stream(zmq_subscriber);

    let key_db_inner = key_db.clone();

    tokio::run(futures::lazy(|| {
        tokio::spawn(broker.map_err(|e| error!("{:?}", e)));
        tokio::spawn(insertion_loop(item_stream, key_db_inner));
        Ok(())
    }));
}
