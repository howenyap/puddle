use crate::config::Config;
use puddle::RaindropClient;

pub(crate) struct CliApp {
    pub(crate) client: RaindropClient,
}

impl CliApp {
    pub(crate) fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::load()?;

        Ok(Self {
            client: RaindropClient::new(config.access_token)?,
        })
    }
}
