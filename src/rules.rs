use crate::git::Commit;
use regex::Regex;

/// A single problem found in a commit, produced by a Rule.
pub struct Finding {
    pub rule_name: &'static str,
    pub message: String,
    /// A purely illustrative example of a better message, filled in from
    /// `Rule::example` after `check` reports a problem. Never applied to
    /// git — display only.
    pub example: Option<String>,
}

/// Something that inspects a commit and optionally reports a problem.
/// Adding a new check means implementing this trait and registering it
/// in `all_rules()` below — nothing else needs to change.
pub trait Rule {
    fn name(&self) -> &'static str;
    fn check(&self, commit: &Commit) -> Option<Finding>;

    /// A purely illustrative example of what a good message could look like
    /// for this commit — never applied to git, just shown in the report.
    fn example(&self, _commit: &Commit) -> Option<String> {
        None
    }
}

/// The description part of a subject, stripping a leading Conventional
/// Commits `type(scope): ` prefix when present.
fn description_part(subject: &str) -> &str {
    match subject.split_once(": ") {
        Some((_, rest)) => rest,
        None => subject,
    }
}

pub struct VagueSubject;
impl Rule for VagueSubject {
    fn name(&self) -> &'static str {
        "vague-subject"
    }
    fn check(&self, commit: &Commit) -> Option<Finding> {
        const VAGUE: [&str; 6] = ["fix", "wip", "update", "stuff", "changes", "test"];
        let subject = commit.subject.trim().to_lowercase();
        if VAGUE
            .iter()
            .any(|v| subject == *v || subject.starts_with(&format!("{v} ")))
        {
            return Some(Finding {
                rule_name: self.name(),
                message: format!(
                    "subject \"{}\" is too vague to be useful later",
                    commit.subject
                ),
                example: None,
            });
        }
        None
    }
    fn example(&self, _commit: &Commit) -> Option<String> {
        Some("feat: reject expired refresh tokens on renew".to_string())
    }
}

pub struct SubjectTooLong;
impl Rule for SubjectTooLong {
    fn name(&self) -> &'static str {
        "subject-too-long"
    }
    fn check(&self, commit: &Commit) -> Option<Finding> {
        const MAX_LEN: usize = 72;
        if commit.subject.len() > MAX_LEN {
            return Some(Finding {
                rule_name: self.name(),
                message: format!(
                    "subject is {} chars, keep it under {MAX_LEN} for readability",
                    commit.subject.len()
                ),
                example: None,
            });
        }
        None
    }
}

pub struct MissingBodyOnBigChange;
impl Rule for MissingBodyOnBigChange {
    fn name(&self) -> &'static str {
        "missing-body-on-big-change"
    }
    fn check(&self, commit: &Commit) -> Option<Finding> {
        const BIG_CHANGE_FILES: usize = 5;
        if commit.files_changed >= BIG_CHANGE_FILES && commit.body.is_empty() {
            return Some(Finding {
                rule_name: self.name(),
                message: format!(
                    "touches {} files but has no body explaining why",
                    commit.files_changed
                ),
                example: None,
            });
        }
        None
    }
    fn example(&self, commit: &Commit) -> Option<String> {
        Some(format!(
            "{}\n\n<a short paragraph explaining WHY this change touches {} files>",
            commit.subject, commit.files_changed
        ))
    }
}

pub struct OversizedCommit;
impl Rule for OversizedCommit {
    fn name(&self) -> &'static str {
        "oversized-commit"
    }
    fn check(&self, commit: &Commit) -> Option<Finding> {
        const TOO_MANY_FILES: usize = 15;
        if commit.files_changed >= TOO_MANY_FILES {
            return Some(Finding {
                rule_name: self.name(),
                message: format!(
                    "touches {} files, consider splitting into smaller commits",
                    commit.files_changed
                ),
                example: None,
            });
        }
        None
    }
}

/// Checks the subject against the Conventional Commits shape:
/// `type(scope): description`, with an optional `!` for breaking changes.
pub struct ConventionalCommitFormat {
    regex: Regex,
}

impl ConventionalCommitFormat {
    pub fn new() -> Self {
        // Compiled once per run; the pattern is a fixed literal so this
        // can never fail in practice.
        let regex = Regex::new(
            r"^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([a-z0-9_-]+\))?!?: .+",
        )
        .expect("hardcoded regex is valid");
        Self { regex }
    }
}

impl Rule for ConventionalCommitFormat {
    fn name(&self) -> &'static str {
        "conventional-commit-format"
    }
    fn check(&self, commit: &Commit) -> Option<Finding> {
        if self.regex.is_match(&commit.subject) {
            None
        } else {
            Some(Finding {
                rule_name: self.name(),
                message: "subject doesn't follow the Conventional Commits `type(scope): description` format".to_string(),
                example: None,
            })
        }
    }
    fn example(&self, _commit: &Commit) -> Option<String> {
        Some("feat(scope): add a short imperative description".to_string())
    }
}

pub struct SubjectFullStop;
impl Rule for SubjectFullStop {
    fn name(&self) -> &'static str {
        "subject-full-stop"
    }
    fn check(&self, commit: &Commit) -> Option<Finding> {
        if commit.subject.trim_end().ends_with('.') {
            Some(Finding {
                rule_name: self.name(),
                message: "subject ends with a period, drop it".to_string(),
                example: None,
            })
        } else {
            None
        }
    }
    fn example(&self, commit: &Commit) -> Option<String> {
        Some(commit.subject.trim_end().trim_end_matches('.').to_string())
    }
}

/// Heuristic only: flags description text that starts with a past-tense or
/// gerund verb ("added", "fixing") instead of the imperative mood git itself
/// uses in its own generated messages ("add", "fix"). Simple suffix check,
/// not real NLP — false positives on words like "existing" are possible.
pub struct ImperativeMood;
impl Rule for ImperativeMood {
    fn name(&self) -> &'static str {
        "imperative-mood"
    }
    fn check(&self, commit: &Commit) -> Option<Finding> {
        let description = description_part(&commit.subject);
        let first_word = description.split_whitespace().next()?.to_lowercase();
        let looks_past_tense = first_word.len() > 3 && first_word.ends_with("ed");
        let looks_gerund = first_word.len() > 4 && first_word.ends_with("ing");
        if looks_past_tense || looks_gerund {
            Some(Finding {
                rule_name: self.name(),
                message: format!(
                    "\"{first_word}\" doesn't look imperative — prefer \"add\" over \"added\"/\"adding\""
                ),
                example: None,
            })
        } else {
            None
        }
    }
}

/// All active rules, in the order they should be evaluated.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(ConventionalCommitFormat::new()),
        Box::new(VagueSubject),
        Box::new(SubjectTooLong),
        Box::new(SubjectFullStop),
        Box::new(ImperativeMood),
        Box::new(MissingBodyOnBigChange),
        Box::new(OversizedCommit),
    ]
}
