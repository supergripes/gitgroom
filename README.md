# gitgroom

A CLI tool that checks your git commit messages and shows you what a better
one would look like — before they become permanent regrets.

## Why

Commit messages are documentation nobody reviews until something breaks and
you need to know *why* a change happened. `gitgroom` scans your history for
common problems (vague messages, missing context on big changes, oversized
commits, messages that don't follow Conventional Commits) and reports them,
alongside an illustrative example of a better message. It never touches your
history — read-only by design.

## Usage

```
# Scan the last 20 commits (default)
gitgroom check

# Scan a specific number of recent commits (max 200)
gitgroom check --limit 50

# Scan a single commit by hash
gitgroom check --commit a3f9c21

# Scan only the last commit (handy in a commit-msg hook)
gitgroom check --last

# Also save the report as Markdown
gitgroom check --output report.md
```

`--limit`, `--commit`, and `--last` are mutually exclusive.

When more than one commit is flagged and you're running in a real terminal,
`check` walks through them one at a time with a `[n]ext [p]revious [q]uit`
prompt. Piping the output (or using `--output`) prints everything at once
instead, so scripts and CI never hang waiting on stdin.

## What `check` looks for

- **conventional-commit-format** — subject doesn't match Conventional Commits'
  `type(scope): description` shape
- **vague-subject** — messages like "fix", "wip", "update" that say nothing
  useful six months from now
- **subject-too-long** — first line over 72 characters
- **subject-full-stop** — first line ends with a period
- **imperative-mood** — heuristic check for non-imperative phrasing ("added",
  "fixing") instead of git's own convention ("add", "fix")
- **missing-body-on-big-change** — 5+ files changed with no body explaining why
- **oversized-commit** — 15+ files changed, probably should've been split up

Rules live in `src/rules.rs` as small, independent checks — adding a new one
means implementing the `Rule` trait and adding it to `all_rules()`, nothing
else needs to change. A rule can optionally implement `example()` to show a
purely illustrative "this is what a good message could look like" hint next
to its finding — it's never applied to git, just displayed.

## Building

```
cargo build --release
./target/release/gitgroom check
```

This produces a native binary at `target/release/gitgroom` — no crates.io
publish or package manager needed to try it.

## Trying it on another repo locally

`gitgroom` isn't published anywhere yet, but you don't need it to be to test
it against a real repository:

**Option 1 — run the compiled binary directly.** From inside the repo you
want to check:

```
cd /path/to/some/other/repo
/path/to/gitgroom/target/release/gitgroom check
```

`gitgroom` shells out to `git` in the current working directory, so you need
to `cd` into the target repo first — it doesn't take a `--repo-path`-style
flag.

**Option 2 — install it on your `PATH`.** From inside this repo:

```
cargo install --path .
```

This builds in release mode and copies the binary to `~/.cargo/bin/gitgroom`
(already on `PATH` if you installed Rust via rustup). From then on, just:

```
cd /path/to/some/other/repo
gitgroom check --limit 30
```

After changing the code, re-run `cargo install --path . --force` to update
the installed binary.

## License

MIT
