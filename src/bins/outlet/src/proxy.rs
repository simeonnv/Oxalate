use crate::HARVESTER_URL;

pub async fn proxy() {
    tokio::spawn(async move {
        let url = format!("{}/proxy", *HARVESTER_URL);
    });
}
