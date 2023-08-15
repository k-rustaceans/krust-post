use std::fmt::Display;

use event_driven_library::responses::{AnyError, ApplicationResponse, BaseError};
use event_driven_library::ApplicationError;
use serde::Serialize;

use crate::domain::post::PostAggregate;

#[derive(Debug, Serialize)]
pub enum ServiceResponse {
    Post(PostAggregate),
    String(String),
    Bool(bool),
    I64(i64),
    F64(f64),
    Empty(()),
}
impl ApplicationResponse for ServiceResponse {}

impl From<PostAggregate> for ServiceResponse {
    fn from(value: PostAggregate) -> Self {
        ServiceResponse::Post(value)
    }
}

impl From<String> for ServiceResponse {
    fn from(value: String) -> Self {
        ServiceResponse::String(value)
    }
}

impl From<bool> for ServiceResponse {
    fn from(value: bool) -> Self {
        ServiceResponse::Bool(value)
    }
}
impl From<()> for ServiceResponse {
    fn from(_value: ()) -> Self {
        ServiceResponse::Empty(())
    }
}

impl From<i64> for ServiceResponse {
    fn from(value: i64) -> Self {
        ServiceResponse::I64(value)
    }
}

impl From<f64> for ServiceResponse {
    fn from(value: f64) -> Self {
        ServiceResponse::F64(value)
    }
}

#[derive(Debug,ApplicationError!)]
pub enum ServiceError {
    DeserializationError(Box<AnyError>),
    EntityNotFound,
    InvalidURL,
    ParsingError,
    StopSentinel,
    HttpError(Box<AnyError>),
    BadRequest,
    BaseError(BaseError),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::DeserializationError(res) => write!(f, "{}", res),
            ServiceError::EntityNotFound => write!(f, "EntityNotFound"),
            ServiceError::InvalidURL => write!(f, "InvalidURL"),
            ServiceError::StopSentinel => write!(f, "StopSentinel"),
            ServiceError::ParsingError => write!(f, "ParsingError"),
            ServiceError::HttpError(res) => write!(f, "{}", res),
            ServiceError::BadRequest => write!(f, "BadRequest"),
            ServiceError::BaseError(base) => write!(f, "{}", &base.to_string()),
        }
    }
}
