use crate::client::RaindropClient;
use crate::error::Error;
use crate::http::Response;
use crate::models::common::{
    BoolResponse, CollectionScope, ItemResponse, ItemsResponse, ModifiedResponse,
};
use crate::models::raindrops::{
    CreateRaindrop, DeleteManyParams, DeleteManyRaindrops, Raindrop, RaindropListParams,
    UpdateManyParams, UpdateManyRaindrops, UpdateRaindrop,
};
use mime::Mime;
use reqwest::Method;

#[derive(Clone)]
pub struct RaindropsApi {
    client: RaindropClient,
}

impl RaindropsApi {
    pub(crate) fn new(client: RaindropClient) -> Self {
        Self { client }
    }

    pub async fn get(&self, id: i64) -> Result<Response<Raindrop>, Error> {
        let path = format!("raindrop/{id}");
        let res = self
            .client
            .send_json::<ItemResponse<Raindrop>, (), ()>(Method::GET, &path, None, None)
            .await?;

        Ok(Response {
            data: res.data.item,
            meta: res.meta,
        })
    }

    pub async fn create(&self, payload: &CreateRaindrop) -> Result<Response<Raindrop>, Error> {
        let res = self
            .client
            .send_json::<ItemResponse<Raindrop>, (), _>(
                Method::POST,
                "raindrop",
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
        payload: &UpdateRaindrop,
    ) -> Result<Response<Raindrop>, Error> {
        let path = format!("raindrop/{id}");
        let res = self
            .client
            .send_json::<ItemResponse<Raindrop>, (), _>(Method::PUT, &path, None, Some(payload))
            .await?;

        Ok(Response {
            data: res.data.item,
            meta: res.meta,
        })
    }

    pub async fn delete(&self, id: i64) -> Result<Response<bool>, Error> {
        let path = format!("raindrop/{id}");
        let res = self
            .client
            .send_json::<BoolResponse, (), ()>(Method::DELETE, &path, None, None)
            .await?;

        Ok(Response {
            data: res.data.result,
            meta: res.meta,
        })
    }

    pub async fn upload_file(
        &self,
        bytes: Vec<u8>,
        mime: Mime,
        file_name: &str,
    ) -> Result<Response<Raindrop>, Error> {
        let part = reqwest::multipart::Part::bytes(bytes)
            .mime_str(mime.as_ref())
            .map_err(|e| Error::Deserialize(e.to_string()))?
            .file_name(file_name.to_string());
        let form = reqwest::multipart::Form::new().part("file", part);

        let res = self
            .client
            .send_multipart::<ItemResponse<Raindrop>, ()>(Method::PUT, "raindrop/file", None, form)
            .await?;

        Ok(Response {
            data: res.data.item,
            meta: res.meta,
        })
    }

    pub async fn upload_cover(
        &self,
        id: i64,
        bytes: Vec<u8>,
        mime: Mime,
        file_name: &str,
    ) -> Result<Response<Raindrop>, Error> {
        let path = format!("raindrop/{id}/cover");
        let part = reqwest::multipart::Part::bytes(bytes)
            .mime_str(mime.as_ref())
            .map_err(|e| Error::Deserialize(e.to_string()))?
            .file_name(file_name.to_string());
        let form = reqwest::multipart::Form::new().part("cover", part);

        let res = self
            .client
            .send_multipart::<ItemResponse<Raindrop>, ()>(Method::PUT, &path, None, form)
            .await?;

        Ok(Response {
            data: res.data.item,
            meta: res.meta,
        })
    }

    pub async fn list(
        &self,
        collection_id: CollectionScope,
        params: &RaindropListParams,
    ) -> Result<Response<ItemsResponse<Raindrop>>, Error> {
        let path = format!("raindrops/{}", collection_id);
        let res = self
            .client
            .send_json::<ItemsResponse<Raindrop>, _, ()>(Method::GET, &path, Some(params), None)
            .await?;

        Ok(Response {
            data: res.data,
            meta: res.meta,
        })
    }

    pub async fn create_many(
        &self,
        items: &[CreateRaindrop],
    ) -> Result<Response<Vec<Raindrop>>, Error> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            items: &'a [CreateRaindrop],
        }

        let res = self
            .client
            .send_json::<ItemsResponse<Raindrop>, (), _>(
                Method::POST,
                "raindrops",
                None,
                Some(&Body { items }),
            )
            .await?;

        Ok(Response {
            data: res.data.items,
            meta: res.meta,
        })
    }

    pub async fn update_many(
        &self,
        collection_id: CollectionScope,
        payload: &UpdateManyRaindrops,
        params: Option<&UpdateManyParams>,
    ) -> Result<Response<u64>, Error> {
        let path = format!("raindrops/{}", collection_id);
        #[derive(serde::Serialize)]
        struct Body<'a> {
            ids: Vec<i64>,
            #[serde(flatten)]
            payload: &'a UpdateManyRaindrops,
        }

        let ids = params
            .and_then(|params| params.ids.as_deref())
            .map(|ids| {
                ids.split(',')
                    .filter_map(|id| id.trim().parse::<i64>().ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let body = Body { ids, payload };
        let res = self
            .client
            .send_json::<ModifiedResponse, (), _>(Method::PUT, &path, None, Some(&body))
            .await?;

        Ok(Response {
            data: res.data.modified,
            meta: res.meta,
        })
    }

    pub async fn delete_many(
        &self,
        collection_id: CollectionScope,
        payload: &DeleteManyRaindrops,
        params: Option<&DeleteManyParams>,
    ) -> Result<Response<bool>, Error> {
        let path = format!("raindrops/{}", collection_id);
        let res = self
            .client
            .send_json::<BoolResponse, _, _>(Method::DELETE, &path, params, Some(payload))
            .await?;

        Ok(Response {
            data: res.data.result,
            meta: res.meta,
        })
    }
}
