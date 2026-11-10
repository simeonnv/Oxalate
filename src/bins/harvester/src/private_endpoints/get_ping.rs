// Copyright 2026 Simeon Vangelov
// SPDX-License-Identifier: GPL-3.0-only

#[utoipa::path(
    get,
    path = "/ping",
    responses(
        (status = 200),
    ),
    tag = "Health",
)]
pub async fn get_ping() -> &'static str {
    "pong"
}
