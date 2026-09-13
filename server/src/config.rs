use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub network: Network,
    pub directory: Directory,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Network {
    pub ip: String,
    pub port: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Directory {
    pub path: PathBuf,
}

impl Config {
    pub fn init() -> Self {
        Self {
            network: Network {
                ip: "127.0.0.1".to_string(),
                port: "8080".to_string(),
            },
            directory: Directory {
                path: PathBuf::from(crate::FILE_PATH),
            },
        }
    }

    pub fn new(ip: String, port: String, path: PathBuf) -> Self {
        Self {
            network: Network { ip, port },
            directory: Directory { path },
        }
    }

    pub async fn create_config_file(&self) -> common::Result<()> {
        let _ = tokio::fs::File::create(crate::CONFIG_PATH).await?;
        let mut config_file = tokio::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(crate::CONFIG_PATH)
            .await?;
        let string_content = toml::to_string(&self)?;
        config_file.write_all(string_content.as_bytes()).await?;
        config_file.flush().await?;
        Ok(())
    }

    pub async fn load_from_file() -> common::Result<Self> {
        let mut config_file = tokio::fs::File::open(crate::CONFIG_PATH).await?;
        let mut content = String::new();
        config_file.read_to_string(&mut content).await?;
        let config_struct: Config = toml::from_str(&content)?;
        Ok(config_struct)
    }
}
