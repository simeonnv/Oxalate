use axum::{Json, extract::State};
use exn::ResultExt;
use http_error::HttpError;

use oxalate_schemas::indexer::post_search::{Req, Res, SearchResult};
use url::Url;

use crate::AppState;

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
    struct DbRes {
        pub url: String,
        pub keywords: String,
        pub score: Option<f32>,
    }

    let db_res = sqlx::query_as!(
        DbRes,
        "
            SELECT url, keywords, paradedb.score(url)
            FROM Webpages
            WHERE keywords ||| $1
            ORDER BY paradedb.score(url) DESC
            LIMIT 25;
        ",
        req.text
    )
    .fetch_all(&state.db_pool)
    .await
    .or_raise(|| HttpError::Internal("".into()))?;

    let search_results = db_res
        .into_iter()
        .map(|e| {
            let url = Url::parse(&e.url).map_err(|_| HttpError::Internal("".into()))?;
            Ok(SearchResult {
                url,
                score: e.score.unwrap_or(0.),
            })
        })
        .collect::<Result<Vec<_>, HttpError>>()?;

    let res = Res { search_results };

    Ok(Json(res))
}
