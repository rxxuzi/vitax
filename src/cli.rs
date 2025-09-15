//! Command line interface definitions.

use clap::Parser;

/// Command line arguments for vitax.
#[derive(Parser, Debug, Clone)]
#[command(name = "vitax")]
#[command(about = "A safe directory analysis tool")]
#[command(version)]
pub struct Args {
    /// Input paths to analyze
    pub paths: Vec<String>,

    /// Maximum recursion depth
    #[arg(short = 'd', long = "depth", default_value = "10")]
    pub max_depth: usize,

    /// Patterns to ignore (can be used multiple times)
    #[arg(short = 'I', long = "ignore")]
    pub ignore: Vec<String>,

    /// Filter by file extensions (can be used multiple times)
    #[arg(short = 'e', long = "ext")]
    pub extensions: Vec<String>,

    /// Show hidden files and directories
    #[arg(short = 'a', long = "all")]
    pub show_hidden: bool,

    /// Verbose output (show skipped files and errors)
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}