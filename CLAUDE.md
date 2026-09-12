# CLAUDE.md

Guidance for Claude Code when working in this repository.

## What this is

`gitgroom` is intentionally small in scope. Keep changes minimal and
focused — flag anything that would meaningfully grow the project instead of
just doing it.

It scans git commit messages and reports problems (vague subjects, missing
context, not following Conventional Commits), showing an illustrative example
of a better message. **It is read-only by design and must stay that way** —
it never writes to git history. A `fix` command that would amend/rebase
commits was deliberately dropped as too risky for what it added; don't
reintroduce automatic history rewriting without the user explicitly asking
for it again.

## Architecture

- [src/cli.rs](src/cli.rs) — clap-derive argument parsing only. No logic.
- [src/git.rs](src/git.rs) — the only module that shells out to `git`
  (`std::process::Command`). Runs in the process's current working directory;
  there's no `--repo-path`-style flag.
- [src/rules.rs](src/rules.rs) — the `Rule` trait (`check` + optional
  `example`) and all rule implementations, registered in `all_rules()`.
  Adding a rule means implementing the trait and adding one line there —
  nothing else should need to change.
- [src/report.rs](src/report.rs) — terminal rendering (colors via
  `owo-colors`, a next/previous/quit pager for multi-commit results when
  stdout is a real terminal) and Markdown export.
- [src/main.rs](src/main.rs) — orchestrates the above; thin on purpose.

## Conventions

- Commit messages in this repo follow Conventional Commits
  (`type(scope): description`) — the tool would flag its own history
  otherwise. Keep subjects imperative and under 72 chars.
- Only commit or push when explicitly asked.
- No new dependencies unless they clearly earn their place (current set:
  `clap`, `owo-colors`, `anyhow`, `regex` — deliberately small).

## Build & test

```bash
cargo build --release
./target/release/gitgroom check
```

To try it against another repo without publishing anywhere, either run the
compiled binary with an absolute path from inside that repo's directory, or
`cargo install --path .` to get `gitgroom` on PATH.
