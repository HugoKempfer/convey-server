use actix_web::dev::HttpResponseBuilder;
use actix_web::error::ResponseError;
use actix_web::http::{header, StatusCode};
use actix_web::HttpResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConveyError {
    #[error("Temporary error while storing/accessing data from cache.")]
    CacheError(#[from] redis::RedisError),

    #[error("Serialization error")]
    SerializationError(#[from] serde_json::Error),

    #[error("You don't have access over this resource.")]
    UnauthorizedError(),

    #[error("The requested resource cannot be found.")]
    NotFound(),

    #[error("Internal communication error")]
    MailboxError(#[from] actix::MailboxError),

    #[error("{0}")]
    BadRequest(String),
}

impl ResponseError for ConveyError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ConveyError::CacheError(_)
            | ConveyError::SerializationError(_)
            | ConveyError::MailboxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConveyError::UnauthorizedError() => StatusCode::UNAUTHORIZED,
            ConveyError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ConveyError::NotFound() => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .body(self.to_string())
    }
}
