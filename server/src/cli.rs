use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
