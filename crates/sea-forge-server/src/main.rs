#![forbid(unsafe_code)]

use sea_forge_server::{run, ServerConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_current_span(false)
        .with_span_list(false)
        .init();

    let root = std::env::var("SEA_FORGE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".sea-forge"));
    let config_path = root.join("server.yaml");
    let config = ServerConfig::load(&config_path).unwrap_or_else(|e| {
        tracing::warn!("config load failed ({e}), using defaults");
        ServerConfig {
            root: root.clone(),
            ..Default::default()
        }
    });

    tracing::info!(
        socket = %config.socket_path.display(),
        max_concurrent = config.max_concurrent_runs,
        "starting sea-forge-server"
    );

    run(config).await
}
