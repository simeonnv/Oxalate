use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

use crate::Error;

#[derive(Deserialize, Serialize, ToSchema, Clone)]
#[schema(as = Urls::Ipv4UrlRange)]
pub struct Ipv4UrlRange {
    pub from: u32,
    pub to: u32,

    #[serde(skip_serializing, default)]
    pub index: u32,

    pub port: Option<u16>,
    pub protocol: Protocol,
}

impl Iterator for Ipv4UrlRange {
    type Item = Result<Url, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let current_ip = self.from + self.index;
        if current_ip > self.to {
            return None;
        }
        let current_ip = Ipv4Addr::from(current_ip);
        self.index += 1;

        let url = Url::parse(&format!("{}://{}", self.protocol.as_str(), current_ip));

        let mut url = match url {
            Ok(e) => e,
            Err(err) => return Some(Err(Error::InvalidProtocol(err))),
        };
        if let Err(_) = url.set_port(self.port) {
            return Some(Err(Error::SetPortError));
        }

        Some(Ok(url))
    }
}

#[derive(Deserialize, Serialize, ToSchema, Clone)]
#[schema(as = Post::Proxy::Res::Protocol)]
pub enum Protocol {
    Http,
    Https,
    Msp,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Http => "http",
            Protocol::Https => "https",
            Protocol::Msp => "msp",
        }
    }
}
