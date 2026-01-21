use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sled::Db;

use exn::{Result, ResultExt};

#[derive(Clone)]
pub struct KvDb(Db);

impl KvDb {
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let db = sled::open(path).or_raise(|| Error::FailedToInit)?;
        Ok(Self(db))
    }

    pub fn insert<T: AsRef<[u8]>, S: Serialize>(&self, key: &T, value: &S) -> Result<(), Error> {
        let json = serde_json::to_vec(value).or_raise(|| Error::JsonSerializeError)?;

        self.0
            .insert(key, json)
            .or_raise(|| Error::FailedToInsertInKv)?;

        Ok(())
    }

    pub fn get<T: AsRef<[u8]>, S: for<'a> Deserialize<'a>>(
        &self,
        key: &T,
    ) -> Result<Option<S>, Error> {
        let val = self
            .0
            .get(key)
            .or_raise(|| Error::FailedToGetFromKv)?
            .map(|e| serde_json::from_slice(&e).or_raise(|| Error::JsonDeserializeError))
            .transpose()?;

        Ok(val)
    }

    pub fn flush(&self) -> Result<(), Error> {
        self.0.flush().or_raise(|| Error::FailedToSaveToDisk)?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to serialize value to json for kv")]
    JsonSerializeError,

    #[error("failed to deserialize value to json for kv")]
    JsonDeserializeError,

    #[error("failed to insert in kv")]
    FailedToInsertInKv,

    #[error("failed to get from kv")]
    FailedToGetFromKv,

    #[error("failed to save kv to disk")]
    FailedToSaveToDisk,

    #[error("failed to init kv db")]
    FailedToInit,
}
