use common::monitoring::monitor_client::MonitorClient;
use std::ops::{Deref, DerefMut};
use tonic_web_wasm_client::Client;

pub struct RpcClient {
    channel: MonitorClient<Client>,
}

impl RpcClient {
    pub fn new(dst: String) -> Self {
        let client = Client::new(dst);
        Self {
            channel: MonitorClient::new(client),
        }
    }
}

impl Deref for RpcClient {
    type Target = MonitorClient<Client>;

    fn deref(&self) -> &Self::Target {
        &self.channel
    }
}

impl DerefMut for RpcClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.channel
    }
}
