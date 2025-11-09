use std::net::Ipv4Addr;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicU16, AtomicU32};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::kv_db::DB;
use crate::kv_db::Error as KvError;

const SCRAPPER_STATE_KEY: &[u8; 14] = b"scrapper state";

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub enum Stage {
    #[default]
    GlobalScan,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ScrapperState {
    pub enabled: bool,
    pub current_stage: Stage,
}

impl ScrapperState {
    pub fn load() -> Result<Self, Error> {
        let scrapper_state = DB.get(SCRAPPER_STATE_KEY)?;
        let scrapper_state = match scrapper_state {
            Some(e) => e,
            None => {
                let scrapper_state = Self::default();
                Self::save_state(&scrapper_state)?;
                scrapper_state
            }
        };
        Ok(scrapper_state)
    }

    pub fn save_state(&self) -> Result<(), Error> {
        DB.insert(SCRAPPER_STATE_KEY, self)?;
        DB.flush()?;
        Ok(())
    }

    pub fn enable(&mut self) {
        self.enabled = true
    }

    // pub fn req_addresses(&mut self) -> Option<Ipv4Range> {
    //     if !self.enabled {
    //         return None;
    //     }
    //     let old = self.scanned_addr_counter.fetch_add(256, Ordering::Relaxed);
    //     if old > u32::MAX.wrapping_sub(255) {
    //         self.enabled = false;
    //         let _ = self.save_state();
    //         return None;
    //     }

    //     Some(Ipv4Range {
    //         current: old,
    //         end: old.wrapping_add(255),
    //     })
    // }

    pub fn reset(&mut self) -> Result<(), Error> {
        *self = Self {
            enabled: false,
            current_stage: self.current_stage,
        };
        self.save_state()?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("kv error -> {0}")]
    KvError(#[from] KvError),
}
