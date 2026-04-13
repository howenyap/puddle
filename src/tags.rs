use crate::app::CliApp;
use crate::previews::{DeleteTagPreview, RenameTagPreview};
use clap::{Args, Subcommand};
use puddle::models::tags::Tag;

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub(crate) enum TagsCommand {
    #[command(about = "List all tags")]
    List,
    #[command(about = "List tags for a collection")]
    Get(GetTagsArgs),
    #[command(about = "Rename a tag within a collection")]
    Rename(RenameTagArgs),
    #[command(about = "Delete a tag from a collection")]
    Delete(DeleteTagArgs),
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct GetTagsArgs {
    #[arg(allow_hyphen_values = true)]
    pub(crate) collection_id: i64,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct RenameTagArgs {
    #[arg(allow_hyphen_values = true)]
    pub(crate) collection_id: i64,
    pub(crate) find: String,
    pub(crate) replace: String,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct DeleteTagArgs {
    #[arg(allow_hyphen_values = true)]
    pub(crate) collection_id: i64,
    pub(crate) tag: String,
}

impl CliApp {
    pub(crate) async fn run_tags(
        &self,
        command: TagsCommand,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            TagsCommand::List => self.tags_list().await,
            TagsCommand::Get(args) => self.tags_get(args).await,
            TagsCommand::Rename(args) => self.tags_rename(args).await,
            TagsCommand::Delete(args) => self.tags_delete(args).await,
        }
    }

    async fn tags_list(&self) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.tags().list().await?;
        print_tags(&response.data);

        Ok(())
    }

    async fn tags_get(&self, args: GetTagsArgs) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.tags().get(args.collection_id).await?;
        print_tags(&response.data);

        Ok(())
    }

    async fn tags_rename(&self, args: RenameTagArgs) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_dry_run() {
            println!("{}", RenameTagPreview::new(&args));

            return Ok(());
        }

        let response = self
            .client
            .tags()
            .rename(args.collection_id, &args.find, &args.replace)
            .await?;
        println!("renamed: {}", response.data);

        Ok(())
    }

    async fn tags_delete(&self, args: DeleteTagArgs) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_dry_run() {
            println!("{}", DeleteTagPreview::new(&args));

            return Ok(());
        }

        let response = self
            .client
            .tags()
            .delete(args.collection_id, &args.tag)
            .await?;
        println!("deleted: {}", response.data);

        Ok(())
    }
}

fn print_tags(items: &[Tag]) {
    if items.is_empty() {
        println!("No tags found.");
        return;
    }

    for item in items {
        match item.count {
            Some(count) => println!("{} ({count})", item.id),
            None => println!("{}", item.id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Command};
    use clap::Parser;

    #[test]
    fn parses_nested_tag_rename_command() {
        let cli =
            Cli::try_parse_from(["puddle", "tags", "rename", "10", "api", "backend"]).unwrap();

        assert_eq!(
            Some(Command::Tags {
                command: TagsCommand::Rename(RenameTagArgs {
                    collection_id: 10,
                    find: "api".to_string(),
                    replace: "backend".to_string(),
                }),
            }),
            cli.command
        );
    }

    #[test]
    fn parses_negative_collection_id_for_tag_scope() {
        let cli = Cli::try_parse_from(["puddle", "tags", "get", "-1"]).unwrap();

        assert_eq!(
            Some(Command::Tags {
                command: TagsCommand::Get(GetTagsArgs { collection_id: -1 }),
            }),
            cli.command
        );
    }
}
