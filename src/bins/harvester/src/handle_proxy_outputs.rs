use std::collections::HashSet;

use scraper::{Html, Selector};
use sqlx::{Pool, Postgres};
use url::Url;

use crate::scrapper_state::ProxyOutput;

pub async fn save_proxy_outputs(proxy_outputs: &[ProxyOutput], db_pool: Pool<Postgres>) {
    let mut urls = HashSet::new();
    let mut compressed_outputs = vec![];

    for output in proxy_outputs {
        let body = &output.body;
        let html = Html::parse_document(body);

        let href_sel = Selector::parse(r#"a[href], area[href]"#).unwrap();
        for el in html.select(&href_sel) {
            if let Some(link) = el.value().attr("href") {
                let mut parsed = if let Ok(e) = Url::parse(link) {
                    e
                } else if let Ok(e) = output.url.join(link) {
                    e
                } else {
                    continue;
                };

                parsed.set_query(None);
                parsed.set_fragment(None);

                urls.insert(parsed);
            }
        }

        let text: Box<[String]> = html
            .root_element()
            .text()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .map(|t| t.to_owned())
            .collect();
        compressed_outputs.push((output, text));
    }

    // task work
    todo!();
}
