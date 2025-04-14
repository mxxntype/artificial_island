use clap::Parser;
use epicentre_diagnostics::color_eyre::eyre::{Context, eyre};
use epicentre_diagnostics::{DiagnosticLayer, Report, tracing};
use iroh::protocol::Router;
use iroh::{Endpoint, NodeAddr, RelayMode};
use iroh_blobs::net_protocol::Blobs;
use iroh_docs::protocol::Docs;
use iroh_gossip::net::Gossip;
use neurotransmitter::Identity;
use neurotransmitter::cli::ServerOptions;

#[tokio::main]
async fn main() -> Result<(), Report> {
    DiagnosticLayer
        .setup()
        .wrap_err("Failed to setup the diagnostic layer")?;

    let options = ServerOptions::parse();
    let file_path = options.file.canonicalize()?;
    tracing::info!(?file_path);

    let secret_key = Identity.from_cache_or_generate_new()?;
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .discovery_n0()
        .relay_mode(RelayMode::Default)
        .bind()
        .await
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to create an Iroh endpoint")?;

    let blobs = Blobs::memory().build(&endpoint);
    let gossip = Gossip::builder()
        .spawn(endpoint.clone())
        .await
        .map_err(|error| eyre!("{error}"))?;
    let docs = Docs::memory()
        .spawn(&blobs, &gossip)
        .await
        .map_err(|error| eyre!("{error}"))?;

    let router = Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs)
        .accept(iroh_gossip::ALPN, gossip)
        .accept(iroh_docs::ALPN, docs.clone())
        .spawn()
        .await
        .inspect(|_| tracing::debug!("Router accept loop is ready, send SIGINT to shutdown"))
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to spawn Iroh router")?;

    // let docs_client = docs.client();

    let node_id = router.endpoint().node_id();
    let node_addr = NodeAddr::from(node_id);
    tracing::debug!(?node_addr, home_relay = ?router.endpoint().home_relay());

    tokio::signal::ctrl_c().await?;
    tracing::warn!("Caught SIGINT signal, shutting down");
    router.shutdown().await.map_err(|error| eyre!("{error}"))?;

    Ok(())
}
