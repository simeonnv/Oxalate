pub mod scrapper_controller;
pub use scrapper_controller::ScrapperController;

pub mod global_scan;

pub mod save_proxy_outputs;
pub use save_proxy_outputs::save_proxy_outputs;

mod proxy_id;
pub use proxy_id::HEADER_KEY;
pub use proxy_id::ProxyId;

mod error;
pub use error::Error;
