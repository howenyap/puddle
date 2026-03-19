use crate::client::RaindropClient;
use crate::error::Error;
use crate::http::Response;
use crate::models::common::{BoolResponse, ItemsResponse};
use crate::models::tags::Tag;
use reqwest::Method;

#[derive(Clone)]
pub struct TagsApi {
    client: RaindropClient,
}

impl TagsApi {
    pub(crate) fn new(client: RaindropClient) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<Response<Vec<Tag>>, Error> {
        let res = self
            .client
            .send_json::<ItemsResponse<Tag>, (), ()>(Method::GET, "tags", None, None)
            .await?;

        Ok(Response {
            data: res.data.items,
            meta: res.meta,
        })
    }

    pub async fn get(&self, collection_id: i64) -> Result<Response<Vec<Tag>>, Error> {
        let path = format!("tags/{collection_id}");
        let res = self
            .client
            .send_json::<ItemsResponse<Tag>, (), ()>(Method::GET, &path, None, None)
            .await?;

        Ok(Response {
            data: res.data.items,
            meta: res.meta,
        })
    }

    pub async fn rename(
        &self,
        collection_id: i64,
        find: &str,
        replace: &str,
    ) -> Result<Response<bool>, Error> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            replace: &'a str,
            tags: [&'a str; 1],
        }
        let path = format!("tags/{collection_id}");
        let res = self
            .client
            .send_json::<BoolResponse, (), _>(
                Method::PUT,
                &path,
                None,
                Some(&Body {
                    replace,
                    tags: [find],
                }),
            )
            .await?;

        Ok(Response {
            data: res.data.result,
            meta: res.meta,
        })
    }

    pub async fn delete(&self, collection_id: i64, tag: &str) -> Result<Response<bool>, Error> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            tags: [&'a str; 1],
        }
        let path = format!("tags/{collection_id}");
        let res = self
            .client
            .send_json::<BoolResponse, (), _>(
                Method::DELETE,
                &path,
                None,
                Some(&Body { tags: [tag] }),
            )
            .await?;

        Ok(Response {
            data: res.data.result,
            meta: res.meta,
        })
    }
}
