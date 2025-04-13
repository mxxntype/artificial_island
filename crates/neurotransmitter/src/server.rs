use clap::Parser;
use epicentre_diagnostics::color_eyre::eyre::{Context, eyre};
use epicentre_diagnostics::{DiagnosticLayer, Report, tracing};
use iroh::protocol::Router;
use iroh::{Endpoint, NodeAddr, RelayMode, SecretKey};
use iroh_blobs::net_protocol::Blobs;
use iroh_docs::protocol::Docs;
use iroh_gossip::net::Gossip;
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
