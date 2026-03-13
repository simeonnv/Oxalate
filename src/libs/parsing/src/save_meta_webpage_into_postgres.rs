use exn::{Result, ResultExt};
use sqlx::{Pool, Postgres};
use url::Url;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to insert meta page into postgres")]
    InsertMetaWebpages,

    #[error("Failed to insert webpages into db")]
    InsertUrls,
}

pub async fn save_meta_webpage_into_postgres(
    db_pool: &Pool<Postgres>,
    keywords: &[String],
    title: &str,
    url: &Url,
    search_engine: &str,
) -> Result<(), Error> {
    sqlx::query!(
        "
            INSERT INTO MetaWebpages
                (url, keywords, title, search_engine)
            VALUES
                ($1, $2, $3, $4)
            ON CONFLICT (url) DO NOTHING;   
        ",
        url.as_str(),
        keywords.join(" "),
        title,
        search_engine
    )
    .execute(db_pool)
    .await
    .or_raise(|| Error::InsertMetaWebpages)?;

    exn::Ok(())
}
