use crate::app::CliApp;
use crate::common::{display_text, read_upload_file};
use crate::previews::{
    CreateCollectionPreview, DeleteCollectionPreview, UpdateCollectionPreview,
    UploadCollectionCoverPreview,
};
use clap::{Args, Subcommand};
use puddle::models::collections::{Collection, CreateCollection, UpdateCollection};
use puddle::models::common::CollectionScope;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub(crate) enum CollectionsCommand {
    #[command(about = "List collections")]
    List,
    #[command(about = "Get a collection by ID")]
    Get(GetCollectionArgs),
    #[command(about = "Create a collection")]
    Create(CreateCollectionArgs),
    #[command(about = "Update a collection")]
    Update(UpdateCollectionArgs),
    #[command(about = "Delete a collection")]
    Delete(DeleteCollectionArgs),
    #[command(about = "Upload a collection cover")]
    UploadCover(UploadCollectionCoverArgs),
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct GetCollectionArgs {
    #[arg(allow_hyphen_values = true)]
    pub(crate) id: CollectionScope,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct CreateCollectionArgs {
    pub(crate) title: String,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) parent: Option<i64>,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct UpdateCollectionArgs {
    #[arg(allow_hyphen_values = true)]
    pub(crate) id: i64,
    #[arg(long)]
    pub(crate) title: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) parent: Option<i64>,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct DeleteCollectionArgs {
    #[arg(allow_hyphen_values = true)]
    pub(crate) id: i64,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct UploadCollectionCoverArgs {
    #[arg(allow_hyphen_values = true)]
    pub(crate) id: i64,
    pub(crate) path: PathBuf,
}

impl CliApp {
    pub(crate) async fn run_collections(
        &self,
        command: CollectionsCommand,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            CollectionsCommand::List => self.collections_list().await,
            CollectionsCommand::Get(args) => self.collections_get(args).await,
            CollectionsCommand::Create(args) => self.collections_create(args).await,
            CollectionsCommand::Update(args) => self.collections_update(args).await,
            CollectionsCommand::Delete(args) => self.collections_delete(args).await,
            CollectionsCommand::UploadCover(args) => self.collections_upload_cover(args).await,
        }
    }

    async fn collections_list(&self) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.collections().get_children().await?;
        print_collection_list(&response.data);

        Ok(())
    }

    async fn collections_get(
        &self,
        args: GetCollectionArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.collections().get(args.id).await?;
        print_collection_detail(&response.data);

        Ok(())
    }

    async fn collections_create(
        &self,
        args: CreateCollectionArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = CreateCollection {
            title: args.title,
            parent: args.parent,
            extra: HashMap::new(),
        };

        if self.is_dry_run() {
            println!("{}", CreateCollectionPreview::new(&payload));

            return Ok(());
        }

        let response = self.client.collections().create(&payload).await?;
        print_collection_detail(&response.data);

        Ok(())
    }

    async fn collections_update(
        &self,
        args: UpdateCollectionArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = UpdateCollection {
            title: args.title,
            parent: args.parent,
            extra: HashMap::new(),
        };

        if self.is_dry_run() {
            let existing_collection = self
                .client
                .collections()
                .get(CollectionScope::from(args.id))
                .await?
                .data;

            println!(
                "{}",
                UpdateCollectionPreview::new(&existing_collection, &payload)
            );

            return Ok(());
        }

        let response = self.client.collections().update(args.id, &payload).await?;
        print_collection_detail(&response.data);

        Ok(())
    }

    async fn collections_delete(
        &self,
        args: DeleteCollectionArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_dry_run() {
            let existing_collection = self
                .client
                .collections()
                .get(CollectionScope::from(args.id))
                .await?
                .data;

            println!("{}", DeleteCollectionPreview::new(&existing_collection));

            return Ok(());
        }

        let response = self.client.collections().delete(args.id).await?;
        println!("deleted: {}", response.data);

        Ok(())
    }

    async fn collections_upload_cover(
        &self,
        args: UploadCollectionCoverArgs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_dry_run() {
            let _ = read_upload_file(&args.path)?;
            let existing_collection = self
                .client
                .collections()
                .get(CollectionScope::from(args.id))
                .await?
                .data;

            println!(
                "{}",
                UploadCollectionCoverPreview::new(&existing_collection, &args.path)
            );

            return Ok(());
        }

        let (bytes, mime, file_name) = read_upload_file(&args.path)?;
        let response = self
            .client
            .collections()
            .upload_cover(args.id, bytes, mime, &file_name)
            .await?;
        print_collection_detail(&response.data);

        Ok(())
    }
}

fn print_collection_list(items: &[Collection]) {
    if items.is_empty() {
        println!("No collections found.");
        return;
    }

    for item in items {
        println!("{}", format_collection_summary(item));
    }
}

fn print_collection_detail(item: &Collection) {
    println!("id: {}", item.id);
    println!(
        "title: {}",
        display_text(item.title.as_deref(), "(untitled)")
    );

    if let Some(parent) = item.parent {
        println!("parent: {parent}");
    }

    if let Some(count) = item.count {
        println!("count: {count}");
    }
}

fn format_collection_summary(item: &Collection) -> String {
    let mut parts = vec![
        format!("#{}", item.id),
        display_text(item.title.as_deref(), "(untitled)").to_string(),
    ];

    if let Some(parent) = item.parent {
        parts.push(format!("parent={parent}"));
    }

    if let Some(count) = item.count {
        parts.push(format!("count={count}"));
    }

    parts.join(" | ")
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Command};
    use clap::Parser;

    #[test]
    fn parses_nested_collection_update_command() {
        let cli = Cli::try_parse_from([
            "puddle",
            "collections",
            "update",
            "42",
            "--title",
            "Reading",
            "--parent",
            "1",
        ])
        .unwrap();

        assert_eq!(
            Some(Command::Collections {
                command: CollectionsCommand::Update(UpdateCollectionArgs {
                    id: 42,
                    title: Some("Reading".to_string()),
                    parent: Some(1),
                }),
            }),
            cli.command
        );
    }

    #[test]
    fn parses_negative_collection_id_without_double_dash() {
        let cli = Cli::try_parse_from(["puddle", "collections", "get", "-1"]).unwrap();

        assert_eq!(
            Some(Command::Collections {
                command: CollectionsCommand::Get(GetCollectionArgs {
                    id: CollectionScope::Unsorted,
                }),
            }),
            cli.command
        );
    }

    #[test]
    fn parses_collections_list_command() {
        let cli = Cli::try_parse_from(["puddle", "collections", "list"]).unwrap();

        assert_eq!(
            Some(Command::Collections {
                command: CollectionsCommand::List,
            }),
            cli.command
        );
    }
}
