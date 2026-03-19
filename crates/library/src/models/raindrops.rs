use crate::pagination::PageParams;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Raindrop {
    #[serde(rename = "_id")]
    pub id: i64,
    pub title: Option<String>,
    pub link: Option<String>,
    pub excerpt: Option<String>,
    pub collection: Option<CollectionRef>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRef {
    #[serde(rename = "$id")]
    pub id: i64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateRaindrop {
    pub link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<CollectionRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateRaindrop {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<CollectionRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RaindropListParams {
    #[serde(flatten)]
    pub page: PageParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nested: Option<bool>,
}

impl RaindropListParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page.page = Some(page);
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.page.per_page = Some(per_page);
        self
    }

    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    pub fn nested(mut self, nested: bool) -> Self {
        self.nested = Some(nested);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateManyRaindrops {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<CollectionRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateManyParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeleteManyRaindrops {
    #[serde(default)]
    pub ids: Vec<i64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeleteManyParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<String>,
}
