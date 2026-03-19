use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod oauth {
    use super::*;

    const DEFAULT_OAUTH_URL: &str = "https://raindrop.io/oauth/access_token";
    const OAUTH_REQUEST_FAILED_MESSAGE: &str = "oauth request failed";
    const OAUTH_EMPTY_RESPONSE_MESSAGE: &str = "oauth request failed with empty response";
    const OAUTH_MISSING_ACCESS_TOKEN_MESSAGE: &str = "oauth response missing access_token";

    #[derive(Debug, Clone, Copy, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum GrantType {
        AuthorizationCode,
        RefreshToken,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct ExchangeCodeRequest<'a> {
        pub client_id: &'a str,
        pub client_secret: &'a str,
        pub code: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub redirect_uri: Option<&'a str>,
        pub grant_type: GrantType,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct RefreshTokenRequest<'a> {
        pub client_id: &'a str,
        pub client_secret: &'a str,
        pub refresh_token: &'a str,
        pub grant_type: GrantType,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct TokenResponse {
        pub access_token: String,
        #[serde(default)]
        pub refresh_token: Option<String>,
        #[serde(default)]
        pub expires_in: Option<u64>,
        #[serde(default)]
        pub token_type: Option<String>,
        #[serde(flatten)]
        pub extra: HashMap<String, serde_json::Value>,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct OauthErrorPayload {
        #[serde(default)]
        error: Option<String>,
        #[serde(default, alias = "error_description", alias = "errorMessage")]
        message: Option<String>,
        #[allow(dead_code)]
        #[serde(flatten)]
        extra: HashMap<String, serde_json::Value>,
    }

    struct RequestContext<'a> {
        oauth_url: &'a str,
        client_id: &'a str,
        client_secret: &'a str,
    }

    pub struct TokenRequestBuilder;

    pub struct ExchangeCodeTokenRequestBuilder<'a> {
        context: RequestContext<'a>,
        code: &'a str,
        redirect_uri: Option<&'a str>,
    }

    pub struct RefreshTokenRequestBuilder<'a> {
        context: RequestContext<'a>,
        refresh_token: &'a str,
    }

    impl TokenRequestBuilder {
        pub fn exchange_code<'a>(
            client_id: &'a str,
            client_secret: &'a str,
            code: &'a str,
        ) -> ExchangeCodeTokenRequestBuilder<'a> {
            ExchangeCodeTokenRequestBuilder {
                context: RequestContext {
                    oauth_url: DEFAULT_OAUTH_URL,
                    client_id,
                    client_secret,
                },
                code,
                redirect_uri: None,
            }
        }

        pub fn refresh<'a>(
            client_id: &'a str,
            client_secret: &'a str,
            refresh_token: &'a str,
        ) -> RefreshTokenRequestBuilder<'a> {
            RefreshTokenRequestBuilder {
                context: RequestContext {
                    oauth_url: DEFAULT_OAUTH_URL,
                    client_id,
                    client_secret,
                },
                refresh_token,
            }
        }
    }

    impl<'a> ExchangeCodeTokenRequestBuilder<'a> {
        pub fn oauth_url(mut self, oauth_url: &'a str) -> Self {
            self.context.oauth_url = oauth_url;
            self
        }

        pub fn redirect_uri(mut self, redirect_uri: &'a str) -> Self {
            self.redirect_uri = Some(redirect_uri);
            self
        }

        pub async fn send(self) -> Result<TokenResponse, Error> {
            let payload = ExchangeCodeRequest {
                client_id: self.context.client_id,
                client_secret: self.context.client_secret,
                code: self.code,
                redirect_uri: self.redirect_uri,
                grant_type: GrantType::AuthorizationCode,
            };
            post_token_request(self.context.oauth_url, &payload).await
        }
    }

    impl<'a> RefreshTokenRequestBuilder<'a> {
        pub fn oauth_url(mut self, oauth_url: &'a str) -> Self {
            self.context.oauth_url = oauth_url;
            self
        }

        pub async fn send(self) -> Result<TokenResponse, Error> {
            let payload = RefreshTokenRequest {
                client_id: self.context.client_id,
                client_secret: self.context.client_secret,
                refresh_token: self.refresh_token,
                grant_type: GrantType::RefreshToken,
            };
            post_token_request(self.context.oauth_url, &payload).await
        }
    }

    async fn post_token_request<T: Serialize>(
        url: &str,
        payload: &T,
    ) -> Result<TokenResponse, Error> {
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .form(payload)
            .send()
            .await
            .map_err(Error::Http)?;
        let status = response.status();
        let body = response.text().await.map_err(Error::Http)?;
        let parsed = serde_json::from_str::<serde_json::Value>(&body).ok();

        if !status.is_success() {
            let (code, message, raw) = if let Some(raw) = parsed {
                let oauth_error = parse_oauth_error_payload(&raw);
                let code = oauth_error.as_ref().and_then(|p| p.error.clone());
                let message = oauth_error_message(&oauth_error, OAUTH_REQUEST_FAILED_MESSAGE);
                (code, message, Some(raw))
            } else {
                let message = if body.trim().is_empty() {
                    OAUTH_EMPTY_RESPONSE_MESSAGE.to_string()
                } else {
                    format!(
                        "oauth request failed with non-json response: {}",
                        body.trim()
                    )
                };
                (None, message, None)
            };

            return Err(Error::Api {
                status,
                code,
                message,
                raw,
            });
        }

        let value = parsed.ok_or_else(|| {
            Error::Deserialize(format!(
                "oauth response is not valid json (status={}): {}",
                status,
                body.trim()
            ))
        })?;

        if value.get("access_token").is_none() {
            let oauth_error = parse_oauth_error_payload(&value);
            let code = oauth_error.as_ref().and_then(|p| p.error.clone());
            let message = oauth_error_message(&oauth_error, OAUTH_MISSING_ACCESS_TOKEN_MESSAGE);

            return Err(Error::Api {
                status,
                code,
                message,
                raw: Some(value),
            });
        }

        serde_json::from_value(value).map_err(|e| Error::Deserialize(e.to_string()))
    }

    fn parse_oauth_error_payload(raw: &serde_json::Value) -> Option<OauthErrorPayload> {
        serde_json::from_value::<OauthErrorPayload>(raw.clone()).ok()
    }

    fn oauth_error_message(parsed: &Option<OauthErrorPayload>, fallback: &str) -> String {
        parsed
            .as_ref()
            .and_then(|p| p.message.clone().or_else(|| p.error.clone()))
            .unwrap_or_else(|| fallback.to_string())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn oauth_error_message_prefers_description_alias() {
            let raw = serde_json::json!({
                "error": "invalid_grant",
                "error_description": "bad authorization code"
            });
            let parsed = parse_oauth_error_payload(&raw);
            assert_eq!(
                "bad authorization code",
                oauth_error_message(&parsed, OAUTH_REQUEST_FAILED_MESSAGE)
            );
        }

        #[test]
        fn oauth_error_message_supports_error_message_alias() {
            let raw = serde_json::json!({
                "error": "invalid_client",
                "errorMessage": "client credentials are invalid"
            });
            let parsed = parse_oauth_error_payload(&raw);
            assert_eq!(
                "client credentials are invalid",
                oauth_error_message(&parsed, OAUTH_REQUEST_FAILED_MESSAGE)
            );
        }

        #[test]
        fn oauth_error_message_falls_back_to_error_code_then_default() {
            let raw_with_error = serde_json::json!({ "error": "invalid_scope" });
            let parsed_with_error = parse_oauth_error_payload(&raw_with_error);
            assert_eq!(
                "invalid_scope",
                oauth_error_message(&parsed_with_error, OAUTH_REQUEST_FAILED_MESSAGE)
            );

            let raw_without_error = serde_json::json!({ "foo": "bar" });
            let parsed_without_error = parse_oauth_error_payload(&raw_without_error);
            assert_eq!(
                OAUTH_REQUEST_FAILED_MESSAGE,
                oauth_error_message(&parsed_without_error, OAUTH_REQUEST_FAILED_MESSAGE)
            );
        }
    }
}
