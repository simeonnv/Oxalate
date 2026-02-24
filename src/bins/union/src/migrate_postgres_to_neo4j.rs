use axum::{Json, extract::State};
use chrono::Utc;
use oxalate_schemas::union::post_insert_keywords::Req;

use crate::{AppState, endpoints::post_insert_keywords};

struct DBRes {
    pub keywords: String,
    pub url: String,
}

pub async fn migrate_postgres_to_neo4j(state: AppState) {
    let now = Utc::now().naive_local();

    for i in 0.. {
        log::info!("migrating field: {}-{}", i * 50, i * 50 + 50);
        let start = Utc::now().time();

        log::info!("getting keywords");
        let webpages = sqlx::query_as!(
            DBRes,
            "
                SELECT keywords, url FROM Webpages
                WHERE created_at < $1
                ORDER BY created_at DESC
                LIMIT 50
                OFFSET $2;
            ",
            now,
            i * 50,
        )
        .fetch_all(&state.db_pool)
        .await
        .unwrap();

        if webpages.is_empty() {
            log::info!("No more webpages, aka we are done");
            break;
        }

        log::info!("iter inserting");
        for webpage in webpages {
            if webpage.keywords.is_empty() {
                continue;
            }

            let keywords: Vec<String> = webpage
                .keywords
                .split_whitespace()
                .map(|e| e.to_owned())
                .collect();

            // peak implementaciq
            post_insert_keywords(
                State(state.clone()),
                Json(Req {
                    keywords,
                    window_size: 5,
                    weight_increase: 1,
                    url: webpage.url,
                }),
            )
            .await
            .unwrap();
        }

        let end = Utc::now().time();
        log::info!(
            "migrated fields {}-{}, took {}",
            i * 50,
            i * 50 + 50,
            end - start
        );
    }
}
