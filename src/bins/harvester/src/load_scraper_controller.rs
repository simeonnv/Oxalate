use exn::{Result, ResultExt};
use oxalate_kv_db::kv_db::KvDb;
use oxalate_scraper_controller::ScraperController;

pub fn load_scraper_controller(
    kv_db: &KvDb,
    key: &'static str,
) -> Result<ScraperController, Error> {
    let scraper_controller = kv_db
        .get(&key)
        .or_raise(|| Error::GetKv)?
        .unwrap_or_default();

    Ok(scraper_controller)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to load scraper controller from kv db")]
    GetKv,
}
