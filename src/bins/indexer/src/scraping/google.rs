use crate::AppState;
use exn::{Ok, Result, ResultExt};
use http_error::HttpError;
use scraper::Selector;
use url::Url;

pub async fn request(query: &str, state: &AppState) -> Result<String, http_error::HttpError> {
    let resp = state
        .wreqclient
        .get(
            Url::parse_with_params(
                "https://www.google.com/search",
                &[("q", query), ("nfpr", "1")],
            )
            .unwrap(),
        )
        .send()
        .await
        .or_raise(|| HttpError::BadRequest("error fetching from google".to_string()))?
        .text()
        .await
        .or_raise(|| HttpError::BadRequest("Error configuring response to string".to_string()))?;
    Ok(resp)
}

pub fn parse_response(body: &str) -> Vec<String> {
    let fragment = scraper::Html::parse_fragment(body);
    let selector = Selector::parse(r#"a[href]"#).unwrap();
    let mut hrefs: Vec<String> = Vec::new();

    for element in fragment.select(&selector) {
        if let Some(href) = element.attr("href") {
            hrefs.push(href.to_string());
        }
    }
    hrefs
}
