#[macro_use]
extern crate clap;

pub mod bitcoin;
pub mod db;
pub mod net;
pub mod settings;

use std::io;

use actix_web::{middleware::Logger, App, HttpServer};
use env_logger::Env;
use lazy_static::lazy_static;
use log::info;

use crate::{
    bitcoin::{tx_stream, BitcoinClient},
    db::KeyDB,
    settings::Settings,
};

pub mod models {
    include!(concat!(env!("OUT_DIR"), "/models.rs"));
}

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().expect("couldn't load config");
}

fn main() -> io::Result<()> {
    let sys = actix_rt::System::new("prefix-server");

    // Init logging
    env_logger::from_env(Env::default().default_filter_or("actix_web=info,prefix-server=info"))
        .init();
    info!("starting server @ {}", SETTINGS.bind);

    // Open DB
    let key_db = KeyDB::try_new(&SETTINGS.db_path).unwrap();

    // Init Bitcoin client
    let bitcoin_client = BitcoinClient::new(
        format!("http://{}:{}", SETTINGS.node_ip.clone(), SETTINGS.rpc_port),
        SETTINGS.rpc_username.clone(),
        SETTINGS.rpc_password.clone(),
    );

    // Init ZMQ
    let (tx_stream, connection) =
        tx_stream::get_tx_stream(&format!("tcp://{}:{}", SETTINGS.node_ip, SETTINGS.zmq_port));
    // let key_stream = tx_stream::extract_details(tx_stream);
    // actix_rt::Arbiter::current().send(connection.map_err(|e| error!("{:?}", e)));

    // Setup peer polling logic
    // actix_rt::Arbiter::current().send(client.peer_polling(key_db.clone(), key_stream));

    // Init REST server
    HttpServer::new(move || {
        let key_db_inner = key_db.clone();
        let bitcoin_client_inner = bitcoin_client.clone();

        // Init app
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
        // .route("/prefix/{prefix}", prefix_search)
    })
    .bind(&SETTINGS.bind)?
    .start();

    sys.run()
}
