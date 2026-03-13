use std::{collections::HashMap, pin::Pin};

use axum::{Json, extract::State};
use exn::ResultExt;
use futures::FutureExt;
use http_error::HttpError;

use oxalate_parsing::split_into_words::split_into_words;
use oxalate_schemas::indexer::post_search::{Req, Res, SearchResult};

use crate::{AppState, scraping::search_text};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to search through the search engines")]
    SearchThoughSearchEngines,

    #[error("failed to send meta results to parser")]
    SendParser,

    #[error("parser responded with a bad error code")]
    ParserResError,
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
    let results = search_text(&req.text, state.wreq_client, state.db_pool.to_owned())
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
        .collect::<HashMap<_, _>>();

    let mut futures = vec![];
    for (search_engine, results) in results.iter() {
        use oxalate_schemas::parser::post_insert_meta_webpage::{Page, Req};

        if search_engine == "oxalate" {
            continue;
        }

        let pages = results
            .iter()
            .map(|e| {
                let keywords = split_into_words(&e.text);
                Page {
                    url: e.url.to_owned(),
                    keywords,
                    title: e.title.to_owned(),
                    search_engine: search_engine.to_owned(),
                }
            })
            .collect();
        let reqwest_client = state.reqwest_client.to_owned();
        let parser_url = state.parser_url.join("/insert_meta_webpage").unwrap();

        futures.push(
            async move {
                reqwest_client
                    .post(parser_url)
                    .json(&Req { pages })
                    .send()
                    .await
            }
            .boxed(),
        );
    }

    tokio::spawn(async move {
        for future in futures {
            let res = future.await.map(|e| e.error_for_status());
            match res {
                Err(err) => log::error!("Parser send task error: {err:?}"),
                Ok(Err(err)) => log::error!("Parser send task error: {err:?}"),
                _ => {}
            };
        }
    });

    Ok(Json(Res {
        search_results: results,
    }))
}
