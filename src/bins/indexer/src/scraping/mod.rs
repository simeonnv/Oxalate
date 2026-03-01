use async_trait::async_trait;
use exn::Result;

pub mod image_search_engines;
pub mod text_search_engines;

mod search_text;
pub use search_text::search_text;

#[async_trait]
pub trait SearchEngine<SearchEngineResult, Args, Error: std::error::Error + Sync + Send> {
    async fn search(query: &str, args: Args) -> Result<Vec<SearchEngineResult>, Error>;
}
