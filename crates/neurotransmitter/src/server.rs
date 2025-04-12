use clap::Parser;
use epicentre_diagnostics::color_eyre::eyre::{Context, eyre};
use epicentre_diagnostics::{DiagnosticLayer, Report, tracing};
use iroh::protocol::Router;
use iroh::{Endpoint, NodeAddr, RelayMode};
use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::rpc::client::blobs::WrapOption;
use iroh_blobs::ticket::BlobTicket;
use iroh_blobs::util::SetTagOption;
use neurotransmitter::cli::ServerOptions;

#[tokio::main]
async fn main() -> Result<(), Report> {
    DiagnosticLayer
        .setup()
        .wrap_err("Failed to setup the diagnostic layer")?;

    let options = ServerOptions::parse();
    let file_path = std::path::absolute(options.file)?;
    tracing::info!(?file_path);

    let endpoint = Endpoint::builder()
        .discovery_n0()
        .relay_mode(RelayMode::Default)
        .bind()
        .await
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to create an Iroh endpoint")?;
    let blobs = Blobs::memory().build(&endpoint);
    let router = Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs.clone())
        .spawn()
        .await
        .inspect(|_| tracing::info!("Router accept loop is ready, send SIGINT to shutdown"))
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to spawn Iroh router")?;

    let blobs_client = blobs.client();
    let in_place = true;
    let blob = blobs_client
        .add_from_path(file_path, in_place, SetTagOption::Auto, WrapOption::NoWrap)
        .await
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to add blob from path")?
        .finish()
        .await
        .inspect(|add_outcome| tracing::info!(?add_outcome, "Blob added"))
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to finish the Iroh stream")?;

    let node_id = router.endpoint().node_id();
    let node_addr = NodeAddr::from(node_id);
    tracing::info!(?node_addr, home_relay = ?router.endpoint().home_relay());
    let ticket = BlobTicket::new(node_addr, blob.hash, blob.format)
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to create Iroh blob ticket")?;

    tracing::info!(%ticket, "File hashed, blob ticket is ready");
    tracing::info!("You can now try reaching this endpoint and grab the file using this ticket");

    tokio::signal::ctrl_c().await?;
    tracing::warn!("Caught SIGINT signal, shutting down");
    router.shutdown().await.map_err(|error| eyre!("{error}"))?;

    Ok(())
}
