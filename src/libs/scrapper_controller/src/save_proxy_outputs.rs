use std::collections::HashSet;

use chrono::NaiveDateTime;
use flate2::Compression;
use log::info;
use scraper::{Html, Selector};
use sqlx::{Pool, Postgres};
use std::io::prelude::*;
use thiserror::Error;
use url::Url;

use crate::scrapper_controller::ProxyOutput;

pub struct CompressedHtml {
    pub keywords: String,
    pub compressed_html: Box<[u8]>,
}

pub async fn save_proxy_outputs(
    proxy_id: &str,
    proxy_outputs: &[ProxyOutput],
    db_pool: Pool<Postgres>,
) -> Result<(), Error> {
    let mut urls = HashSet::new();
    let mut compressed_outputs = vec![];

    for output in proxy_outputs {
        let body = &output.body;

        if body.is_empty() {
            continue;
        }

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

        let keywords: String = html
            .root_element()
            .text()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>() // collect into a Vec<&str>
            .join(" ");

        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(body.as_bytes())
            .map_err(|err| Error::Compression(err.to_string()))?;

        let compressed_html = encoder
            .finish()
            .map_err(|err| Error::Compression(err.to_string()))?
            .into_boxed_slice();
        let compressed_html = CompressedHtml {
            keywords,
            compressed_html,
        };

        compressed_outputs.push((output, compressed_html));
    }

    for (proxy_output, compressed_html) in compressed_outputs.into_iter() {
        let headers_json = serde_json::to_value(&proxy_output.headers)
            .map_err(|err| Error::Json(err.to_string()))?;
        let url = proxy_output.url.to_string();
        let keywords = compressed_html.keywords;
        let compressed_html: &[u8] = &compressed_html.compressed_html;
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
        .execute(&db_pool)
        .await?;
    }

    info!("all webpages sent, sending urls!");
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
        .execute(&db_pool)
        .await?;
    }

    info!("saved outputs!");
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
