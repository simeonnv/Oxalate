use exn::{Exn, Result, ResultExt};
use oxalate_kv_db::kv_db::KvDb;
use oxalate_scraper_controller::ScraperController;

pub fn save_scraper_controller(
    kv_db: &KvDb,
    scraper_controller: &ScraperController,
    key: &'static str,
) -> Result<(), Error> {
    kv_db
        .insert(&key, scraper_controller)
        .or_raise(|| Error::InsertKv)?;
    kv_db.flush().or_raise(|| Error::FlushKv)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to insert scraper controller from kv db")]
    InsertKv,

    #[error("Failed to flush kv with scraper controller")]
    FlushKv,
}
