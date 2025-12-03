use std::collections::HashSet;

use chrono::{NaiveDateTime, Utc};
use flate2::Compression;
use log::info;
use scraper::{Html, Selector};
use sqlx::{Pool, Postgres};
use std::io::prelude::*;
use thiserror::Error;
use url::Url;

use crate::scrapper_controller::{HttpBasedOutput, MspOutput, ProxyOutput};

pub async fn save_proxy_outputs(
    proxy_id: &str,
    proxy_outputs: &[ProxyOutput],
    db_pool: Pool<Postgres>,
) -> Result<(), Error> {
    info!("got outputs, saving them!");
    for output in proxy_outputs {
        match output {
            ProxyOutput::HttpBased(http_based_output) => {
                save_http_https_output(http_based_output, &db_pool, proxy_id).await?
            }
            ProxyOutput::Msp(msp_output) => save_mcp_output(msp_output, &db_pool, proxy_id).await?,
        }
    }
    info!("saved outputs!");
    Ok(())
}

pub async fn save_http_https_output(
    output: &HttpBasedOutput,
    db_pool: &Pool<Postgres>,
    proxy_id: &str,
) -> Result<(), Error> {
    let body = &output.body;

    if body.is_empty() {
        return Ok(());
    }

    let mut urls = HashSet::new();

    let keywords;

    {
        let html = Html::parse_document(body);

        let href_sel = Selector::parse(r#"a[href], area[href]"#)
            .map_err(|err| Error::HtmlParse(err.to_string()))?;
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
        .map_err(|err| Error::Compression(err.to_string()))?;

    let compressed_html = encoder
        .finish()
        .map_err(|err| Error::Compression(err.to_string()))?;

    let headers_json =
        serde_json::to_value(&output.headers).map_err(|err| Error::Json(err.to_string()))?;
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
        proxy_id
    )
    .execute(db_pool)
    .await?;

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
            proxy_id,
        )
        .execute(db_pool)
        .await?;
    }

    Ok(())
}

pub async fn save_mcp_output(
    output: &MspOutput,
    db_pool: &Pool<Postgres>,
    proxy_id: &str,
) -> Result<(), Error> {
    let now = Utc::now().naive_local();
    let url = output.url.as_str();

    let players = output.players.as_deref();
    let mods = output.mods.as_deref();
    sqlx::query!(
        "
            INSERT INTO MinecraftServers
                (url, last_scanned, device_machine_id, online_when_scraped,
                 online_players_count, max_online_players, players, description, server_version, mods)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ;
        ",
        url,
        now,
        proxy_id,
        output.online,
        output.online_players_count as i32,
        output.max_online_players as i32,
        players,
        output.description,
        output.version,
        mods,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to compress html -> {0}!")]
    Compression(String),

    #[error("db error -> {0}")]
    DB(#[from] sqlx::Error),

    #[error("Html parse error -> {0}!")]
    HtmlParse(String),

    #[error("Json parse error -> {0}!")]
    Json(String),
}
