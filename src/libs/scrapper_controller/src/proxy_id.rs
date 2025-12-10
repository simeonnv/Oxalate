use std::{fmt::Display, ops::Deref};

use http::HeaderMap;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::Error;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProxyId(String);

impl Display for ProxyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub const HEADER_KEY: &str = "machine-id";

impl ProxyId {
    pub async fn from_http_headers(
        headers: &HeaderMap,
        db_pool: &Pool<Postgres>,
    ) -> Result<Self, Error> {
        let id = headers.get(HEADER_KEY).ok_or(Error::NoProxyIdHeader)?;
        let id = id.to_str().map_err(|_| Error::ProxyIdContent)?.to_owned();

        let device_exists = sqlx::query_scalar!(
            "
             SELECT EXISTS (
                SELECT 1 
                FROM Devices 
                WHERE machine_id = $1
            ) AS row_exists;    
        ",
            &id
        )
        .fetch_one(db_pool)
        .await?
        .unwrap_or(false);

        if device_exists {
            return Ok(Self(id));
        }

        sqlx::query!(
            "
            INSERT INTO Devices
                (machine_id)
            VALUES
                ($1)
            ;
        ",
            &id
        )
        .execute(db_pool)
        .await?;

        Ok(Self(id))
    }

    pub unsafe fn from_raw(inner: String) -> Self {
        Self(inner)
    }
}

impl AsRef<str> for ProxyId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for ProxyId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
