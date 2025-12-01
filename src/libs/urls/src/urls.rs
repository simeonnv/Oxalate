use log::error;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

use crate::Ipv4UrlRange;

#[derive(Deserialize, Serialize, ToSchema, Clone)]
#[schema(as = Urls::Urls)]
pub enum Urls {
    Ipv4UrlRange(Ipv4UrlRange),
    UrlCollection(Vec<Url>),
}

impl IntoIterator for Urls {
    type Item = Url;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Urls::UrlCollection(urls) => Box::new(urls.into_iter()),
            Urls::Ipv4UrlRange(ip_range) => {
                Box::new(ip_range.into_iter().filter_map(|url| match url {
                    Ok(e) => Some(e),
                    Err(err) => {
                        error!("error at Urls iteration with state ipv4UrlRange: {err}");
                        None
                    }
                }))
            }
        }
    }
}
