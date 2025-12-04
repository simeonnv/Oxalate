use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{GlobalState, HARVESTER_URL};

use async_scoped::TokioScope;
use log::{error, info};
use mc_server_status::{McClient, McError, ServerData};
use oxalate_schemas::harvester::public::proxy::post_proxy::{Req, Res};
use oxalate_scrapper_controller::scrapper_controller::{HttpBasedOutput, MspOutput, ProxyOutput};
use reqwest::{Client, Url};
use tokio::time::sleep;

pub fn proxy(reqwest_client: Client, mc_client: McClient, global_state: Arc<GlobalState>) {
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
                        global_state
                            .request_counter
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        match url.scheme() {
                            "http" | "https" => {
                                handle_http_https_request(&reqwest_client, url, &global_state).await
                            }
                            "msp" => handle_msp_request(&mc_client, url).await,
                            _ => None,
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

async fn handle_msp_request(mc_client: &McClient, url: Url) -> Option<Box<ProxyOutput>> {
    let host = url.host_str();
    let port = url.port();

    let mut protocolless_url;
    match host {
        Some(e) => protocolless_url = String::from(e),
        None => return None,
    };
    if let Some(port) = port {
        protocolless_url.push(':');
        protocolless_url.push_str(&port.to_string());
    }

    // info!("sending mc request!");
    let mc_res = mc_client
        .ping(&protocolless_url, mc_server_status::ServerEdition::Java)
        .await;
    let mc_res = match mc_res {
        Ok(e) => e,
        Err(_) => return None,
    };

    info!("mc hit!");

    let data = match mc_res.data {
        ServerData::Java(java_status) => java_status,
        ServerData::Bedrock(_) => return None,
    };

    let players = data
        .players
        .sample
        .map(|e| e.iter().map(|e| e.name.to_owned()).collect());
    let mods = data
        .mods
        .map(|e| e.iter().map(|e| e.modid.to_owned()).collect());

    let proxy_output = MspOutput {
        url,
        online: mc_res.online,
        online_players_count: data.players.online,
        max_online_players: data.players.max,
        description: data.description,
        players,
        version: data.version.name,
        mods,
    };
    let proxy_output = ProxyOutput::Msp(proxy_output);

    return Some(Box::new(proxy_output));
}

async fn handle_http_https_request(
    reqwest_client: &Client,
    url: Url,
    global_state: &GlobalState,
) -> Option<Box<ProxyOutput>> {
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

            let proxy_output = HttpBasedOutput {
                url,
                status,
                body,
                headers,
            };
            let proxy_output = ProxyOutput::HttpBased(proxy_output);

            Some(Box::new(proxy_output))
        }
        Err(_) => None,
    }
}
