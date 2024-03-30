mod cli;
mod conf;
mod db;

use clap::Parser;
use cli::Cli;

fn main() {
    // Load the CLI...
    let _app = Cli::parse();
}
