// Copyright 2026 Simeon Vangelov
// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = Post::KeyLogger::Req)]
pub struct Req(pub Vec<Key>);

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(as = Post::KeyLogger::Req::Key)]
pub struct Key {
    pub at: NaiveDateTime,
    pub key_pressed: String,
}
