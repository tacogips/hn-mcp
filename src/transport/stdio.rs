use crate::tools::hn::client::HnClient;
use crate::tools::hn::HnRouter;
use anyhow::Result;
use rmcp::transport::stdio;
use rmcp::ServiceExt;

pub async fn run_stdio_server() -> Result<()> {
    // Create an instance of our search router with the API key
    let service = HnRouter::new(HnClient::new());

    // Use the rust-sdk stdio transport implementation
    let server = service.serve(stdio()).await?;

    // Wait for the server to complete
    server.waiting().await?;

    Ok(())
}
