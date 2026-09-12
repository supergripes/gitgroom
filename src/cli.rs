use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(
    name = "gitgroom",
    version,
    about = "Checks and grooms your git commit messages",
    long_about = "gitgroom scans your git history for messy commit messages \
                  (too vague, too long, missing context, not following \
                  Conventional Commits) and shows you what a better message \
                  would look like. Read-only by design."
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze commit history and print a report (read-only)
    Check {
        /// How many recent commits to scan (max 200)
        #[clap(short, long, default_value_t = 20, conflicts_with_all = &["commit", "last"])]
        limit: usize,

        /// Scan a single commit by hash instead of a range
        #[clap(long, conflicts_with_all = &["limit", "last"])]
        commit: Option<String>,

        /// Scan only the last commit (HEAD)
        #[clap(long, conflicts_with_all = &["limit", "commit"])]
        last: bool,

        /// Also write the report to this file as Markdown
        #[clap(long)]
        output: Option<PathBuf>,
    },
}
