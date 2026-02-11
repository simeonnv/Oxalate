use utoipa::OpenApi;

// pub use crate::private_endpoints;
use crate::private_endpoints::control;
use crate::private_endpoints::get_ping;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_ping::get_ping,
        control::get_scraper_state::get_scraper_state,
        control::post_swap_scraper_on_state::post_swap_scraper_on_state,
    ),
    tags(
        (name = "Control", description = "controlling the whole system"),
    ),
    security()
)]
pub struct ApiDoc;
