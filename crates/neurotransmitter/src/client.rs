use clap::Parser;
use epicentre_diagnostics::color_eyre::eyre::{Context, eyre};
use epicentre_diagnostics::{DiagnosticLayer, Report, tracing};
use iroh::{Endpoint, RelayMode};
use iroh_blobs::net_protocol::Blobs;
use neurotransmitter::cli::ClientOptions;

#[tokio::main]
async fn main() -> Result<(), Report> {
    DiagnosticLayer
        .setup()
        .wrap_err("Failed to setup the diagnostic layer")?;

    let options = ClientOptions::parse();
    let ticket = options.blob_ticket;
    tracing::info!(%ticket);

    let endpoint = Endpoint::builder()
        .discovery_n0()
        .relay_mode(RelayMode::Default)
        .bind()
        .await
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to create an Iroh endpoint")?;

    let blobs = Blobs::memory().build(&endpoint);
    let blobs_client = blobs.client();
    let _ = blobs_client
        .download(ticket.hash(), ticket.node_addr().clone())
        .await
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to download blob")?
        .finish()
        .await
        .inspect(|outcome| tracing::info!(?outcome, "Blob downloaded"))
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to download blob")?;

    let blob_bytes = blobs_client.read_to_bytes(ticket.hash()).await.unwrap();
    let blob_string = std::str::from_utf8(&blob_bytes)?;

    eprintln!("{blob_string}");

    Ok(())
}
