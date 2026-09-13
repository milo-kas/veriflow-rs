use std::path::PathBuf;

use clap::Parser;
use server::{cli::Args, cli::Commands, config::Config, server::Listener};
#[tokio::main]

async fn main() -> common::Result<()> {
    let args = Args::parse();
    match args.command {
        Some(Commands::Config { ip, port, dir }) => {
            let config_exists = tokio::fs::try_exists(server::CONFIG_PATH).await?;
            if config_exists {
                let mut config = Config::load_from_file().await?;
                if let Some(ip_str) = ip.flatten() {
                    config.network.ip = ip_str;
                }
                if let Some(port_str) = port.flatten() {
                    config.network.port = port_str;
                }
                if let Some(dir_str) = dir.flatten() {
                    config.directory.path = PathBuf::from(dir_str);
                }
                config.create_config_file().await?;
            } else {
                let ip_str = ip.and_then(|ip_value| ip_value);
                let port_str = port.and_then(|port_value| port_value);
                let dir_path = dir.and_then(|dir_value| dir_value.map(PathBuf::from));
                let config = Config::new(
                    ip_str.unwrap_or_else(|| "127.0.0.1".into()),
                    port_str.unwrap_or_else(|| "8080".into()),
                    dir_path.unwrap_or_else(|| PathBuf::from(server::FILE_PATH)),
                );
                config.create_config_file().await?;
            }
        }
        None => {
            let config_exists = tokio::fs::try_exists(server::CONFIG_PATH).await?;
            if !config_exists {
                let path_exists = tokio::fs::try_exists(server::FILE_PATH).await?;
                if !path_exists {
                    tokio::fs::create_dir_all(server::FILE_PATH).await?;
                }
                let config_content = Config::init();
                config_content.create_config_file().await?;
            }
            let config_struct = Config::load_from_file().await?;
            let path_exists = tokio::fs::try_exists(&config_struct.directory.path).await?;
            if !path_exists {
                tokio::fs::create_dir_all(&config_struct.directory.path).await?;
            }
            tracing_subscriber::fmt::init();
            let mut listener =
                Listener::new(&config_struct.network.ip, &config_struct.network.port).await?;
            listener.listen(config_struct.directory.path).await?;
        }
    }
    Ok(())
}
