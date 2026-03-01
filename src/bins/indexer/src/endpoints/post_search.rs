use axum::{Json, extract::State};
use exn::ResultExt;
use http_error::HttpError;

use oxalate_schemas::indexer::post_search::{Req, Res, SearchResult};

use crate::{AppState, scraping::search_text};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to search through the search engines")]
    SearchThoughSearchEngines,
}

#[utoipa::path(
    post,
    path = "/search",
    request_body = Req,
    responses(
        (status = 200),
    ),
    tag = "Search",
)]
#[axum::debug_handler]
pub async fn post_search(
    State(state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<Json<Res>, HttpError> {
    let results = search_text(&req.text, state.wreq_client, state.db_pool)
        .await
        .or_raise(|| Error::SearchThoughSearchEngines)
        .or_raise(|| HttpError::Internal("".into()))?;

    let results = results
        .into_iter()
        .map(|(k, v)| {
            let v: Vec<_> = v
                .into_iter()
                .map(|e| SearchResult {
                    url: e.url,
                    text: e.text,
                    title: e.title,
                })
                .collect();
            (k.to_owned(), v)
        })
        .collect();

    Ok(Json(Res {
        search_results: results,
    }))
}
