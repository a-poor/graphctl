///! Handles the CLI definition and parsing.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(
    name = "graphctl", 
    version = "v0.1.0", 
    author = "Austin Poor", 
    about = "A CLI for interacting with a local graph database."
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,

    #[clap(
        long, 
        global=true, 
        help="Path to the config directory. Defaults to $HOME/.graphctl",
    )]
    pub config_dir: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Add,
    Get,
    Set,
    Del,
    Meta,
    Init, 
    Cfg,
}

