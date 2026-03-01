use async_trait::async_trait;
use base64::Engine;
use exn::{Result, ResultExt};
use rand::RngExt;
use scraper::{ElementRef, Html, Node, Selector};
use url::Url;
use wreq::Client;

use crate::scraping::{SearchEngine, text_search_engines::TextSearchEngineResult};

#[derive(Hash, Eq, PartialEq)]
pub struct TextSearchBing;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to fetch search html from bing")]
    FetchBing,

    #[error("failed to turn bing raw response into plain text")]
    ResToText,
}

#[async_trait]
impl SearchEngine<TextSearchEngineResult, Client, Error> for TextSearchBing {
    async fn search(query: &str, args: Client) -> Result<Vec<TextSearchEngineResult>, Error> {
        let wreq_client = args;
        let cvid = generate_cvid();

        let res = wreq_client
            .get("https://www.bing.com/search")
            .header("Cookie", &format!("SRCHHPGUSR=IG={}", cvid))
            .query(&[
                ("q", query),
                ("form", "QBLH"),
                ("sp", "-1"),
                ("lq", "0"),
                ("pq", query),
                ("sc", "12-4"),
                ("qs", "n"),
                ("sk", ""),
                ("cvid", &cvid),
                ("filters", "rcrse:\"1\""),
            ])
            .send()
            .await
            .or_raise(|| Error::FetchBing)?
            .text()
            .await
            .or_raise(|| Error::ResToText)?;

        let dom = Html::parse_document(&res);

        let sel_result = Selector::parse("#b_results > li.b_algo").unwrap();
        let sel_title_link = Selector::parse("h2 > a").unwrap();
        let sel_desc_container =
            Selector::parse(".b_caption > p, p.b_algoSlug, .b_caption .ipText").unwrap();

        let results = dom
            .select(&sel_result)
            .filter_map(|element| {
                let title_el = element.select(&sel_title_link).next()?;
                let title = title_el.text().collect::<String>().trim().to_string();

                let url_str = title_el.value().attr("href")?;
                let url = Url::parse(&clean_url(url_str)?).ok()?;

                let mut description = String::new();
                if let Some(desc_container) = element.select(&sel_desc_container).next() {
                    for node in desc_container.children() {
                        match node.value() {
                            Node::Text(t) => {
                                description.push_str(&t.text);
                            }
                            Node::Element(inner_el) => {
                                if !inner_el.has_class(
                                    "algoSlug_icon",
                                    scraper::CaseSensitivity::CaseSensitive,
                                ) && let Some(el_ref) = ElementRef::wrap(node)
                                {
                                    description.push_str(&el_ref.text().collect::<String>());
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if title.is_empty() {
                    return None;
                }

                Some(TextSearchEngineResult {
                    url,
                    title,
                    text: description.trim().to_string(),
                })
            })
            .collect();

        Ok(results)
    }
}

fn generate_cvid() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill(&mut bytes);
    bytes.iter().map(|b| format!("{:02X}", b)).collect()
}

fn clean_url(url: &str) -> Option<String> {
    if url.starts_with("https://www.bing.com/ck/a?") {
        let url = Url::parse(url).ok()?;
        let u = url
            .query_pairs()
            .find(|(key, _)| key == "u")
            .unwrap_or_default()
            .1;
        let u = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&u[2..])
            .unwrap_or_default();
        Some(String::from_utf8_lossy(&u).to_string())
    } else {
        Some(url.to_string())
    }
}
