//! CLI Arg Parsing Struct

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// File transfer operations (upload, download, delete, list)
    #[command(group(
  clap::ArgGroup::new("operation")
    .required(true)
    .args(["upload", "download", "delete", "list"]),
  ))]
    Transfer {
        ///  IP of the server (host is added automatically as per config)
        #[arg(short, long)]
        ip: Option<String>,

        /// Upload file to server
        #[arg(short, long, group = "operation")]
        upload: Option<PathBuf>,

        /// Download file from server
        #[arg(short, long, group = "operation")]
        download: Option<PathBuf>,

        /// Delete file from server (full flag required for precaution)
        #[arg(long, group = "operation")]
        delete: Option<PathBuf>,

        /// List all files on server
        #[arg(short, long, group = "operation")]
        list: bool,
    },

    /// Set configuration file values (ip, port, dir)
    Config {
        /// Set new ip / get ip if no value is provided
        #[arg(short, long, num_args = 0..=1)]
        ip: Option<Option<String>>,

        /// Set new port / get port if no value is provided
        #[arg(short, long, num_args = 0..=1)]
        port: Option<Option<String>>,

        /// Set new download directory / get dir if no value is provided
        #[arg(short, long, num_args = 0..=1)]
        dir: Option<Option<String>>,
    },
}
