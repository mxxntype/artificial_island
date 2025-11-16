#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod metrics;
mod resource_monitor;
pub mod server;
pub mod units;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

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

pub const DEFAULT_API_ADDRESS: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8899));

pub const METRICS_ENDPOINT: &str = "/metrics";
