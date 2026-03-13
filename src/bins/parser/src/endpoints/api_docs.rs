use utoipa::OpenApi;

use crate::endpoints::get_ping;
use crate::endpoints::post_insert_meta_webpage;
use crate::endpoints::post_insert_webpage;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_ping::get_ping,
        post_insert_meta_webpage::post_insert_meta_webpage,
        post_insert_webpage::post_insert_webpage,
    ),
    tags(),
    security()
)]
pub struct ApiDoc;
