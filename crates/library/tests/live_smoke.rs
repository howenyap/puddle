use puddle::{RaindropClient, models::common::CollectionScope};

#[tokio::test]
async fn live_smoke_test() {
    let Ok(token) = std::env::var("RAINDROP_TEST_TOKEN") else {
        eprintln!("skipping live smoke test: RAINDROP_TEST_TOKEN not set");
        return;
    };

    let client = RaindropClient::new(token).expect("client should build");

    let _user = client
        .user()
        .me()
        .await
        .expect("user.me should succeed")
        .data;

    let _list = client
        .raindrops()
        .list(
            CollectionScope::All,
            &puddle::models::raindrops::RaindropListParams::new().per_page(1),
        )
        .await
        .expect("raindrops.list should succeed")
        .data;
}
