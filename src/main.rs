mod cli;
mod git;
mod report;
mod rules;

use anyhow::{bail, Result};
use clap::Parser;
use cli::{Cli, Commands};
use report::CommitReport;
use std::path::PathBuf;

const MAX_LIMIT: usize = 200;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check {
            limit,
            commit,
            last,
            output,
        } => run_check(limit, commit, last, output),
    }
}

fn run_check(
    limit: usize,
    commit: Option<String>,
    last: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    let query = if let Some(hash) = commit {
        git::LogQuery::SingleCommit(hash)
    } else if last {
        git::LogQuery::LastN(1)
    } else {
        if limit == 0 || limit > MAX_LIMIT {
            bail!("--limit must be between 1 and {MAX_LIMIT}");
        }
        git::LogQuery::LastN(limit)
    };

    let commits = git::log(&query)?;
    let active_rules = rules::all_rules();

    let reports: Vec<CommitReport> = commits
        .iter()
        .map(|commit| {
            let findings = active_rules
                .iter()
                .filter_map(|rule| {
                    let mut finding = rule.check(commit)?;
                    finding.example = rule.example(commit);
                    Some(finding)
                })
                .collect();
            CommitReport { commit, findings }
        })
        .collect();

    report::print(&reports, output.is_none())?;

    if let Some(path) = output {
        report::write_markdown(&path, &reports)?;
        println!("\nReport written to {}", path.display());
    }

    Ok(())
}
