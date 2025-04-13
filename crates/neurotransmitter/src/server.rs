use clap::Parser;
use epicentre_diagnostics::color_eyre::eyre::{Context, eyre};
use epicentre_diagnostics::{DiagnosticLayer, Report, tracing};
use iroh::protocol::Router;
use iroh::{Endpoint, NodeAddr, RelayMode, SecretKey};
use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::rpc::client::blobs::WrapOption;
use iroh_blobs::ticket::BlobTicket;
use iroh_blobs::util::SetTagOption;
use neurotransmitter::cli::ServerOptions;
use std::fs;
use std::path::PathBuf;

pub const PRIVATE_KEY_HOME_LOCATION: &str = ".cache/neurotransmitter/";
pub const PRIVATE_KEY_FILENAME: &str = "server_key.bin";

#[tokio::main]
async fn main() -> Result<(), Report> {
    DiagnosticLayer
        .setup()
        .wrap_err("Failed to setup the diagnostic layer")?;

    let options = ServerOptions::parse();
    let file_path = options.file.canonicalize()?;
    tracing::info!(?file_path);

    let identity_path = std::env::var("HOME").map(PathBuf::from).map(|mut path| {
        path.push(PRIVATE_KEY_HOME_LOCATION);
        let _ = fs::create_dir_all(&path);
        path.push(PRIVATE_KEY_FILENAME);
        tracing::warn!(private_key_path = ?path);
        path
    })?;

    let secret_key = if identity_path.exists() {
        let key_bytes: [u8; 32] = fs::read(&identity_path)?
            .try_into()
            .map_err(|_| eyre!("Identity file must contain exactly 32 bytes"))?;
        SecretKey::from_bytes(&key_bytes)
    } else {
        let key = SecretKey::generate(rand::rngs::OsRng);
        fs::write(&identity_path, key.to_bytes())?;
        key
    };

    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
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
        .inspect(|_| tracing::debug!("Router accept loop is ready, send SIGINT to shutdown"))
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
        .inspect(|add_outcome| tracing::debug!(?add_outcome, "Blob added"))
        .map_err(|error| eyre!("{error}"))
        .wrap_err("Failed to finish the Iroh stream")?;

    let node_id = router.endpoint().node_id();
    let node_addr = NodeAddr::from(node_id);
    tracing::debug!(?node_addr, home_relay = ?router.endpoint().home_relay());
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
