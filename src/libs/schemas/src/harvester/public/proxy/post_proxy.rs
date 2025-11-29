use std::net::{Ipv4Addr, Ipv6Addr};

use oxalate_scrapper_controller::scrapper_controller::ProxyOutput;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = Post::Proxy::Req)]
pub enum Req {
    RequestUrls,
    ReturnUrlOutputs(Vec<ProxyOutput>),
}

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = Post::Proxy::Res)]
pub struct Res(pub Option<Urls>);

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = Post::Proxy::Res::ResBody)]
pub enum Urls {
    Urls(Vec<Url>),
    UrlIpRange(Ipv4UrlRange),
}

impl IntoIterator for Urls {
    type Item = Url;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Urls::Urls(urls) => Box::new(urls.into_iter()),
            Urls::UrlIpRange(ip_range) => Box::new(ip_range.into_iter().filter_map(|url| url.ok())),
        }
    }
}
#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = Post::Proxy::Res::IpRange)]
pub struct Ipv4UrlRange {
    pub from: u32,
    pub to: u32,

    #[serde(skip_serializing, default)]
    pub index: u32,

    pub port: Option<u16>,
    pub protocol: Protocol,
}

impl Iterator for Ipv4UrlRange {
    type Item = Result<Url, String>;

    fn next(&mut self) -> Option<Self::Item> {
        let current_ip = self.from + self.index;
        if current_ip > self.to {
            return None;
        }
        let current_ip = Ipv4Addr::from(current_ip);
        self.index += 1;

        let url = Url::parse(&format!("{}://{}", current_ip, self.protocol.as_str()))
            .map_err(|err| err.to_string());
        let mut url = match url {
            Ok(e) => e,
            Err(err) => return Some(Err(err)),
        };
        let _ = url.set_port(self.port);

        Some(Ok(url))
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = Post::Proxy::Res::Protocol)]
pub enum Protocol {
    Http,
    Https,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Http => "http",
            Protocol::Https => "https",
        }
    }
}
