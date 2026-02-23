use utoipa::OpenApi;

use crate::endpoints::get_ping;
use crate::endpoints::post_insert_keywords;

#[derive(OpenApi)]
#[openapi(
    paths(get_ping::get_ping, post_insert_keywords::post_insert_keywords),
    tags(),
    security()
)]
pub struct ApiDoc;
