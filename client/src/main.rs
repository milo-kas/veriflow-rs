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
        Commands::Config { ip, port, dir } => {
            let mut updated = false;

            macro_rules! handle {
                ($flag:expr, $field:ident, $display:expr) => {
                    match $flag {
                        // flag with value => set
                        Some(Some(value)) => {
                            config.$field = value.into();
                            true
                        }
                        // flag with no value => get
                        Some(None) => {
                            // use display due to PathBuf
                            println!("{}", $display);
                            false
                        }
                        // otherwise simply no update or ouput
                        None => false,
                    }
                };
            }

            updated |= handle!(ip, ip, config.ip);
            updated |= handle!(port, port, config.port);
            updated |= handle!(dir, download_dir, config.download_dir.display());

            if updated {
                config.save()?;
                println!("Configuration saved.")
            }
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
