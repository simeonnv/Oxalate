use crate::{PingReq, PingRes, proccesses::HarvesterService};
use tonic::{Request, Response, Status};

pub async fn ping(
    _ctx: &HarvesterService,
    _req: Request<PingReq>,
) -> Result<Response<PingRes>, Status> {
    let reply = PingRes {
        message: "pong".into(),
    };
    Ok(Response::new(reply))
}
