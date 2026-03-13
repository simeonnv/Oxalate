use std::collections::HashMap;

use exn::{Result, ResultExt};
use sqlx::{Pool, Postgres};
use wreq::Client;

use crate::scraping::{
    SearchEngine,
    text_search_engines::{
        TextSearchEngineResult, bing::TextSearchBing, brave::TextSearchBrave,
        google::TextSearchGoogle, oxalate::TextSearchOxalate,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to search with brave")]
    Brave,

    #[error("failed to search with bing")]
    Bing,

    #[error("failed to search with Google")]
    Google,

    #[error("failed to search with Oxalate")]
    Oxalate,
}

// pub struct SearchEngineBias {
//     brave: Option<NonZeroU8>,
// }

pub async fn search_text(
    query: &str,
    wreq_client: Client,
    db_pool: Pool<Postgres>,
) -> Result<HashMap<&'static str, Vec<TextSearchEngineResult>>, Error> {
    let (brave, bing, google, oxalate) = tokio::join!(
        TextSearchBrave::search(query, wreq_client.to_owned()),
        TextSearchBing::search(query, wreq_client.to_owned()),
        TextSearchGoogle::search(query, wreq_client),
        TextSearchOxalate::search(query, db_pool),
    );
    let brave = brave.or_raise(|| Error::Brave)?;
    let bing = bing.or_raise(|| Error::Bing)?;
    let google = google.or_raise(|| Error::Google)?;
    let oxalate = oxalate.or_raise(|| Error::Oxalate)?;

    let mut map = HashMap::new();
    map.insert("brave", brave);
    map.insert("bing", bing);
    map.insert("google", google);
    map.insert("oxalate", oxalate);

    Ok(map)
}
