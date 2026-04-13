use crate::Command;
use crate::config::Config;
use puddle::{Error, RaindropClient};

pub(crate) struct RunContext {
    pub(crate) is_dry_run: bool,
}

pub(crate) struct CliApp {
    pub(crate) config: Config,
    pub(crate) client: RaindropClient,
    pub(crate) context: RunContext,
}

impl CliApp {
    pub(crate) fn new(context: RunContext) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::load()?;
        let client = RaindropClient::builder()
            .access_token(config.values().access_token.clone())
            .build()?;

        Ok(Self {
            config,
            client,
            context,
        })
    }

    pub(crate) fn is_dry_run(&self) -> bool {
        self.context.is_dry_run
    }

    pub(crate) async fn run_command(
        &mut self,
        command: Command,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut attempted = false;

        loop {
            let result = match command.clone() {
                Command::List(args) => self.list(args).await,
                Command::Get(args) => self.get(args).await,
                Command::Create(args) => self.create(args).await,
                Command::Update(args) => self.update(args).await,
                Command::Delete(args) => self.delete(args).await,
                Command::UploadFile(args) => self.upload_file(args).await,
                Command::UploadCover(args) => self.upload_cover(args).await,
                Command::CreateMany(args) => self.create_many(args).await,
                Command::UpdateMany(args) => self.update_many(args).await,
                Command::DeleteMany(args) => self.delete_many(args).await,
                Command::Export(args) => self.export(args).await,
                Command::Collections { command } => self.run_collections(command).await,
                Command::Tags { command } => self.run_tags(command).await,
                Command::Me => self.user_me().await,
                Command::Filters { command } => self.run_filters(command).await,
                Command::Setup | Command::Config => unreachable!(),
            };

            match result {
                Ok(()) => return Ok(()),
                Err(err)
                    if !attempted
                        && err
                            .as_ref()
                            .downcast_ref::<Error>()
                            .is_some_and(Error::is_refreshable_auth_error) =>
                {
                    self.config = self.config.clone().refresh_access_token().await?;
                    self.client = RaindropClient::builder()
                        .access_token(self.config.values().access_token.clone())
                        .build()?;

                    attempted = true;
                }
                Err(err) => return Err(err),
            }
        }
    }
}
