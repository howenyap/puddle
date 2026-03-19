use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    #[serde(rename = "_id")]
    pub id: i64,
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_parent_id")]
    pub parent: Option<i64>,
    pub count: Option<u32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateCollection {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<i64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCollection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<i64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ParentValue {
    Id(i64),
    Ref {
        #[serde(rename = "$id")]
        id: i64,
    },
}

fn deserialize_parent_id<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<ParentValue>::deserialize(deserializer)?;

    Ok(value.map(|value| match value {
        ParentValue::Id(id) => id,
        ParentValue::Ref { id } => id,
    }))
}
