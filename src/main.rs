#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

pub mod bitcoin;
pub mod db;
pub mod net;
pub mod settings;

use std::{net::SocketAddr, sync::Mutex};

use bitcoin_zmq::ZMQSubscriber;
use env_logger::Env;
use futures::{
    future::{self, FutureResult},
    stream, Future, Stream,
};
use lazy_static::lazy_static;
use log::{error, info};
use tokio::net::TcpListener;
use tower_grpc::{Request, Response};
use tower_hyper::server::{Http, Server};

use crate::{
    bitcoin::{streams, BitcoinClient},
    db::KeyDB,
    models::{server, *},
    // net::prefix_search,
    settings::Settings,
};

pub mod models {
    include!(concat!(env!("OUT_DIR"), "/models.rs"));
}

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().expect("couldn't load config");
    pub static ref STATUS: Mutex<ServerStatus> = Mutex::new(ServerStatus::default());
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

#[derive(Clone, Debug)]
pub struct Public {
    key_db: KeyDB,
    bitcoin_client: BitcoinClient,
}

// TODO: Put on private port
impl server::Private for Public {
    type ScrapeFuture = future::FutureResult<Response<()>, tower_grpc::Status>;
    type StatusFuture = future::FutureResult<Response<ServerStatus>, tower_grpc::Status>;

    fn scrape(&mut self, request: Request<BlockInterval>) -> Self::ScrapeFuture {
        let interval = request.into_inner();
        if interval.start > interval.end {
            // TODO: Return error here
            // return future::err()
        }
        let opt_end = match interval.end {
            0 => None,
            other => Some(other),
        };
        let item_stream = insertion_loop(
            streams::scrape(self.bitcoin_client.clone(), interval.start, opt_end),
            self.key_db.clone(),
        )
        .map_err(|err| {
            // Set to idle on error
            let mut status_lock = STATUS.lock().unwrap();
            status_lock.state = 0;
            err
        });

        tokio::spawn(item_stream);
        future::ok(Response::new(()))
    }

    fn status(&mut self, _: Request<()>) -> Self::StatusFuture {
        future::ok(Response::new(STATUS.lock().unwrap().to_owned()))
    }
}

impl server::Public for Public {
    type PrefixSearchStream = Box<dyn Stream<Item = Item, Error = tower_grpc::Status> + Send>;
    type PrefixSearchFuture = FutureResult<Response<Self::PrefixSearchStream>, tower_grpc::Status>;

    fn prefix_search(&mut self, request: Request<SearchParams>) -> Self::PrefixSearchFuture {
        let params = request.into_inner();
        let key_db = self.key_db.clone();
        let bitcoin_client = self.bitcoin_client.clone();
        let item_stream = stream::iter_ok(key_db.prefix_iter(&params.prefix))
            .and_then(move |db_item| {
                bitcoin_client
                    .get_raw_tx(&db_item.tx_id)
                    .join(future::ok(db_item))
            })
            .map(|(raw_tx, db_item)| Item {
                raw_tx,
                input_index: db_item.input_index,
                block_height: db_item.block_height,
            })
            .map_err(|err| tower_grpc::Status::new(tower_grpc::Code::Internal, err.to_string()));
        future::ok(Response::new(Box::new(item_stream)))
    }
}

fn main() {
    // Setup logging
    env_logger::from_env(Env::default().default_filter_or("actix_web=info,prefix-server=info"))
        .init();
    info!("starting server @ {}", SETTINGS.bind);

    // Open DB
    let key_db = KeyDB::try_new(&SETTINGS.db_path).unwrap(); // Unrecoverable
    let key_db_inner = key_db.clone();

    // Setup Bitcoin client
    let bitcoin_client = BitcoinClient::new(
        format!("http://{}:{}", SETTINGS.node_ip.clone(), SETTINGS.rpc_port),
        SETTINGS.rpc_username.clone(),
        SETTINGS.rpc_password.clone(),
    );
    let bitcoin_client_inner = bitcoin_client.clone();

    // Setup gRPC
    let public = Public {
        key_db,
        bitcoin_client,
    };
    let mut server = Server::new(server::PublicServer::new(public));
    let addr: SocketAddr = SETTINGS.bind.parse().unwrap();
    let bind = TcpListener::bind(&addr).unwrap();
    let http = Http::new().http2_only(true).clone();
    let serve = bind
        .incoming()
        .for_each(move |sock| {
            if let Err(e) = sock.set_nodelay(true) {
                return Err(e);
            }

            let serve = server.serve_with(sock, http.clone());
            tokio::spawn(serve.map_err(|e| error!("hyper error: {:?}", e)));

            Ok(())
        })
        .map_err(|e| eprintln!("accept error: {}", e));

    // Setup insertion loop
    let zmq_addr = format!("tcp://{}:{}", SETTINGS.node_ip, SETTINGS.zmq_port);
    let (zmq_subscriber, broker) = ZMQSubscriber::new(&zmq_addr, 1024);
    let item_stream = streams::get_item_stream(zmq_subscriber, bitcoin_client_inner);

    tokio::run(futures::lazy(|| {
        tokio::spawn(broker.map_err(|e| error!("{:?}", e)));
        tokio::spawn(insertion_loop(item_stream, key_db_inner));
        tokio::spawn(serve);
        Ok(())
    }));
}
