// Copyright 2026 Simeon Vangelov
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Req {
    pub url: String,
    pub keywords: Vec<String>,
    pub window_size: usize,
    pub weight_increase: u8,
}
