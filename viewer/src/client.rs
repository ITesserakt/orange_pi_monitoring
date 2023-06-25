use common::monitoring::monitor_client::MonitorClient;
use common::monitoring::Pack;
use futures::future::AbortHandle;
use futures::Stream;
use std::ops::{Deref, DerefMut};
use tonic::{Request, Response, Status};

#[cfg(target_arch = "wasm32")]
type MonitorClientWithTransport = MonitorClient<tonic_web_wasm_client::Client>;
#[cfg(not(target_arch = "wasm32"))]
type MonitorClientWithTransport = MonitorClient<tonic::transport::Channel>;

pub struct RpcClient {
    channel: MonitorClientWithTransport,
}

impl RpcClient {
    #[cfg(target_arch = "wasm32")]
    pub fn new<S: ToString>(dst: S) -> Self {
        let client = tonic_web_wasm_client::Client::new(dst.to_string());
        Self {
            channel: MonitorClient::new(client),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new<S: AsRef<str>>(dst: S) -> Self {
        use std::str::FromStr;
        use tonic::transport::Channel;
        use tonic::transport::Uri;

        let channel =
            Channel::builder(Uri::from_str(dst.as_ref()).expect("Cannot build uri from string"))
                .connect_lazy();
        Self {
            channel: MonitorClient::new(channel),
        }
    }

    pub fn connect(
        self,
    ) -> (
        impl Stream<Item = Result<Response<Pack>, Status>>,
        AbortHandle,
    ) {
        let stream = futures::stream::try_unfold(self, |mut client| async move {
            let pack = client.monitor_all(Request::new(())).await;
            pack.map(|x| Some((x, client)))
        });

        futures::stream::abortable(stream)
    }
}

impl Deref for RpcClient {
    type Target = MonitorClientWithTransport;

    fn deref(&self) -> &Self::Target {
        &self.channel
    }
}

impl DerefMut for RpcClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.channel
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use crate::client::RpcClient;
    use futures::{pin_mut, StreamExt};

    #[tokio::test]
    async fn test_stream_should_be_aborted() {
        let client = RpcClient::new("orangepi:50525".to_owned());
        let (stream, handle) = client.connect();
        pin_mut!(stream);

        assert!(stream.next().await.is_some());
        handle.abort();
        assert!(stream.next().await.is_none());
    }
}
