use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sled::Db;
use thiserror::Error;

#[derive(Clone)]
pub struct KvDb(Db);

impl KvDb {
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let db = sled::open(path).map_err(|err| Error::FailedToInit(err.to_string()))?;
        Ok(Self(db))
    }

    pub fn insert<T: AsRef<[u8]>, S: Serialize>(&self, key: &T, value: &S) -> Result<(), Error> {
        let json =
            serde_json::to_vec(value).map_err(|err| Error::JsonSerializeError(err.to_string()))?;

        self.0
            .insert(key, json)
            .map_err(|err| Error::FailedToInsertInKv(err.to_string()))?;

        Ok(())
    }

    pub fn get<T: AsRef<[u8]>, S: for<'a> Deserialize<'a>>(
        &self,
        key: &T,
    ) -> Result<Option<S>, Error> {
        let val = self
            .0
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
        self.0
            .flush()
            .map_err(|e| Error::FailedToSaveToDisk(e.to_string()))?;
        Ok(())
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

    #[error("failed to init kv db -> {0}")]
    FailedToInit(String),
}
