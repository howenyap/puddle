use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemResponse<T> {
    pub result: bool,
    pub item: T,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsResponse<T> {
    pub result: bool,
    pub items: Vec<T>,
    #[serde(default)]
    pub count: Option<u64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoolResponse {
    pub result: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedResponse {
    pub result: bool,
    pub modified: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdResponse {
    pub result: bool,
    pub item: IdValue,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdValue {
    #[serde(rename = "$id")]
    pub id: i64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionScope {
    Id(i64),
    #[default]
    All,
    Unsorted,
    Trash,
}

impl CollectionScope {
    pub fn id(id: i64) -> Result<Self, InvalidCollectionScopeId> {
        match id {
            0 | -1 | -99 => Err(InvalidCollectionScopeId(id)),
            _ => Ok(Self::Id(id)),
        }
    }
}

impl From<i64> for CollectionScope {
    fn from(id: i64) -> Self {
        match id {
            0 => Self::All,
            -1 => Self::Unsorted,
            -99 => Self::Trash,
            other => Self::Id(other),
        }
    }
}

impl From<CollectionScope> for i64 {
    fn from(scope: CollectionScope) -> Self {
        match scope {
            CollectionScope::Id(id) => id,
            CollectionScope::All => 0,
            CollectionScope::Unsorted => -1,
            CollectionScope::Trash => -99,
        }
    }
}

impl Display for CollectionScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => f.write_str("All"),
            Self::Unsorted => f.write_str("Unsorted"),
            Self::Trash => f.write_str("Trash"),
            Self::Id(id) => write!(f, "{id}"),
        }
    }
}

impl FromStr for CollectionScope {
    type Err = ParseCollectionScopeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "all" | "All" => Ok(Self::All),
            "unsorted" | "Unsorted" => Ok(Self::Unsorted),
            "trash" | "Trash" => Ok(Self::Trash),
            _ => {
                let id = value
                    .parse::<i64>()
                    .map_err(|_| ParseCollectionScopeError(value.to_string()))?;

                Ok(id.into())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCollectionScopeId(pub i64);

impl Display for InvalidCollectionScopeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "reserved collection id: {}", self.0)
    }
}

impl StdError for InvalidCollectionScopeId {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCollectionScopeError(pub String);

impl Display for ParseCollectionScopeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid collection id: {}", self.0)
    }
}

impl StdError for ParseCollectionScopeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_scope_id_rejects_reserved_ids() {
        assert_eq!(Err(InvalidCollectionScopeId(0)), CollectionScope::id(0));
        assert_eq!(Err(InvalidCollectionScopeId(-1)), CollectionScope::id(-1));
        assert_eq!(Err(InvalidCollectionScopeId(-99)), CollectionScope::id(-99));
    }

    #[test]
    fn collection_scope_maps_reserved_raw_ids_to_system_variants() {
        assert_eq!(CollectionScope::All, CollectionScope::from(0));
        assert_eq!(CollectionScope::Unsorted, CollectionScope::from(-1));
        assert_eq!(CollectionScope::Trash, CollectionScope::from(-99));
        assert_eq!(CollectionScope::Id(42), CollectionScope::from(42));
    }

    #[test]
    fn collection_scope_converts_to_wire_ids() {
        assert_eq!(0, i64::from(CollectionScope::All));
        assert_eq!(-1, i64::from(CollectionScope::Unsorted));
        assert_eq!(-99, i64::from(CollectionScope::Trash));
        assert_eq!(42, i64::from(CollectionScope::Id(42)));
    }

    #[test]
    fn collection_scope_displays_labels_for_system_variants() {
        assert_eq!("All", CollectionScope::All.to_string());
        assert_eq!("Unsorted", CollectionScope::Unsorted.to_string());
        assert_eq!("Trash", CollectionScope::Trash.to_string());
        assert_eq!("42", CollectionScope::Id(42).to_string());
    }

    #[test]
    fn collection_scope_parses_labels_and_numeric_ids() {
        assert_eq!(Ok(CollectionScope::All), "All".parse());
        assert_eq!(Ok(CollectionScope::Unsorted), "Unsorted".parse());
        assert_eq!(Ok(CollectionScope::Trash), "Trash".parse());
        assert_eq!(Ok(CollectionScope::Id(42)), "42".parse());
    }

    #[test]
    fn collection_scope_defaults_to_all() {
        assert_eq!(CollectionScope::All, CollectionScope::default());
    }
}
