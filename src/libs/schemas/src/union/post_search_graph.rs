// Copyright 2026 Simeon Vangelov
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Req {
    pub search_keywords: Vec<String>,
    // THIS VARIABLE IS VERY SENCETIVE, ONLY USE UP TO MAX LEVEL 3
    pub recursion_depth: u8,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Res {}
