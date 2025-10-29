use crate::{Harvester, PingReq, PingRes};
use tonic::{Request, Response, Status};

mod ping;
pub use ping::ping as impl_ping;

#[derive(Default)]
pub struct HarvesterService {}

#[tonic::async_trait]
impl Harvester for HarvesterService {
    async fn ping(&self, req: Request<PingReq>) -> Result<Response<PingRes>, Status> {
        impl_ping(self, req).await
    }
}
