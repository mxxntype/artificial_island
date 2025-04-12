use epicentre_diagnostics::color_eyre::eyre::eyre;
use epicentre_diagnostics::{DiagnosticLayer, Report, tracing};
use iroh::Endpoint;
use iroh::protocol::Router;
use iroh_blobs::net_protocol::Blobs;

#[tokio::main]
async fn main() -> Result<(), Report> {
    DiagnosticLayer.setup()?;

    let endpoint = Endpoint::builder()
        .discovery_n0()
        .bind()
        .await
        .map_err(|error| eyre!("{error}"))?;
    let blobs = Blobs::memory().build(&endpoint);
    let router = Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs)
        .spawn()
        .await
        .map_err(|error| eyre!("{error}"))?;

    tracing::info!("Router accept loop is ready, send Ctrl+C to shutdown");

    tokio::signal::ctrl_c().await?;
    tracing::warn!("Caught Ctrl+C signal, shutting down");
    router.shutdown().await.map_err(|error| eyre!("{error}"))?;

    Ok(())
}
