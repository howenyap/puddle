use puddle::{
    Error, RaindropClient,
    models::{
        collections::{CreateCollection, UpdateCollection},
        common::CollectionScope,
        raindrops::{RaindropListParams, UpdateManyParams, UpdateManyRaindrops},
    },
};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn injects_auth_header_and_calls_user() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "result": true,
        "user": {"$id": 1, "email": "dev@example.com", "full_name": "Dev User"}
    });

    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let me = client.user().me().await.unwrap().data;
    assert_eq!(1, me.id);
    assert_eq!(Some("dev@example.com"), me.email.as_deref());
}

#[tokio::test]
async fn encodes_query_params_for_raindrop_list() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "result": true,
        "items": [
            {
                "_id": 42,
                "title": "Rust article",
                "link": "https://example.com/rust",
                "collection": { "$id": -1 },
                "tags": ["rust"]
            }
        ],
        "count": 1
    });

    Mock::given(method("GET"))
        .and(path("/raindrops/0"))
        .and(query_param("search", "rust"))
        .and(query_param("sort", "-created"))
        .and(query_param("nested", "true"))
        .and(query_param("page", "2"))
        .and(query_param("perpage", "25"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let params = RaindropListParams::new()
        .search("rust")
        .sort("-created")
        .nested(true)
        .page(2)
        .per_page(25);

    let list = client
        .raindrops()
        .list(CollectionScope::All, &params)
        .await
        .unwrap()
        .data;
    assert_eq!(Some(1), list.count);
    assert_eq!(1, list.items.len());
    assert_eq!(42, list.items[0].id);
    assert_eq!(Some("Rust article"), list.items[0].title.as_deref());
    assert_eq!(
        Some(-1),
        list.items[0].collection.as_ref().map(|value| value.id)
    );
}

#[tokio::test]
async fn raindrops_list_uses_system_collection_wire_id() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "result": true,
        "items": [],
        "count": 0
    });

    Mock::given(method("GET"))
        .and(path("/raindrops/-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let list = client
        .raindrops()
        .list(CollectionScope::Unsorted, &RaindropListParams::new())
        .await
        .unwrap()
        .data;

    assert_eq!(Some(0), list.count);
    assert_eq!(0, list.items.len());
}

#[tokio::test]
async fn raindrops_get_parses_item_envelope_with_underscore_id() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "result": true,
        "item": {
            "_id": 314,
            "title": "Detailed entry",
            "link": "https://example.com/detail",
            "collection": { "$id": 7 },
            "tags": ["cli", "rust"]
        }
    });

    Mock::given(method("GET"))
        .and(path("/raindrop/314"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let item = client.raindrops().get(314).await.unwrap().data;
    assert_eq!(314, item.id);
    assert_eq!(Some("Detailed entry"), item.title.as_deref());
    assert_eq!(Some("https://example.com/detail"), item.link.as_deref());
    assert_eq!(Some(7), item.collection.as_ref().map(|value| value.id));
    assert_eq!(vec!["cli", "rust"], item.tags);
}

#[tokio::test]
async fn maps_api_error_payload() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "result": false,
        "error": "not_found",
        "errorMessage": "collection not found"
    });

    Mock::given(method("GET"))
        .and(path("/collection/999"))
        .respond_with(ResponseTemplate::new(404).set_body_json(body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let err = client
        .collections()
        .get(CollectionScope::id(999).unwrap())
        .await
        .unwrap_err();
    match err {
        Error::Api {
            status,
            code,
            message,
            ..
        } => {
            assert_eq!(404, status.as_u16());
            assert_eq!(Some("not_found"), code.as_deref());
            assert_eq!("collection not found", message);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn collections_get_root_uses_special_root_collection_id() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "result": true,
        "item": {"_id": -1, "title": ""}
    });

    Mock::given(method("GET"))
        .and(path("/collection/-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let root = client.collections().get_root().await.unwrap().data;
    assert_eq!(-1, root.id);
    assert_eq!(Some(""), root.title.as_deref());
}

#[tokio::test]
async fn collections_get_uses_system_collection_wire_id() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "result": true,
        "item": {"_id": -99, "title": "Trash"}
    });

    Mock::given(method("GET"))
        .and(path("/collection/-99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let trash = client
        .collections()
        .get(CollectionScope::Trash)
        .await
        .unwrap()
        .data;
    assert_eq!(-99, trash.id);
    assert_eq!(Some("Trash"), trash.title.as_deref());
}

#[tokio::test]
async fn collections_list_roots_uses_plural_collections_path() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "result": true,
        "items": [
            {"_id": 10, "title": "Work"},
            {"_id": 11, "title": "Personal"}
        ]
    });

    Mock::given(method("GET"))
        .and(path("/collections"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let roots = client.collections().list_roots().await.unwrap().data;
    assert_eq!(2, roots.len());
    assert_eq!(10, roots[0].id);
    assert_eq!(Some("Work"), roots[0].title.as_deref());
    assert_eq!(11, roots[1].id);
}

#[tokio::test]
async fn collections_item_routes_use_singular_collection_path() {
    let server = MockServer::start().await;

    let get_body = serde_json::json!({
        "result": true,
        "item": {"_id": 123, "title": "existing"}
    });
    Mock::given(method("GET"))
        .and(path("/collection/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(get_body))
        .mount(&server)
        .await;

    let create_body = serde_json::json!({
        "result": true,
        "item": {"_id": 124, "title": "created"}
    });
    Mock::given(method("POST"))
        .and(path("/collection"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_body))
        .mount(&server)
        .await;

    let update_body = serde_json::json!({
        "result": true,
        "item": {"_id": 123, "title": "updated"}
    });
    Mock::given(method("PUT"))
        .and(path("/collection/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(update_body))
        .mount(&server)
        .await;

    let delete_body = serde_json::json!({
        "result": true
    });
    Mock::given(method("DELETE"))
        .and(path("/collection/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(delete_body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let got = client
        .collections()
        .get(CollectionScope::id(123).unwrap())
        .await
        .unwrap()
        .data;
    assert_eq!(123, got.id);

    let created = client
        .collections()
        .create(&CreateCollection {
            title: "created".to_string(),
            ..Default::default()
        })
        .await
        .unwrap()
        .data;
    assert_eq!(124, created.id);

    let updated = client
        .collections()
        .update(
            123,
            &UpdateCollection {
                title: Some("updated".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .data;
    assert_eq!(Some("updated"), updated.title.as_deref());

    let deleted = client.collections().delete(123).await.unwrap().data;
    assert!(deleted);
}

#[tokio::test]
async fn tags_list_and_get_parse_items_envelope() {
    let server = MockServer::start().await;

    let list_body = serde_json::json!({
        "result": true,
        "items": [{"_id": "api", "count": 100}]
    });
    Mock::given(method("GET"))
        .and(path("/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(list_body))
        .mount(&server)
        .await;

    let get_body = serde_json::json!({
        "result": true,
        "items": [{"_id": "rust", "count": 5}]
    });
    Mock::given(method("GET"))
        .and(path("/tags/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(get_body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let tags = client.tags().list().await.unwrap().data;
    assert_eq!(1, tags.len());
    assert_eq!("api", tags[0].id);
    assert_eq!(Some(100), tags[0].count);

    let scoped = client.tags().get(1).await.unwrap().data;
    assert_eq!(1, scoped.len());
    assert_eq!("rust", scoped[0].id);
    assert_eq!(Some(5), scoped[0].count);
}

#[tokio::test]
async fn tags_rename_and_delete_send_tags_array_body() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/tags/1"))
        .and(body_json(serde_json::json!({
            "replace": "backend",
            "tags": ["api"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"result": true})))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/tags/1"))
        .and(body_json(serde_json::json!({
            "tags": ["backend"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"result": true})))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    assert!(
        client
            .tags()
            .rename(1, "api", "backend")
            .await
            .unwrap()
            .data
    );
    assert!(client.tags().delete(1, "backend").await.unwrap().data);
}

#[tokio::test]
async fn filters_list_calls_scoped_route_and_parses_object_payload() {
    let server = MockServer::start().await;

    let filters_body = serde_json::json!({
        "result": true,
        "total": { "count": 1400 },
        "highlights": { "count": 59 },
        "notag": { "count": 1366 },
        "tags": [
            { "_id": "guides", "count": 9, "color": "blue" }
        ],
        "types": [
            { "_id": "article", "count": 313 }
        ],
        "created": [
            { "_id": "2026-03", "count": 10 }
        ],
        "collectionId": 1,
        "additionalField": "preserved"
    });

    Mock::given(method("GET"))
        .and(path("/filters/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(filters_body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let filters = client.filters().list(1).await.unwrap().data;
    assert!(filters.result);
    assert_eq!(Some(1400), filters.total.as_ref().map(|count| count.count));
    assert_eq!(
        Some(59),
        filters.highlights.as_ref().map(|count| count.count)
    );
    assert_eq!(Some(1366), filters.notag.as_ref().map(|count| count.count));
    assert_eq!(None, filters.broken.as_ref().map(|count| count.count));
    assert_eq!("guides", filters.tags[0].id);
    assert_eq!(9, filters.tags[0].count);
    assert_eq!("article", filters.types[0].id);
    assert_eq!(313, filters.types[0].count);
    assert_eq!("2026-03", filters.created[0].id);
    assert_eq!(Some(1), filters.collection_id);
}

#[tokio::test]
async fn upload_endpoints_use_put_routes() {
    let server = MockServer::start().await;

    let raindrop_body = serde_json::json!({
        "result": true,
        "item": {"_id": 55, "title": "uploaded"}
    });
    let collection_body = serde_json::json!({
        "result": true,
        "item": {"_id": 99, "title": "covered"}
    });

    Mock::given(method("PUT"))
        .and(path("/raindrop/file"))
        .respond_with(ResponseTemplate::new(200).set_body_json(raindrop_body.clone()))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/raindrop/55/cover"))
        .respond_with(ResponseTemplate::new(200).set_body_json(raindrop_body))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/collection/99/cover"))
        .respond_with(ResponseTemplate::new(200).set_body_json(collection_body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let uploaded = client
        .raindrops()
        .upload_file(b"hello".to_vec(), mime::TEXT_PLAIN, "hello.txt")
        .await
        .unwrap()
        .data;
    assert_eq!(55, uploaded.id);

    let covered = client
        .raindrops()
        .upload_cover(55, b"cover".to_vec(), mime::IMAGE_PNG, "cover.png")
        .await
        .unwrap()
        .data;
    assert_eq!(55, covered.id);

    let collection = client
        .collections()
        .upload_cover(99, b"cover".to_vec(), mime::IMAGE_PNG, "cover.png")
        .await
        .unwrap()
        .data;
    assert_eq!(99, collection.id);
}

#[tokio::test]
async fn update_many_sends_ids_in_json_body_and_reads_modified_count() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/raindrops/7"))
        .and(body_json(serde_json::json!({
            "ids": [10, 12],
            "tags": ["rust"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": true,
            "modified": 2
        })))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let modified = client
        .raindrops()
        .update_many(
            CollectionScope::id(7).unwrap(),
            &UpdateManyRaindrops {
                tags: Some(vec!["rust".to_string()]),
                ..Default::default()
            },
            Some(&UpdateManyParams {
                ids: Some("10,12".to_string()),
            }),
        )
        .await
        .unwrap()
        .data;

    assert_eq!(2, modified);
}

#[tokio::test]
async fn raindrop_collection_scope_maps_system_routes() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "result": true,
        "items": [],
        "count": 0
    });

    Mock::given(method("GET"))
        .and(path("/raindrops/-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body.clone()))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/raindrops/-99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let unsorted = client
        .raindrops()
        .list(CollectionScope::Unsorted, &RaindropListParams::new())
        .await
        .unwrap()
        .data;
    let trash = client
        .raindrops()
        .list(CollectionScope::Trash, &RaindropListParams::new())
        .await
        .unwrap()
        .data;

    assert_eq!(0, unsorted.items.len());
    assert_eq!(0, trash.items.len());
}

#[tokio::test]
async fn custom_reqwest_client_still_sends_auth_header() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("authorization", "Bearer custom-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "user": {"$id": 7, "email": "custom@example.com"}
        })))
        .mount(&server)
        .await;

    let raw_client = reqwest::Client::new();
    let client = RaindropClient::builder()
        .access_token("custom-token")
        .base_url(server.uri())
        .reqwest_client(raw_client)
        .build()
        .unwrap();

    let me = client.user().me().await.unwrap().data;
    assert_eq!(7, me.id);
}

#[tokio::test]
async fn maps_rate_limit_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("x-ratelimit-limit", "120")
                .insert_header("x-ratelimit-remaining", "0")
                .insert_header("x-ratelimit-reset", "1700000000")
                .set_body_string("{}"),
        )
        .mount(&server)
        .await;

    let client = RaindropClient::builder()
        .access_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let err = client.user().me().await.unwrap_err();
    match err {
        Error::RateLimited { rate_limit } => {
            assert_eq!(Some(120), rate_limit.limit);
            assert_eq!(Some(0), rate_limit.remaining);
            assert!(rate_limit.reset_at.is_some());
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
