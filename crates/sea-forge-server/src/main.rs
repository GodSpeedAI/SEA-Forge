#![forbid(unsafe_code)]

use sea_forge_server::{run, ServerConfig};

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

    // One cell contract: the resolved root owns the records *and* the socket.
    // `SEA_FORGE_ROOT` relocates both together; `SEA_FORGE_SOCKET` is the only
    // way to split them, and only on purpose.
    let root = sea_forge_server::resolve_cell_root();
    let config_path = root.join("server.yaml");
    // Fail closed (§8.3 `server_config_error`). An absent `server.yaml` is a
    // first run and yields defaults; a file that *exists* and does not parse or
    // validate blocks the start. Falling back to defaults there would silently
    // run the cell under a configuration the operator never wrote — the wrong
    // agent endpoint, the wrong concurrency, no notify hook — while every log
    // line claimed a healthy server.
    let mut config = ServerConfig::load(&config_path).map_err(|message| {
        tracing::error!(
            config_path = %config_path.display(),
            error_class = "server_config_error",
            "{message}"
        );
        message
    })?;
    // The resolved root always wins over a `root:` key in the file: the file
    // lives *inside* the cell, so it cannot name a different one.
    config.root = root;
    if let Some(socket_override) = sea_forge_server::resolve_socket_override() {
        config.socket_path = socket_override;
    }

    tracing::info!(
        root = %config.root.display(),
        socket = %config.resolved_socket_path().display(),
        max_concurrent = config.max_concurrent_runs,
        "starting sea-forge-server"
    );

    run(config).await
}
