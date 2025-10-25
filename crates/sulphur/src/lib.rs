#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod metrics;
mod resource_monitor;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use clap::Parser;
use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
pub use metrics::*;
pub use resource_monitor::*;

pub const CLAP_STYLE: Styles = Styles::styled()
    .usage(AnsiColor::White.on_default().bold().underline())
    .header(AnsiColor::White.on_default().bold().underline())
    .literal(AnsiColor::Magenta.on_default())
    .placeholder(AnsiColor::White.on_default())
    .invalid(AnsiColor::Red.on_default().bold())
    .valid(AnsiColor::Cyan.on_default());

pub const DEFAULT_SOCKET_ADDR: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8899));

// NOTE: The refresh interval is controlled by the caller of the client CLI.
#[derive(Parser, Debug)]
#[must_use]
#[command(version, author, about, styles(CLAP_STYLE))]
pub struct ServerOptions {
    /// Which address the [`axum`] server should bind to.
    #[arg(short('a'), long, default_value_t = DEFAULT_SOCKET_ADDR)]
    pub socket_addr: SocketAddr,

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
