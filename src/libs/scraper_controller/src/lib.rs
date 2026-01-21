pub mod scraper_controller;
pub use scraper_controller::ScraperController;

pub mod save_proxy_outputs;
pub use save_proxy_outputs::save_proxy_outputs;

mod proxy_id;
pub use proxy_id::HEADER_KEY;
pub use proxy_id::ProxyId;

mod file_iterator_task_generator;
pub use file_iterator_task_generator::FileIteratorTaskGenerator;

// pub mod ipv4_iterator_task_generator;
