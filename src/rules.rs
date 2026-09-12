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

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(subject: &str, body: &str, files_changed: usize) -> Commit {
        Commit {
            hash: "abc1234def5678".to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            files_changed,
        }
    }

    #[test]
    fn vague_subject_flags_known_vague_words() {
        assert!(VagueSubject.check(&commit("fix", "", 1)).is_some());
        assert!(VagueSubject.check(&commit("wip", "", 1)).is_some());
        assert!(VagueSubject
            .check(&commit("update login page", "", 1))
            .is_some());
    }

    #[test]
    fn vague_subject_allows_descriptive_messages() {
        assert!(VagueSubject
            .check(&commit("feat: reject expired refresh tokens", "", 1))
            .is_none());
    }

    #[test]
    fn subject_too_long_flags_over_72_chars() {
        let long_subject = "a".repeat(73);
        assert!(SubjectTooLong
            .check(&commit(&long_subject, "", 1))
            .is_some());
    }

    #[test]
    fn subject_too_long_allows_exactly_72_chars() {
        let subject = "a".repeat(72);
        assert!(SubjectTooLong.check(&commit(&subject, "", 1)).is_none());
    }

    #[test]
    fn missing_body_flags_big_change_without_body() {
        assert!(MissingBodyOnBigChange
            .check(&commit("feat: big change", "", 5))
            .is_some());
    }

    #[test]
    fn missing_body_allows_big_change_with_body() {
        assert!(MissingBodyOnBigChange
            .check(&commit("feat: big change", "explains why", 5))
            .is_none());
    }

    #[test]
    fn missing_body_allows_small_change_without_body() {
        assert!(MissingBodyOnBigChange
            .check(&commit("feat: small change", "", 4))
            .is_none());
    }

    #[test]
    fn oversized_commit_flags_15_or_more_files() {
        assert!(OversizedCommit
            .check(&commit("feat: sprawling change", "", 15))
            .is_some());
    }

    #[test]
    fn oversized_commit_allows_under_15_files() {
        assert!(OversizedCommit
            .check(&commit("feat: normal change", "", 14))
            .is_none());
    }

    #[test]
    fn conventional_commit_format_accepts_valid_shapes() {
        let rule = ConventionalCommitFormat::new();
        assert!(rule
            .check(&commit("feat: add token refresh support", "", 1))
            .is_none());
        assert!(rule
            .check(&commit("fix(auth): handle expired sessions", "", 1))
            .is_none());
        assert!(rule
            .check(&commit("feat!: breaking change to config format", "", 1))
            .is_none());
    }

    #[test]
    fn conventional_commit_format_rejects_invalid_shapes() {
        let rule = ConventionalCommitFormat::new();
        assert!(rule.check(&commit("random subject line", "", 1)).is_some());
        assert!(rule
            .check(&commit("Feat: wrong case for type", "", 1))
            .is_some());
    }

    #[test]
    fn subject_full_stop_flags_trailing_period() {
        assert!(SubjectFullStop
            .check(&commit("fix(auth): resolve token bug.", "", 1))
            .is_some());
    }

    #[test]
    fn subject_full_stop_allows_no_trailing_period() {
        assert!(SubjectFullStop
            .check(&commit("fix(auth): resolve token bug", "", 1))
            .is_none());
    }

    #[test]
    fn subject_full_stop_example_strips_the_period() {
        let example = SubjectFullStop
            .example(&commit("fix(auth): resolve token bug.", "", 1))
            .unwrap();
        assert_eq!(example, "fix(auth): resolve token bug");
    }

    #[test]
    fn imperative_mood_flags_past_tense_and_gerund() {
        assert!(ImperativeMood
            .check(&commit("feat: added token refresh", "", 1))
            .is_some());
        assert!(ImperativeMood
            .check(&commit("fixing the auth bug", "", 1))
            .is_some());
    }

    #[test]
    fn imperative_mood_allows_imperative_verbs() {
        assert!(ImperativeMood
            .check(&commit("feat: add token refresh support", "", 1))
            .is_none());
    }

    #[test]
    fn description_part_strips_conventional_commit_prefix() {
        assert_eq!(
            description_part("feat(auth): add token refresh"),
            "add token refresh"
        );
        assert_eq!(description_part("no prefix here"), "no prefix here");
    }
}
