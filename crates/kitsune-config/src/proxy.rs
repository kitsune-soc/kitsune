use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::net::SocketAddr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Authentication {
    pub username: SmolStr,
    pub password: SmolStr,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProxyType {
    Socks5,
    Tor,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConnectionConfiguration {
    pub r#type: ProxyType,
    pub addr: SocketAddr,
    pub auth: Option<Authentication>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub incoming: Option<ConnectionConfiguration>,
    pub outgoing: Option<ConnectionConfiguration>,
}
