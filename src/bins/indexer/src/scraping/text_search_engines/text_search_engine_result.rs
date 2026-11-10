// Copyright 2026 Simeon Vangelov
// SPDX-License-Identifier: GPL-3.0-only

use url::Url;

#[derive(Debug, Eq, PartialEq)]
pub struct TextSearchEngineResult {
    pub url: Url,
    pub title: String,
    pub text: String,
}
