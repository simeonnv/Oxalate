use async_trait::async_trait;
use exn::{Result, ResultExt};
use scraper::{ElementRef, Html, Selector};
use url::Url;
use wreq::Client;

use crate::scraping::{SearchEngine, text_search_engines::TextSearchEngineResult};

#[derive(Hash, Eq, PartialEq)]
pub struct TextSearchGoogle;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to fetch search html from google")]
    FetchGoogle,

    #[error("failed to turn google raw response into plain text")]
    ResToText,
}

#[async_trait]
impl SearchEngine<TextSearchEngineResult, Client, Error> for TextSearchGoogle {
    async fn search(query: &str, args: Client) -> Result<Vec<TextSearchEngineResult>, Error> {
        let wreq_client = args;

        let res = wreq_client
            .get("https://www.google.com/search")
            .query(&[("q", query), ("nfpr", "1"), ("filter", "0"), ("start", "0")])
            .send()
            .await
            .or_raise(|| Error::FetchGoogle)?
            .text()
            .await
            .or_raise(|| Error::ResToText)?;
        dbg!(&res);

        let dom = Html::parse_document(&res);
        let mut results = Vec::new();

        let sel_featured = Selector::parse("block-component").unwrap();

        if let Some(featured_el) = dom.select(&sel_featured).next()
            && let Some(res) = parse_featured_snippet(&featured_el)
        {
            results.push(res);
        }

        let sel_result = Selector::parse("[jscontroller=SC7lYd]").unwrap();
        let sel_title = Selector::parse("h3").unwrap();
        let sel_href = Selector::parse("a[href]").unwrap();
        let sel_desc = Selector::parse(
            "div[data-sncf='2'], div[data-sncf='1,2'], div[style='-webkit-line-clamp:2']",
        )
        .unwrap();

        for element in dom.select(&sel_result) {
            let title = element
                .select(&sel_title)
                .next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default();

            let url_str = element
                .select(&sel_href)
                .next()
                .and_then(|e| e.value().attr("href"))
                .unwrap_or_default();

            let text = element
                .select(&sel_desc)
                .next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default();

            if let Ok(url) = Url::parse(url_str)
                && !title.is_empty()
            {
                results.push(TextSearchEngineResult { url, title, text });
            }
        }

        Ok(results)
    }
}

fn parse_featured_snippet(el: &ElementRef) -> Option<TextSearchEngineResult> {
    let sel_title =
        Selector::parse(".g > div[lang] a h3, div[lang] > div[style='position:relative'] a h3")
            .unwrap();
    let sel_href = Selector::parse(
        ".g > div[lang] a:has(h3), div[lang] > div[style='position:relative'] a:has(h3)",
    )
    .unwrap();

    let title = el.select(&sel_title).next()?.text().collect::<String>();
    let url_str = el.select(&sel_href).next()?.value().attr("href")?;
    let url = Url::parse(url_str).ok()?;

    let mut description = String::new();

    let sel_heading = Selector::parse("div[role='heading']").unwrap();
    if let Some(h) = el.select(&sel_heading).next() {
        description.push_str(&format!("{}\n\n", h.text().collect::<String>()));
    }

    let sel_desc_span =
        Selector::parse("div[data-attrid='wa:/description'] > span:first-child").unwrap();
    let sel_list = Selector::parse("ul").unwrap();
    let sel_li = Selector::parse("li").unwrap();

    if let Some(desc_span) = el.select(&sel_desc_span).next() {
        description.push_str(&desc_span.text().collect::<String>());
    } else if let Some(list_el) = el.select(&sel_list).next() {
        for li in list_el.select(&sel_li) {
            description.push_str(&format!("• {}\n", li.text().collect::<String>()));
        }
    }

    Some(TextSearchEngineResult {
        url,
        title,
        text: description.trim().to_string(),
    })
}
