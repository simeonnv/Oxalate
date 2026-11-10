// Copyright 2026 Simeon Vangelov
// SPDX-License-Identifier: GPL-3.0-only

use serde::Serialize;

#[derive(Serialize)]
pub struct Res {
    pub enabled: bool,
}
