mod metrics;
mod resource_monitor;

use std::net::SocketAddr;

use clap::Parser;
pub use metrics::*;
pub use resource_monitor::*;

// NOTE: The refresh interval is controlled by the caller of the client CLI.
#[derive(Parser, Debug)]
#[must_use]
pub struct ServerOptions {
    #[arg(long)]
    pub listen_addr: SocketAddr,

    /// The length of the unicode graph printed by this program, in characters.
    ///
    /// Note that the graph is printed using braille symbols, which means
    /// two distinct "measurements" can fit inside a single character.
    #[arg(short('l'), long, default_value_t = 5)]
    pub graph_length: u8,

    // TODO(@mxxntype): Figure out a decent docstring for this.
    #[arg(short('s'), long, default_value_t = 5)]
    pub span_seconds: u8,
}
