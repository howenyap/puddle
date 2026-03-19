use crate::client::RaindropClient;
use crate::error::Error;
use crate::http::Response;
use crate::models::user::User;
use reqwest::Method;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone)]
pub struct UserApi {
    client: RaindropClient,
}

impl UserApi {
    pub(crate) fn new(client: RaindropClient) -> Self {
        Self { client }
    }

    pub async fn me(&self) -> Result<Response<User>, Error> {
        #[derive(Debug, Deserialize)]
        struct UserResponse {
            user: User,
            #[allow(dead_code)]
            #[serde(flatten)]
            extra: HashMap<String, serde_json::Value>,
        }

        let res = self
            .client
            .send_json::<UserResponse, (), ()>(Method::GET, "user", None, None)
            .await?;

        Ok(Response {
            data: res.data.user,
            meta: res.meta,
        })
    }
}
