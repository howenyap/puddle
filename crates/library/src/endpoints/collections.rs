use crate::client::RaindropClient;
use crate::error::Error;
use crate::http::Response;
use crate::models::collections::{Collection, CreateCollection, UpdateCollection};
use crate::models::common::{BoolResponse, CollectionScope, ItemResponse, ItemsResponse};
use mime::Mime;
use reqwest::Method;

const COLLECTIONS_CHILDREN_PATH: &str = "collections/childrens";

#[derive(Clone)]
pub struct CollectionsApi {
    client: RaindropClient,
}

impl CollectionsApi {
    pub(crate) fn new(client: RaindropClient) -> Self {
        Self { client }
    }

    pub async fn list_roots(&self) -> Result<Response<Vec<Collection>>, Error> {
        let res = self
            .client
            .send_json::<ItemsResponse<Collection>, (), ()>(Method::GET, "collections", None, None)
            .await?;

        Ok(Response {
            data: res.data.items,
            meta: res.meta,
        })
    }

    pub async fn get_root(&self) -> Result<Response<Collection>, Error> {
        self.get(CollectionScope::Unsorted).await
    }

    pub async fn get_children(&self) -> Result<Response<Vec<Collection>>, Error> {
        let res = self
            .client
            .send_json::<ItemsResponse<Collection>, (), ()>(
                Method::GET,
                COLLECTIONS_CHILDREN_PATH,
                None,
                None,
            )
            .await?;

        Ok(Response {
            data: res.data.items,
            meta: res.meta,
        })
    }

    pub async fn get(&self, id: CollectionScope) -> Result<Response<Collection>, Error> {
        let path = format!("collection/{}", id);
        let res = self
            .client
            .send_json::<ItemResponse<Collection>, (), ()>(Method::GET, &path, None, None)
            .await?;

        Ok(Response {
            data: res.data.item,
            meta: res.meta,
        })
    }

    pub async fn create(&self, payload: &CreateCollection) -> Result<Response<Collection>, Error> {
        let res = self
            .client
            .send_json::<ItemResponse<Collection>, (), _>(
                Method::POST,
                "collection",
                None,
                Some(payload),
            )
            .await?;

        Ok(Response {
            data: res.data.item,
            meta: res.meta,
        })
    }

    pub async fn update(
        &self,
        id: i64,
        payload: &UpdateCollection,
    ) -> Result<Response<Collection>, Error> {
        let path = format!("collection/{id}");
        let res = self
            .client
            .send_json::<ItemResponse<Collection>, (), _>(Method::PUT, &path, None, Some(payload))
            .await?;

        Ok(Response {
            data: res.data.item,
            meta: res.meta,
        })
    }

    pub async fn delete(&self, id: i64) -> Result<Response<bool>, Error> {
        let path = format!("collection/{id}");
        let res = self
            .client
            .send_json::<BoolResponse, (), ()>(Method::DELETE, &path, None, None)
            .await?;

        Ok(Response {
            data: res.data.result,
            meta: res.meta,
        })
    }

    pub async fn upload_cover(
        &self,
        id: i64,
        bytes: Vec<u8>,
        mime: Mime,
        file_name: &str,
    ) -> Result<Response<Collection>, Error> {
        let path = format!("collection/{id}/cover");
        let part = reqwest::multipart::Part::bytes(bytes)
            .mime_str(mime.as_ref())
            .map_err(|e| Error::Deserialize(e.to_string()))?
            .file_name(file_name.to_string());
        let form = reqwest::multipart::Form::new().part("cover", part);

        let res = self
            .client
            .send_multipart::<ItemResponse<Collection>, ()>(Method::PUT, &path, None, form)
            .await?;

        Ok(Response {
            data: res.data.item,
            meta: res.meta,
        })
    }
}
