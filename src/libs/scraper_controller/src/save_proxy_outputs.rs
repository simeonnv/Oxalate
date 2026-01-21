use chrono::{NaiveDateTime, Utc};
use exn::{OptionExt, Result, ResultExt, bail};
use flate2::Compression;
use log::{debug, info};
use scraper::{Html, Selector};
use serde::Serialize;
use sqlx::{Pool, Postgres};
use std::io::prelude::*;
use std::{collections::HashSet, ops::Deref};
use url::Url;

use crate::{
    ProxyId,
    scraper_controller::{HttpRes, ProxyRes},
};

pub async fn save_proxy_outputs<LoggingCTX: Serialize>(
    proxy_id: &ProxyId,
    proxy_res: &[ProxyRes],
    db_pool: &Pool<Postgres>,
    logging_ctx: &LoggingCTX,
) -> Result<(), Error> {
    info!(ctx:serde = logging_ctx; "starting to save proxy responses into db");
    for output in proxy_res {
        match output {
            ProxyRes::HttpRes(http_based_output) => {
                save_http_https_output(http_based_output, &db_pool, proxy_id, logging_ctx)
                    .await
                    .or_raise(|| Error::FailedToHandleHttpRes)?
            } // ProxyRes::Msp(msp_output) => save_mcp_output(msp_output, &db_pool, proxy_id).await?,
        }
    }
    info!(ctx:serde = logging_ctx; "successfully saved proxy res into db");
    Ok(())
}

pub async fn save_http_https_output<LoggingCTX: Serialize>(
    output: &HttpRes,
    db_pool: &Pool<Postgres>,
    proxy_id: &ProxyId,
    logging_ctx: &LoggingCTX,
) -> Result<(), Error> {
    info!(ctx:serde = logging_ctx; "starting to save proxy http res");
    let body = &output.body;

    // TODO handle empty bodies by not extracting keywords
    if body.is_empty() {
        debug!(ctx:serde = logging_ctx; "http res body is empty; returning early");
        return Ok(());
    }

    let mut urls = HashSet::new();

    let keywords;

    {
        let html = Html::parse_document(body);

        let href_sel = Selector::parse(r#"a[href], area[href]"#)
            .map_err(|e| Error::HtmlParse(e.to_string()))
            .or_raise(|| Error::HtmlExtract)?;

        for el in html.select(&href_sel) {
            if let Some(link) = el.value().attr("href") {
                let mut parsed = if let Ok(e) = Url::parse(link) {
                    e
                } else if let Ok(e) = output.url.join(link) {
                    e
                } else {
                    continue;
                };

                parsed.set_query(None);
                parsed.set_fragment(None);

                urls.insert(parsed);
            }
        }

        keywords = html
            .root_element()
            .text()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>() // collect into a Vec<&str>
            .join(" ");
    }

    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(body.as_bytes())
        .or_raise(|| Error::HtmlCompression)?;
    let compressed_html = encoder.finish().or_raise(|| Error::HtmlCompression)?;

    let headers_json = serde_json::to_value(&output.headers).or_raise(|| Error::JsonHeaders)?;
    let url = output.url.to_string();

    sqlx::query!(
        "
                INSERT INTO Webpages
                    (url, compressed_body, keywords, headers, device_machine_id)
                VALUES
                    ($1, $2, $3, $4, $5)
                ON CONFLICT (url) DO NOTHING;   
            ",
        url,
        compressed_html,
        keywords,
        headers_json,
        proxy_id.deref()
    )
    .execute(db_pool)
    .await
    .or_raise(|| Error::DBQuery)?;

    for url in urls {
        let url = url.to_string();
        sqlx::query!(
            "
            INSERT INTO Urls
                (url, last_scanned, device_machine_id)
            VALUES
                ($1, $2, $3)
            ON CONFLICT (url) DO NOTHING;     
        ",
            url,
            None::<NaiveDateTime>,
            proxy_id.deref(),
        )
        .execute(db_pool)
        .await
        .or_raise(|| Error::DBQuery)?;
    }

    info!(ctx:serde = logging_ctx; "successfully saved proxy http res");
    Ok(())
}

// pub async fn save_mcp_output(
//     output: &MspOutput,
//     db_pool: &Pool<Postgres>,
//     proxy_id: &ProxyId,
// ) -> Result<(), Error> {
//     let now = Utc::now().naive_local();
//     let url = output.url.as_str();

//     let players = output.players.as_deref();
//     let mods = output.mods.as_deref();
//     sqlx::query!(
//         "
//             INSERT INTO MinecraftServers
//                 (url, last_scanned, device_machine_id, online_when_scraped,
//                  online_players_count, max_online_players, players, description, server_version, mods, ping)
//             VALUES
//                 ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
//             ;
//         ",
//         url,
//         now,
//         proxy_id.deref(),
//         output.online,
//         output.online_players_count as i32,
//         output.max_online_players as i32,
//         players,
//         output.description,
//         output.version,
//         mods,
//         //TODO FIX
//         0_f64,
//     )
//     .execute(db_pool)
//     .await?;

//     Ok(())
// }

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to handle proxy http res")]
    FailedToHandleHttpRes,

    #[error("failed to compress html")]
    HtmlCompression,

    #[error("failed to query db")]
    DBQuery,

    #[error("Failed to extract contents from the html")]
    HtmlExtract,

    #[error("Failed to parse html {0}")]
    HtmlParse(String),

    #[error("Failed to json parse http headers")]
    JsonHeaders,
}
