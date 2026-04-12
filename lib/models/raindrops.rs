use crate::models::common::CollectionScope;
use crate::pagination::{PageParams, PerPage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Raindrop {
    #[serde(rename = "_id")]
    pub id: RaindropId,
    pub title: Option<String>,
    pub link: Option<String>,
    pub excerpt: Option<String>,
    pub collection: Option<CollectionRef>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Raindrop {
    pub fn matches_scope(&self, scope: CollectionScope) -> bool {
        match scope {
            CollectionScope::All => true,
            CollectionScope::Id(id) => self.collection.as_ref().is_some_and(|value| value.id == id),
            CollectionScope::Unsorted | CollectionScope::Trash => self
                .collection
                .as_ref()
                .is_some_and(|value| value.id == i64::from(scope)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RaindropId(pub i64);

impl RaindropId {
    pub const fn new(id: i64) -> Self {
        Self(id)
    }

    pub const fn into_inner(self) -> i64 {
        self.0
    }
}

impl From<i64> for RaindropId {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl From<RaindropId> for i64 {
    fn from(id: RaindropId) -> Self {
        id.0
    }
}

impl Display for RaindropId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RaindropId {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<i64>()
            .map(Self)
            .map_err(|_| format!("invalid raindrop id: {value}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRef {
    #[serde(rename = "$id")]
    pub id: i64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl CollectionRef {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            extra: HashMap::new(),
        }
    }
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

    pub fn per_page(mut self, per_page: PerPage) -> Self {
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<CollectionRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeleteManyRaindrops {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<i64>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
