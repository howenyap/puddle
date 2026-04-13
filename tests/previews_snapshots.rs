mod common {
    pub(crate) fn display_text<'a>(value: Option<&'a str>, fallback: &'a str) -> &'a str {
        value
            .filter(|text| !text.trim().is_empty())
            .unwrap_or(fallback)
    }
}

mod raindrops {
    use puddle::models::common::CollectionScope;
    use puddle::models::raindrops::Raindrop;

    #[allow(dead_code)]
    #[derive(Debug, Clone)]
    pub(crate) struct UpdateManyArgs {
        pub(crate) ids: Vec<i64>,
        pub(crate) from_collection: Vec<CollectionScope>,
        pub(crate) exclude_collection: Vec<CollectionScope>,
        pub(crate) search: Option<String>,
        pub(crate) tags: Vec<String>,
        pub(crate) to_collection: Option<CollectionScope>,
    }

    #[derive(Debug, Clone)]
    pub(crate) struct ResolvedUpdateManySelection {
        pub(crate) targets: Vec<Raindrop>,
        pub(crate) from_collections: Vec<CollectionScope>,
        pub(crate) excluded_collections: Vec<CollectionScope>,
        pub(crate) search: Option<String>,
    }
}

mod tags {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct RenameTagArgs {
        pub(crate) collection_id: i64,
        pub(crate) find: String,
        pub(crate) replace: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct DeleteTagArgs {
        pub(crate) collection_id: i64,
        pub(crate) tag: String,
    }
}

#[path = "../src/previews.rs"]
mod previews;

use insta::assert_snapshot;
use previews::*;
use puddle::models::collections::{Collection, CreateCollection, UpdateCollection};
use puddle::models::common::CollectionScope;
use puddle::models::raindrops::{
    CollectionRef, CreateRaindrop, Raindrop, RaindropId, UpdateRaindrop,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn create_raindrop() {
    let payload = create_raindrop_payload();

    assert_snapshot!(
        "create_raindrop",
        CreateRaindropPreview::new(&payload).to_string()
    );
}

#[test]
fn update_raindrop() {
    let payload = update_raindrop_payload();

    assert_snapshot!(
        "update_raindrop",
        UpdateRaindropPreview::new(&full_raindrop(), &payload).to_string()
    );
}

#[test]
fn delete_raindrop() {
    assert_snapshot!(
        "delete_raindrop",
        DeleteRaindropPreview::new(&full_raindrop()).to_string()
    );
}

#[test]
fn upload_raindrop_file() {
    let path = report_path();

    assert_snapshot!(
        "upload_raindrop_file",
        UploadRaindropFilePreview::new(&path).to_string()
    );
}

#[test]
fn upload_raindrop_cover() {
    let path = report_path();

    assert_snapshot!(
        "upload_raindrop_cover",
        UploadRaindropCoverPreview::new(&full_raindrop(), &path).to_string()
    );
}

#[test]
fn create_many_raindrops() {
    let input = PathBuf::from("fixtures/create-many.json");
    let payloads = vec![
        create_raindrop_payload(),
        CreateRaindrop {
            link: "https://example.com/untitled".to_string(),
            title: None,
            excerpt: None,
            collection: None,
            tags: Vec::new(),
            extra: HashMap::new(),
        },
    ];

    assert_snapshot!(
        "create_many_raindrops",
        CreateManyRaindropsPreview::new(&input, &payloads).to_string()
    );
}

#[test]
fn update_many_raindrops() {
    let selection = selection_with_scope();
    let args = update_many_args();

    assert_snapshot!(
        "update_many_raindrops",
        UpdateManyRaindropsPreview::new(&selection, &args).to_string()
    );
}

#[test]
fn delete_many_raindrops() {
    let mut second = full_raindrop();
    second.id = RaindropId::new(43);
    second.title = Some("Server notes".to_string());
    second.link = Some("https://example.com/server".to_string());
    let targets = vec![full_raindrop(), second];

    assert_snapshot!(
        "delete_many_raindrops",
        DeleteManyRaindropsPreview::new(CollectionScope::Unsorted, &targets).to_string()
    );
}

#[test]
fn create_collection() {
    let payload = create_collection_payload();

    assert_snapshot!(
        "create_collection",
        CreateCollectionPreview::new(&payload).to_string()
    );
}

#[test]
fn update_collection() {
    let payload = update_collection_payload();

    assert_snapshot!(
        "update_collection",
        UpdateCollectionPreview::new(&full_collection(), &payload).to_string()
    );
}

#[test]
fn delete_collection() {
    assert_snapshot!(
        "delete_collection",
        DeleteCollectionPreview::new(&full_collection()).to_string()
    );
}

#[test]
fn upload_collection_cover() {
    let path = report_path();

    assert_snapshot!(
        "upload_collection_cover",
        UploadCollectionCoverPreview::new(&full_collection(), &path).to_string()
    );
}

#[test]
fn rename_tag() {
    let args = rename_tag_args();

    assert_snapshot!("rename_tag", RenameTagPreview::new(&args).to_string());
}

#[test]
fn delete_tag() {
    let args = delete_tag_args();

    assert_snapshot!("delete_tag", DeleteTagPreview::new(&args).to_string());
}

fn full_raindrop() -> Raindrop {
    Raindrop {
        id: RaindropId::new(42),
        title: Some("Rust article".to_string()),
        link: Some("https://example.com/rust".to_string()),
        excerpt: Some("Original excerpt".to_string()),
        collection: Some(CollectionRef::new(-1)),
        tags: vec!["rust".to_string(), "cli".to_string()],
        extra: HashMap::new(),
    }
}

fn create_raindrop_payload() -> CreateRaindrop {
    CreateRaindrop {
        link: "https://example.com/create".to_string(),
        title: Some("Saved article".to_string()),
        excerpt: Some("Queued for later".to_string()),
        collection: Some(CollectionRef::new(7)),
        tags: vec!["rust".to_string(), "backend".to_string()],
        extra: HashMap::new(),
    }
}

fn update_raindrop_payload() -> UpdateRaindrop {
    UpdateRaindrop {
        title: Some("Better title".to_string()),
        excerpt: Some("Updated excerpt".to_string()),
        collection: Some(CollectionRef::new(7)),
        tags: Some(vec!["backend".to_string(), "docs".to_string()]),
        extra: HashMap::new(),
    }
}

fn full_collection() -> Collection {
    Collection {
        id: 12,
        title: Some("Reading list".to_string()),
        parent: Some(5),
        count: Some(9),
        extra: HashMap::new(),
    }
}

fn create_collection_payload() -> CreateCollection {
    CreateCollection {
        title: "Architecture".to_string(),
        parent: Some(5),
        extra: HashMap::new(),
    }
}

fn update_collection_payload() -> UpdateCollection {
    UpdateCollection {
        title: Some("Architecture notes".to_string()),
        parent: Some(8),
        extra: HashMap::new(),
    }
}

fn selection_with_scope() -> raindrops::ResolvedUpdateManySelection {
    raindrops::ResolvedUpdateManySelection {
        targets: vec![
            full_raindrop(),
            Raindrop {
                id: RaindropId::new(43),
                title: None,
                link: Some("https://example.com/untitled".to_string()),
                excerpt: Some("Second excerpt".to_string()),
                collection: Some(CollectionRef::new(11)),
                tags: vec!["inbox".to_string()],
                extra: HashMap::new(),
            },
        ],
        from_collections: vec![CollectionScope::Unsorted, CollectionScope::Id(7)],
        excluded_collections: vec![CollectionScope::Trash],
        search: Some("rust async".to_string()),
    }
}

fn update_many_args() -> raindrops::UpdateManyArgs {
    raindrops::UpdateManyArgs {
        ids: vec![42, 43],
        from_collection: vec![CollectionScope::Unsorted, CollectionScope::Id(7)],
        exclude_collection: vec![CollectionScope::Trash],
        search: Some("rust async".to_string()),
        tags: vec!["backend".to_string(), "docs".to_string()],
        to_collection: Some(CollectionScope::Id(7)),
    }
}

fn rename_tag_args() -> tags::RenameTagArgs {
    tags::RenameTagArgs {
        collection_id: 7,
        find: "api".to_string(),
        replace: "backend".to_string(),
    }
}

fn delete_tag_args() -> tags::DeleteTagArgs {
    tags::DeleteTagArgs {
        collection_id: 7,
        tag: "deprecated".to_string(),
    }
}

fn report_path() -> PathBuf {
    PathBuf::from("fixtures/report.pdf")
}
