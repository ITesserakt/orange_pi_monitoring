use std::sync::Arc;
use sysinfo::System;
use tokio::sync::Mutex;
use tokio::try_join;
use tonic::{async_trait, Request, Response, Status};

use crate::collect_info::{Cpu, MonitoringData, Network};

use common::monitoring::{
    monitor_server::Monitor, CpuResponse, NetworkInterface, NetworkResponse, Pack,
};

pub struct MonitorService {
    system: Arc<Mutex<System>>,
}

#[async_trait]
impl Monitor for MonitorService {
    async fn monitor_cpu(&self, _request: Request<()>) -> Result<Response<CpuResponse>, Status> {
        let mut system = self.system.clone().lock_owned().await;
        let data = MonitoringData::<Cpu>::new(&mut system).await;

        match data.map(|x| x.into_data()) {
            Ok(data) => Ok(Response::new(CpuResponse {
                usage: data.usage,
                temperature: data.temperature,
            })),
            Err(e) => Err(Status::from_error(Box::new(e))),
        }
    }

    async fn monitor_network(
        &self,
        _request: Request<()>,
    ) -> Result<Response<NetworkResponse>, Status> {
        let mut system = self.system.clone().lock_owned().await;
        let data = MonitoringData::<Network>::new(&mut system).await;

        match data.map(|x| x.into_data()) {
            Ok(data) => Ok(Response::new(NetworkResponse {
                interfaces: data
                    .names
                    .into_iter()
                    .map(|x| NetworkInterface {
                        name: x.clone(),
                        bytes_in: data.bytes_in.get(x.as_str()).copied().unwrap_or_default(),
                        bytes_out: data.bytes_out.get(x.as_str()).copied().unwrap_or_default(),
                    })
                    .collect(),
            })),
            Err(e) => Err(Status::from_error(Box::new(e))),
        }
    }

    async fn monitor_all(&self, request: Request<()>) -> Result<Response<Pack>, Status> {
        try_join!(
            self.monitor_cpu(Request::new(())),
            self.monitor_network(request)
        )
        .map(|x| {
            Response::new(Pack {
                network: Some(x.1.into_inner()),
                cpu: Some(x.0.into_inner()),
            })
        })
    }
}

impl MonitorService {
    pub fn new(system: System) -> Self {
        MonitorService {
            system: Arc::new(Mutex::new(system)),
        }
    }
}
