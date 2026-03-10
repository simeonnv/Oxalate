use std::{collections::HashSet, fmt::Display};

use exn::{Result, ResultExt};
use scraper::{Html, Selector};
use tokio::task::spawn_blocking;
use url::Url;

use crate::{ParsedHtml, split_into_words::split_into_words};

pub async fn parse_html(html: String, url: Url) -> Result<ParsedHtml, Error> {
    let (keywords, title, urls) = spawn_blocking(move || {
        let mut urls = HashSet::new();
        let html = Html::parse_document(&html);

        let title_sel = Selector::parse("title")
            .map_err(|e| HtmlParse(e.to_string()))
            .or_raise(|| Error::HtmlExtract)?;

        let title = html
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let href_sel = Selector::parse(r#"a[href], area[href]"#)
            .map_err(|e| HtmlParse(e.to_string()))
            .or_raise(|| Error::HtmlExtract)?;

        for el in html.select(&href_sel) {
            if let Some(link) = el.value().attr("href") {
                let mut parsed = if let Ok(e) = Url::parse(link) {
                    e
                } else if let Ok(e) = url.join(link) {
                    e
                } else {
                    continue;
                };

                parsed.set_query(None);
                parsed.set_fragment(None);

                urls.insert(parsed);
            }
        }

        let mut text_parts = Vec::new();
        let root = html.root_element();

        fn extract_text(element: scraper::ElementRef, buffer: &mut Vec<String>) {
            for node in element.children() {
                if let Some(child_el) = scraper::ElementRef::wrap(node) {
                    let tag = child_el.value().name();
                    if matches!(
                        tag,
                        "script"
                            | "style"
                            | "svg"
                            | "head"
                            | "noscript"
                            | "iframe"
                            | "object"
                            | "embed"
                            | "template"
                            | "code"
                            | "pre"
                    ) {
                        continue;
                    }
                    extract_text(child_el, buffer);
                } else if let Some(text) = node.value().as_text() {
                    let t = text.trim();
                    if !t.is_empty() {
                        buffer.push(t.to_lowercase());
                    }
                }
            }
        }

        extract_text(root, &mut text_parts);
        let raw_text = text_parts.join(" ");
        let keywords = split_into_words(&raw_text);

        exn::Ok((keywords, title, urls))
    })
    .await
    .or_raise(|| Error::HtmlThreadPanic)??;

    Ok(ParsedHtml {
        keywords,
        title,
        urls,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to extract contents from the html")]
    HtmlExtract,

    #[error("blocking html parsing thread paniced")]
    HtmlThreadPanic,
}

#[derive(Debug)]
pub struct HtmlParse(String);
impl Display for HtmlParse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse html {}", self.0)
    }
}
impl std::error::Error for HtmlParse {}
