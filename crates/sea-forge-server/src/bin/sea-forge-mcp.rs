#![forbid(unsafe_code)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let config = arguments
        .next()
        .ok_or("usage: sea-forge-mcp <trusted-config.json>")?;
    if arguments.next().is_some() {
        return Err("usage: sea-forge-mcp <trusted-config.json>".into());
    }
    sea_forge_server::mcp::run(std::path::Path::new(&config))
}
