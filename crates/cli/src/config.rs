use crate::constants::{
    TOML_ACCESS_TOKEN, TOML_CLIENT_ID, TOML_CLIENT_SECRET, TOML_REDIRECT_URI, TOML_REFRESH_TOKEN,
};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use toml::Value;

pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub access_token: String,
    pub refresh_token: String,
}

impl Config {
    pub fn from_path(path: impl AsRef<Path>) -> io::Result<Self> {
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

    pub fn write_to_global_path(&self) -> io::Result<()> {
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
