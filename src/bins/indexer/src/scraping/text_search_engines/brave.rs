use async_trait::async_trait;
use exn::{Result, ResultExt};
use scraper::{Html, Selector};
use url::Url;
use wreq::Client;

use crate::scraping::{SearchEngine, text_search_engines::TextSearchEngineResult};

#[derive(Hash, Eq, PartialEq)]
pub struct TextSearchBrave;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to fetch search html from brave")]
    FetchBrave,

    #[error("failed to turn brave raw response into plain text")]
    ResToText,
}

#[async_trait]
impl SearchEngine<TextSearchEngineResult, Client, Error> for TextSearchBrave {
    async fn search(query: &str, args: Client) -> Result<Vec<TextSearchEngineResult>, Error> {
        let wreq_client = args;

        let res = wreq_client
            .get("https://search.brave.com/search".to_string())
            .query(&[("q", query), ("nfpr", "1")])
            .send()
            .await
            .or_raise(|| Error::FetchBrave)?
            .text()
            .await
            .or_raise(|| Error::ResToText)?;

        let dom = Html::parse_document(&res);

        let sel_result = Selector::parse("#results > .snippet[data-pos]:not(.standalone)").unwrap();
        let sel_title = Selector::parse(".title").unwrap();
        let sel_href = Selector::parse("a").unwrap();
        let sel_desc =
            Selector::parse(".generic-snippet, .video-snippet > .snippet-description").unwrap();

        let results = dom
            .select(&sel_result)
            .filter_map(|element| {
                let title = element
                    .select(&sel_title)
                    .next()?
                    .text()
                    .collect::<String>()
                    .trim()
                    .to_string();

                let url_str = element.select(&sel_href).next()?.value().attr("href")?;

                let url = Url::parse(url_str).ok()?;

                let text = element
                    .select(&sel_desc)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                if title.is_empty() && text.is_empty() {
                    return None;
                }

                Some(TextSearchEngineResult { url, title, text })
            })
            .collect();

        Ok(results)
    }
}
