// Copyright 2026 Simeon Vangelov
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Clone)]
#[schema(as = Post::Info::Logs::Req)]
pub struct Req {
    pub logs: Vec<Value>,
}
