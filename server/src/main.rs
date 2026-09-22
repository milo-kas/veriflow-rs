use std::path::{Path, PathBuf};

use clap::Parser;
use common::config::ServerConfig;
use server::{cli::Args, cli::Commands, server::Listener};
#[tokio::main]

async fn main() -> common::Result<()> {
    let args = Args::parse();
    match args.command {
        Some(Commands::Config { ip, port, dir }) => {
            let config_exists = tokio::fs::try_exists(server::CONFIG_PATH).await?;
            if config_exists {
                let mut config = ServerConfig::load_from(Path::new(server::CONFIG_PATH));
                if let Some(ip_str) = ip.flatten() {
                    config.ip = ip_str;
                }
                if let Some(port_str) = port.flatten() {
                    config.port = port_str;
                }
                if let Some(dir_str) = dir.flatten() {
                    config.resource_dir = PathBuf::from(dir_str);
                }
                config.save_to(Path::new(server::CONFIG_PATH))?;
            } else {
                let ip_str = ip.and_then(|ip_value| ip_value);
                let port_str = port.and_then(|port_value| port_value);
                let dir_path = dir.and_then(|dir_value| dir_value.map(PathBuf::from));
                let config = ServerConfig {
                    ip: ip_str.unwrap_or_else(|| "127.0.0.1".into()),
                    port: port_str.unwrap_or_else(|| "8080".into()),
                    resource_dir: dir_path.unwrap_or_else(|| PathBuf::from(server::FILE_PATH)),
                };
                config.save_to(Path::new(server::CONFIG_PATH))?;
            }
        }
        None => {
            let config_exists = tokio::fs::try_exists(server::CONFIG_PATH).await?;
            if !config_exists {
                let path_exists = tokio::fs::try_exists(server::FILE_PATH).await?;
                if !path_exists {
                    tokio::fs::create_dir_all(server::FILE_PATH).await?;
                }
                let config_content = ServerConfig {
                    ip: "127.0.0.1".into(),
                    port: "8080".into(),
                    resource_dir: PathBuf::from(server::FILE_PATH),
                };
                config_content.save_to(Path::new(server::CONFIG_PATH))?;
            }
            let config_struct = ServerConfig::load_from(Path::new(server::CONFIG_PATH));
            let path_exists = tokio::fs::try_exists(&config_struct.resource_dir).await?;
            if !path_exists {
                tokio::fs::create_dir_all(&config_struct.resource_dir).await?;
            }
            tracing_subscriber::fmt::init();
            let mut listener = Listener::new(&config_struct.ip, &config_struct.port).await?;
            listener.listen(config_struct.resource_dir).await?;
        }
    }
    Ok(())
}
