use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorPayload {
    pub result: Option<bool>,
    pub error: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
    pub message: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RateLimitInfo {
    pub limit: Option<u32>,
    pub remaining: Option<u32>,
    pub reset_at: Option<DateTime<Utc>>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("api error (status={status}, code={code:?}, message={message})")]
    Api {
        status: StatusCode,
        code: Option<String>,
        message: String,
        raw: Option<serde_json::Value>,
    },

    #[error("deserialize error: {0}")]
    Deserialize(String),

    #[error("auth error: {0}")]
    Auth(String),

    #[error("rate limited")]
    RateLimited { rate_limit: RateLimitInfo },

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("invalid url: {0}")]
    Url(String),
}

impl Error {
    pub fn is_refreshable_auth_error(&self) -> bool {
        matches!(self, Error::Api { status, .. } if status.as_u16() == 401)
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Self::Url(value.to_string())
    }
}
