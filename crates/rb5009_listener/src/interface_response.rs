use eui48::MacAddress;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[expect(dead_code)]
#[serde(try_from = "RawInterfaceResponse")]
pub struct InterfaceResponse {
    pub r#type: String,
    pub mac_address: Option<MacAddress>,

    pub running: Option<bool>,
    pub disabled: Option<bool>,
    pub slave: Option<bool>,

    pub name: String,
    pub default_name: Option<String>,

    pub mtu: Option<String>,
    pub actual_mtu: Option<String>,
    pub l2mtu: Option<String>,
    pub max_l2mtu: Option<String>,

    pub link_downs: Option<String>,
    pub last_link_down_time: Option<String>,
    pub last_link_up_time: Option<String>,

    pub rx_byte: usize,
    pub rx_packet: usize,
    pub fp_rx_byte: usize,
    pub fp_rx_packet: usize,

    pub tx_byte: usize,
    pub tx_packet: usize,
    pub tx_queue_drop: usize,
    pub fp_tx_byte: usize,
    pub fp_tx_packet: usize,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RawInterfaceResponse {
    pub r#type: Option<String>,
    pub mac_address: Option<String>,

    pub running: Option<String>,
    pub disabled: Option<String>,
    pub slave: Option<String>,

    pub name: Option<String>,
    pub default_name: Option<String>,

    pub mtu: Option<String>,
    pub actual_mtu: Option<String>,
    pub l2mtu: Option<String>,
    pub max_l2mtu: Option<String>,

    pub link_downs: Option<String>,
    pub last_link_down_time: Option<String>,
    pub last_link_up_time: Option<String>,

    pub rx_byte: Option<String>,
    pub rx_packet: Option<String>,
    pub fp_rx_byte: Option<String>,
    pub fp_rx_packet: Option<String>,

    pub tx_byte: Option<String>,
    pub tx_packet: Option<String>,
    pub tx_queue_drop: Option<String>,
    pub fp_tx_byte: Option<String>,
    pub fp_tx_packet: Option<String>,
}

impl TryFrom<RawInterfaceResponse> for InterfaceResponse {
    type Error = self::ParseError;

    fn try_from(raw: RawInterfaceResponse) -> Result<Self, Self::Error> {
        let interface_response = Self {
            r#type: raw.r#type.unwrap(),
            mac_address: raw.mac_address.map(|mac_str| mac_str.parse()).transpose()?,
            running: raw.running.map(|str| str.parse()).transpose()?,
            disabled: raw.disabled.map(|str| str.parse()).transpose()?,
            slave: raw.slave.map(|str| str.parse()).transpose()?,

            name: raw.name.unwrap(),
            default_name: raw.default_name,

            mtu: raw.mtu,
            actual_mtu: raw.actual_mtu,
            l2mtu: raw.l2mtu,
            max_l2mtu: raw.max_l2mtu,

            link_downs: raw.link_downs,
            last_link_down_time: raw.last_link_down_time,
            last_link_up_time: raw.last_link_up_time,

            rx_byte: raw
                .rx_byte
                .map(|str| str.parse())
                .transpose()?
                .unwrap_or_default(),
            rx_packet: raw
                .rx_packet
                .map(|str| str.parse())
                .transpose()?
                .unwrap_or_default(),
            fp_rx_byte: raw
                .fp_rx_byte
                .map(|str| str.parse())
                .transpose()?
                .unwrap_or_default(),
            fp_rx_packet: raw
                .fp_rx_packet
                .map(|str| str.parse())
                .transpose()?
                .unwrap_or_default(),
            tx_byte: raw
                .tx_byte
                .map(|str| str.parse())
                .transpose()?
                .unwrap_or_default(),
            tx_packet: raw
                .tx_packet
                .map(|str| str.parse())
                .transpose()?
                .unwrap_or_default(),
            tx_queue_drop: raw
                .tx_queue_drop
                .map(|str| str.parse())
                .transpose()?
                .unwrap_or_default(),
            fp_tx_byte: raw
                .fp_tx_byte
                .map(|str| str.parse())
                .transpose()?
                .unwrap_or_default(),
            fp_tx_packet: raw
                .fp_tx_packet
                .map(|str| str.parse())
                .transpose()?
                .unwrap_or_default(),
        };

        Ok(interface_response)
    }
}

#[derive(thiserror::Error, Debug)]
#[expect(clippy::enum_variant_names)]
pub enum ParseError {
    #[error("Failed to parse MAC address")]
    MalformedMac(#[from] eui48::ParseError),
    #[error("Failed to parse a boolean field")]
    MalformedBool(#[from] std::str::ParseBoolError),
    #[error("Failed to parse an integer field")]
    MalformedInt(#[from] std::num::ParseIntError),
}
