use async_trait::async_trait;
use exn::{Result, ResultExt};
use sqlx::{Pool, Postgres};
use url::Url;

use crate::scraping::{SearchEngine, text_search_engines::TextSearchEngineResult};

#[derive(Hash, Eq, PartialEq)]
pub struct TextSearchOxalate;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to fetch oxalate search results from db")]
    DB,
}

#[async_trait]
impl SearchEngine<TextSearchEngineResult, Pool<Postgres>, Error> for TextSearchOxalate {
    async fn search(
        query: &str,
        args: Pool<Postgres>,
    ) -> Result<Vec<TextSearchEngineResult>, Error> {
        let db_pool = args;
        struct DbRes {
            pub url: String,
            pub keywords: String,
            pub title: String,
        }

        let db_res = sqlx::query_as!(
            DbRes,
            "
                SELECT url, keywords, title
                FROM Webpages
                WHERE keywords ||| $1
                ORDER BY paradedb.score(url) DESC
                LIMIT 25;
            ",
            query
        )
        .fetch_all(&db_pool)
        .await
        .or_raise(|| Error::DB)?;

        let results = db_res
            .into_iter()
            .filter_map(|e| {
                Some(TextSearchEngineResult {
                    url: Url::parse(&e.url).ok()?,
                    title: e.title,
                    text: e.keywords.chars().take(180).collect::<String>(),
                })
            })
            .collect();

        Ok(results)
    }
}
