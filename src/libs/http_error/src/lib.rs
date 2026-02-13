use std::ops::Deref;

use axum::{http::StatusCode, response::IntoResponse};
use exn::Exn;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum HttpError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal: {0}")]
    Internal(String),
}

impl From<Exn<HttpError>> for HttpError {
    fn from(err: Exn<HttpError>) -> Self {
        let e = err.deref();
        if let HttpError::Internal(_) = e {
            log::error!("{:?}", err);
        } else {
            log::debug!("{:?}", err);
        };
        e.to_owned()
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::BadRequest(e) => (StatusCode::BAD_REQUEST, e).into_response(),
            Self::Unauthorized(e) => (StatusCode::UNAUTHORIZED, e).into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    }
}
