use utoipa::OpenApi;

use crate::endpoints::get_ping;
use crate::endpoints::post_keyword_graph;
use crate::endpoints::post_search;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_ping::get_ping,
        post_search::post_search,
        post_keyword_graph::post_keyword_graph
    ),
    tags(),
    security()
)]
pub struct ApiDoc;
