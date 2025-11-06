use std::net::Ipv4Addr;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicU16, AtomicU32};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::kv_db::DB;
use crate::kv_db::Error as KvError;

const MAIN_STATE_KEY: &[u8; 10] = b"Main State";

#[derive(Serialize, Deserialize, Default)]
pub struct ScrapperState {
    pub enabled: bool,
    pub scanned_addr_counter: AtomicU32,
    pub connected_devices: AtomicU16,
}

impl ScrapperState {
    pub fn load() -> Result<Self, Error> {
        let scrapper_state = DB.get(MAIN_STATE_KEY)?;
        let scrapper_state = match scrapper_state {
            Some(e) => e,
            None => {
                let scrapper_state = Self {
                    enabled: false,
                    scanned_addr_counter: 0.into(),
                    connected_devices: 0.into(),
                };
                Self::save_state(&scrapper_state)?;
                scrapper_state
            }
        };
        Ok(scrapper_state)
    }

    pub fn save_state(&self) -> Result<(), Error> {
        DB.insert(MAIN_STATE_KEY, self)?;
        DB.flush()?;
        Ok(())
    }

    pub fn enable(&mut self) {
        self.enabled = true
    }

    pub fn req_addresses(&mut self) -> Option<Ipv4Range> {
        if !self.enabled {
            return None;
        }
        let old = self.scanned_addr_counter.fetch_add(256, Ordering::Relaxed);
        if old > u32::MAX.wrapping_sub(255) {
            self.enabled = false;
            let _ = self.save_state();
            return None;
        }

        Some(Ipv4Range {
            current: old,
            end: old.wrapping_add(255),
        })
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        *self = Self {
            enabled: false,
            scanned_addr_counter: 0.into(),
            connected_devices: 0.into(),
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

#[derive(Clone, Debug)]
pub struct Ipv4Range {
    pub current: u32,
    pub end: u32,
}

impl Iterator for Ipv4Range {
    type Item = Ipv4Addr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.end {
            return None;
        }
        let ip = Ipv4Addr::from(self.current);
        self.current = self.current.wrapping_add(1);
        Some(ip)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = if self.current <= self.end {
            (self.end - self.current + 1) as usize
        } else {
            0
        };
        (remaining, Some(remaining))
    }
}
