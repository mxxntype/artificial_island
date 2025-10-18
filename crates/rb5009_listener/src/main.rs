mod interface_response;

use std::fmt;

use base64::prelude::*;
use epicentre_diagnostics::DiagnosticLayer;
use epicentre_diagnostics::color_eyre::eyre::{Context, Report};

fn main() -> Result<(), Report> {
    DiagnosticLayer.setup()?;

    let routeros_host = "192.168.0.1";
    let rest_api_http_url = format!("http://{routeros_host}/rest/interface");

    let credentials =
        Credentials::from_env().context("Failed to read credentials from the environment")?;
    let basic_auth = format!("Basic {}", BASE64_STANDARD.encode(credentials.to_string()));

    let response = ureq::get(rest_api_http_url)
        .header(axum::http::header::AUTHORIZATION, basic_auth)
        .call()
        .context("Failed to query the RouterOS REST API")?
        .into_body()
        .read_json::<Vec<interface_response::InterfaceResponse>>()?;

    dbg!(response);

    Ok(())
}

#[derive(Debug)]
struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub const VAR_USERNAME: &'static str = "ROS_REST_API_USER";
    pub const VAR_PASSWORD: &'static str = "ROS_REST_API_PASS";

    pub const fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn from_env() -> Result<Self, std::env::VarError> {
        let username = std::env::var(Self::VAR_USERNAME)?;
        let password = std::env::var(Self::VAR_PASSWORD)?;
        Ok(Self::new(username, password))
    }
}

impl fmt::Display for Credentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { username, password } = self;
        write!(formatter, "{username}:{password}")
    }
}
