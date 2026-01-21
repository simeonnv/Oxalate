use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{GlobalState, HARVESTER_URL};

use craftping::tokio::ping;
use futures::future;
use futures::stream::{self, StreamExt};
use log::{error, info};
use oxalate_schemas::harvester::public::proxy::post_proxy::{Req, Res};
use oxalate_scraper_controller::scraper_controller::{HttpBasedOutput, MspOutput, ProxyOutput};
use oxalate_urls::urls::ProxyReq;
use reqwest::{Client, Url};
use tokio::{
    net::TcpStream,
    time::{sleep, timeout},
};

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
            let reqs = match res.0 {
                Some(e) => e,
                None => {
                    info!("proxy controller is paused / disabled, waiting for it to re-activate");
                    sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };

            info!("got urls!");

            let mut outputs = vec![];
            stream::iter(reqs.0)
                .map(|req| {
                    global_state
                        .request_counter
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    async {
                        match req {
                            ProxyReq::Msp(e) => handle_msp_request(e.url).await,
                            ProxyReq::Http(e) | ProxyReq::Https(e) => {
                                handle_http_https_request(&reqwest_client, e.url, &global_state)
                                    .await
                            }
                        }
                    }
                })
                .buffer_unordered(800)
                .filter_map(|e| future::ready(e.map(|e| *e)))
                .for_each(|e| {
                    outputs.push(e);
                    future::ready(())
                })
                .await;

            info!("all outputs are retrived, returning back to main scrapper server!");

            let req = Req::ReturnUrlOutputs(outputs);

            if let Err(err) = reqwest_client.post(&url).json(&req).send().await {
                info!("failed to send back http outputs!: {err}");
                sleep(Duration::from_secs(30)).await;
            };
        }
    });
}

async fn handle_msp_request(url: Url) -> Option<Box<ProxyOutput>> {
    let host = match url.host_str() {
        Some(e) => e,
        None => return None,
    };
    let port = url.port().unwrap_or(25565);

    let mut stream = match timeout(Duration::from_secs(5), TcpStream::connect((host, port))).await {
        Ok(Ok(e)) => e,
        Ok(Err(err)) => {
            error!("msp tcpstream err: {err}");
            return None;
        }
        _ => return None,
    };

    let mc_res = match timeout(Duration::from_secs(5), ping(&mut stream, host, port)).await {
        Ok(Ok(e)) => e,
        _ => return None,
    };
    info!("mc hit!");

    let players = mc_res
        .sample
        .map(|e| e.iter().map(|e| e.name.to_owned()).collect());
    let proxy_output = MspOutput {
        url,
        // TODO get rid of this
        online: true,
        online_players_count: mc_res.online_players as i64,
        max_online_players: mc_res.max_players as i64,
        description: mc_res
            .description
            .map(|e| e.to_string())
            .unwrap_or("".into()),
        players,
        version: mc_res.version,
        // TODO get rid of both of these
        ping: 0_f64,
        mods: None,
    };
    let proxy_output = ProxyOutput::Msp(proxy_output);

    Some(Box::new(proxy_output))
}

async fn handle_http_https_request(
    reqwest_client: &Client,
    url: Url,
    global_state: &GlobalState,
) -> Option<Box<ProxyOutput>> {
    // dbg!(&url);
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
            info!("website hit");
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
        Err(err) => {
            // let code = err.status();
            // let is_timeout = err.is_timeout();
            // let is_connect = err.is_connect();
            // error!(
            //     "http proxy err: {err}, code: {code:?}, is_timeout: {is_timeout}, is_connect: {is_connect}"
            // );
            None
        }
    }
}
