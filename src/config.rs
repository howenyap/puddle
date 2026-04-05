use crate::constants::{
    ENV_ACCESS_TOKEN, ENV_CLIENT_ID, ENV_CLIENT_SECRET, ENV_REDIRECT_URI, ENV_REFRESH_TOKEN,
    TOML_ACCESS_TOKEN, TOML_CLIENT_ID, TOML_CLIENT_SECRET, TOML_REDIRECT_URI, TOML_REFRESH_TOKEN,
};
use puddle::auth::oauth;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use toml::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfigValues {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Config {
    Env(ConfigValues),
    File { path: PathBuf, values: ConfigValues },
}

impl Config {
    pub(crate) fn load() -> io::Result<Self> {
        if let Some(values) = ConfigValues::from_env()? {
            return Ok(Self::Env(values));
        }

        let path = global_config_path()?;
        let values = ConfigValues::from_path(&path)?;

        Ok(Self::File { path, values })
    }

    pub(crate) fn values(&self) -> &ConfigValues {
        match self {
            Self::Env(values) => values,
            Self::File { values, .. } => values,
        }
    }

    pub(crate) fn with_refreshed_tokens(
        &self,
        access_token: String,
        refresh_token: Option<String>,
    ) -> Self {
        match self {
            Self::Env(values) => {
                Self::Env(values.with_refreshed_tokens(access_token, refresh_token))
            }
            Self::File { path, values } => Self::File {
                path: path.clone(),
                values: values.with_refreshed_tokens(access_token, refresh_token),
            },
        }
    }

    pub(crate) fn persist(&self) -> io::Result<()> {
        if let Self::File { path, values } = self {
            let mut table = toml::Table::new();

            table.insert(
                TOML_CLIENT_ID.to_string(),
                Value::String(values.client_id.clone()),
            );
            table.insert(
                TOML_CLIENT_SECRET.to_string(),
                Value::String(values.client_secret.clone()),
            );
            table.insert(
                TOML_REDIRECT_URI.to_string(),
                Value::String(values.redirect_uri.clone()),
            );
            table.insert(
                TOML_ACCESS_TOKEN.to_string(),
                Value::String(values.access_token.clone()),
            );
            table.insert(
                TOML_REFRESH_TOKEN.to_string(),
                Value::String(values.refresh_token.clone()),
            );

            let content = toml::to_string_pretty(&table).map_err(|e| {
                io::Error::other(format!("failed to serialize global config toml: {e}"))
            })?;
            fs::write(path, format!("{content}\n"))?;
        }

        Ok(())
    }

    pub(crate) async fn refresh_access_token(self) -> Result<Self, Box<dyn std::error::Error>> {
        let values = self.values();
        let token = oauth::TokenRequestBuilder::refresh(
            &values.client_id,
            &values.client_secret,
            &values.refresh_token,
        )
        .send()
        .await?;

        let config = self.with_refreshed_tokens(token.access_token, token.refresh_token);
        config.persist()?;

        Ok(config)
    }
}

impl ConfigValues {
    pub(crate) fn from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("failed to read config at {}: {err}", path.display()),
            )
        })?;
        let table = content.parse::<toml::Table>().map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse config at {}: {err}", path.display()),
            )
        })?;

        Ok(Self {
            client_id: Self::required_toml_string(&table, TOML_CLIENT_ID, path)?,
            client_secret: Self::required_toml_string(&table, TOML_CLIENT_SECRET, path)?,
            redirect_uri: Self::required_toml_string(&table, TOML_REDIRECT_URI, path)?,
            access_token: Self::required_toml_string(&table, TOML_ACCESS_TOKEN, path)?,
            refresh_token: Self::required_toml_string(&table, TOML_REFRESH_TOKEN, path)?,
        })
    }

    pub(crate) fn write_to_global_path(&self) -> io::Result<()> {
        let path = global_config_path()?;
        let mut table = toml::Table::new();

        table.insert(
            TOML_CLIENT_ID.to_string(),
            Value::String(self.client_id.clone()),
        );
        table.insert(
            TOML_CLIENT_SECRET.to_string(),
            Value::String(self.client_secret.clone()),
        );
        table.insert(
            TOML_REDIRECT_URI.to_string(),
            Value::String(self.redirect_uri.clone()),
        );
        table.insert(
            TOML_ACCESS_TOKEN.to_string(),
            Value::String(self.access_token.clone()),
        );
        table.insert(
            TOML_REFRESH_TOKEN.to_string(),
            Value::String(self.refresh_token.clone()),
        );

        let content = toml::to_string_pretty(&table).map_err(|e| {
            io::Error::other(format!("failed to serialize global config toml: {e}"))
        })?;
        fs::write(path, format!("{content}\n"))?;

        Ok(())
    }

    pub(crate) fn with_refreshed_tokens(
        &self,
        access_token: String,
        refresh_token: Option<String>,
    ) -> Self {
        Self {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            redirect_uri: self.redirect_uri.clone(),
            access_token,
            refresh_token: refresh_token.unwrap_or_else(|| self.refresh_token.clone()),
        }
    }

    fn from_env() -> io::Result<Option<Self>> {
        let entries = [
            ENV_CLIENT_ID,
            ENV_CLIENT_SECRET,
            ENV_REDIRECT_URI,
            ENV_ACCESS_TOKEN,
            ENV_REFRESH_TOKEN,
        ]
        .map(|key| {
            (
                key,
                env::var(key).ok().filter(|value| !value.trim().is_empty()),
            )
        });

        if entries.iter().all(|(_, value)| value.is_none()) {
            return Ok(None);
        }

        let [
            (_, Some(client_id)),
            (_, Some(client_secret)),
            (_, Some(redirect_uri)),
            (_, Some(access_token)),
            (_, Some(refresh_token)),
        ] = entries
        else {
            let missing = entries
                .iter()
                .filter(|(_, value)| value.is_none())
                .map(|(env_key, _)| *env_key)
                .collect::<Vec<_>>()
                .join(", ");

            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "partial PUDDLE_* environment configuration detected. Missing or empty: {missing}"
                ),
            ));
        };

        Ok(Some(Self {
            client_id,
            client_secret,
            redirect_uri,
            access_token,
            refresh_token,
        }))
    }

    fn required_toml_string(table: &toml::Table, key: &str, path: &Path) -> io::Result<String> {
        let value = table.get(key).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("missing `{key}` in global config at {}", path.display()),
            )
        })?;
        let value = value.as_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "`{key}` in global config at {} must be a string",
                    path.display()
                ),
            )
        })?;
        if value.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "`{key}` in global config at {} cannot be empty",
                    path.display()
                ),
            ));
        }
        Ok(value.to_string())
    }
}

pub fn global_config_path() -> io::Result<PathBuf> {
    let base_dir = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or_else(|| io::Error::other("HOME is not set and XDG_CONFIG_HOME is unset"))?;

    let dir = base_dir.join("puddle");
    fs::create_dir_all(&dir)?;

    Ok(dir.join("config.toml"))
}
