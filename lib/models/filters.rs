use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiltersResponse {
    pub result: bool,
    #[serde(default)]
    pub broken: Option<FilterCount>,
    #[serde(default)]
    pub duplicates: Option<FilterCount>,
    #[serde(default)]
    pub important: Option<FilterCount>,
    #[serde(default)]
    pub notag: Option<FilterCount>,
    #[serde(default)]
    pub total: Option<FilterCount>,
    #[serde(default)]
    pub highlights: Option<FilterCount>,
    #[serde(default)]
    pub created: Vec<FilterBucket>,
    #[serde(default)]
    pub tags: Vec<FilterBucket>,
    #[serde(default)]
    pub types: Vec<FilterBucket>,
    #[serde(default, rename = "collectionId")]
    pub collection_id: Option<i64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCount {
    pub count: u32,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterBucket {
    #[serde(rename = "_id")]
    pub id: String,
    pub count: u32,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
