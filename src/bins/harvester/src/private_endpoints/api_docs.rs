use utoipa::OpenApi;

// pub use crate::private_endpoints;
use crate::private_endpoints::control;
use crate::private_endpoints::get_ping;
use crate::private_endpoints::metric;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_ping::get_ping,
        control::get_scraper_state::get_scraper_state,
        control::post_swap_scraper_on_state::post_swap_scraper_on_state,
        metric::get_active_tasks::get_active_tasks,
        metric::get_connected_proxies::get_connected_proxies,
    ),
    tags(
        (name = "Control", description = "controlling the whole system"),
    ),
    security()
)]
pub struct ApiDoc;
