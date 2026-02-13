use utoipa::OpenApi;

use crate::endpoints::get_ping;

#[derive(OpenApi)]
#[openapi(paths(get_ping::get_ping,), tags(), security())]
pub struct ApiDoc;
