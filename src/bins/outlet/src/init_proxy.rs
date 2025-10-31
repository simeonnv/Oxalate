use tonic::transport::Channel;

use crate::harvester::harvester_client::HarvesterClient;

pub async fn init_proxy(client: &mut HarvesterClient<Channel>) {
    // client.send_compressed(encoding)
}
