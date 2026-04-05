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
use crate::config::{ConfigValues, global_config_path};
use crate::constants::{
    AUTHORIZE_URL_BASE, DEFAULT_OAUTH_DEBUG_REDIRECT_URI, RAINDROP_INTEGRATIONS_URI,
};
use crate::raindrops::{
    CreateArgs, CreateManyArgs, DeleteArgs, DeleteManyArgs, ExportArgs, GetArgs, ListArgs,
    UpdateArgs, UpdateManyArgs, UploadCoverArgs, UploadFileArgs,
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
    #[command(about = "Export all raindrops to a JSON file")]
    Export(ExportArgs),
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
    #[command(about = "Show the authenticated user")]
    Me,
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
            let mut app = CliApp::new()?;
            app.run_command(command).await?;
        }
    }

    Ok(())
}

async fn run_setup() -> Result<(), Box<dyn std::error::Error>> {
    println!("Create a new app at Raindrop integrations: {RAINDROP_INTEGRATIONS_URI}\n");
    println!(
        "Use this redirect URI: {}",
        DEFAULT_OAUTH_DEBUG_REDIRECT_URI
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

    let config = ConfigValues {
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
