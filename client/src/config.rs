//! Client Config Struct

use common::VeriflowError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Config Struct
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)] // to only fill missing blanks
pub struct ClientConfig {
    pub ip: String,
    pub port: String,
    pub download_dir: PathBuf,
}

// Skeleton for the config file
impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            ip: String::from("127.0.0.1"),
            port: String::from("8080"),
            download_dir: PathBuf::from("../Veriflow/Downloads"),
        }
    }
}

impl ClientConfig {
    pub fn save(&self) -> Result<(), VeriflowError> {
        self.save_to(Path::new("config.toml")) // default path
    }

    // save configuration to path
    pub fn save_to(&self, path: &Path) -> Result<(), VeriflowError> {
        let toml_str = toml::to_string_pretty(self)?;

        std::fs::write(path, toml_str)?;

        Ok(())
    }

    pub fn load() -> Self {
        Self::load_from(Path::new("config.toml")) // default path
    }

    // load configuration from path
    pub fn load_from(path: &Path) -> Self {
        // Attempt to read the file
        let config_str = match std::fs::read_to_string(path) {
            Ok(content) => content,
            // Cant read file / no file found
            Err(_) => {
                eprintln!("Config file not found.");
                eprintln!("Creating a new one...");

                let default_config = Self::default();

                // create new config file
                let _ = default_config.save_to(path);

                return default_config;
            }
        };

        // Parse the TOML
        match toml::from_str(&config_str) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Config Error: {e}.");
                eprintln!("Using default settings...");
                Self::default()
            }
        }
    }

    // Helper function for full address (ip + port)
    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = ClientConfig::default();

        assert_eq!(config.ip, "127.0.0.1");
        assert_eq!(config.port, "8080");
        assert_eq!(config.download_dir, PathBuf::from("../Veriflow/Downloads"));
    }

    #[test]
    fn test_full_address_getter() {
        let mut config = ClientConfig::default();
        config.ip = "10001".to_string();
        config.port = "576".to_string();

        assert_eq!(config.address(), "10001:576");
    }

    #[test]
    fn test_save_and_load_custom_path() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let config_path = dir.path().join("random123.toml");

        let test_ip = "164.100.1.1".to_string();
        let test_port = "4040".to_string();
        let test_download_dir = PathBuf::from("tmp/some dir");

        // set custom config
        let config = ClientConfig {
            ip: test_ip,
            port: test_port,
            download_dir: test_download_dir,
        };

        // save
        config.save_to(&config_path)?;

        // load
        let loaded_config = ClientConfig::load_from(&config_path);
        assert_eq!(loaded_config.ip, config.ip);
        assert_eq!(loaded_config.port, config.port);
        assert_eq!(loaded_config.download_dir, config.download_dir);

        Ok(())
    }

    #[test]
    fn test_load_from_malformed_toml() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "malformed...")?;

        let config = ClientConfig::load_from(&path);
        let default = ClientConfig::default();

        assert_eq!(config.ip, default.ip);
        assert_eq!(config.port, default.port);
        assert_eq!(config.download_dir, default.download_dir);
        Ok(())
    }

    #[test]
    fn test_load_from_partial_toml() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "ip = \"164.100.1.1\"")?;

        let config = ClientConfig::load_from(&path);
        let default = ClientConfig::default();

        assert_eq!(config.ip, "164.100.1.1");
        assert_eq!(config.port, default.port);
        assert_eq!(config.download_dir, default.download_dir);
        Ok(())
    }

    #[test]
    fn test_load_from_missing_config_file() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().join("config.toml");
        // verify the file does not exist
        assert!(!path.exists());

        let config = ClientConfig::load_from(&path);
        let default = ClientConfig::default();

        assert_eq!(config.ip, default.ip);
        assert_eq!(config.port, default.port);
        assert_eq!(config.download_dir, default.download_dir);
        Ok(())
    }
}
