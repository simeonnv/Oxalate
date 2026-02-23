use std::ops::Deref;

use axum::{Json, extract::State};
use exn::ResultExt;
use http_error::HttpError;
use neo4rs::{BoltType, Query, query};
use oxalate_schemas::union::post_insert_keywords::*;

use crate::AppState;
use itertools::Itertools;

#[utoipa::path(
    post,
    path = "/insert_keywords",
    request_body = Req,
    responses(
        (status = 200),
    ),
    tag = "Graph",
)]
#[axum::debug_handler]
pub async fn post_insert_keywords(
    State(state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), HttpError> {
    let mut queries = vec![];
    let keywords = req.keywords;
    let window_size = req.window_size;

    for window in keywords.windows(window_size) {
        for (word_1, word_2) in window.iter().tuple_combinations() {
            let (first, second) = if word_1 < word_2 {
                (word_1, word_2)
            } else {
                (word_2, word_1)
            };

            let query = query(
                "
                MERGE (w1:Word {text: $1})
                MERGE (w2:Word {text: $2})
                MERGE (w1)-[r:RELATED]->(w2)
                  ON CREATE SET r.weight = 1
                  ON MATCH SET r.weight = r.weight + 1
                ",
            )
            .param("1", first.to_owned())
            .param("2", second.to_owned());

            queries.push(query);
        }
    }

    let mut txn = state
        .log4j_pool
        .start_txn()
        .await
        .or_raise(|| HttpError::Internal("".into()))?;

    txn.run_queries(queries.into_iter())
        .await
        .or_raise(|| HttpError::Internal("".into()))?;

    txn.commit()
        .await
        .or_raise(|| HttpError::Internal("Failed to commit".into()))?;

    Ok(())
}
