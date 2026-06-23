use clap::Parser;

use crate::cli::{Args, Commands};
use common::VeriflowError;

mod cli;
mod config;
mod transfer;
mod ui;

// Start tokio engine
#[tokio::main]
async fn main() -> Result<(), VeriflowError> {
    // Parse CLI arguments
    let args = Args::parse();

    // Load config
    let mut config = config::ClientConfig::load();

    // Handle CLI arguments
    match args.command {
        // Config
        Commands::Config { 
            ip, 
            port, 
            dir 
        } => {
            if let Some(new_ip) = ip {
                config.ip = new_ip;
            }
            if let Some(new_port) = port {
                config.port = new_port;
            }
            if let Some(new_dir) = dir {
                config.download_dir = new_dir.into();
            }

            config.save()?;
            println!("Configuration saved.")
        }

        // Transfer
        Commands::Transfer {
            ip,
            upload,
            download,
            delete,
            list,
        } => {
            // See if CLI argument was passed otherwise use config
            let target_ip = ip.unwrap_or_else(|| config.address());

            // Let the result of the function that is called via cli args be handled by VeriflowError
            // Use Some operator for Option
            if let Some(path) = upload {
                // Upload
                transfer::upload_file(&path, &target_ip).await?;
            } else if let Some(path) = download {
                // Download
                transfer::download_file(&path, &target_ip, &config.download_dir).await?;
            } else if let Some(path) = delete {
                // Delete
                transfer::delete_file(&path, &target_ip).await?;
            } else if list {
                // List
                transfer::list_files(&target_ip).await?;
            };

            println!("Success!");
        }
    }
    Ok(())
}
