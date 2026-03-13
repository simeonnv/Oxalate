use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use reqwest::Client;
use tokio::time::sleep;

use crate::AppState;
use systemstat::{Platform, System};

pub fn resources(_reqwest_client: Client, global_state: Arc<AppState>) {
    let sys = System::new();

    tokio::spawn(async move {
        loop {
            let old = global_state.request_counter.load(Ordering::Relaxed);
            sleep(Duration::from_secs(1)).await;
            let new = global_state.request_counter.load(Ordering::Relaxed);
            let _request_in_a_min = new - old;

            let _mem = sys
                .memory()
                .ok()
                .map(|e| (e.total.as_u64(), e.free.as_u64()));

            let _cpu_temp = sys.cpu_temp().ok();
        }
    });
}
