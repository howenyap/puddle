use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "$id", alias = "_id")]
    pub id: i64,
    pub email: Option<String>,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
