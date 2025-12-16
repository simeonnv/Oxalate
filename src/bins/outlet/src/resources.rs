use std::{
    fmt::Debug,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use reqwest::Client;
use tokio::time::sleep;

use crate::GlobalState;
use systemstat::{CPULoad, Platform, System};

pub fn resources(global_state: Arc<GlobalState>, reqwest_client: Client) {
    let sys = System::new();

    tokio::spawn(async move {
        loop {
            let old = global_state.request_counter.load(Ordering::Relaxed);
            sleep(Duration::from_secs(1)).await;
            let new = global_state.request_counter.load(Ordering::Relaxed);
            let request_in_a_min = new - old;

            let mem = sys
                .memory()
                .ok()
                .map(|e| (e.total.as_u64(), e.free.as_u64()));

            let cpu_temp = sys.cpu_temp().ok();
        }
    });
}
