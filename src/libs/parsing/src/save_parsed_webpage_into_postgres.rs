use std::{collections::HashMap, ops::Deref};

use chrono::NaiveDateTime;
use exn::{Result, ResultExt};
use oxalate_scraper_controller::ProxyId;
use serde_json::Value;
use sqlx::{Pool, Postgres};
use url::Url;

use crate::ParsedHtml;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to insert page into postgres")]
    InsertWebpages,

    #[error("Failed to insert webpages into db")]
    InsertUrls,
}

pub async fn save_parsed_webpage_into_postgres(
    db_pool: &Pool<Postgres>,
    parsed_html: &ParsedHtml,
    compressed_html: &[u8],
    headers_json: Value,
    proxy_id: ProxyId,
    url: Url,
) -> Result<(), Error> {
    sqlx::query!(
        "
            INSERT INTO Webpages
                (url, compressed_body, keywords, headers, device_machine_id, title)
            VALUES
                ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (url) DO NOTHING;   
        ",
        url.as_str(),
        compressed_html,
        parsed_html.keywords.join(" "),
        headers_json,
        proxy_id.deref(),
        parsed_html.title
    )
    .execute(db_pool)
    .await
    .or_raise(|| Error::InsertWebpages)?;

    for url in parsed_html.urls.iter() {
        let url = url.as_str();
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
        .or_raise(|| Error::InsertUrls)?;
    }

    exn::Ok(())
}
