mod app;
mod collections;
mod common;
mod config;
mod constants;
mod filters;
mod raindrops;
mod tags;
mod user;

use crate::app::CliApp;
use crate::config::{Config, global_config_path};
use crate::constants::{
    AUTHORIZE_URL_BASE, DEFAULT_OAUTH_DEBUG_REDIRECT_URI, RAINDROP_INTEGRATIONS_URI,
};
use crate::raindrops::{
    CreateArgs, CreateManyArgs, DeleteArgs, DeleteManyArgs, GetArgs, ListArgs, UpdateArgs,
    UpdateManyArgs, UploadCoverArgs, UploadFileArgs,
};
use clap::{CommandFactory, Parser, Subcommand};
use puddle::auth::oauth;
use std::io::{self, Write};
use url::form_urlencoded;

#[derive(Debug, Parser)]
#[command(name = "puddle", version, about = "")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
enum Command {
    #[command(about = "List raindrops")]
    List(ListArgs),
    #[command(about = "Get a raindrop by ID")]
    Get(GetArgs),
    #[command(about = "Create a raindrop")]
    Create(CreateArgs),
    #[command(about = "Update a raindrop")]
    Update(UpdateArgs),
    #[command(about = "Delete a raindrop")]
    Delete(DeleteArgs),
    #[command(about = "Upload a file as a raindrop")]
    UploadFile(UploadFileArgs),
    #[command(about = "Upload a cover for a raindrop")]
    UploadCover(UploadCoverArgs),
    #[command(about = "Create multiple raindrops from a JSON file")]
    CreateMany(CreateManyArgs),
    #[command(about = "Update multiple raindrops")]
    UpdateMany(UpdateManyArgs),
    #[command(about = "Delete multiple raindrops")]
    DeleteMany(DeleteManyArgs),
    #[command(about = "Collection operations")]
    Collections {
        #[command(subcommand)]
        command: collections::CollectionsCommand,
    },
    #[command(about = "Tag operations")]
    Tags {
        #[command(subcommand)]
        command: tags::TagsCommand,
    },
    #[command(about = "User operations")]
    User {
        #[command(subcommand)]
        command: user::UserCommand,
    },
    #[command(about = "Filter operations")]
    Filters {
        #[command(subcommand)]
        command: filters::FiltersCommand,
    },
    #[command(about = "Setup a raindrop integration")]
    Setup,
    #[command(about = "Print path to config")]
    Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let Some(command) = cli.command else {
        let mut cmd = Cli::command();
        cmd.print_help()?;
        println!();
        return Ok(());
    };

    match command {
        Command::Setup => run_setup().await?,
        Command::Config => {
            let path = global_config_path()?;
            println!("{}", path.display());
        }
        command => {
            let app = CliApp::new()?;

            match command {
                Command::List(args) => app.list(args).await?,
                Command::Get(args) => app.get(args).await?,
                Command::Create(args) => app.create(args).await?,
                Command::Update(args) => app.update(args).await?,
                Command::Delete(args) => app.delete(args).await?,
                Command::UploadFile(args) => app.upload_file(args).await?,
                Command::UploadCover(args) => app.upload_cover(args).await?,
                Command::CreateMany(args) => app.create_many(args).await?,
                Command::UpdateMany(args) => app.update_many(args).await?,
                Command::DeleteMany(args) => app.delete_many(args).await?,
                Command::Collections { command } => app.run_collections(command).await?,
                Command::Tags { command } => app.run_tags(command).await?,
                Command::User { command } => app.run_user(command).await?,
                Command::Filters { command } => app.run_filters(command).await?,
                Command::Setup | Command::Config => unreachable!(),
            }
        }
    }

    Ok(())
}

async fn run_setup() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Create a new app at {}",
        terminal_link("Raindrop integrations", RAINDROP_INTEGRATIONS_URI),
    );
    println!(
        "Use this redirect URI: {}",
        terminal_link(
            DEFAULT_OAUTH_DEBUG_REDIRECT_URI,
            DEFAULT_OAUTH_DEBUG_REDIRECT_URI,
        )
    );
    let client_id = prompt_required("\nClient ID")?;
    let client_secret = prompt_required("Client Secret")?;
    let redirect_uri = DEFAULT_OAUTH_DEBUG_REDIRECT_URI.to_string();

    let query = form_urlencoded::Serializer::new(String::new())
        .append_pair("response_type", "code")
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .finish();
    let authorise_url = format!("{AUTHORIZE_URL_BASE}?{query}");

    println!("\nOpen this URL to authorise and copy the code:");
    println!("{authorise_url}");

    let code = prompt_required("\nAuthorization code")?;
    let token = oauth::TokenRequestBuilder::exchange_code(&client_id, &client_secret, &code)
        .redirect_uri(&redirect_uri)
        .send()
        .await?;
    let refresh_token = token
        .refresh_token
        .ok_or_else(|| io::Error::other("missing refresh token in OAuth response"))?;

    let config = Config {
        client_id,
        client_secret,
        redirect_uri,
        access_token: token.access_token,
        refresh_token,
    };

    config.write_to_global_path()?;

    println!("\nYou're all set.");

    Ok(())
}
fn terminal_link(label: &str, url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{label}\x1b]8;;\x1b\\")
}

fn prompt_required(label: &str) -> io::Result<String> {
    loop {
        print!("{label}: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
        println!("{label} is required.");
    }
}
