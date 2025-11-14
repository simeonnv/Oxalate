use crate::env::ENVVARS;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sled::Db;
use thiserror::Error;

pub struct KvDb {
    pub db: Db,
}

lazy_static! {
    pub static ref DB: KvDb = KvDb::new();
}

impl KvDb {
    pub fn new() -> Self {
        let db = sled::open(&ENVVARS.harvester_kv_db_path).unwrap();
        Self { db }
    }

    pub fn insert<T: AsRef<[u8]>, S: Serialize>(&self, key: &T, value: &S) -> Result<(), Error> {
        let json =
            serde_json::to_vec(value).map_err(|err| Error::JsonSerializeError(err.to_string()))?;

        self.db
            .insert(key, json)
            .map_err(|err| Error::FailedToInsertInKv(err.to_string()))?;

        Ok(())
    }

    pub fn get<T: AsRef<[u8]>, S: for<'a> Deserialize<'a>>(
        &self,
        key: &T,
    ) -> Result<Option<S>, Error> {
        let val = self
            .db
            .get(key)
            .map_err(|err| Error::FailedToGetFromKv(err.to_string()))?
            .map(|e| {
                serde_json::from_slice(&e)
                    .map_err(|err| Error::JsonDeserializeError(err.to_string()))
            })
            .transpose()?;

        Ok(val)
    }

    pub fn flush(&self) -> Result<(), Error> {
        self.db
            .flush()
            .map_err(|e| Error::FailedToSaveToDisk(e.to_string()))?;
        Ok(())
    }
}

impl Default for KvDb {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to serialize value to json for kv -> {0}")]
    JsonSerializeError(String),

    #[error("failed to deserialize value to json for kv -> {0}")]
    JsonDeserializeError(String),

    #[error("failed to insert in kv -> {0}")]
    FailedToInsertInKv(String),

    #[error("failed to get from kv -> {0}")]
    FailedToGetFromKv(String),

    #[error("failed to save kv to disk -> {0}")]
    FailedToSaveToDisk(String),
}
