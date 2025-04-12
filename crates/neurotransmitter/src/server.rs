use epicentre_diagnostics::color_eyre::eyre::{Context, eyre};
use epicentre_diagnostics::{DiagnosticLayer, Report, tracing};
use iroh::Endpoint;
use iroh::protocol::Router;
use iroh_blobs::net_protocol::Blobs;

#[tokio::main]
async fn main() -> Result<(), Report> {
    DiagnosticLayer
        .setup()
        .wrap_err("Failed to setup the diagnostic layer")?;

    let endpoint = Endpoint::builder()
        .discovery_n0()
        .bind()
        .await
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to create an Iroh endpoint")?;
    let blobs = Blobs::memory().build(&endpoint);
    let router = Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs)
        .spawn()
        .await
        .inspect(|_| tracing::info!("Router accept loop is ready, send SIGINT to shutdown"))
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to spawn Iroh router")?;

    tokio::signal::ctrl_c().await?;
    tracing::warn!("Caught SIGINT signal, shutting down");
    router.shutdown().await.map_err(|error| eyre!("{error}"))?;

    Ok(())
}
