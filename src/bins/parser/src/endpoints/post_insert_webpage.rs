use axum::{Json, extract::State};
use exn::ResultExt;
use http_error::HttpError;
use oxalate_parsing::save_parsed_webpage_into_postgres::save_parsed_webpage_into_postgres;

use crate::AppState;
use oxalate_schemas::parser::post_insert_webpage::*;

pub use oxalate_parsing::{
    compress_html::compress_html, parse_html::parse_html, save_into_neo4j::save_into_neo4j,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed parse html")]
    Parse,

    #[error("failed to compress html")]
    Compress,

    #[error("failed to insert parsed html into neo4j")]
    InsertNeo4j,

    #[error("failed to insert parsed html into postges")]
    InsertPg,
}

#[utoipa::path(
    post,
    path = "/insert_webpage",
    request_body = Req,
    responses(
        (status = 200),
    ),
    tag = "Insert",
)]
#[axum::debug_handler]
pub async fn post_insert_webpage(
    State(state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), HttpError> {
    for page in req.pages {
        if page.raw_html.is_empty() {
            return Err(HttpError::BadRequest("empty html field!".to_owned()));
        }

        let headers = serde_json::to_value(page.headers.unwrap_or_default())
            .or_raise(|| HttpError::BadRequest("invalid headers".into()))?;

        let compressed_html = compress_html(&page.raw_html)
            .or_raise(|| Error::Compress)
            .or_raise(|| HttpError::Internal("".into()))?;

        let parsed_html = parse_html(page.raw_html, page.url.to_owned())
            .await
            .or_raise(|| Error::Parse)
            .or_raise(|| HttpError::Internal("".into()))?;

        // we dont joint the db futures, bc it will send such a high amount of request to the databases that there is a real risk they can crash
        let neo4j_result =
            save_into_neo4j(&state.neo4j_pool, &parsed_html.keywords, &page.url, 5).await;
        let pg_result = save_parsed_webpage_into_postgres(
            &state.db_pool,
            &parsed_html,
            &compressed_html,
            headers,
            page.proxy_id,
            page.url,
        )
        .await;

        neo4j_result
            .or_raise(|| Error::InsertNeo4j)
            .or_raise(|| HttpError::Internal("".into()))?;
        pg_result
            .or_raise(|| Error::InsertPg)
            .or_raise(|| HttpError::Internal("".into()))?;
    }

    Ok(())
}
