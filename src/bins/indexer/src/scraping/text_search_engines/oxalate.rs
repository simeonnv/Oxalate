use crate::scraping::{SearchEngine, text_search_engines::TextSearchEngineResult};
use async_trait::async_trait;
use exn::{Result, ResultExt};
use sqlx::{Pool, Postgres};
use url::Url;

#[derive(Hash, Eq, PartialEq)]
pub struct TextSearchOxalate;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to fetch oxalate search webpage results from db")]
    DBWebpage,

    #[error("failed to fetch oxalate search meta webpage results from db")]
    DBMetaWebpage,
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
            pub score: Option<f32>,
        }

        let mut db_webpage_res = sqlx::query_as!(
            DbRes,
            r#"
                SELECT url, keywords, title, paradedb.score(url) 
                FROM Webpages
                WHERE keywords ||| $1
                ORDER BY score DESC
                LIMIT 25;
            "#,
            query
        )
        .fetch_all(&db_pool)
        .await
        .or_raise(|| Error::DBWebpage)?;

        let db_meta_webpage_res = sqlx::query_as!(
            DbRes,
            r#"
                SELECT url, keywords, title, paradedb.score(url) 
                FROM MetaWebpages
                WHERE keywords ||| $1
                ORDER BY score DESC
                LIMIT 25;
            "#,
            query
        )
        .fetch_all(&db_pool)
        .await
        .or_raise(|| Error::DBMetaWebpage)?;

        db_webpage_res.extend(db_meta_webpage_res);
        db_webpage_res.sort_by(|a, b| {
            b.score
                .unwrap_or_default()
                .partial_cmp(&a.score.unwrap_or_default())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let results = db_webpage_res
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
