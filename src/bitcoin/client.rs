use crate::net::jsonrpc_client::*;

use std::sync::Arc;

use futures::Future;
use serde_json::Value;

#[derive(Clone)]
pub struct BitcoinClient(Arc<JsonClient>);

impl BitcoinClient {
    pub fn new(endpoint: String, username: String, password: String) -> BitcoinClient {
        BitcoinClient(Arc::new(JsonClient::new(endpoint, username, password)))
    }

    pub fn get_raw_tx(
        &self,
        tx_id: &[u8],
    ) -> Box<dyn Future<Item = String, Error = ClientError> + Send> {
        let request = self.0.build_request(
            "getrawtransaction".to_string(),
            vec![Value::String(hex::encode(tx_id))],
        );
        Box::new(
            self.0
                .send_request(&request)
                .and_then(|resp| resp.into_result::<String>()),
        )
    }
}
