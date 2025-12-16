use oxalate_kv_db::kv_db::Error as KvError;
use thiserror::Error;

use crate::save_proxy_outputs;

#[derive(Debug, Error)]
pub enum Error {
    #[error("kv error -> {0}")]
    Kv(#[from] KvError),

    #[error("No proxy id in header")]
    NoProxyIdHeader,

    #[error("Invalid proxy header content")]
    ProxyIdContent,

    #[error("Proxy has not been seen before therefore has not been saved in internal register")]
    ProxyHasNotBeenSeenBefore,

    #[error("The proxy has no registrated proxy job!")]
    ProxyHasNoJob,

    #[error("failed to save proxy outputs -> {0}")]
    SaveProxyOutput(#[from] save_proxy_outputs::Error),

    #[error("DB error -> {0}")]
    DbError(#[from] sqlx::Error),

    #[error("file read error-> {0}")]
    FileReadError(String),
}
