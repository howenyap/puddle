use crate::app::CliApp;
use crate::common::{display_text, read_upload_file};
use clap::Args;
use puddle::RaindropClient;
use puddle::models::common::{CollectionScope, ItemsResponse};
use puddle::models::raindrops::{
    CollectionRef, CreateRaindrop, DeleteManyRaindrops, Raindrop, RaindropListParams,
    UpdateManyParams, UpdateManyRaindrops, UpdateRaindrop,
};
use puddle::pagination::{MAX_PER_PAGE, PageParams};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CliPage(u32);

impl CliPage {
    fn as_display(self) -> u32 {
        self.0
    }

    fn to_api_page(self) -> u32 {
        self.0 - 1
    }
}

impl FromStr for CliPage {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let page = value
            .parse::<u32>()
            .map_err(|_| format!("invalid value '{value}' for '--page <PAGE>'"))?;

        if page == 0 {
            return Err("page must be at least 1".to_string());
        }

        Ok(Self(page))
    }
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct ListArgs {
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) collection: Option<CollectionScope>,
    #[arg(long)]
    pub(crate) search: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) sort: Option<String>,
    #[arg(long)]
    pub(crate) page: Option<CliPage>,
    #[arg(long = "per-page")]
    pub(crate) per_page: Option<u32>,
    #[arg(long)]
    pub(crate) nested: bool,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct GetArgs {
    pub(crate) id: i64,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct CreateArgs {
    pub(crate) link: String,
    #[arg(long)]
    pub(crate) title: Option<String>,
    #[arg(long)]
    pub(crate) excerpt: Option<String>,
    #[arg(long = "tag")]
    pub(crate) tags: Vec<String>,
    #[arg(long)]
    pub(crate) collection: Option<i64>,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct UpdateArgs {
    pub(crate) id: i64,
    #[arg(long)]
    pub(crate) title: Option<String>,
    #[arg(long)]
    pub(crate) excerpt: Option<String>,
    #[arg(long = "tag")]
    pub(crate) tags: Vec<String>,
    #[arg(long)]
    pub(crate) collection: Option<i64>,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct DeleteArgs {
    pub(crate) id: i64,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct UploadFileArgs {
    pub(crate) path: PathBuf,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct UploadCoverArgs {
    pub(crate) id: i64,
    pub(crate) path: PathBuf,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct CreateManyArgs {
    #[arg(
        long,
        help = "Path to a JSON file containing an array of raindrop payloads"
    )]
    pub(crate) input: PathBuf,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct UpdateManyArgs {
    #[arg(long, allow_hyphen_values = true, default_value_t = CollectionScope::default())]
    pub(crate) collection_id: CollectionScope,
    #[arg(long = "id", required = true)]
    pub(crate) ids: Vec<i64>,
    #[arg(long = "tag")]
    pub(crate) tags: Vec<String>,
    #[arg(long)]
    pub(crate) collection: Option<i64>,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct DeleteManyArgs {
    #[arg(long, allow_hyphen_values = true, default_value_t = CollectionScope::default())]
    pub(crate) collection_id: CollectionScope,
    #[arg(long = "id", required = true)]
    pub(crate) ids: Vec<i64>,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct ExportArgs {
    #[arg(
        long,
        default_value = "export.json",
        help = "Path to write the exported bookmarks JSON"
    )]
    pub(crate) output: PathBuf,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) collection: Option<CollectionScope>,
}

impl CliApp {
    pub(crate) async fn list(&self, args: ListArgs) -> Result<(), Box<dyn std::error::Error>> {
        let current_page = args.page.map(CliPage::as_display).unwrap_or(1);
        let params = RaindropListParams {
            page: PageParams {
                page: args.page.map(CliPage::to_api_page),
                per_page: args.per_page,
            },
            search: args.search,
            sort: args.sort,
            nested: args.nested.then_some(true),
        };

        let collection_id = args.collection.unwrap_or_default();
        let response = self.client.raindrops().list(collection_id, &params).await?;
        print_raindrop_list(&response.data, current_page);

        Ok(())
    }

    pub(crate) async fn get(&self, args: GetArgs) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.raindrops().get(args.id).await?;
        println!("{}", RaindropDetailDisplay(&response.data));

        Ok(())
    }

    pub(crate) async fn create(&self, args: CreateArgs) -> Result<(), Box<dyn std::error::Error>> {
        let payload = create_payload(args);
        let response = self.client.raindrops().create(&payload).await?;
        println!("{}", RaindropDetailDisplay(&response.data));

        Ok(())
    }

    pub(crate) async fn update(&self, args: UpdateArgs) -> Result<(), Box<dyn std::error::Error>> {
        let payload = UpdateRaindrop {
            title: args.title,
            excerpt: args.excerpt,
            collection: args.collection.map(collection_ref),
            tags: (!args.tags.is_empty()).then_some(args.tags),
            extra: HashMap::new(),
        };
        let response = self.client.raindrops().update(args.id, &payload).await?;
        println!("{}", RaindropDetailDisplay(&response.data));

        Ok(())
    }

    pub(crate) async fn delete(&self, args: DeleteArgs) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.raindrops().delete(args.id).await?;
        println!("deleted: {}", response.data);

        Ok(())
    }

    pub(crate) async fn upload_file(
        &self,
        args: UploadFileArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (bytes, mime, file_name) = read_upload_file(&args.path)?;
        let response = self
            .client
            .raindrops()
            .upload_file(bytes, mime, &file_name)
            .await?;
        println!("{}", RaindropDetailDisplay(&response.data));

        Ok(())
    }

    pub(crate) async fn upload_cover(
        &self,
        args: UploadCoverArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (bytes, mime, file_name) = read_upload_file(&args.path)?;
        let response = self
            .client
            .raindrops()
            .upload_cover(args.id, bytes, mime, &file_name)
            .await?;
        println!("{}", RaindropDetailDisplay(&response.data));

        Ok(())
    }

    pub(crate) async fn create_many(
        &self,
        args: CreateManyArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let input = std::fs::read_to_string(&args.input)?;
        let payloads: Vec<CreateRaindrop> = serde_json::from_str(&input)?;
        let response = self.client.raindrops().create_many(&payloads).await?;

        if response.data.is_empty() {
            println!("No raindrops created.");
            return Ok(());
        }

        for item in &response.data {
            println!("{}", format_raindrop_summary(item));
        }

        Ok(())
    }

    pub(crate) async fn update_many(
        &self,
        args: UpdateManyArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = UpdateManyRaindrops {
            collection: args.collection.map(collection_ref),
            tags: (!args.tags.is_empty()).then_some(args.tags),
            extra: HashMap::new(),
        };
        let params = UpdateManyParams {
            ids: Some(
                args.ids
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        };
        let response = self
            .client
            .raindrops()
            .update_many(args.collection_id, &payload, Some(&params))
            .await?;
        println!("modified: {}", response.data);
        println!(
            "ids: {}",
            args.ids
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );

        Ok(())
    }

    pub(crate) async fn delete_many(
        &self,
        args: DeleteManyArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = DeleteManyRaindrops {
            ids: args.ids.clone(),
            extra: HashMap::new(),
        };
        let response = self
            .client
            .raindrops()
            .delete_many(args.collection_id, &payload, None)
            .await?;
        println!("deleted: {}", response.data);

        Ok(())
    }

    pub(crate) async fn export(&self, args: ExportArgs) -> Result<(), Box<dyn std::error::Error>> {
        let collection_id = args.collection.unwrap_or_default();
        let items = fetch_all_raindrops(&self.client, collection_id).await?;
        write_raindrop_export(&args.output, &items)?;
        println!(
            "exported {} raindrops to {}",
            items.len(),
            args.output.display()
        );

        Ok(())
    }
}

fn create_payload(args: CreateArgs) -> CreateRaindrop {
    CreateRaindrop {
        link: args.link,
        title: args.title,
        excerpt: args.excerpt,
        collection: args.collection.map(collection_ref),
        tags: args.tags,
        extra: HashMap::new(),
    }
}

fn collection_ref(id: i64) -> CollectionRef {
    CollectionRef {
        id,
        extra: HashMap::new(),
    }
}

fn print_raindrop_list(items: &ItemsResponse<Raindrop>, current_page: u32) {
    if items.items.is_empty() {
        println!("No raindrops found.");
        return;
    }

    println!("Listed raindrops: {}", items.items.len());
    println!("{}", format_next_page_hint(current_page));
    println!();

    for (index, item) in items.items.iter().enumerate() {
        if index > 0 {
            println!();
        }

        println!("{}", format_raindrop_list_item(item));
    }
}

fn format_next_page_hint(current_page: u32) -> String {
    let previous_page = current_page.saturating_sub(1);
    let next_page = current_page + 1;

    if current_page > 1 {
        format!(
            "Hint: use\n`--page {previous_page}` for the previous page\n`--page {next_page}` for the next page"
        )
    } else {
        format!("Hint: use\n`--page {next_page}` for the next page")
    }
}

fn format_raindrop_list_item(item: &Raindrop) -> String {
    [
        format!("id: {}", item.id),
        format!(
            "title: {}",
            display_text(item.title.as_deref(), "(untitled)")
        ),
        format!("link: {}", display_text(item.link.as_deref(), "(none)")),
        format!(
            "tags: {}",
            if item.tags.is_empty() {
                "(none)".to_string()
            } else {
                item.tags.join(", ")
            }
        ),
    ]
    .join("\n")
}

struct RaindropDetailDisplay<'a>(&'a Raindrop);

impl Display for RaindropDetailDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let item = self.0;
        writeln!(f, "id: {}", item.id)?;
        writeln!(
            f,
            "title: {}",
            display_text(item.title.as_deref(), "(untitled)")
        )?;
        write!(f, "link: {}", display_text(item.link.as_deref(), "(none)"))?;

        if let Some(excerpt) = item.excerpt.as_deref() {
            write!(f, "\nexcerpt: {excerpt}")?;
        }
        if let Some(collection) = item.collection.as_ref() {
            write!(f, "\ncollection: {}", collection.id)?;
        }
        if !item.tags.is_empty() {
            write!(f, "\ntags: {}", item.tags.join(", "))?;
        }

        Ok(())
    }
}

fn format_raindrop_summary(item: &Raindrop) -> String {
    let mut parts = vec![
        format!("#{}", item.id),
        display_text(item.title.as_deref(), "(untitled)").to_string(),
        display_text(item.link.as_deref(), "(no link)").to_string(),
    ];

    if let Some(collection) = item.collection.as_ref() {
        parts.push(format!("collection={}", collection.id));
    }
    if !item.tags.is_empty() {
        parts.push(format!("tags=[{}]", item.tags.join(", ")));
    }

    parts.join(" | ")
}

async fn fetch_all_raindrops(
    client: &RaindropClient,
    collection_id: CollectionScope,
) -> Result<Vec<Raindrop>, Box<dyn std::error::Error>> {
    let mut page = 0;
    let mut all_items = Vec::new();

    loop {
        let params = RaindropListParams::new().page(page).per_page(MAX_PER_PAGE);
        let response = client
            .raindrops()
            .list(collection_id, &params)
            .await?;
        let mut items = response.data.items;
        let item_count = items.len();

        all_items.append(&mut items);

        if item_count < MAX_PER_PAGE as usize {
            break;
        }

        page += 1;
    }

    Ok(all_items)
}

fn write_raindrop_export(
    output_path: &std::path::Path,
    items: &[Raindrop],
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(items)?;
    std::fs::write(output_path, format!("{json}\n"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Command};
    use clap::Parser;
    use std::collections::HashMap;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn parses_top_level_list_command() {
        let cli = Cli::try_parse_from([
            "puddle",
            "list",
            "--search",
            "rust",
            "--sort",
            "-created",
            "--page",
            "2",
            "--per-page",
            "25",
            "--nested",
        ])
        .unwrap();

        assert_eq!(
            Some(Command::List(ListArgs {
                collection: None,
                search: Some("rust".to_string()),
                sort: Some("-created".to_string()),
                page: Some(CliPage(2)),
                per_page: Some(25),
                nested: true,
            })),
            cli.command
        );
    }

    #[test]
    fn parses_top_level_create_command() {
        let cli = Cli::try_parse_from([
            "puddle",
            "create",
            "https://example.com",
            "--title",
            "Example",
            "--excerpt",
            "Notes",
            "--tag",
            "rust",
            "--tag",
            "cli",
            "--collection",
            "42",
        ])
        .unwrap();

        assert_eq!(
            Some(Command::Create(CreateArgs {
                link: "https://example.com".to_string(),
                title: Some("Example".to_string()),
                excerpt: Some("Notes".to_string()),
                tags: vec!["rust".to_string(), "cli".to_string()],
                collection: Some(42),
            })),
            cli.command
        );
    }

    #[test]
    fn parses_top_level_export_command() {
        let cli = Cli::try_parse_from([
            "puddle",
            "export",
            "--collection",
            "42",
            "--output",
            "bookmarks.json",
        ])
        .unwrap();

        assert_eq!(
            Some(Command::Export(ExportArgs {
                output: PathBuf::from("bookmarks.json"),
                collection: Some(CollectionScope::id(42).unwrap()),
            })),
            cli.command
        );
    }

    #[test]
    fn export_defaults_output_path() {
        let cli = Cli::try_parse_from(["puddle", "export"]).unwrap();

        assert_eq!(
            Some(Command::Export(ExportArgs {
                output: PathBuf::from("export.json"),
                collection: None,
            })),
            cli.command
        );
    }

    #[test]
    fn parses_named_collection_scope_for_export() {
        let cli = Cli::try_parse_from(["puddle", "export", "--collection", "Unsorted"]).unwrap();

        assert_eq!(
            Some(Command::Export(ExportArgs {
                output: PathBuf::from("export.json"),
                collection: Some(CollectionScope::Unsorted),
            })),
            cli.command
        );
    }

    #[test]
    fn export_defaults_to_all_collection() {
        let args = ExportArgs {
            output: PathBuf::from("export.json"),
            collection: None,
        };

        assert_eq!(CollectionScope::All, args.collection.unwrap_or_default());
    }

    #[test]
    fn parses_top_level_update_many_command() {
        let cli = Cli::try_parse_from([
            "puddle",
            "update-many",
            "--collection-id",
            "5",
            "--id",
            "10",
            "--id",
            "12",
            "--tag",
            "rust",
            "--collection",
            "7",
        ])
        .unwrap();

        assert_eq!(
            Some(Command::UpdateMany(UpdateManyArgs {
                collection_id: CollectionScope::id(5).unwrap(),
                ids: vec![10, 12],
                tags: vec!["rust".to_string()],
                collection: Some(7),
            })),
            cli.command
        );
    }

    #[test]
    fn list_defaults_to_all_collection() {
        let args = ListArgs {
            collection: None,
            search: None,
            sort: None,
            page: None,
            per_page: None,
            nested: false,
        };

        assert_eq!(CollectionScope::All, args.collection.unwrap_or_default());
    }

    #[test]
    fn parses_system_collection_scope_for_list() {
        let cli = Cli::try_parse_from(["puddle", "list", "--collection", "-1"]).unwrap();

        assert_eq!(
            Some(Command::List(ListArgs {
                collection: Some(CollectionScope::Unsorted),
                search: None,
                sort: None,
                page: None,
                per_page: None,
                nested: false,
            })),
            cli.command
        );
    }

    #[test]
    fn parses_named_collection_scope_for_list() {
        let cli = Cli::try_parse_from(["puddle", "list", "--collection", "Unsorted"]).unwrap();

        assert_eq!(
            Some(Command::List(ListArgs {
                collection: Some(CollectionScope::Unsorted),
                search: None,
                sort: None,
                page: None,
                per_page: None,
                nested: false,
            })),
            cli.command
        );
    }

    #[test]
    fn parses_numeric_collection_scope_for_list() {
        let cli = Cli::try_parse_from(["puddle", "list", "--collection", "42"]).unwrap();

        assert_eq!(
            Some(Command::List(ListArgs {
                collection: Some(CollectionScope::id(42).unwrap()),
                search: None,
                sort: None,
                page: None,
                per_page: None,
                nested: false,
            })),
            cli.command
        );
    }

    #[test]
    fn create_payload_preserves_optional_fields() {
        let args = CreateArgs {
            link: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            excerpt: Some("Summary".to_string()),
            tags: vec!["rust".to_string(), "cli".to_string()],
            collection: Some(7),
        };

        let payload = create_payload(args);

        assert_eq!("https://example.com", payload.link);
        assert_eq!(Some("Example"), payload.title.as_deref());
        assert_eq!(Some("Summary"), payload.excerpt.as_deref());
        assert_eq!(Some(7), payload.collection.as_ref().map(|value| value.id));
        assert_eq!(vec!["rust", "cli"], payload.tags);
    }

    #[test]
    fn formats_list_item_with_requested_fields() {
        let item = Raindrop {
            id: 42,
            title: Some("Rust article".to_string()),
            link: Some("https://example.com".to_string()),
            excerpt: None,
            collection: Some(CollectionRef {
                id: 7,
                extra: HashMap::new(),
            }),
            tags: vec!["rust".to_string(), "cli".to_string()],
            extra: HashMap::new(),
        };

        let expected = [
            "id: 42",
            "title: Rust article",
            "link: https://example.com",
            "tags: rust, cli",
        ]
        .join("\n");

        let actual = format_raindrop_list_item(&item);

        assert_eq!(expected, actual);
    }

    #[test]
    fn formats_raindrop_detail_via_display() {
        let item = Raindrop {
            id: 42,
            title: Some("Rust article".to_string()),
            link: Some("https://example.com".to_string()),
            excerpt: Some("CLI notes".to_string()),
            collection: Some(CollectionRef {
                id: 7,
                extra: HashMap::new(),
            }),
            tags: vec!["rust".to_string(), "cli".to_string()],
            extra: HashMap::new(),
        };

        let expected = [
            "id: 42",
            "title: Rust article",
            "link: https://example.com",
            "excerpt: CLI notes",
            "collection: 7",
            "tags: rust, cli",
        ]
        .join("\n");

        let actual = RaindropDetailDisplay(&item).to_string();

        assert_eq!(expected, actual);
    }

    #[test]
    fn next_page_hint_on_later_page_shows_previous_and_next() {
        let actual = format_next_page_hint(2);

        assert_eq!(
            "Hint: use\n`--page 1` for the previous page\n`--page 3` for the next page",
            actual
        );
    }

    #[test]
    fn next_page_hint_on_first_page_shows_only_next() {
        let actual = format_next_page_hint(1);

        assert_eq!("Hint: use\n`--page 2` for the next page", actual);
    }

    #[test]
    fn translates_cli_page_numbers_to_api_page_numbers() {
        assert_eq!(0, CliPage(1).to_api_page());
        assert_eq!(1, CliPage(2).to_api_page());
        assert_eq!(4, CliPage(5).to_api_page());
    }

    #[test]
    fn cli_page_keeps_display_values_one_indexed() {
        assert_eq!(1, CliPage(1).as_display());
        assert_eq!(2, CliPage(2).as_display());
    }

    #[test]
    fn rejects_zero_page_for_list() {
        let cli = Cli::try_parse_from(["puddle", "list", "--page", "0"]);

        assert!(cli.is_err());
    }

    #[tokio::test]
    async fn fetch_all_raindrops_collects_paginated_results() {
        let server = MockServer::start().await;

        let first_page_items = (1..=25)
            .map(|id| {
                serde_json::json!({
                    "_id": id,
                    "title": format!("Item {id}"),
                    "link": format!("https://example.com/{id}"),
                    "tags": []
                })
            })
            .collect::<Vec<_>>();
        let second_page_items = vec![
            serde_json::json!({
                "_id": 26,
                "title": "Item 26",
                "link": "https://example.com/26",
                "tags": ["export"]
            }),
            serde_json::json!({
                "_id": 27,
                "title": "Item 27",
                "link": "https://example.com/27",
                "tags": []
            }),
        ];

        Mock::given(method("GET"))
            .and(path("/raindrops/0"))
            .and(query_param("page", "0"))
            .and(query_param("perpage", "25"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": true,
                "items": first_page_items,
                "count": 25
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/raindrops/0"))
            .and(query_param("page", "1"))
            .and(query_param("perpage", "25"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": true,
                "items": second_page_items,
                "count": 2
            })))
            .mount(&server)
            .await;

        let client = RaindropClient::builder()
            .access_token("test-token")
            .base_url(server.uri())
            .build()
            .unwrap();

        let items = fetch_all_raindrops(&client, CollectionScope::All)
            .await
            .unwrap();

        assert_eq!(27, items.len());
        assert_eq!(1, items[0].id);
        assert_eq!(27, items[26].id);
    }

    #[tokio::test]
    async fn fetch_all_raindrops_uses_requested_collection_scope() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/raindrops/-1"))
            .and(query_param("page", "0"))
            .and(query_param("perpage", "25"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": true,
                "items": [],
                "count": 0
            })))
            .mount(&server)
            .await;

        let client = RaindropClient::builder()
            .access_token("test-token")
            .base_url(server.uri())
            .build()
            .unwrap();

        let items = fetch_all_raindrops(&client, CollectionScope::Unsorted)
            .await
            .unwrap();

        assert_eq!(0, items.len());
    }
}
