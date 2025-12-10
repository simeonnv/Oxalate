use std::{array::IntoIter, collections::HashMap, ops::Deref};

use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
#[schema(as = Urls::Urls)]
pub struct ProxyReqs(pub Vec<ProxyReq>);

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub enum ProxyReq {
    Msp(MspContent),
    Http(HttpBasedContent),
    Https(HttpBasedContent),
    Tcp(TcpContent),
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct HttpBasedContent {
    pub url: Url,
    pub headers: HashMap<String, String>,
    pub body: Option<Box<[u8]>>,
    pub method: HttpMethod,
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct MspContent {
    pub url: Url,
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct TcpContent {
    pub url: Url,
    pub sequence_of_requests: Box<[Box<[u8]>]>,
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Head,
    Put,
    Patch,
    Delete,
    Connect,
    Options,
    Trace,
}
