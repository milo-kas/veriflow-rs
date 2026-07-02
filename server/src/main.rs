use std::path::PathBuf;

use server::{server::Listener, Config, Directory, Network};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[tokio::main]

async fn main() -> common::Result<()> {
    let config_exists = tokio::fs::try_exists(server::CONFIG_PATH).await?;
    if !config_exists {
        let path_exists = tokio::fs::try_exists(server::FILE_PATH).await?;
        if !path_exists {
            tokio::fs::create_dir_all(server::FILE_PATH).await?;
        }
        let config_content: Config = Config {
            network: (Network {
                ip: "127.0.0.1".to_string(),
                port: "8080".to_string(),
            }),
            directory: (Directory {
                path: PathBuf::from(server::FILE_PATH),
            }),
        };
        let _ = tokio::fs::File::create(server::CONFIG_PATH).await?;
        let mut config_file = tokio::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(server::CONFIG_PATH)
            .await?;
        let string_content = toml::to_string(&config_content)?;
        config_file.write_all(string_content.as_bytes()).await?;
        config_file.flush().await?;
    }
    let mut config_file = tokio::fs::File::open(server::CONFIG_PATH).await?;
    let mut content = String::new();
    config_file.read_to_string(&mut content).await?;
    let config_struct: Config = toml::from_str(&content)?;
    let path_exists = tokio::fs::try_exists(&config_struct.directory.path).await?;
    if !path_exists {
        tokio::fs::create_dir_all(&config_struct.directory.path).await?;
    }
    tracing_subscriber::fmt::init();
    let mut listener =
        Listener::new(&config_struct.network.ip, &config_struct.network.port).await?;
    listener.listen(config_struct.directory.path).await?;
    Ok(())
}
