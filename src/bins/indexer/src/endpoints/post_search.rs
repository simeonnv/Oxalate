use std::collections::HashMap;

use axum::{Json, extract::State};
use exn::ResultExt;
use http_error::HttpError;

use oxalate_schemas::indexer::post_search::{Req, Res, SearchResult};
use tokio::try_join;
use url::Url;

use crate::{
    AppState,
    scraping::{self},
};

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
        #[allow(dead_code)]
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

    let res_metasearch = get_metasearch_results(&req.text, &state)
        .await
        .or_raise(|| HttpError::Internal("Error during metasearch results".to_string()))?;

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

    dbg!(&search_results);

    let res = Res {
        search_results,
        metasearch_results: res_metasearch,
    };

    Ok(Json(res))
}

async fn get_metasearch_results(
    query: &str,
    state: &AppState,
) -> Result<Vec<SearchResult>, HttpError> {
    let (brave_html, mojeek_html) = try_join!(
        scraping::brave::request(query, &state),
        scraping::mojeek::request(query, &state)
    )
    .or_raise(|| HttpError::Internal("External search providers failed".into()))?;
    let mut results = scraping::brave::parse_response(&brave_html);
    let mut mojeek_parsed_results = scraping::mojeek::parse_response(&mojeek_html);

    results.append(&mut mojeek_parsed_results);

    let mut counts: HashMap<String, f32> = HashMap::new();

    for value in results {
        *counts.entry(value).or_insert(0.0) += 1.0;
    }

    let response: Vec<SearchResult> = counts
        .into_iter()
        .map(|(url_string, score)| SearchResult {
            url: Url::parse(&url_string).unwrap(), // FIXXXXXX
            score,
        })
        .collect();
    Ok(response)
}
