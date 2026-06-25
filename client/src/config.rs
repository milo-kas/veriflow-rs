//! Client Config Struct

use common::VeriflowError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub fn load() -> Self {
        // Attempt to read the file
        let config_str = match std::fs::read_to_string("config.toml") {
            Ok(content) => content,
            // Cant read file / no file found
            Err(_) => {
                eprintln!("Config file not found.");
                eprintln!("Creating a new one...");

                let default_config = Self::default();

                // create new config file
                let _ = default_config.save();

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

    pub fn save(&self) -> Result<(), VeriflowError> {
        let toml_str = toml::to_string_pretty(self)?;

        std::fs::write("config.toml", toml_str)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
