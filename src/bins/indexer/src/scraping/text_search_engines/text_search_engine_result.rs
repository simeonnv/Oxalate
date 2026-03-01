use url::Url;

#[derive(Debug, Eq, PartialEq)]
pub struct TextSearchEngineResult {
    pub url: Url,
    pub title: String,
    pub text: String,
}
