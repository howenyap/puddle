use crate::endpoints::{
    collections::CollectionsApi, filters::FiltersApi, raindrops::RaindropsApi, tags::TagsApi,
    user::UserApi,
};
use crate::error::Error;
use reqwest::header::HeaderValue;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

const DEFAULT_BASE_URL: &str = "https://api.raindrop.io/rest/v1";
const DEFAULT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug)]
pub struct RaindropClient {
    pub(crate) inner: Arc<ClientInner>,
}

#[derive(Debug)]
pub(crate) struct ClientInner {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: Url,
    pub(crate) auth_header: String,
    pub(crate) user_agent: String,
}

impl RaindropClient {
    pub fn new(access_token: impl Into<String>) -> Result<Self, Error> {
        Self::builder().access_token(access_token).build()
    }

    pub fn builder() -> RaindropClientBuilder {
        RaindropClientBuilder::default()
    }

    pub fn collections(&self) -> CollectionsApi {
        CollectionsApi::new(self.clone())
    }

    pub fn raindrops(&self) -> RaindropsApi {
        RaindropsApi::new(self.clone())
    }

    pub fn tags(&self) -> TagsApi {
        TagsApi::new(self.clone())
    }

    pub fn user(&self) -> UserApi {
        UserApi::new(self.clone())
    }

    pub fn filters(&self) -> FiltersApi {
        FiltersApi::new(self.clone())
    }
}

#[derive(Debug, Default)]
pub struct RaindropClientBuilder {
    access_token: Option<String>,
    base_url: Option<String>,
    timeout: Option<Duration>,
    user_agent: Option<String>,
    custom_client: Option<reqwest::Client>,
}

impl RaindropClientBuilder {
    pub fn access_token(mut self, access_token: impl Into<String>) -> Self {
        self.access_token = Some(access_token.into());
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn reqwest_client(mut self, client: reqwest::Client) -> Self {
        self.custom_client = Some(client);
        self
    }

    pub fn build(self) -> Result<RaindropClient, Error> {
        let token = self
            .access_token
            .ok_or_else(|| Error::Auth("missing access token".to_string()))?;
        let bearer = format!("Bearer {token}");
        HeaderValue::from_str(&bearer)
            .map_err(|e| Error::Auth(format!("invalid bearer header: {e}")))?;

        let ua = self
            .user_agent
            .unwrap_or_else(|| DEFAULT_USER_AGENT.to_string());
        HeaderValue::from_str(&ua)
            .map_err(|e| Error::Auth(format!("invalid user-agent header: {e}")))?;

        let base = self
            .base_url
            .as_deref()
            .unwrap_or(DEFAULT_BASE_URL)
            .trim()
            .to_string();

        let base = if base.ends_with('/') {
            base
        } else {
            format!("{base}/")
        };

        let base_url = Url::parse(&base)?;

        let http = if let Some(client) = self.custom_client {
            client
        } else {
            let mut builder = reqwest::Client::builder();
            if let Some(timeout) = self.timeout {
                builder = builder.timeout(timeout);
            }
            builder.build().map_err(Error::Http)?
        };

        Ok(RaindropClient {
            inner: Arc::new(ClientInner {
                http,
                base_url,
                auth_header: bearer,
                user_agent: ua,
            }),
        })
    }
}
