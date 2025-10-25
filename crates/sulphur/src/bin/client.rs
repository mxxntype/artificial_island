use std::net::SocketAddr;

use clap::Parser;
use epicentre_diagnostics::{DiagnosticLayer, Report};
use sulphur::{CLAP_STYLE, DEFAULT_SOCKET_ADDR, Metrics};

fn main() -> Result<(), Report> {
    DiagnosticLayer.setup()?;

    let options = ClientOptions::parse();
    let uri = format!("http://{}/metrics", options.api_address);
    let metrics = reqwest::blocking::get(uri)?.json::<Metrics>()?;
    let graph = metrics.render_cpu_usage_graph()?;
    println!("{graph}");

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, author, about, styles(CLAP_STYLE))]
struct ClientOptions {
    /// The address at which the server is configured to listen.
    #[arg(long, default_value_t = DEFAULT_SOCKET_ADDR)]
    api_address: SocketAddr,
}
