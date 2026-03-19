use crate::client::RaindropClient;
use crate::error::Error;
use crate::http::Response;
use crate::models::filters::FiltersResponse;
use reqwest::Method;

#[derive(Clone)]
pub struct FiltersApi {
    client: RaindropClient,
}

impl FiltersApi {
    pub(crate) fn new(client: RaindropClient) -> Self {
        Self { client }
    }

    pub async fn list(&self, collection_id: i64) -> Result<Response<FiltersResponse>, Error> {
        let path = format!("filters/{collection_id}");
        let res = self
            .client
            .send_json::<FiltersResponse, (), ()>(Method::GET, &path, None, None)
            .await?;

        Ok(Response {
            data: res.data,
            meta: res.meta,
        })
    }
}
