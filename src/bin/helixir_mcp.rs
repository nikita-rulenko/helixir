

use helixir::mcp::run_server;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    
    
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(
            EnvFilter::from_default_env()
                .add_directive("helixir=warn".parse()?)
                .add_directive("helixir::mcp=info".parse()?) 
        )
        .init();

    run_server().await
}

