#![deny(clippy::correctness)]
#![deny(clippy::suspicious)]
#![warn(clippy::complexity)]
#![warn(clippy::style)]
#![warn(clippy::perf)]
#![deny(clippy::unwrap_used)]
#![forbid(unsafe_code)]

mod collect_info;
mod cpu_service;

extern crate clap;

use std::{
    error::Error, io::stdin, net::SocketAddr, process::exit, str::FromStr, sync::Arc,
    time::Duration,
};

use clap::{arg, Parser};
use common::monitoring::monitor_server::MonitorServer;
use fslock::LockFile;
use sysinfo::{
    CpuRefreshKind, Pid, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt,
};
use tokio::sync::Mutex;
use tonic::{codegen::http::HeaderName, transport::Server};
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::cpu_service::MonitorService;

const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_EXPOSED_HEADERS: [&str; 4] = [
    "grpc-status",
    "grpc-message",
    "grpc-status-details-bin",
    "grpc-encoding",
];
const DEFAULT_ALLOW_HEADERS: [&str; 5] = [
    "x-grpc-web",
    "content-type",
    "x-user-agent",
    "grpc-timeout",
    "grpc-accept-encoding",
];

#[derive(Parser)]
struct ServerCli {
    #[arg(short = 'a', long = "address", default_value = "127.0.0.1")]
    address: String,
    #[arg(short = 'p', long = "port", default_value = "50501")]
    port: u16,
    #[arg(short = 'u', long = "update", default_value = "100")]
    update_every_ms: u64,
    #[arg(short = 'l', long = "lock", default_value = ".service.lock")]
    lock_file: String,
}

async fn launch(system: System, cli: ServerCli) -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = format!("{}:{}", cli.address, cli.port).parse()?;

    let service = MonitorService::new(system);

    println!("Listening server on {addr}");

    Server::builder()
        .accept_http1(true)
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::mirror_request())
                .allow_credentials(true)
                .allow_headers(
                    DEFAULT_ALLOW_HEADERS
                        .iter()
                        .copied()
                        .map(HeaderName::from_static)
                        .collect::<Vec<_>>(),
                )
                .max_age(DEFAULT_MAX_AGE)
                .expose_headers(
                    DEFAULT_EXPOSED_HEADERS
                        .iter()
                        .copied()
                        .map(HeaderName::from_static)
                        .collect::<Vec<_>>(),
                ),
        )
        .layer(GrpcWebLayer::new())
        .add_service(MonitorServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

fn disable_lock(cli: &ServerCli, system: &System) -> Result<bool, Box<dyn Error>> {
    eprintln!("Service already running. Kill it? [Y/n]");
    let mut choise = String::new();
    stdin().read_line(&mut choise)?;
    if choise.trim().to_lowercase().as_str() == "n" {
        return Ok(false);
    }
    let pid = std::fs::read_to_string(&cli.lock_file)?;

    Ok(system
        .process(Pid::from_str(pid.trim())?)
        .expect("Process already killed")
        .kill())
}

fn register_on_kill(lock: Arc<Mutex<LockFile>>) {
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to receive an event from system");
        lock.lock().await.unlock().expect("Cannot unlock service");
        exit(0);
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = ServerCli::parse();
    let system = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_networks()
            .with_networks_list()
            .with_processes(ProcessRefreshKind::new())
            .with_components()
            .with_components_list(),
    );
    let lock = Arc::new(Mutex::new(LockFile::open(&cli.lock_file)?));
    register_on_kill(lock.clone());

    if lock.lock().await.try_lock_with_pid()? {
        launch(system, cli).await?;
    } else if disable_lock(&cli, &system)? {
        lock.lock().await.lock_with_pid()?;
        launch(system, cli).await?;
    }

    Ok(())
}
