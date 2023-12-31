use std::{collections::HashMap, time::Instant};

use sysinfo::{ComponentExt, CpuExt, CpuRefreshKind, NetworkData, NetworkExt, System, SystemExt};
use tonic::async_trait;

#[async_trait]
pub trait AsyncNew: Sized {
    async fn new(system: &mut System) -> std::io::Result<Self>;
}

#[derive(Debug)]
pub struct CpuLoad {
    pub user: f32,
    pub nice: f32,
    pub system: f32,
    pub idle: f32,
}

#[derive(Debug)]
pub struct Cpu {
    pub time: Instant,
    pub freq: Vec<u64>,
    pub temperature: Option<f32>,
    pub usage: Vec<f32>,
}

#[derive(Debug)]
pub struct Network {
    pub names: Vec<String>,
    pub bytes_in: HashMap<String, u64>,
    pub bytes_out: HashMap<String, u64>,
}

#[derive(Debug)]
pub struct MonitoringData<T> {
    data: T,
}

#[async_trait]
impl AsyncNew for Cpu {
    async fn new(system: &mut System) -> std::io::Result<Self> {
        Cpu::new(system)
    }
}

#[async_trait]
impl AsyncNew for Network {
    async fn new(system: &mut System) -> std::io::Result<Self> {
        Network::new(system)
    }
}

#[async_trait]
impl<T: AsyncNew + Send, V: AsyncNew + Send> AsyncNew for (T, V) {
    async fn new(system: &mut System) -> std::io::Result<Self> {
        Ok((T::new(system).await?, V::new(system).await?))
    }
}

impl Cpu {
    fn read_freq(system: &System) -> std::io::Result<Vec<u64>> {
        Ok(system.cpus().iter().map(|c| c.frequency()).collect())
    }

    fn read_loads(system: &System) -> std::io::Result<Vec<f32>> {
        let cpus = system.cpus();
        Ok(cpus.iter().map(|c| c.cpu_usage()).collect())
    }

    fn read_temp(system: &System) -> std::io::Result<f32> {
        let temps = system
            .components()
            .iter()
            .map(|c| c.temperature())
            .collect::<Vec<_>>();

        let temps_len = temps.len();
        Ok(temps.into_iter().sum::<f32>() / temps_len as f32)
    }

    fn new(system: &mut System) -> std::io::Result<Self> {
        system.refresh_all();
        system.refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage());

        Ok(Self {
            time: Instant::now(),
            freq: Cpu::read_freq(system).unwrap_or_default(),
            temperature: Cpu::read_temp(system).ok(),
            usage: Cpu::read_loads(system).unwrap_or_default(),
        })
    }
}

impl Network {
    fn read_networks_stats<F: Fn(&NetworkData) -> T, T>(
        system: &System,
        f: F,
    ) -> std::io::Result<HashMap<String, T>> {
        let networks = system.networks();

        Ok(networks
            .into_iter()
            .map(|x| (x.0.clone(), f(x.1)))
            .collect())
    }

    fn read_in(system: &System) -> std::io::Result<HashMap<String, u64>> {
        Network::read_networks_stats(system, |x| x.received())
    }

    fn read_out(system: &System) -> std::io::Result<HashMap<String, u64>> {
        Network::read_networks_stats(system, |x| x.transmitted())
    }

    fn new(system: &mut System) -> std::io::Result<Self> {
        system.refresh_all();

        Ok(Self {
            names: system.networks().into_iter().map(|x| x.0.clone()).collect(),
            bytes_in: Network::read_in(system)?,
            bytes_out: Network::read_out(system)?,
        })
    }
}

impl<T> MonitoringData<T> {
    pub async fn new(system: &mut System) -> std::io::Result<Self>
    where
        T: AsyncNew,
    {
        Ok(MonitoringData {
            data: T::new(system).await?,
        })
    }

    pub fn into_data(self) -> T {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use sysinfo::{CpuRefreshKind, RefreshKind, System, SystemExt};

    use crate::collect_info::Cpu;

    use super::{MonitoringData, Network};

    #[tokio::test]
    async fn test_cpu_works() {
        let mut system = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_components(),
        );
        let data = MonitoringData::<Cpu>::new(&mut system).await;

        assert!(data.is_ok());
        let data = data.unwrap().data;
        assert!(data.temperature.is_some());
        assert!(data.freq.into_iter().all(|x| x > 0));
    }

    #[tokio::test]
    async fn test_network_works() {
        let mut system = System::new_with_specifics(RefreshKind::new().with_networks());
        let data = MonitoringData::<Network>::new(&mut system).await;

        assert!(data.is_ok());
    }
}
