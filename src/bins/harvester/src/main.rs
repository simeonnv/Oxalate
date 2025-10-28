use tonic::{Request, Response, Status, transport::Server};
pub mod harvester {

    tonic::include_proto!("harvester");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("harvester_descriptor");
}

pub use harvester::harvester_server::Harvester;
pub use harvester::{PingReq, PingRes};

use crate::harvester::harvester_server::HarvesterServer;

#[derive(Default)]
pub struct HarvesterService {}

#[tonic::async_trait]
impl Harvester for HarvesterService {
    async fn ping(&self, req: Request<PingReq>) -> Result<Response<PingRes>, Status> {
        println!("Got a request from {:?}", req.remote_addr());
        let reply = PingRes {
            message: format!("pong"),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse().unwrap();
    let harvester_server = HarvesterService::default();

    println!("HealthServer + GreeterServer listening on {addr}");

    let reflector = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(harvester::FILE_DESCRIPTOR_SET)
        .build_v1()
        .expect("failed to read reflector");

    Server::builder()
        .add_service(reflector)
        .add_service(HarvesterServer::new(harvester_server))
        .serve(addr)
        .await?;

    Ok(())
}
