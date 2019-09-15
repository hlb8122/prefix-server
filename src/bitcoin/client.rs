use crate::net::jsonrpc_client::*;

use std::sync::Arc;

use futures::Future;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct BitcoinClient(Arc<JsonClient>);

impl BitcoinClient {
    pub fn new(endpoint: String, username: String, password: String) -> BitcoinClient {
        BitcoinClient(Arc::new(JsonClient::new(endpoint, username, password)))
    }

    pub fn get_raw_tx(
        &self,
        tx_id: &[u8],
    ) -> Box<dyn Future<Item = Vec<u8>, Error = ClientError> + Send> {
        let request = self.0.build_request(
            "getrawtransaction".to_string(),
            vec![Value::String(hex::encode(tx_id))],
        );
        Box::new(
            self.0
                .send_request(&request)
                .and_then(|resp| resp.into_result::<String>())
                .map(|hex_tx| hex::decode(hex_tx).unwrap()),
        )
    }

    pub fn get_raw_block(
        &self,
        block_id: &[u8],
    ) -> Box<dyn Future<Item = Vec<u8>, Error = ClientError> + Send> {
        let request = self.0.build_request(
            "getrawblock".to_string(),
            vec![
                Value::String(hex::encode(block_id)),
                Value::Number(0.into()),
            ],
        );
        Box::new(
            self.0
                .send_request(&request)
                .and_then(|resp| resp.into_result::<String>())
                .map(|hex_block| hex::decode(hex_block).unwrap()),
        )
    }

    pub fn get_block_number(
        &self,
        block_id: &[u8],
    ) -> Box<dyn Future<Item = u32, Error = ClientError> + Send> {
        let request = self.0.build_request(
            "getblockheader".to_string(),
            vec![Value::String(hex::encode(block_id))],
        );
        Box::new(
            self.0
                .send_request(&request)
                .and_then(|resp| resp.into_result::<Value>())
                .map(move |value_header| {
                    let height_value = value_header.get("height").unwrap().to_owned();
                    serde_json::from_value(height_value).unwrap()
                }),
        )
    }
}
