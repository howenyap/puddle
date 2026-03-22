use crate::client::RaindropClient;
use crate::error::{ApiErrorPayload, Error, RateLimitInfo};
use chrono::{TimeZone, Utc};
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use reqwest::{Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Default)]
pub struct ResponseMetadata {
    pub rate_limit: RateLimitInfo,
}

#[derive(Debug, Clone)]
pub struct Response<T> {
    pub data: T,
    pub meta: ResponseMetadata,
}

impl RaindropClient {
    pub(crate) async fn send_json<T, Q, B>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        body: Option<&B>,
    ) -> Result<Response<T>, Error>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
        B: Serialize + ?Sized,
    {
        let mut request = self.request(method, path)?;

        if let Some(query) = query {
            request = request.query(query);
        }
        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await.map_err(Error::Http)?;
        Self::decode_response(response).await
    }

    pub(crate) async fn send_multipart<T, Q>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        body: reqwest::multipart::Form,
    ) -> Result<Response<T>, Error>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
    {
        let mut request = self.request(method, path)?;

        if let Some(query) = query {
            request = request.query(query);
        }

        let response = request.multipart(body).send().await.map_err(Error::Http)?;

        Self::decode_response(response).await
    }

    fn request(&self, method: Method, path: &str) -> Result<reqwest::RequestBuilder, Error> {
        let path = path.trim_start_matches('/');
        let url = self.inner.base_url.join(path)?;

        Ok(self
            .inner
            .http
            .request(method, url)
            .header(AUTHORIZATION, self.inner.auth_header.as_str())
            .header(USER_AGENT, self.inner.user_agent.as_str()))
    }

    async fn decode_response<T>(response: reqwest::Response) -> Result<Response<T>, Error>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let rate_limit = parse_rate_limit(response.headers());

        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(Error::RateLimited { rate_limit });
        }

        let bytes = response.bytes().await.map_err(Error::Http)?;
        let body_text = String::from_utf8_lossy(&bytes).to_string();
        let raw_json = if bytes.is_empty() {
            Some(serde_json::Value::Null)
        } else {
            serde_json::from_slice::<serde_json::Value>(&bytes).ok()
        };

        if !status.is_success() {
            if let Some(raw) = raw_json {
                return Err(map_api_error(status, raw));
            }

            let message = if body_text.trim().is_empty() {
                format!("request failed with status {status}")
            } else {
                format!(
                    "request failed with status {} and non-json response: {}",
                    status,
                    body_text.trim()
                )
            };

            return Err(Error::Api {
                status,
                code: None,
                message,
                raw: None,
            });
        }

        let raw = raw_json.ok_or_else(|| {
            Error::Deserialize(format!(
                "response is not valid json (status={}): {}",
                status,
                body_text.trim()
            ))
        })?;

        let data = serde_json::from_value(raw.clone()).map_err(|e| {
            if let Some(api_error) = success_status_api_error(status, &raw) {
                api_error
            } else {
                Error::Deserialize(e.to_string())
            }
        })?;

        Ok(Response {
            data,
            meta: ResponseMetadata { rate_limit },
        })
    }
}

fn map_api_error(status: StatusCode, raw: serde_json::Value) -> Error {
    let parsed = serde_json::from_value::<ApiErrorPayload>(raw.clone()).ok();

    let code = parsed.as_ref().and_then(|p| p.error.clone());
    let message = parsed
        .as_ref()
        .and_then(|p| p.error_message.clone().or_else(|| p.message.clone()))
        .unwrap_or_else(|| format!("request failed with status {status}"));

    Error::Api {
        status,
        code,
        message,
        raw: Some(raw),
    }
}

fn success_status_api_error(status: StatusCode, raw: &serde_json::Value) -> Option<Error> {
    let parsed = serde_json::from_value::<ApiErrorPayload>(raw.clone()).ok()?;

    if parsed.result != Some(false) {
        return None;
    }

    let has_error_details = parsed.error.is_some()
        || parsed.error_message.is_some()
        || parsed.message.is_some();

    if !has_error_details {
        return None;
    }

    Some(map_api_error(status, raw.clone()))
}

fn parse_rate_limit(headers: &reqwest::header::HeaderMap) -> RateLimitInfo {
    let limit = header_as_u32(headers, "x-ratelimit-limit");
    let remaining = header_as_u32(headers, "x-ratelimit-remaining");
    let reset_at = header_as_i64(headers, "x-ratelimit-reset")
        .and_then(|epoch| Utc.timestamp_opt(epoch, 0).single());

    RateLimitInfo {
        limit,
        remaining,
        reset_at,
    }
}

fn header_as_u32(headers: &reqwest::header::HeaderMap, name: &str) -> Option<u32> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u32>().ok())
}

fn header_as_i64(headers: &reqwest::header::HeaderMap, name: &str) -> Option<i64> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
}
