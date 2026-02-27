use std::collections::HashMap;

use axum::{Json, extract::State};
use exn::ResultExt;
use http_error::HttpError;

use neo4rs::query;
use oxalate_schemas::indexer::post_keyword_graph::{Node, Relation, Req, Res};
use oxalate_utils::parse_into_words;

use crate::AppState;

#[utoipa::path(
    post,
    path = "/keyword_graph",
    request_body = Req,
    responses(
        (status = 200),
    ),
    tag = "Search",
)]
#[axum::debug_handler]
pub async fn post_keyword_graph(
    State(state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<Json<Res>, HttpError> {
    let keywords = parse_into_words(req.text);

    let row_stream = state
        .neo4j_pool
        .execute(
            query(
                r#"
            UNWIND $words AS word_text
            MATCH (start:Word {text: word_text})
            OPTIONAL MATCH (start)-[r:RELATED]-(neighbor:Word)
            WITH start, neighbor, r
            ORDER BY r.weight DESC

            WITH start, collect(neighbor)[0..20] AS topNodes
            WITH start, topNodes + start AS nodesPerWord

            UNWIND nodesPerWord AS node
            WITH DISTINCT node
            WHERE node IS NOT NULL
            WITH collect(node) AS totalPool

            UNWIND totalPool AS n1
            MATCH (n1)-[rel:RELATED]->(n2)
            WHERE n2 IN totalPool

            RETURN
                n1.text AS source,
                n1.usage AS sourceUsage,
                rel.weight AS weight,
                n2.text AS target,
                n2.usage AS targetUsage
            ORDER BY weight DESC
        "#,
            )
            .param("words", keywords),
        )
        .await
        .or_raise(|| Error::Query)
        .or_raise(|| HttpError::Internal("".into()))?;

    struct WordRelation {
        pub source: String,
        pub source_usage: i64,
        pub weight: i64,
        pub target: String,
        pub target_usage: i64,
    }

    let relations_res = {
        let mut relations = Vec::new();
        let mut row_stream = row_stream;
        while let Ok(Some(row)) = row_stream.next().await {
            let relation = WordRelation {
                source: row.get("source").unwrap_or_default(),
                source_usage: row.get("sourceUsage").unwrap_or(0),
                weight: row.get("weight").unwrap_or(0),
                target: row.get("target").unwrap_or_default(),
                target_usage: row.get("targetUsage").unwrap_or(0),
            };

            relations.push(relation);
        }
        relations
    };

    let (nodes, relations) = {
        let mut nodes = HashMap::new();
        let mut relations = vec![];
        for rel_res in relations_res {
            nodes
                .entry(rel_res.source.to_owned())
                .or_insert(rel_res.source_usage);
            nodes
                .entry(rel_res.target.to_owned())
                .or_insert(rel_res.target_usage);

            relations.push(Relation {
                source_word: rel_res.source,
                weight: rel_res.weight,
                target_word: rel_res.target,
            });
        }

        (
            nodes
                .into_iter()
                .map(|(word, usage)| Node { word, usage })
                .collect::<Vec<_>>(),
            relations,
        )
    };

    Ok(Json(Res { relations, nodes }))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get relations from neo4j")]
    Query,
}

// UNWIND ["hello"] AS word_text
// MATCH (start:Word {text: word_text})
// OPTIONAL MATCH (start)-[r:RELATED]-(neighbor:Word)
// WITH start, neighbor, r
// ORDER BY r.weight DESC

// WITH start, collect(neighbor)[0..10] AS topNodes
// WITH start, topNodes + start AS nodesPerWord

// UNWIND nodesPerWord AS node
// WITH DISTINCT node
// WHERE node IS NOT NULL
// WITH collect(node) AS totalPool

// UNWIND totalPool AS n1
// MATCH (n1)-[rel:RELATED]->(n2)
// WHERE n2 IN totalPool

// RETURN
//     n1.text AS source,
//     n1.usage AS sourceUsage,
//     rel.weight AS weight,
//     n2.text AS target,
//     n2.usage AS targetUsage
// ORDER BY weight DESC
