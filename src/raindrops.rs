use crate::app::CliApp;
use crate::common::{display_text, read_upload_file};
use crate::previews::{
    CreateManyRaindropsPreview, CreateRaindropPreview, DeleteManyRaindropsPreview,
    DeleteRaindropPreview, UpdateManyRaindropsPreview, UpdateRaindropPreview,
    UploadRaindropCoverPreview, UploadRaindropFilePreview,
};
use clap::{ArgGroup, Args};
use puddle::RaindropClient;
use puddle::models::common::{CollectionScope, ItemsResponse};
use puddle::models::raindrops::{
    CreateRaindrop, DeleteManyRaindrops, Raindrop, RaindropId, RaindropListParams,
    UpdateManyRaindrops, UpdateRaindrop,
};
use puddle::pagination::{MAX_PER_PAGE, PageParams, PerPage};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
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
    pub(crate) per_page: Option<PerPage>,
    #[arg(long)]
    pub(crate) nested: bool,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct GetArgs {
    pub(crate) id: RaindropId,
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
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) collection: Option<CollectionScope>,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct UpdateArgs {
    pub(crate) id: RaindropId,
    #[arg(long)]
    pub(crate) title: Option<String>,
    #[arg(long)]
    pub(crate) excerpt: Option<String>,
    #[arg(long = "tag")]
    pub(crate) tags: Vec<String>,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) collection: Option<CollectionScope>,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct DeleteArgs {
    pub(crate) id: RaindropId,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct UploadFileArgs {
    pub(crate) path: PathBuf,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct UploadCoverArgs {
    pub(crate) id: RaindropId,
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
#[command(group(
    ArgGroup::new("selector")
        .args(["ids", "from_collection"])
        .required(true)
        .multiple(true)
))]
pub(crate) struct UpdateManyArgs {
    #[arg(long = "id")]
    pub(crate) ids: Vec<i64>,
    #[arg(long = "from-collection", allow_hyphen_values = true)]
    pub(crate) from_collection: Vec<CollectionScope>,
    #[arg(long = "exclude-collection", allow_hyphen_values = true)]
    pub(crate) exclude_collection: Vec<CollectionScope>,
    #[arg(long)]
    pub(crate) search: Option<String>,
    #[arg(long = "tag")]
    pub(crate) tags: Vec<String>,
    #[arg(long = "to-collection", allow_hyphen_values = true)]
    pub(crate) to_collection: Option<CollectionScope>,
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
        let payload = create_payload(args.clone())?;

        if self.is_dry_run() {
            println!("{}", CreateRaindropPreview::new(&payload));

            return Ok(());
        }

        let response = self.client.raindrops().create(&payload).await?;
        println!("{}", RaindropDetailDisplay(&response.data));

        Ok(())
    }

    pub(crate) async fn update(&self, args: UpdateArgs) -> Result<(), Box<dyn std::error::Error>> {
        let payload = UpdateRaindrop {
            title: args.title,
            excerpt: args.excerpt,
            collection: args.collection.map(TryInto::try_into).transpose()?,
            tags: (!args.tags.is_empty()).then_some(args.tags),
            extra: HashMap::new(),
        };

        if self.is_dry_run() {
            let existing_raindrop = self.client.raindrops().get(args.id).await?.data;

            println!(
                "{}",
                UpdateRaindropPreview::new(&existing_raindrop, &payload)
            );

            return Ok(());
        }

        let response = self.client.raindrops().update(args.id, &payload).await?;
        println!("{}", RaindropDetailDisplay(&response.data));

        Ok(())
    }

    pub(crate) async fn delete(&self, args: DeleteArgs) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_dry_run() {
            let existing_raindrop = self.client.raindrops().get(args.id).await?.data;

            println!("{}", DeleteRaindropPreview::new(&existing_raindrop));

            return Ok(());
        }

        let response = self.client.raindrops().delete(args.id).await?;
        println!("deleted: {}", response.data);

        Ok(())
    }

    pub(crate) async fn upload_file(
        &self,
        args: UploadFileArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_dry_run() {
            let _ = read_upload_file(&args.path)?;

            println!("{}", UploadRaindropFilePreview::new(&args.path));

            return Ok(());
        }

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
        if self.is_dry_run() {
            let _ = read_upload_file(&args.path)?;
            let existing_raindrop = self.client.raindrops().get(args.id).await?.data;

            println!(
                "{}",
                UploadRaindropCoverPreview::new(&existing_raindrop, &args.path)
            );

            return Ok(());
        }

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

        if self.is_dry_run() {
            println!(
                "{}",
                CreateManyRaindropsPreview::new(&args.input, &payloads)
            );

            return Ok(());
        }

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
        if self.is_dry_run() {
            let selection = resolve_update_many_selection(&self.client, &args).await?;

            println!("{}", UpdateManyRaindropsPreview::new(&selection, &args));

            return Ok(());
        }

        let outcome = execute_update_many(&self.client, &args).await?;

        if outcome.modified == 0 {
            println!("No raindrops matched the requested selectors.");

            return Ok(());
        }

        let selection = outcome.selection;

        println!("matched: {}", selection.targets.len());
        println!("modified: {}", outcome.modified);

        if !selection.from_collections.is_empty() {
            println!(
                "from collections: {}",
                format_collection_scopes(&selection.from_collections)
            );
        }

        if !selection.excluded_collections.is_empty() {
            println!(
                "excluded collections: {}",
                format_collection_scopes(&selection.excluded_collections)
            );
        }

        if let Some(search) = selection.search {
            println!("search: {search}");
        }

        Ok(())
    }

    pub(crate) async fn delete_many(
        &self,
        args: DeleteManyArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_dry_run() {
            let targets = self.client.raindrops().get_many(&args.ids).await?.data;

            println!(
                "{}",
                DeleteManyRaindropsPreview::new(args.collection_id, &targets)
            );

            return Ok(());
        }

        let payload = DeleteManyRaindrops {
            ids: Some(args.ids.clone()),
            extra: HashMap::new(),
        };
        let response = self
            .client
            .raindrops()
            .delete_many(args.collection_id, &payload)
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

fn create_payload(
    args: CreateArgs,
) -> Result<CreateRaindrop, puddle::models::common::InvalidDestinationCollectionScope> {
    Ok(CreateRaindrop {
        link: args.link,
        title: args.title,
        excerpt: args.excerpt,
        collection: args.collection.map(TryInto::try_into).transpose()?,
        tags: args.tags,
        extra: HashMap::new(),
    })
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
    collection_scope: CollectionScope,
) -> Result<Vec<Raindrop>, Box<dyn std::error::Error>> {
    fetch_all_raindrops_matching(client, collection_scope, None).await
}

async fn fetch_all_raindrops_matching(
    client: &RaindropClient,
    collection_scope: CollectionScope,
    search: Option<&str>,
) -> Result<Vec<Raindrop>, Box<dyn std::error::Error>> {
    let mut page = 0;
    let mut all_items = Vec::new();

    loop {
        let mut params = RaindropListParams::new()
            .page(page)
            .per_page(PerPage::new_unchecked(MAX_PER_PAGE));
        if let Some(search) = search {
            params = params.search(search);
        }
        let response = client.raindrops().list(collection_scope, &params).await?;
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

#[derive(Debug, Clone)]
pub(crate) struct ResolvedUpdateManySelection {
    pub(crate) targets: Vec<Raindrop>,
    pub(crate) from_collections: Vec<CollectionScope>,
    pub(crate) excluded_collections: Vec<CollectionScope>,
    pub(crate) search: Option<String>,
}

#[derive(Debug, Clone)]
struct UpdateManyOutcome {
    selection: ResolvedUpdateManySelection,
    modified: u64,
}

async fn execute_update_many(
    client: &RaindropClient,
    args: &UpdateManyArgs,
) -> Result<UpdateManyOutcome, Box<dyn std::error::Error>> {
    let selection = resolve_update_many_selection(client, args).await?;

    if selection.targets.is_empty() {
        return Ok(UpdateManyOutcome {
            selection,
            modified: 0,
        });
    }

    let payload = UpdateManyRaindrops {
        ids: Some(
            selection
                .targets
                .iter()
                .map(|target| target.id.into_inner())
                .collect(),
        ),
        collection: args.to_collection.map(TryInto::try_into).transpose()?,
        tags: (!args.tags.is_empty()).then_some(args.tags.clone()),
        extra: HashMap::new(),
    };
    let response = client
        .raindrops()
        .update_many(CollectionScope::All, &payload)
        .await?;

    Ok(UpdateManyOutcome {
        selection,
        modified: response.data,
    })
}

async fn resolve_update_many_selection(
    client: &RaindropClient,
    args: &UpdateManyArgs,
) -> Result<ResolvedUpdateManySelection, Box<dyn std::error::Error>> {
    let mut selected = BTreeMap::new();

    for id in &args.ids {
        let response = client.raindrops().get(RaindropId::new(*id)).await?;
        selected.insert(*id, response.data);
    }

    for scope in &args.from_collection {
        let items = fetch_all_raindrops_matching(client, *scope, args.search.as_deref()).await?;
        selected.extend(items.into_iter().map(|item| (item.id.into_inner(), item)));
    }

    if !args.exclude_collection.is_empty() {
        selected.retain(|_, item| {
            !args
                .exclude_collection
                .iter()
                .any(|scope| item.matches_scope(*scope))
        });
    }

    Ok(ResolvedUpdateManySelection {
        targets: selected.into_values().collect(),
        from_collections: args.from_collection.clone(),
        excluded_collections: args.exclude_collection.clone(),
        search: args.search.clone(),
    })
}

fn format_collection_scopes(scopes: &[CollectionScope]) -> String {
    scopes
        .iter()
        .map(|scope| scope.to_string())
        .collect::<Vec<_>>()
        .join(", ")
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
    use puddle::models::raindrops::CollectionRef;
    use std::collections::HashMap;
    use wiremock::matchers::{body_json, method, path, query_param};
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
                per_page: Some(PerPage::new_unchecked(25)),
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
                collection: Some(CollectionScope::id(42).unwrap()),
            })),
            cli.command
        );
    }

    #[test]
    fn parses_named_system_collection_for_create() {
        let cli = Cli::try_parse_from([
            "puddle",
            "create",
            "https://example.com",
            "--collection",
            "Unsorted",
        ])
        .unwrap();

        assert_eq!(
            Some(Command::Create(CreateArgs {
                link: "https://example.com".to_string(),
                title: None,
                excerpt: None,
                tags: vec![],
                collection: Some(CollectionScope::Unsorted),
            })),
            cli.command
        );
    }

    #[test]
    fn parses_named_system_collection_for_update() {
        let cli =
            Cli::try_parse_from(["puddle", "update", "42", "--collection", "Unsorted"]).unwrap();

        assert_eq!(
            Some(Command::Update(UpdateArgs {
                id: RaindropId::new(42),
                title: None,
                excerpt: None,
                tags: vec![],
                collection: Some(CollectionScope::Unsorted),
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
            "--id",
            "10",
            "--id",
            "12",
            "--from-collection",
            "5",
            "--from-collection",
            "Unsorted",
            "--exclude-collection",
            "Trash",
            "--search",
            "rust",
            "--tag",
            "rust",
            "--to-collection",
            "7",
        ])
        .unwrap();

        assert_eq!(
            Some(Command::UpdateMany(UpdateManyArgs {
                ids: vec![10, 12],
                from_collection: vec![CollectionScope::id(5).unwrap(), CollectionScope::Unsorted],
                exclude_collection: vec![CollectionScope::Trash],
                search: Some("rust".to_string()),
                tags: vec!["rust".to_string()],
                to_collection: Some(CollectionScope::id(7).unwrap()),
            })),
            cli.command
        );
    }

    #[test]
    fn parses_named_system_collection_for_update_many_destination() {
        let cli = Cli::try_parse_from([
            "puddle",
            "update-many",
            "--id",
            "10",
            "--to-collection",
            "Unsorted",
        ])
        .unwrap();

        assert_eq!(
            Some(Command::UpdateMany(UpdateManyArgs {
                ids: vec![10],
                from_collection: vec![],
                exclude_collection: vec![],
                search: None,
                tags: vec![],
                to_collection: Some(CollectionScope::Unsorted),
            })),
            cli.command
        );
    }

    #[test]
    fn rejects_update_many_without_selector() {
        let cli = Cli::try_parse_from(["puddle", "update-many", "--tag", "rust"]);

        assert!(cli.is_err());
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
            collection: Some(CollectionScope::id(7).unwrap()),
        };

        let payload = create_payload(args).unwrap();

        assert_eq!("https://example.com", payload.link);
        assert_eq!(Some("Example"), payload.title.as_deref());
        assert_eq!(Some("Summary"), payload.excerpt.as_deref());
        assert_eq!(Some(7), payload.collection.as_ref().map(|value| value.id));
        assert_eq!(vec!["rust", "cli"], payload.tags);
    }

    #[test]
    fn create_payload_accepts_unsorted_destination() {
        let args = CreateArgs {
            link: "https://example.com".to_string(),
            title: None,
            excerpt: None,
            tags: vec![],
            collection: Some(CollectionScope::Unsorted),
        };

        let payload = create_payload(args).unwrap();

        assert_eq!(Some(-1), payload.collection.as_ref().map(|value| value.id));
    }

    #[test]
    fn destination_collection_rejects_all() {
        let error = CollectionScope::All.into_destination_id().unwrap_err();

        assert_eq!(
            "`All` is not a valid destination collection; use a specific collection or `Unsorted`",
            error.to_string()
        );
    }

    #[test]
    fn destination_collection_rejects_trash() {
        let error = CollectionScope::Trash.into_destination_id().unwrap_err();

        assert_eq!(
            "`Trash` is not a valid destination collection",
            error.to_string()
        );
    }

    #[test]
    fn formats_list_item_with_requested_fields() {
        let item = Raindrop {
            id: RaindropId::new(42),
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
            id: RaindropId::new(42),
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

    #[test]
    fn rejects_zero_per_page_for_list() {
        let cli = Cli::try_parse_from(["puddle", "list", "--per-page", "0"]);

        assert!(cli.is_err());
    }

    #[test]
    fn rejects_per_page_above_documented_max_for_list() {
        let cli = Cli::try_parse_from(["puddle", "list", "--per-page", "51"]);

        assert!(cli.is_err());
    }

    #[tokio::test]
    async fn fetch_all_raindrops_collects_paginated_results() {
        let server = MockServer::start().await;

        let first_page_items = (1..=50)
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
                "_id": 51,
                "title": "Item 51",
                "link": "https://example.com/51",
                "tags": ["export"]
            }),
            serde_json::json!({
                "_id": 52,
                "title": "Item 52",
                "link": "https://example.com/52",
                "tags": []
            }),
        ];

        Mock::given(method("GET"))
            .and(path("/raindrops/0"))
            .and(query_param("page", "0"))
            .and(query_param("perpage", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": true,
                "items": first_page_items,
                "count": 50
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/raindrops/0"))
            .and(query_param("page", "1"))
            .and(query_param("perpage", "50"))
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

        assert_eq!(52, items.len());
        assert_eq!(RaindropId::new(1), items[0].id);
        assert_eq!(RaindropId::new(52), items[51].id);
    }

    #[tokio::test]
    async fn fetch_all_raindrops_uses_requested_collection_scope() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/raindrops/-1"))
            .and(query_param("page", "0"))
            .and(query_param("perpage", "50"))
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

    #[tokio::test]
    async fn execute_update_many_updates_union_of_discovered_and_explicit_ids() {
        let server = MockServer::start().await;

        mock_get_raindrop(&server, 10, 100, &[]).await;
        mock_list_page(&server, 42, 0, None, &[raindrop_json(12, 42, &[])], false).await;
        Mock::given(method("PUT"))
            .and(path("/raindrops/0"))
            .and(body_json(serde_json::json!({
                "ids": [10, 12],
                "tags": ["rust"],
                "collection": { "$id": 7 }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": true,
                "modified": 2
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server);
        let outcome = execute_update_many(
            &client,
            &UpdateManyArgs {
                ids: vec![10],
                from_collection: vec![CollectionScope::id(42).unwrap()],
                exclude_collection: vec![],
                search: None,
                tags: vec!["rust".to_string()],
                to_collection: Some(CollectionScope::id(7).unwrap()),
            },
        )
        .await
        .unwrap();

        assert_eq!(2, outcome.modified);
        assert_eq!(
            vec![10, 12],
            outcome
                .selection
                .targets
                .iter()
                .map(|target| target.id.into_inner())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            vec![CollectionScope::id(42).unwrap()],
            outcome.selection.from_collections
        );
        assert_eq!(
            Vec::<CollectionScope>::new(),
            outcome.selection.excluded_collections
        );
        assert_eq!(None, outcome.selection.search);
    }

    #[tokio::test]
    async fn execute_update_many_deduplicates_unioned_results() {
        let server = MockServer::start().await;

        mock_list_page(
            &server,
            42,
            0,
            None,
            &[raindrop_json(10, 42, &[]), raindrop_json(12, 42, &[])],
            false,
        )
        .await;
        mock_list_page(
            &server,
            -1,
            0,
            None,
            &[raindrop_json(10, -1, &[]), raindrop_json(15, -1, &[])],
            false,
        )
        .await;
        Mock::given(method("PUT"))
            .and(path("/raindrops/0"))
            .and(body_json(serde_json::json!({
                "ids": [10, 12, 15]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": true,
                "modified": 3
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server);
        let outcome = execute_update_many(
            &client,
            &UpdateManyArgs {
                ids: vec![],
                from_collection: vec![CollectionScope::id(42).unwrap(), CollectionScope::Unsorted],
                exclude_collection: vec![],
                search: None,
                tags: vec![],
                to_collection: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(
            vec![10, 12, 15],
            outcome
                .selection
                .targets
                .iter()
                .map(|target| target.id.into_inner())
                .collect::<Vec<_>>()
        );
        assert_eq!(3, outcome.modified);
    }

    #[tokio::test]
    async fn execute_update_many_applies_excluded_collections_after_union() {
        let server = MockServer::start().await;

        mock_get_raindrop(&server, 10, -99, &[]).await;
        mock_list_page(
            &server,
            42,
            0,
            None,
            &[raindrop_json(12, 42, &[]), raindrop_json(13, -99, &[])],
            false,
        )
        .await;
        Mock::given(method("PUT"))
            .and(path("/raindrops/0"))
            .and(body_json(serde_json::json!({
                "ids": [12]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": true,
                "modified": 1
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server);
        let outcome = execute_update_many(
            &client,
            &UpdateManyArgs {
                ids: vec![10],
                from_collection: vec![CollectionScope::id(42).unwrap()],
                exclude_collection: vec![CollectionScope::Trash],
                search: None,
                tags: vec![],
                to_collection: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(
            vec![12],
            outcome
                .selection
                .targets
                .iter()
                .map(|target| target.id.into_inner())
                .collect::<Vec<_>>()
        );
        assert_eq!(1, outcome.modified);
    }

    #[tokio::test]
    async fn execute_update_many_passes_search_to_list_requests_only() {
        let server = MockServer::start().await;

        mock_get_raindrop(&server, 10, 42, &[]).await;
        mock_list_page(
            &server,
            42,
            0,
            Some("rust"),
            &[raindrop_json(12, 42, &["rust"])],
            false,
        )
        .await;
        Mock::given(method("PUT"))
            .and(path("/raindrops/0"))
            .and(body_json(serde_json::json!({
                "ids": [10, 12]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": true,
                "modified": 2
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server);
        let outcome = execute_update_many(
            &client,
            &UpdateManyArgs {
                ids: vec![10],
                from_collection: vec![CollectionScope::id(42).unwrap()],
                exclude_collection: vec![],
                search: Some("rust".to_string()),
                tags: vec![],
                to_collection: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(
            vec![10, 12],
            outcome
                .selection
                .targets
                .iter()
                .map(|target| target.id.into_inner())
                .collect::<Vec<_>>()
        );
        assert_eq!(Some("rust".to_string()), outcome.selection.search);
    }

    fn test_client(server: &MockServer) -> RaindropClient {
        RaindropClient::builder()
            .access_token("test-token")
            .base_url(server.uri())
            .build()
            .unwrap()
    }

    async fn mock_get_raindrop(server: &MockServer, id: i64, collection_id: i64, tags: &[&str]) {
        Mock::given(method("GET"))
            .and(path(format!("/raindrop/{id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": true,
                "item": raindrop_json(id, collection_id, tags)
            })))
            .mount(server)
            .await;
    }

    async fn mock_list_page(
        server: &MockServer,
        collection_scope: i64,
        page: u32,
        search: Option<&str>,
        items: &[serde_json::Value],
        has_next_page: bool,
    ) {
        let response_items = items.to_vec();
        let mut mock = Mock::given(method("GET"))
            .and(path(format!("/raindrops/{collection_scope}")))
            .and(query_param("page", page.to_string()))
            .and(query_param("perpage", "50"));
        if let Some(search) = search {
            mock = mock.and(query_param("search", search));
        }
        mock.respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": true,
            "items": response_items,
            "count": items.len(),
        })))
        .expect(1)
        .mount(server)
        .await;

        if has_next_page {
            unreachable!(
                "test helper currently expects the caller to mount the next page explicitly"
            );
        }
    }

    fn raindrop_json(id: i64, collection_id: i64, tags: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "_id": id,
            "title": format!("Item {id}"),
            "link": format!("https://example.com/{id}"),
            "collection": { "$id": collection_id },
            "tags": tags
        })
    }
}
