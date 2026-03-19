use crate::config::{Config, global_config_path};
use puddle::RaindropClient;

pub(crate) struct CliApp {
    pub(crate) client: RaindropClient,
}

impl CliApp {
    pub(crate) fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = global_config_path()?;
        let config = Config::from_path(&config_path)?;

        Ok(Self {
            client: RaindropClient::new(config.access_token)?,
        })
    }
}
