use anyhow::{bail, Context, Result};
use std::process::Command;

/// A single commit, as far as gitgroom cares.
pub struct Commit {
    pub hash: String,
    pub subject: String,
    pub body: String,
    pub files_changed: usize,
}

/// What slice of history to read.
pub enum LogQuery {
    /// The N most recent commits, counted from HEAD backwards.
    LastN(usize),
    /// A single, specific commit.
    SingleCommit(String),
}

/// Runs `git log` and returns the matching commits.
pub fn log(query: &LogQuery) -> Result<Vec<Commit>> {
    // %x1f (unit separator) and %x1e (record separator) keep parsing simple
    // even if commit messages contain newlines or unusual characters.
    let format = "%H%x1f%s%x1f%b%x1e";

    let mut args = vec!["log".to_string(), format!("--pretty=format:{format}")];
    match query {
        // `-n` counts from HEAD backwards and never errors on short
        // histories the way a "HEAD~N..HEAD" range does.
        LogQuery::LastN(n) => args.push(format!("-n{n}")),
        LogQuery::SingleCommit(hash) => {
            args.push("-n1".to_string());
            args.push(hash.clone());
        }
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .context("failed to run `git log` — is git installed and is this a git repo?")?;

    if !output.status.success() {
        bail!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for record in raw.split('\u{1e}').filter(|r| !r.trim().is_empty()) {
        let parts: Vec<&str> = record.trim_start_matches('\n').split('\u{1f}').collect();
        if parts.len() < 3 {
            continue;
        }
        let hash = parts[0].to_string();
        let files_changed = files_changed_in(&hash).unwrap_or(0);
        commits.push(Commit {
            hash,
            subject: parts[1].to_string(),
            body: parts[2].trim().to_string(),
            files_changed,
        });
    }

    Ok(commits)
}

fn files_changed_in(hash: &str) -> Result<usize> {
    let output = Command::new("git")
        .args(["show", "--stat", "--format=", hash])
        .output()?;
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.lines().filter(|l| l.contains('|')).count())
}
