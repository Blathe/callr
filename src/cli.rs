use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "callr", about = "A TUI HTTP client")]
pub struct Cli {
    /// Path to a collection directory to open on launch
    pub collection_path: Option<PathBuf>,
}
