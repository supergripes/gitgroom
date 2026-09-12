use crate::git::Commit;
use crate::rules::Finding;
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::fmt::Write as _;
use std::io::{self, IsTerminal, Write as _};
use std::path::Path;

pub struct CommitReport<'a> {
    pub commit: &'a Commit,
    pub findings: Vec<Finding>,
}

fn short_hash(commit: &Commit) -> &str {
    &commit.hash[..7.min(commit.hash.len())]
}

fn render_page(r: &CommitReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{} {}", short_hash(r.commit).yellow().bold(), r.commit.subject);
    for finding in &r.findings {
        let _ = writeln!(
            out,
            "  {} [{}] {}",
            "!".red().bold(),
            finding.rule_name.dimmed(),
            finding.message
        );
        if let Some(example) = &finding.example {
            let _ = writeln!(
                out,
                "      {} {}",
                "e.g. \u{2192}".green(),
                example.replace('\n', "\n      ")
            );
        }
    }
    out
}

/// Prints a human-friendly report to stdout and returns the number of
/// commits that had at least one finding.
///
/// When `paginate` is true and there's more than one flagged commit on a
/// real terminal, findings are shown one commit at a time with a
/// next/previous/quit prompt. Otherwise everything is printed at once, so
/// piping output or writing to `--output` never hangs waiting on stdin.
pub fn print(reports: &[CommitReport], paginate: bool) -> Result<usize> {
    let flagged: Vec<&CommitReport> = reports.iter().filter(|r| !r.findings.is_empty()).collect();

    if flagged.is_empty() {
        println!("{}", "All checked commits look clean.".green().bold());
        return Ok(0);
    }

    let pages: Vec<String> = flagged.iter().map(|r| render_page(r)).collect();
    let interactive = paginate && pages.len() > 1 && io::stdout().is_terminal();

    if interactive {
        run_pager(&pages)?;
    } else {
        for page in &pages {
            print!("\n{page}");
        }
    }

    println!(
        "\n{} {} commit(s) flagged out of {} checked.",
        "Summary:".bold(),
        flagged.len(),
        reports.len()
    );

    Ok(flagged.len())
}

fn run_pager(pages: &[String]) -> Result<()> {
    let mut index = 0usize;
    loop {
        print!("\n{}", pages[index]);
        println!(
            "{}",
            format!(
                "-- commit {}/{} -- [n]ext  [p]revious  [q]uit",
                index + 1,
                pages.len()
            )
            .dimmed()
        );
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "p" => index = index.saturating_sub(1),
            "q" => break,
            _ => {
                // Enter, "n", or anything else advances; falls through to
                // quit once the last page has been seen.
                if index + 1 < pages.len() {
                    index += 1;
                } else {
                    break;
                }
            }
        }
    }
    Ok(())
}

/// Writes the same report to `path` as Markdown — handy as a CI artifact or
/// a screenshot-free way to show off results.
pub fn write_markdown(path: &Path, reports: &[CommitReport]) -> Result<()> {
    let flagged: Vec<&CommitReport> = reports.iter().filter(|r| !r.findings.is_empty()).collect();
    let mut out = String::new();

    let _ = writeln!(out, "# gitgroom report\n");

    if flagged.is_empty() {
        let _ = writeln!(out, "All checked commits look clean.");
    } else {
        for r in &flagged {
            let _ = writeln!(out, "## `{}` {}\n", short_hash(r.commit), r.commit.subject);
            for finding in &r.findings {
                let _ = writeln!(out, "- **[{}]** {}", finding.rule_name, finding.message);
                if let Some(example) = &finding.example {
                    let _ = writeln!(out, "\n  ```\n  {}\n  ```\n", example.replace('\n', "\n  "));
                }
            }
            let _ = writeln!(out);
        }
    }

    let _ = writeln!(
        out,
        "**Summary:** {} commit(s) flagged out of {} checked.",
        flagged.len(),
        reports.len()
    );

    std::fs::write(path, out)
        .with_context(|| format!("failed to write report to {}", path.display()))?;
    Ok(())
}
