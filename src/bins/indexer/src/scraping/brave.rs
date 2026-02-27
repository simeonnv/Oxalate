use crate::AppState;
use exn::{Result, ResultExt};
use http_error::HttpError;
use scraper::Selector;
use url::Url;

pub async fn request(query: &str, state: &AppState) -> Result<String, http_error::HttpError> {
    let resp = state
        .wreqclient
        .get(format!("https://search.brave.com/search"))
        .query(&[("q", query), ("nfpr", "1")])
        .send()
        .await
        .or_raise(|| HttpError::BadRequest("Error fetching from google".to_string()))?
        .text()
        .await
        .or_raise(|| HttpError::BadRequest("Error while converting request to text".to_string()))?;
    dbg!(&resp);
    Ok(resp)
}

pub fn parse_response(body: &str) -> Vec<String> {
    let fragment = scraper::Html::parse_fragment(body);
    let selector = Selector::parse(r#"a[href]"#).unwrap();
    let mut hrefs: Vec<String> = Vec::new();

    for element in fragment.select(&selector) {
        if let Some(href) = element.attr("href")
            && let Ok(url) = Url::parse(href)
        {
            if matches!(url.scheme(), "http" | "https")
                && url.host().is_some()
                && !href.contains("brave")
            {
                hrefs.push(href.to_string());
            }
        }
    }
    hrefs
}
