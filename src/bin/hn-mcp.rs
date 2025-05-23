use anyhow::Result;
use clap::{Parser, Subcommand};
use hn_mcp::tools::{hn::client::HnClient, HnRouter};
use std::net::SocketAddr;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(author, version = "0.1.0", about = "HN MCP Server", long_about = None)]
#[command(propagate_version = true)]
#[command(disable_version_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the server in stdin/stdout mode
    Stdio {
        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,
    },
    /// Run the server with HTTP/SSE interface
    Http {
        /// Address to bind the HTTP server to
        #[arg(short, long, default_value = "0.0.0.0:3000")]
        address: String,

        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Stdio { debug } => run_stdio_server(debug).await,
        Commands::Http { address, debug } => run_http_server(address, debug).await,
    }
}

async fn run_stdio_server(debug: bool) -> Result<()> {
    // Initialize the tracing subscriber with stderr logging
    let level = if debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(level.into()))
        .with_writer(std::io::stderr) // Explicitly use stderr for logging
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false) // Disable ANSI color codes
        .init();

    tracing::info!("Starting HN MCP server in STDIN/STDOUT mode");

    // Run the server using the implementation
    hn_mcp::transport::stdio::run_stdio_server()
        .await
        .map_err(|e| anyhow::anyhow!("Error running STDIO server: {}", e))
}

async fn run_http_server(address: String, debug: bool) -> Result<()> {
    // Setup tracing
    let level = if debug { "debug" } else { "info" };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{},{}", level, env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(false)) // Disable ANSI color codes
        .init();

    // Parse socket address
    let addr: SocketAddr = address.parse()?;

    tracing::debug!("HN MCP Server listening on {}", addr);
    tracing::info!("Access the HN MCP Server at http://{}/sse", addr);

    // Create and run server
    let service = HnRouter::new(HnClient::new());
    let server = hn_mcp::transport::sse_server::serve(service, addr.port())
        .await
        .map_err(|e| anyhow::anyhow!("Error starting SSE server: {}", e))?;

    // Wait for server to complete
    let _ = server.await?;

    Ok(())
}
