use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{GlobalState, HARVESTER_URL};

use async_scoped::TokioScope;
use log::{error, info};
use oxalate_schemas::harvester::public::proxy::post_proxy::{Req, Res};
use oxalate_scrapper_controller::scrapper_controller::ProxyOutput;
use reqwest::Client;
use tokio::time::sleep;

pub fn proxy(reqwest_client: Client, global_state: Arc<GlobalState>) {
    tokio::spawn(async move {
        let url = format!("http://{}/proxy", *HARVESTER_URL);
        loop {
            info!("requesting urls");
            let res = match reqwest_client
                .post(&url)
                .json(&Req::RequestUrls)
                .send()
                .await
            {
                Ok(e) => e,
                Err(err) => {
                    error!("failed to fetch /proxy urls!: {err}");
                    sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };
            let res = match res.json::<Res>().await {
                Ok(e) => e,
                Err(err) => {
                    error!("failed to deserialize json while fetching urls at /proxy!: {err}");
                    sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };
            let urls = match res.0 {
                Some(e) => e,
                None => {
                    info!("proxy controller is paused / disabled, waiting for it to re-activate");
                    sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };
            info!("got urls!");
            let (_, outputs) = TokioScope::scope_and_block(|spawner| {
                for url in urls {
                    spawner.spawn(async {
                        let res = reqwest_client
                            .get(url.as_str())
                            .header("machine-id", "")
                            .send()
                            .await;
                        global_state
                            .request_counter
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        match res {
                            Ok(e) => {
                                info!("hit a actual website");
                                let status = e.status().as_u16();
                                let raw_headers = e.headers().to_owned();
                                let body = e.text().await.unwrap_or_default();

                                let mut headers = HashMap::with_capacity(raw_headers.len());
                                for (key, val) in raw_headers.iter() {
                                    let key = key.to_string();
                                    let val = match val.to_str() {
                                        Ok(e) => e,
                                        Err(_) => continue,
                                    };

                                    headers.insert(key.to_string(), val.to_string());
                                }

                                Some(Box::new(ProxyOutput {
                                    url,
                                    status,
                                    body,
                                    headers,
                                }))
                            }
                            Err(err) => None,
                        }
                    });
                }
            });

            let outputs = outputs
                .into_iter()
                .filter_map(|e| e.unwrap())
                .map(|boxed| *boxed)
                .collect();
            info!("all outputs are retrived, returning back to main scrapper server!");

            let req = Req::ReturnUrlOutputs(outputs);

            if let Err(err) = reqwest_client.post(&url).json(&req).send().await {
                info!("failed to send back http outputs!: {err}");
                sleep(Duration::from_secs(30)).await;
            };
        }
    });
}
