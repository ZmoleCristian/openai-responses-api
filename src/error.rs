use std::error::Error;
use std::fmt::{self, Display, Formatter};

use eventsource_stream::EventStreamError;
use reqwest::StatusCode;
use reqwest::header::InvalidHeaderValue;
use serde_json::Value;
use url::{ParseError, Url};

use crate::response::ApiError;

#[derive(Debug)]
pub enum ResponsesError {
    Http(reqwest::Error),
    Header(InvalidHeaderValue),
    Json(serde_json::Error),
    Sse(EventStreamError<reqwest::Error>),
    Api {
        status: StatusCode,
        error: ApiError,
    },
    Status {
        status: StatusCode,
        body: String,
        parse: serde_json::Error,
    },
    Untyped(Value),
    BaseUrl(ParseError),
    CannotBeABase(Url),
}

impl Display for ResponsesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ResponsesError::Http(e) => write!(f, "http: {e}"),
            ResponsesError::Header(e) => write!(f, "header value: {e}"),
            ResponsesError::Json(e) => write!(f, "json: {e}"),
            ResponsesError::Sse(e) => write!(f, "sse: {e}"),
            ResponsesError::Api { status, error } => {
                write!(f, "api {status}: {:?}: {}", error.kind, error.message)
            }
            ResponsesError::Status {
                status,
                body,
                parse,
            } => write!(f, "api {status}, unparsed error body ({parse}): {body}"),
            ResponsesError::Untyped(value) => write!(f, "object without a type tag: {value}"),
            ResponsesError::BaseUrl(e) => write!(f, "base url: {e}"),
            ResponsesError::CannotBeABase(url) => write!(f, "base url cannot take a path: {url}"),
        }
    }
}

impl Error for ResponsesError {}

impl From<reqwest::Error> for ResponsesError {
    fn from(e: reqwest::Error) -> Self {
        ResponsesError::Http(e)
    }
}

impl From<InvalidHeaderValue> for ResponsesError {
    fn from(e: InvalidHeaderValue) -> Self {
        ResponsesError::Header(e)
    }
}

impl From<serde_json::Error> for ResponsesError {
    fn from(e: serde_json::Error) -> Self {
        ResponsesError::Json(e)
    }
}

impl From<ParseError> for ResponsesError {
    fn from(e: ParseError) -> Self {
        ResponsesError::BaseUrl(e)
    }
}
