# SP3 Repository Resolution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver V3 §35 phase 2 plus `link`/`unlink`: Git repository discovery that never runs Git, canonical repository identity, mappings and `default_profile` in `config.toml`, the full V3 §12 resolver, the `current`, `resolve`, `status`, `link` and `unlink` commands, and `agent-profile <agent>` launching the profile the current repository selects.

**Architecture:** `repo.rs` walks up from the working directory (or `--repo`) to the first `.git`, validates Git's `gitdir`/`commondir` metadata and refuses network and device targets, and canonicalizes paths. `config.rs` stores mappings keyed by the canonical root and edits them under the existing lock. `resolve.rs` is pure over the configuration and the discovery result. `cli.rs` dispatches the five command words before Clap, binds `--repo`, and runs discovery for a launch without a profile word.

**Tech Stack:** Rust 1.98 (edition 2024), clap 4, toml 1.1, toml_edit 0.25, tempfile 3, thiserror 2, cargo-nextest. No new dependencies.

**Design:** `docs/superpowers/specs/2026-09-15-sp3-resolution-design.md` (the oracle for behaviour; V3 `agent-profile-implementation-spec-v3.md` wins over both). SP1 design: `docs/superpowers/specs/2026-09-14-sp1-core-launch-design.md`; SP2 design: `docs/superpowers/specs/2026-09-15-sp2-adapters-design.md`.

---

## How this plan was produced, and how to execute it

Every code block below is copied byte-for-byte from a prototype of the whole design built against `e3f091d` (the SP3 design merged with `main` at `875ddc7`). The prototype passed CI run 35008163339 (draft PR #17, closed unmerged): Windows 219/219, Linux 207/207, macOS 207/207, Clippy, Format, Typos, Cargo deny, the docs build and the PR title check. Plan panel round 1 then found that a `link` or `unlink` that changes nothing rewrote a CRLF or byte-order-mark `config.toml`; the fix and its test, plus a Unix test for a `.git` that is neither a directory nor a file, were folded into the prototype, which then passed locally on Windows (220/220) and Linux (WSL Ubuntu 26.04: 209/209) with `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `typos` clean (macOS was not re-run for that fold). The plan was replayed task by task in a fresh worktree from `e3f091d`, running the gate after every task (Windows counts 158, 177, 191, 194, 199, 205, 220, 220); the replayed tree equals the prototype byte for byte, and every mutant below was run against the prototype and made its named test fail.

Rules for every task:

1. **Step 0 - state check.** On branch `sp3-resolution`, run `git status --short` (expect no output). `HEAD` must be the previous task's commit; for Task 1 it is the commit that added this plan or a later documentation commit. Then check the "Before" facts listed for the task. If any check fails, STOP and report `STATE_MISMATCH: <what>`.
2. **Byte-exact files.** Write each file with exactly the content shown: the whole file, not a merge. An "Edit" step replaces exactly the quoted text, which occurs once. The gate includes `cargo fmt --all -- --check`, so do not reformat. Do not "improve" any code or test.
3. **Shape-divergence stop.** If making the code compile would change the shape, type or encoding of anything shown, STOP and report `[original] -> [yours] because <reason>`. "It compiles" is not a justification.
4. **Oracle.** The named tests pin the behaviour, and the design is the oracle behind them. If a test fails, fix the code to match the test and the design; never edit a test to match the code.
5. **Gate.** Run the gate exactly as written; do not add flags.
6. **Mutant.** After committing, apply the mutant, run the command, confirm the named test FAILS, then restore with `git checkout -- <file>` and confirm `git status --short` prints nothing.
7. **Toolchain drift.** `rust-toolchain.toml` pins `stable`. If a gate fails with a clippy lint or compiler diagnostic in code copied from this plan, STOP and report the diagnostic; do not change the code to silence it.
8. **Windows desktop.** Every full-workspace test run on Windows opens short-lived console windows (SP1's console tests); do not type into them while tests run.
9. **No real agents.** Never install or run a coding agent for this plan. The tests use the `fake-agent` fixture and, in `repo.rs` only, the `git` command with an isolated configuration.
10. **Tests may not touch this checkout.** Every test layout lives in a temp directory checked by `assert_outside_any_repository` (or `guarded()` in `repo.rs`); if that check fails on your machine because a parent of the temp directory holds a `.git`, STOP and report it rather than moving the layout.

Gate commands (every task after its own checks):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
typos
cargo nextest run --workspace --no-tests=pass
```

Expected: `cargo fmt` and `typos` print nothing; clippy prints no warnings; nextest ends with `N tests run: N passed`, 0 failed. The `repo.rs` tests need `git` on `PATH` (present on every CI runner).

## File structure

| File | Task | Responsibility |
|---|---|---|
| `crates/agent-profile/src/error.rs` | 1 | `Error::Repository` (exit 4), `NoProfile { agent, config_file, in_repository }` (design §4.2, §5.3.1) |
| `crates/agent-profile/src/repo.rs` | 2 | `Discovery`, `discover`, `canonical`, `strip_verbatim`, `target_allowed`, `unlink_keys` (design §5, §6.2) |
| `crates/agent-profile/src/config.rs` | 3 | schema `default_profile` and `repositories`, `Mapping`, `Reference`, queries, `link`, `unlink`, `check_case_twins` (design §4.2, §6) |
| `crates/agent-profile/src/adapter/mod.rs` | 3, 4 | uses the moved `check_case_twins`; test call of the new `resolve` |
| `crates/agent-profile/src/resolve.rs` | 4 | the V3 §12 resolver (design §4.3) |
| `crates/agent-profile/src/output.rs` | 4, 5 | `resolve_lines`, `status_lines`, `link_line`, `unlink_lines`, `note` (design §7.5) |
| `crates/agent-profile/src/cli.rs` | 1, 4, 6 | top-level dispatch, `--repo` binding, command checks and usage, the commands, launch discovery (design §7) |
| `crates/agent-profile/tests/support/mod.rs` | 6 | `assert_outside_any_repository`; `Root` runs from its own guarded directory (design §8.4, §8.5) |
| `crates/agent-profile/tests/launch.rs` | 6 | the SP1 `link work extra` row (design §7.2) |
| `crates/agent-profile/tests/resolution.rs` | 7 | end-to-end resolution (design §8.4) |
| `README.md`, `TODO.md` | 8 | usage, resolution, configuration example, SP3 known limits (design §9) |

### Task 1: Repository and no-profile errors

`Error::Repository` (exit 4) and the `NoProfile` message that names every way to select a profile (design §4.2, §5.3.1).

**Files:**
- Modify (whole file): `crates/agent-profile/src/error.rs` (the two error variants, their messages and tests)
- Modify: `crates/agent-profile/src/cli.rs` (the launch fills the new fields; Task 6 replaces this code)

**Before:** `crates/agent-profile/src/error.rs` contains `NoProfile { agent: String },`; `crates/agent-profile/src/error.rs` does not contain `Repository {`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/error.rs`** (the two error variants, their messages and tests; the whole file, byte-exact)

```rust
//! Structured errors and their exit codes (spec §33).

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::name::InvalidReason;

/// Why an agent's executable was not found (spec §20).
#[derive(Debug)]
pub enum NotInstalledReason {
    /// No absolute `PATH` entry holds it; `ignored_relative` relative entries were skipped.
    NotOnPath {
        ignored_relative: usize,
    },
    ExplicitMissing(PathBuf),
}

/// Every error `agent-profile` reports. `exit_code` maps each variant to spec §33.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message}")]
    Usage { message: String },

    #[error("`{command}` is not yet implemented")]
    NotYetImplemented { command: String },

    #[error("{}", unknown_agent_message(agent, known, unknown_configured))]
    UnknownAgent { agent: String, known: Vec<String>, unknown_configured: Vec<String> },

    #[error(
        "`{option}` conflicts with how agent-profile selects the {agent} profile ({mechanism}); remove it from \
         the agent arguments"
    )]
    ArgumentConflict { agent: String, option: String, mechanism: &'static str },

    #[error("{}", not_installed_message(agent, reason, unknown_configured))]
    AgentNotInstalled { agent: String, reason: NotInstalledReason, unknown_configured: Vec<String> },

    #[error("invalid profile name {name:?}: {reason}")]
    InvalidProfileName { name: String, reason: InvalidReason },

    #[error("{}", no_profile_message(agent, config_file, *in_repository))]
    NoProfile { agent: String, config_file: PathBuf, in_repository: bool },

    #[error("repository {}: {reason}", path.display())]
    Repository { path: PathBuf, reason: String },

    #[error("{message}")]
    AppRoot { message: String },

    #[error("{}", config_invalid_message(path, key.as_deref(), detail))]
    ConfigInvalid { path: PathBuf, key: Option<String>, detail: String },

    #[error("could not write {}: {source}", path.display())]
    ConfigWrite { path: PathBuf, source: io::Error },

    #[error("profile path {}: {source}", path.display())]
    ProfileDir { path: PathBuf, source: io::Error },

    #[error(
        "profile `{requested}` differs only in letter case from the existing profile entry `{existing}`; \
         use `{existing}` or choose a different name"
    )]
    ProfileCaseConflict { requested: String, existing: String },

    #[error("{}", unsupported_message(agent, path, config_file))]
    UnsupportedExecutable { agent: String, path: PathBuf, config_file: PathBuf },

    #[error("could not launch {}: {source}", executable.display())]
    Launch { executable: PathBuf, source: io::Error },

    #[error("{context}: {source}")]
    Io { context: String, source: io::Error },
}

impl Error {
    /// The spec §33 exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Usage { .. }
            | Error::NotYetImplemented { .. }
            | Error::UnknownAgent { .. }
            | Error::ArgumentConflict { .. } => 2,
            Error::AgentNotInstalled { .. } => 3,
            Error::InvalidProfileName { .. }
            | Error::NoProfile { .. }
            | Error::AppRoot { .. }
            | Error::ConfigInvalid { .. }
            | Error::ConfigWrite { .. }
            | Error::ProfileDir { .. }
            | Error::ProfileCaseConflict { .. }
            | Error::Repository { .. } => 4,
            Error::UnsupportedExecutable { .. } | Error::Launch { .. } => 6,
            Error::Io { .. } => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn unknown_configured_suffix(unknown_configured: &[String]) -> String {
    if unknown_configured.is_empty() {
        String::new()
    } else {
        format!("; config.toml also configures unknown agents: {}", backticked(unknown_configured))
    }
}

fn backticked(items: &[String]) -> String {
    items.iter().map(|item| format!("`{item}`")).collect::<Vec<_>>().join(", ")
}

fn unknown_agent_message(agent: &str, known: &[String], unknown_configured: &[String]) -> String {
    let known = if known.is_empty() {
        "no agents are available in this build".to_owned()
    } else {
        format!("known agents: {}", backticked(known))
    };
    format!("unknown agent `{agent}` ({known}){}", unknown_configured_suffix(unknown_configured))
}

fn not_installed_message(
    agent: &str,
    reason: &NotInstalledReason,
    unknown_configured: &[String],
) -> String {
    format!("`{agent}` is not installed: {reason}{}", unknown_configured_suffix(unknown_configured))
}

fn no_profile_message(agent: &str, config_file: &Path, in_repository: bool) -> String {
    let link =
        if in_repository { "link this repository (agent-profile link <profile>), " } else { "" };
    format!(
        "no profile selected for `{agent}`; name one (agent-profile {agent} <profile>), {link}or set \
         default_profile in {}",
        config_file.display()
    )
}

fn unsupported_message(agent: &str, path: &Path, config_file: &Path) -> String {
    format!(
        "`{agent}` resolves to {}, which agent-profile cannot launch without a shell. Install the agent's native \
         executable (for example the vendor's standalone installer) or set [agents.{agent}] executable = \
         \"<absolute path to a native .exe>\" in {}",
        path.display(),
        config_file.display()
    )
}

fn config_invalid_message(path: &Path, key: Option<&str>, detail: &str) -> String {
    let key = key.map(|key| format!(" (key `{key}`)")).unwrap_or_default();
    format!(
        "invalid configuration {}{key}: {detail}. agent-profile never rewrites an invalid \
         configuration; fix or move the file.",
        path.display()
    )
}

impl fmt::Display for NotInstalledReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotInstalledReason::NotOnPath { ignored_relative: 0 } => write!(f, "not found in PATH"),
            NotInstalledReason::NotOnPath { ignored_relative } => {
                write!(
                    f,
                    "not found in PATH ({ignored_relative} relative PATH entries were ignored)"
                )
            }
            NotInstalledReason::ExplicitMissing(path) => write!(
                f,
                "the configured executable {} does not exist or is not a file",
                path.display()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn io() -> io::Error {
        io::Error::other("x")
    }

    #[test]
    fn exit_codes_follow_spec_33() {
        let cases: Vec<(Error, i32)> = vec![
            (Error::Usage { message: "m".into() }, 2),
            (Error::NotYetImplemented { command: "doctor".into() }, 2),
            (
                Error::UnknownAgent {
                    agent: "a".into(),
                    known: vec![],
                    unknown_configured: vec![],
                },
                2,
            ),
            (
                Error::ArgumentConflict {
                    agent: "aider".into(),
                    option: "--conf".into(),
                    mechanism: "argument --config <file>",
                },
                2,
            ),
            (
                Error::AgentNotInstalled {
                    agent: "a".into(),
                    reason: NotInstalledReason::NotOnPath { ignored_relative: 0 },
                    unknown_configured: vec![],
                },
                3,
            ),
            (Error::InvalidProfileName { name: "".into(), reason: InvalidReason::Empty }, 4),
            (
                Error::NoProfile {
                    agent: "a".into(),
                    config_file: "c".into(),
                    in_repository: true,
                },
                4,
            ),
            (Error::Repository { path: "r".into(), reason: "not a directory".into() }, 4),
            (Error::AppRoot { message: "m".into() }, 4),
            (Error::ConfigInvalid { path: "c".into(), key: None, detail: "d".into() }, 4),
            (Error::ConfigWrite { path: "c".into(), source: io() }, 4),
            (Error::ProfileDir { path: "p".into(), source: io() }, 4),
            (Error::ProfileCaseConflict { requested: "WORK".into(), existing: "work".into() }, 4),
            (
                Error::UnsupportedExecutable {
                    agent: "a".into(),
                    path: "a.cmd".into(),
                    config_file: "c".into(),
                },
                6,
            ),
            (Error::Launch { executable: "a".into(), source: io() }, 6),
            (Error::Io { context: "c".into(), source: io() }, 1),
        ];
        for (error, code) in cases {
            assert_eq!(error.exit_code(), code, "{error:?}");
        }
    }

    #[test]
    fn unknown_agent_message_lists_known_and_unknown_configured() {
        let none =
            Error::UnknownAgent { agent: "zzz".into(), known: vec![], unknown_configured: vec![] };
        assert_eq!(none.to_string(), "unknown agent `zzz` (no agents are available in this build)");
        let some = Error::UnknownAgent {
            agent: "fakr".into(),
            known: vec!["fake".into()],
            unknown_configured: vec!["fakr".into()],
        };
        assert_eq!(
            some.to_string(),
            "unknown agent `fakr` (known agents: `fake`); config.toml also configures unknown agents: `fakr`"
        );
    }

    #[test]
    fn config_invalid_message_names_file_key_and_guidance() {
        let error = Error::ConfigInvalid {
            path: "/r/config.toml".into(),
            key: Some("agents.fake.path".into()),
            detail: "unknown field".into(),
        };
        let message = error.to_string();
        assert!(message.contains("/r/config.toml"), "{message}");
        assert!(message.contains("`agents.fake.path`"), "{message}");
        assert!(message.contains("never rewrites an invalid configuration"), "{message}");
    }

    #[test]
    fn not_installed_messages_share_one_text_and_count_ignored_entries() {
        let error = |reason| Error::AgentNotInstalled {
            agent: "codex".into(),
            reason,
            unknown_configured: vec![],
        };
        assert_eq!(
            error(NotInstalledReason::NotOnPath { ignored_relative: 0 }).to_string(),
            "`codex` is not installed: not found in PATH"
        );
        assert_eq!(
            error(NotInstalledReason::NotOnPath { ignored_relative: 2 }).to_string(),
            "`codex` is not installed: not found in PATH (2 relative PATH entries were ignored)"
        );
        let missing = NotInstalledReason::ExplicitMissing("/x/codex".into());
        assert_eq!(
            missing.to_string(),
            format!(
                "the configured executable {} does not exist or is not a file",
                Path::new("/x/codex").display()
            )
        );
    }

    #[test]
    fn conflict_and_unsupported_messages() {
        let conflict = Error::ArgumentConflict {
            agent: "aider".into(),
            option: "--conf".into(),
            mechanism: "argument --config <file>",
        };
        assert_eq!(
            conflict.to_string(),
            "`--conf` conflicts with how agent-profile selects the aider profile (argument --config <file>); \
             remove it from the agent arguments"
        );
        let unsupported = Error::UnsupportedExecutable {
            agent: "codex".into(),
            path: "codex.cmd".into(),
            config_file: "config.toml".into(),
        };
        assert_eq!(
            unsupported.to_string(),
            "`codex` resolves to codex.cmd, which agent-profile cannot launch without a shell. Install the \
             agent's native executable (for example the vendor's standalone installer) or set \
             [agents.codex] executable = \"<absolute path to a native .exe>\" in config.toml"
        );
    }

    #[test]
    fn no_profile_message_names_every_way_to_select_a_profile() {
        let error = |in_repository| Error::NoProfile {
            agent: "claude".into(),
            config_file: "config.toml".into(),
            in_repository,
        };
        assert_eq!(
            error(true).to_string(),
            "no profile selected for `claude`; name one (agent-profile claude <profile>), link this \
             repository (agent-profile link <profile>), or set default_profile in config.toml"
        );
        assert_eq!(
            error(false).to_string(),
            "no profile selected for `claude`; name one (agent-profile claude <profile>), or set \
             default_profile in config.toml"
        );
    }

    #[test]
    fn repository_message_names_the_path_and_the_reason() {
        let error =
            Error::Repository { path: "repo/.git".into(), reason: "not a directory".into() };
        assert_eq!(
            error.to_string(),
            format!("repository {}: not a directory", Path::new("repo/.git").display())
        );
    }

    #[test]
    fn profile_path_message_fits_files_and_directories() {
        let error = Error::ProfileDir {
            path: ".aider.conf.yml".into(),
            source: io::Error::other("denied"),
        };
        assert_eq!(error.to_string(), "profile path .aider.conf.yml: denied");
    }
}
```

- [ ] **Step 2: Edit `crates/agent-profile/src/cli.rs`** (the launch fills the new fields; Task 6 replaces this code). Replace exactly this text, which occurs once:

```rust
        return Err(Error::NoProfile { agent: agent.to_string() });
```

with:

```rust
        return Err(Error::NoProfile {
            agent: agent.to_string(),
            config_file: root.config_path(),
            in_repository: false,
        });
```

- [ ] **Step 3: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib error::
```

Expected: every test passes, including `exit_codes_follow_spec_33`, `no_profile_message_names_every_way_to_select_a_profile`, `repository_message_names_the_path_and_the_reason`.

- [ ] **Step 4: Run the gate** (see "Gate commands"). Expected on Windows: `158 tests run: 158 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 5: Commit**

```bash
git add crates/agent-profile/src/error.rs crates/agent-profile/src/cli.rs
git commit -m "feat: repository and no-profile errors

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 6: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/error.rs` replace exactly:

```rust
if in_repository { "link this
```

with:

```rust
if !in_repository { "link this
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib error::`. Expected: FAIL, and the failing test list includes `no_profile_message_names_every_way_to_select_a_profile`. Then restore with `git checkout -- crates/agent-profile/src/error.rs` and confirm `git status --short` prints nothing.


### Task 2: Repository discovery and path identity

`repo::{Discovery, discover, canonical, strip_verbatim, target_allowed, unlink_keys}` (design §5, §6.2) with the unit tests of §8.1-§8.3, including the tests against real `git`.

**Files:**
- Modify (whole file): `crates/agent-profile/src/repo.rs` (discovery, the network allow-list, path identity, unlink keys and tests)

**Before:** `crates/agent-profile/src/repo.rs` contains `//! Repository discovery (spec §13) and canonical repository identity (spec §14).`; `crates/agent-profile/src/repo.rs` does not contain `pub fn discover`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/repo.rs`** (discovery, the network allow-list, path identity, unlink keys and tests; the whole file, byte-exact)

```rust
//! Repository discovery (spec §13) and canonical repository identity (spec §14). SP3 design §5.
//!
//! Discovery never runs Git and never reads Git configuration: it reads the `.git` entry, the existence of
//! `HEAD`, and the capped contents of a `.git` file and its `commondir` (design D1).

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use crate::error::{Error, Result};

/// The largest `.git` file or `commondir` file discovery reads.
const MAX_METADATA_FILE: u64 = 64 * 1024;

/// The outcome of discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Discovery {
    /// The canonical working-tree root.
    Repository(PathBuf),
    NotInRepository,
}

/// `fs::canonicalize`, then `strip_verbatim` (design §5.4).
pub fn canonical(path: &Path) -> io::Result<PathBuf> {
    fs::canonicalize(path).map(|path| strip_verbatim(&path))
}

/// `\\?\C:\x` becomes `C:\x` and `\\?\UNC\server\share\x` becomes `\\server\share\x`; any other path is
/// returned unchanged.
#[cfg(windows)]
pub fn strip_verbatim(path: &Path) -> PathBuf {
    use std::path::Prefix;
    let mut components = path.components();
    let Some(Component::Prefix(prefix)) = components.next() else {
        return path.to_path_buf();
    };
    let mut stripped = match prefix.kind() {
        Prefix::VerbatimDisk(letter) => PathBuf::from(format!("{}:", char::from(letter))),
        Prefix::VerbatimUNC(server, share) => {
            let mut unc = std::ffi::OsString::from(r"\\");
            unc.push(server);
            unc.push(r"\");
            unc.push(share);
            PathBuf::from(unc)
        }
        _ => return path.to_path_buf(),
    };
    stripped.push(components.as_path());
    stripped
}

/// Identity: Unix paths have no verbatim form.
#[cfg(not(windows))]
pub fn strip_verbatim(path: &Path) -> PathBuf {
    path.to_path_buf()
}

/// Whether discovery may touch `target`, a `gitdir` or `commondir` target joined to its base (design §5.3).
/// An allow-list: on Windows only a local drive, or the network share `repository_dir` itself lives on.
pub fn target_allowed(target: &Path, repository_dir: &Path) -> bool {
    if target.as_os_str().as_encoded_bytes().contains(&0) {
        return false;
    }
    prefix_allowed(target, repository_dir)
}

#[cfg(windows)]
fn prefix_allowed(target: &Path, repository_dir: &Path) -> bool {
    use std::path::Prefix;
    fn share(path: &Path) -> Option<(&OsStr, &OsStr)> {
        match path.components().next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => {
                    Some((server, share))
                }
                _ => None,
            },
            _ => None,
        }
    }
    match target.components().next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(_) | Prefix::VerbatimDisk(_) => true,
            Prefix::UNC(..) | Prefix::VerbatimUNC(..) => {
                match (share(target), share(repository_dir)) {
                    (Some((server, name)), Some((repo_server, repo_name))) => {
                        server.eq_ignore_ascii_case(repo_server)
                            && name.eq_ignore_ascii_case(repo_name)
                    }
                    _ => false,
                }
            }
            _ => false,
        },
        _ => false,
    }
}

#[cfg(not(windows))]
fn prefix_allowed(_target: &Path, _repository_dir: &Path) -> bool {
    true
}

/// Finds the repository containing `start` (design §5.1-§5.2).
pub fn discover(start: &Path) -> Result<Discovery> {
    let canonical_start = canonical(start)
        .map_err(|error| repository(start, format!("cannot resolve the directory: {error}")))?;
    match fs::metadata(&canonical_start) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => return Err(repository(start, "not a directory")),
        Err(error) => {
            return Err(repository(start, format!("cannot resolve the directory: {error}")));
        }
    }
    for dir in canonical_start.ancestors() {
        if examine(dir)? {
            return Ok(Discovery::Repository(dir.to_path_buf()));
        }
    }
    Ok(Discovery::NotInRepository)
}

/// Whether `dir` is a repository root; `false` when `dir/.git` does not exist (design §5.2).
fn examine(dir: &Path) -> Result<bool> {
    let dot_git = dir.join(".git");
    let metadata = match fs::metadata(&dot_git) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return match fs::symlink_metadata(&dot_git) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
                Ok(_) => Err(repository(&dot_git, ".git is a broken symbolic link")),
                Err(error) => Err(repository(&dot_git, format!("cannot read .git: {error}"))),
            };
        }
        Err(error) => return Err(repository(&dot_git, format!("cannot read .git: {error}"))),
    };
    if metadata.is_dir() {
        return if is_file(&dot_git.join("HEAD")) {
            Ok(true)
        } else {
            Err(repository(&dot_git, "invalid .git directory: no HEAD file"))
        };
    }
    if !metadata.is_file() {
        return Err(repository(&dot_git, ".git is neither a directory nor a file"));
    }
    let bytes = read_capped(&dot_git)
        .map_err(|error| repository(&dot_git, format!("cannot read .git: {error}")))?
        .ok_or_else(|| repository(&dot_git, "invalid .git file: too large"))?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| repository(&dot_git, "invalid .git file: not UTF-8"))?;
    let value = first_line(text)
        .strip_prefix("gitdir: ")
        .ok_or_else(|| repository(&dot_git, "invalid .git file: no gitdir line"))?;

    let target = dir.join(value);
    if !target_allowed(&target, dir) {
        return Err(repository(
            &dot_git,
            format!("gitdir points to a network or device path: {}", target.display()),
        ));
    }
    let missing = || {
        repository(
            &dot_git,
            format!("gitdir {} is missing or is not a Git directory", target.display()),
        )
    };
    let gitdir = canonical(&target).map_err(|_| missing())?;
    if !fs::metadata(&gitdir).is_ok_and(|metadata| metadata.is_dir())
        || !is_file(&gitdir.join("HEAD"))
    {
        return Err(missing());
    }
    check_commondir(dir, &gitdir)?;
    Ok(true)
}

/// A `commondir` entry, when present, must name an existing directory (design §5.2).
fn check_commondir(dir: &Path, gitdir: &Path) -> Result<()> {
    let file = gitdir.join("commondir");
    match fs::symlink_metadata(&file) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        _ => {}
    }
    let invalid = |detail: String| repository(&file, format!("invalid commondir file: {detail}"));
    match fs::metadata(&file) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return Err(invalid("not a regular file".to_owned())),
        Err(error) => return Err(invalid(format!("cannot read: {error}"))),
    }
    let bytes = read_capped(&file)
        .map_err(|error| invalid(format!("cannot read: {error}")))?
        .ok_or_else(|| invalid("too large".to_owned()))?;
    let text = std::str::from_utf8(&bytes).map_err(|_| invalid("not UTF-8".to_owned()))?;
    let value = first_line(text);
    if value.is_empty() {
        return Err(invalid("empty".to_owned()));
    }
    let target = gitdir.join(value);
    if !target_allowed(&target, dir) {
        return Err(repository(
            &file,
            format!("commondir points to a network or device path: {}", target.display()),
        ));
    }
    if !fs::metadata(&target).is_ok_and(|metadata| metadata.is_dir()) {
        return Err(repository(
            &file,
            format!("commondir {} is missing or is not a directory", target.display()),
        ));
    }
    Ok(())
}

/// The candidate mapping keys for `unlink --repo <repo>`, in order, duplicates dropped (design §6.2).
pub fn unlink_keys(cwd: &Path, repo: &OsStr) -> Vec<PathBuf> {
    let joined = cwd.join(repo);
    let mut keys = Vec::new();
    let mut push = |key: PathBuf| {
        if !keys.contains(&key) {
            keys.push(key);
        }
    };
    if let Ok(key) = canonical(&joined) {
        push(key);
    }
    if let Some(key) = resolved(&joined) {
        push(key);
    }
    push(strip_verbatim(&joined));
    keys
}

/// The canonical deepest existing ancestor of `path`, followed by the remaining components with `.` dropped
/// and `..` applied lexically.
fn resolved(path: &Path) -> Option<PathBuf> {
    let components: Vec<Component<'_>> = path.components().collect();
    for split in (1..=components.len()).rev() {
        let base: PathBuf = components[..split].iter().collect();
        let Ok(mut key) = canonical(&base) else { continue };
        for component in &components[split..] {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    key.pop();
                }
                Component::Normal(name) => key.push(name),
                Component::Prefix(_) | Component::RootDir => return None,
            }
        }
        return Some(key);
    }
    None
}

fn repository(path: &Path, reason: impl Into<String>) -> Error {
    Error::Repository { path: path.to_path_buf(), reason: reason.into() }
}

fn is_file(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

/// The file's bytes, or `None` when it is larger than `MAX_METADATA_FILE`.
fn read_capped(path: &Path) -> io::Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(MAX_METADATA_FILE + 1).read_to_end(&mut bytes)?;
    Ok((bytes.len() as u64 <= MAX_METADATA_FILE).then_some(bytes))
}

/// The first line without its line ending.
fn first_line(text: &str) -> &str {
    text.split('\n').next().unwrap_or_default().trim_end_matches('\r')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// A temp directory that no enclosing `.git` can turn into a repository (design §8.5).
    fn guarded() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for ancestor in dir.path().ancestors() {
            assert!(
                fs::symlink_metadata(ancestor.join(".git")).is_err(),
                "{} has a .git entry, so this test cannot build a layout outside a repository",
                ancestor.display()
            );
        }
        dir
    }

    fn root_of(dir: &tempfile::TempDir) -> PathBuf {
        canonical(dir.path()).unwrap()
    }

    fn git_dir(path: &Path) {
        fs::create_dir_all(path.join(".git")).unwrap();
        fs::write(path.join(".git").join("HEAD"), "ref: refs/heads/main\n").unwrap();
    }

    fn write(path: &Path, contents: impl AsRef<[u8]>) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn found(start: &Path) -> PathBuf {
        match discover(start) {
            Ok(Discovery::Repository(root)) => root,
            other => panic!("{}: {other:?}", start.display()),
        }
    }

    /// Asserts `Error::Repository` with `path` and a reason starting with `reason`.
    fn refused(start: &Path, path: &Path, reason: &str) {
        match discover(start) {
            Err(Error::Repository { path: actual, reason: actual_reason }) => {
                assert_eq!(actual, path, "{actual_reason}");
                assert!(actual_reason.starts_with(reason), "{actual_reason:?} !~ {reason:?}");
            }
            other => panic!("{}: {other:?}", start.display()),
        }
    }

    #[test]
    fn a_git_directory_with_head_is_the_root_from_the_root_and_a_deep_subdirectory() {
        let dir = guarded();
        let root = root_of(&dir);
        git_dir(&root);
        fs::create_dir_all(root.join("a").join("b").join("c")).unwrap();
        assert_eq!(found(&root), root);
        assert_eq!(found(&root.join("a").join("b").join("c")), root);
    }

    #[test]
    fn a_worktree_git_file_with_gitdir_and_commondir_is_a_root() {
        let dir = guarded();
        let root = root_of(&dir);
        let main = root.join("main");
        git_dir(&main);
        let admin = main.join(".git").join("worktrees").join("wt");
        write(&admin.join("HEAD"), "ref: refs/heads/wt\n");
        write(&admin.join("commondir"), "../..\n");
        let worktree = root.join("wt");
        write(&worktree.join(".git"), format!("gitdir: {}\n", admin.to_str().unwrap()));
        fs::create_dir_all(worktree.join("src")).unwrap();
        assert_eq!(found(&worktree.join("src")), worktree);
        assert_eq!(found(&main), main);
    }

    #[test]
    fn a_submodule_git_file_into_modules_is_a_root() {
        let dir = guarded();
        let root = root_of(&dir);
        git_dir(&root);
        write(&root.join(".git").join("modules").join("x").join("HEAD"), "0123\n");
        let submodule = root.join("x");
        write(&submodule.join(".git"), "gitdir: ../.git/modules/x\n");
        assert_eq!(found(&submodule), submodule);
    }

    #[test]
    fn a_nested_repository_is_its_own_root() {
        let dir = guarded();
        let root = root_of(&dir);
        git_dir(&root);
        let nested = root.join("vendor").join("lib");
        git_dir(&nested);
        assert_eq!(found(&nested), nested);
        assert_eq!(found(&root.join("vendor")), root);
    }

    #[test]
    fn a_start_inside_the_git_directory_reaches_the_repository() {
        let dir = guarded();
        let root = root_of(&dir);
        git_dir(&root);
        let objects = root.join(".git").join("objects");
        fs::create_dir_all(&objects).unwrap();
        assert_eq!(found(&objects), root);
    }

    #[test]
    fn no_git_anywhere_is_not_in_a_repository() {
        let dir = guarded();
        let deep = root_of(&dir).join("a").join("b");
        fs::create_dir_all(&deep).unwrap();
        assert_eq!(discover(&deep).unwrap(), Discovery::NotInRepository);
    }

    #[test]
    fn a_missing_start_and_a_file_start_are_repository_errors() {
        let dir = guarded();
        let missing = dir.path().join("missing");
        refused(&missing, &missing, "cannot resolve the directory: ");
        let file = dir.path().join("file");
        fs::write(&file, b"x").unwrap();
        refused(&file, &file, "not a directory");
    }

    #[test]
    fn broken_git_entries_are_errors_never_the_parent_root() {
        let big = "gitdir: x\n".to_owned() + &"#".repeat(MAX_METADATA_FILE as usize);
        type Layout = Box<dyn Fn(&Path, &Path)>;
        // Each layout builds a nested `inner` inside the repository `outer`; `(file, reason)` is expected.
        let cases: Vec<(&str, Layout, &str, &str)> = vec![
            (
                "empty .git directory",
                Box::new(|_, inner| fs::create_dir_all(inner.join(".git")).unwrap()),
                ".git",
                "invalid .git directory: no HEAD file",
            ),
            (
                "HEAD is a directory",
                Box::new(|_, inner| fs::create_dir_all(inner.join(".git").join("HEAD")).unwrap()),
                ".git",
                "invalid .git directory: no HEAD file",
            ),
            (
                "garbage .git file",
                Box::new(|_, inner| write(&inner.join(".git"), "garbage\n")),
                ".git",
                "invalid .git file: no gitdir line",
            ),
            (
                "gitdir that does not exist",
                Box::new(|_, inner| write(&inner.join(".git"), "gitdir: ../nowhere\n")),
                ".git",
                "gitdir ",
            ),
            (
                "gitdir without HEAD",
                Box::new(|outer, inner| {
                    fs::create_dir_all(outer.join("admin")).unwrap();
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                ".git",
                "gitdir ",
            ),
            (
                "commondir target missing",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    write(&outer.join("admin").join("commondir"), "../gone\n");
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "commondir ",
            ),
            (
                ".git file over 64 KiB",
                Box::new(move |_, inner| write(&inner.join(".git"), &big)),
                ".git",
                "invalid .git file: too large",
            ),
            (
                "commondir over 64 KiB",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    write(
                        &outer.join("admin").join("commondir"),
                        "#".repeat(MAX_METADATA_FILE as usize + 1),
                    );
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "invalid commondir file: too large",
            ),
            (
                "commondir is a directory",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    fs::create_dir_all(outer.join("admin").join("commondir")).unwrap();
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "invalid commondir file: not a regular file",
            ),
            (
                "empty commondir",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    write(&outer.join("admin").join("commondir"), "\n");
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "invalid commondir file: empty",
            ),
            (
                "non-UTF-8 .git file",
                Box::new(|_, inner| write(&inner.join(".git"), b"gitdir: \xff\n")),
                ".git",
                "invalid .git file: not UTF-8",
            ),
            (
                "non-UTF-8 commondir",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    write(&outer.join("admin").join("commondir"), b"\xff\n");
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "invalid commondir file: not UTF-8",
            ),
            (
                "gitdir with a NUL",
                Box::new(|_, inner| write(&inner.join(".git"), "gitdir: ../a\0b\n")),
                ".git",
                "gitdir points to a network or device path: ",
            ),
            (
                "commondir with a NUL",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    write(&outer.join("admin").join("commondir"), "../a\0b\n");
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "commondir points to a network or device path: ",
            ),
        ];
        for (name, layout, file, reason) in cases {
            let dir = guarded();
            let outer = root_of(&dir);
            git_dir(&outer);
            let inner = outer.join("inner");
            fs::create_dir_all(&inner).unwrap();
            layout(&outer, &inner);
            let expected = if file == ".git" {
                inner.join(".git")
            } else {
                outer.join("admin").join("commondir")
            };
            match discover(&inner) {
                Err(Error::Repository { path, reason: actual }) => {
                    assert_eq!(path, expected, "{name}: {actual}");
                    assert!(actual.starts_with(reason), "{name}: {actual:?} !~ {reason:?}");
                }
                other => panic!("{name}: {other:?}"),
            }
        }
    }

    #[test]
    fn a_git_file_with_crlf_line_endings_is_accepted() {
        let dir = guarded();
        let root = root_of(&dir);
        write(&root.join("admin").join("HEAD"), "x\r\n");
        write(&root.join("admin").join("commondir"), ".\r\n");
        let worktree = root.join("wt");
        write(&worktree.join(".git"), "gitdir: ../admin\r\nsecond line\r\n");
        assert_eq!(found(&worktree), worktree);
    }

    #[test]
    fn a_gitdir_or_commondir_error_names_the_joined_target() {
        let dir = guarded();
        let root = root_of(&dir);
        let worktree = root.join("wt");
        write(&worktree.join(".git"), "gitdir: ../nowhere\n");
        let target = worktree.join("../nowhere");
        refused(
            &worktree,
            &worktree.join(".git"),
            &format!("gitdir {} is missing or is not a Git directory", target.display()),
        );
        write(&root.join("admin").join("HEAD"), "x\n");
        write(&root.join("admin").join("commondir"), "../gone\n");
        write(&worktree.join(".git"), "gitdir: ../admin\n");
        let target = root.join("admin").join("../gone");
        refused(
            &worktree,
            &root.join("admin").join("commondir"),
            &format!("commondir {} is missing or is not a directory", target.display()),
        );
    }

    #[test]
    fn a_nul_target_is_refused_on_every_platform() {
        assert!(!target_allowed(Path::new("a\0b"), Path::new("repo")));
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_start_resolves_to_the_real_root() {
        let dir = guarded();
        let root = root_of(&dir);
        let real = root.join("real");
        git_dir(&real);
        std::os::unix::fs::symlink(&real, root.join("link")).unwrap();
        assert_eq!(found(&root.join("link")), real);
    }

    #[cfg(unix)]
    #[test]
    fn a_dangling_git_symlink_is_an_error() {
        let dir = guarded();
        let root = root_of(&dir);
        std::os::unix::fs::symlink(root.join("nowhere"), root.join(".git")).unwrap();
        refused(&root, &root.join(".git"), ".git is a broken symbolic link");
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_commondir_is_an_error() {
        use std::os::unix::fs::PermissionsExt;
        let dir = guarded();
        let root = root_of(&dir);
        let commondir = root.join("admin").join("commondir");
        write(&root.join("admin").join("HEAD"), "x\n");
        write(&commondir, "..\n");
        write(&root.join("wt").join(".git"), "gitdir: ../admin\n");
        fs::set_permissions(&commondir, fs::Permissions::from_mode(0o000)).unwrap();
        if fs::read(&commondir).is_ok() {
            eprintln!("skipped: running as a user that ignores file permissions");
            return;
        }
        refused(&root.join("wt"), &commondir, "invalid commondir file: cannot read: ");
    }

    #[cfg(unix)]
    #[test]
    fn a_fifo_commondir_is_refused_without_blocking() {
        let dir = guarded();
        let root = root_of(&dir);
        let commondir = root.join("admin").join("commondir");
        write(&root.join("admin").join("HEAD"), "x\n");
        let status = Command::new("mkfifo").arg(&commondir).status().unwrap();
        assert!(status.success());
        write(&root.join("wt").join(".git"), "gitdir: ../admin\n");
        let start = root.join("wt");
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || sender.send(discover(&start)).unwrap());
        let result = receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("discovery blocked on a FIFO");
        match result {
            Err(Error::Repository { path, reason }) => {
                assert_eq!(path, commondir);
                assert_eq!(reason, "invalid commondir file: not a regular file");
            }
            other => panic!("{other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_git_entry_that_is_neither_a_directory_nor_a_file_is_an_error() {
        let dir = guarded();
        let root = root_of(&dir);
        let status = Command::new("mkfifo").arg(root.join(".git")).status().unwrap();
        assert!(status.success());
        fs::create_dir_all(root.join("sub")).unwrap();
        refused(&root.join("sub"), &root.join(".git"), ".git is neither a directory nor a file");
    }

    #[cfg(windows)]
    #[test]
    fn a_junction_start_resolves_to_the_real_root() {
        let dir = guarded();
        let root = root_of(&dir);
        let real = root.join("real");
        git_dir(&real);
        let junction = root.join("junction");
        let status = Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(&junction)
            .arg(&real)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(found(&junction.join(".")), real);
    }

    #[cfg(windows)]
    #[test]
    fn strip_verbatim_handles_drive_unc_and_other_verbatim_forms() {
        for (input, expected) in [
            (r"\\?\C:\x\y", r"C:\x\y"),
            (r"\\?\C:\", r"C:\"),
            (r"\\?\UNC\server\share\x", r"\\server\share\x"),
            (r"\\?\Volume{0f0e}\x", r"\\?\Volume{0f0e}\x"),
            (r"\\?\GLOBALROOT\Device\x", r"\\?\GLOBALROOT\Device\x"),
            (r"C:\plain", r"C:\plain"),
            (r"\\server\share\x", r"\\server\share\x"),
        ] {
            assert_eq!(strip_verbatim(Path::new(input)), Path::new(expected), "{input}");
            assert_eq!(
                strip_verbatim(Path::new(input)).as_os_str(),
                OsStr::new(expected),
                "{input}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn target_allowed_accepts_local_drives_and_the_repository_share_only() {
        let local = Path::new(r"C:\repo");
        let shared = Path::new(r"\\server\share\repo");
        for (target, repository_dir, allowed) in [
            (r"\\server\share\x", local, false),
            (r"//server/share/x", local, false),
            (r"\/server/share/x", local, false),
            (r"//./pipe/x", local, false),
            (r"\\.\pipe\x", local, false),
            (r"\\?\UNC\s\x", local, false),
            (r"\\?\Volume{0f0e}\x", local, false),
            (r"\\?\GLOBALROOT\x", local, false),
            (r"\\evil\\share\x", local, false),
            (r"C:\??\UNC\s\x", local, true),
            (r"C:\x", local, true),
            (r"\\?\C:\x", local, true),
            (r"\\SERVER\Share\other", shared, true),
            (r"\\?\UNC\server\share\x", shared, true),
            (r"\\server\elsewhere\x", shared, false),
            (r"C:\a\0b", local, true),
            ("C:\\a\0b", local, false),
        ] {
            assert_eq!(
                target_allowed(Path::new(target), repository_dir),
                allowed,
                "{target:?} from {}",
                repository_dir.display()
            );
        }
        // Rust joins a rooted value onto the drive of the base, so the value stays local.
        #[allow(clippy::join_absolute_paths)]
        let joined = local.join(r"\??\UNC\s\x");
        assert_eq!(joined, Path::new(r"C:\??\UNC\s\x"));
    }

    #[cfg(windows)]
    #[test]
    fn network_and_device_gitdir_targets_are_refused_before_any_filesystem_call() {
        for value in
            [r"\\server\share\x", r"\\.\pipe\x", r"\\?\Volume{0f0e}\x", r"//server/share/x"]
        {
            let dir = guarded();
            let root = root_of(&dir);
            write(&root.join(".git"), format!("gitdir: {value}\n"));
            refused(
                &root,
                &root.join(".git"),
                &format!(
                    "gitdir points to a network or device path: {}",
                    root.join(value).display()
                ),
            );
            write(&root.join("admin").join("HEAD"), "x\n");
            write(&root.join("admin").join("commondir"), format!("{value}\n"));
            write(&root.join(".git"), "gitdir: admin\n");
            refused(
                &root,
                &root.join("admin").join("commondir"),
                "commondir points to a network or device path: ",
            );
        }
    }

    /// `git` isolated from the developer's and the machine's configuration (design §8.2).
    fn git(cwd: &Path, args: &[&str]) -> String {
        let empty = cwd.parent().unwrap().join("empty-gitconfig");
        fs::write(&empty, b"").unwrap();
        let mut command = Command::new("git");
        for (name, _) in std::env::vars_os() {
            if name.to_string_lossy().starts_with("GIT_") {
                command.env_remove(name);
            }
        }
        let output = command
            .current_dir(cwd)
            .env("GIT_CONFIG_GLOBAL", &empty)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"])
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    fn git_toplevel(cwd: &Path) -> PathBuf {
        canonical(Path::new(git(cwd, &["rev-parse", "--show-toplevel"]).trim_end())).unwrap()
    }

    fn git_repository(path: &Path) {
        fs::create_dir_all(path).unwrap();
        git(path, &["init", "-q"]);
        git(path, &["commit", "-q", "--allow-empty", "-m", "init"]);
    }

    #[test]
    fn discovery_matches_git_for_init_worktree_and_submodule() {
        let dir = guarded();
        let root = root_of(&dir);
        let main = root.join("main");
        git_repository(&main);
        let library = root.join("library");
        git_repository(&library);
        git(&main, &["worktree", "add", "-q", "../wt"]);
        git(
            &main,
            &[
                "-c",
                "protocol.file.allow=always",
                "submodule",
                "add",
                "-q",
                library.to_str().unwrap(),
                "sub",
            ],
        );
        git(&main, &["commit", "-q", "-m", "submodule"]);
        let deep = main.join("sub").join("nested");
        fs::create_dir_all(&deep).unwrap();
        for start in [main.clone(), root.join("wt"), main.join("sub"), deep] {
            assert_eq!(found(&start), git_toplevel(&start), "{}", start.display());
        }
    }

    #[test]
    fn broken_nested_git_entries_inside_a_real_repository_are_errors() {
        for (name, layout) in [
            ("gitdir to nowhere", "gitdir: ../nowhere\n"),
            ("garbage", "garbage\n"),
            ("empty directory", ""),
        ] {
            let dir = guarded();
            let main = root_of(&dir).join("main");
            git_repository(&main);
            let nested = main.join("nested");
            fs::create_dir_all(&nested).unwrap();
            if layout.is_empty() {
                fs::create_dir(nested.join(".git")).unwrap();
            } else {
                fs::write(nested.join(".git"), layout).unwrap();
            }
            assert!(matches!(discover(&nested), Err(Error::Repository { .. })), "{name}");
        }
    }

    #[test]
    fn unlink_keys_prefer_the_canonical_path_then_the_resolved_then_the_literal() {
        let dir = guarded();
        let root = root_of(&dir);
        let repo = root.join("repo");
        fs::create_dir_all(&repo).unwrap();
        assert_eq!(unlink_keys(&root, OsStr::new("repo")), [repo]);

        let cwd = root.join("cwd");
        fs::create_dir_all(&cwd).unwrap();
        assert_eq!(
            unlink_keys(&cwd, OsStr::new("../gone")),
            [root.join("gone"), cwd.join("../gone")]
        );
        assert_eq!(
            unlink_keys(&cwd, OsStr::new("./missing/../gone")),
            [cwd.join("gone"), cwd.join("./missing/../gone")]
        );
    }

    #[test]
    fn unlink_keys_resolve_a_deleted_directory_under_a_non_canonical_ancestor() {
        let dir = guarded();
        let spelled = dir.path().join("sub").join("..").join("gone");
        let keys = unlink_keys(dir.path(), OsStr::new("sub/../gone"));
        assert_eq!(keys[0], root_of(&dir).join("gone"), "{keys:?}");
        assert_eq!(keys.last().unwrap(), &strip_verbatim(&spelled), "{keys:?}");
    }
}
```

- [ ] **Step 2: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib repo::
```

Expected: every test passes, including `broken_git_entries_are_errors_never_the_parent_root`, `discovery_matches_git_for_init_worktree_and_submodule`, `unlink_keys_prefer_the_canonical_path_then_the_resolved_then_the_literal`.

- [ ] **Step 3: Run the gate** (see "Gate commands"). Expected on Windows: `177 tests run: 177 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 4: Commit**

```bash
git add crates/agent-profile/src/repo.rs
git commit -m "feat: repository discovery and canonical identity

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 5: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/repo.rs` replace exactly:

```rust
            Err(repository(&dot_git, "invalid .git directory: no HEAD file"))
```

with:

```rust
            Ok(false)
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib repo::`. Expected: FAIL, and the failing test list includes `broken_git_entries_are_errors_never_the_parent_root`. Then restore with `git checkout -- crates/agent-profile/src/repo.rs` and confirm `git status --short` prints nothing.


### Task 3: Configuration mappings, link and unlink

`default_profile` and `[repositories.<root>]` in the strict schema, the mapping queries, `link`, `unlink` and `check_case_twins` moved into `config` (design §4.2, §6).

**Files:**
- Modify (whole file): `crates/agent-profile/src/config.rs` (schema, queries, link/unlink, the moved case-twin check and tests)
- Modify: `crates/agent-profile/src/adapter/mod.rs` (the adapters use the moved check; remove the old copy)

**Before:** `crates/agent-profile/src/config.rs` does not contain `default_profile`; `crates/agent-profile/src/adapter/mod.rs` contains `fn check_case_twins(root: &AppRoot, profile: &ProfileName) -> Result<()> {`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/config.rs`** (schema, queries, link/unlink, the moved case-twin check and tests; the whole file, byte-exact)

```rust
//! The application root (spec §7), `config.toml` strict reads (spec §17) and locked, atomic writes
//! (spec §18, §18.1).

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use tempfile::NamedTempFile;

use crate::error::{Error, Result};
use crate::name::{AgentId, Platform, ProfileName};

/// Environment variable that overrides the application root.
pub const HOME_ENV: &str = "AGENT_PROFILE_HOME";

const CONFIG_FILE: &str = "config.toml";
const LOCK_FILE: &str = "config.toml.lock";
const TEMP_PREFIX: &str = ".config.toml.";
const TEMP_SUFFIX: &str = ".tmp";
const LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const LOCK_RETRY: Duration = Duration::from_millis(50);

/// The directory holding `config.toml` and `profiles/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRoot(PathBuf);

impl AppRoot {
    /// A root at an explicit path. Tests use this instead of changing the process environment.
    pub fn from_path(path: PathBuf) -> AppRoot {
        AppRoot(path)
    }

    /// `AGENT_PROFILE_HOME` when set, otherwise `<home>/.agent-profile`. Never creates anything.
    pub fn resolve() -> Result<AppRoot> {
        AppRoot::resolve_from(std::env::var_os(HOME_ENV), std::env::home_dir())
    }

    fn resolve_from(home_override: Option<OsString>, home_dir: Option<PathBuf>) -> Result<AppRoot> {
        match home_override {
            Some(value) => {
                let path = PathBuf::from(value);
                if path.as_os_str().is_empty() || !path.is_absolute() {
                    return Err(Error::AppRoot {
                        message: format!(
                            "{HOME_ENV} must be an absolute path, got {:?}",
                            path.as_os_str()
                        ),
                    });
                }
                Ok(AppRoot(path))
            }
            None => match home_dir {
                Some(home) if home.is_absolute() => Ok(AppRoot(home.join(".agent-profile"))),
                _ => Err(Error::AppRoot {
                    message: format!(
                        "cannot determine an absolute home directory; set {HOME_ENV} to an absolute path"
                    ),
                }),
            },
        }
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn config_path(&self) -> PathBuf {
        self.0.join(CONFIG_FILE)
    }

    pub fn profiles_dir(&self) -> PathBuf {
        self.0.join("profiles")
    }

    fn lock_path(&self) -> PathBuf {
        self.0.join(LOCK_FILE)
    }
}

/// The validated contents of `config.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    agents: BTreeMap<String, Option<PathBuf>>,
    default_profile: Option<ProfileName>,
    /// Keyed by the stored key string, so iteration is ordered by it.
    repositories: BTreeMap<String, Mapping>,
}

/// One `[repositories.'<root>']` entry (SP3 design §6.1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mapping {
    pub profile: Option<ProfileName>,
    pub agents: BTreeMap<AgentId, ProfileName>,
}

/// A mapping field that names a profile (SP3 design §4.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// The stored key.
    pub root: PathBuf,
    /// `None` for the entry's `profile`, the agent for an `agents.<id>` field.
    pub agent: Option<AgentId>,
    /// The stored spelling.
    pub profile: ProfileName,
}

/// What `link` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkOutcome {
    Linked,
    Changed { old: ProfileName },
    AlreadyLinked,
}

/// What `unlink` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnlinkOutcome {
    /// `root` is the stored key of the entry the mapping was removed from.
    Unlinked { root: PathBuf, old: ProfileName },
    /// `shown` is the first candidate key.
    NothingToRemove { shown: PathBuf },
}

impl Config {
    /// Reads `<root>/config.toml`. A missing file is an empty configuration; anything invalid is an error.
    pub fn load(root: &AppRoot) -> Result<Config> {
        let path = root.config_path();
        match fs::read(&path) {
            Ok(bytes) => parse(&path, &bytes),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
            Err(error) => Err(invalid(&path, None, format!("cannot read the file: {error}"))),
        }
    }

    /// The explicit executable override for an agent, if configured.
    pub fn agent_executable(&self, id: &str) -> Option<&Path> {
        self.agents.get(id).and_then(|executable| executable.as_deref())
    }

    /// Every agent id that has an `agents.<id>` table, in sorted order.
    pub fn configured_agents(&self) -> impl Iterator<Item = &str> {
        self.agents.keys().map(String::as_str)
    }

    /// The hand-edited global default (SP3 design D3).
    pub fn default_profile(&self) -> Option<&ProfileName> {
        self.default_profile.as_ref()
    }

    /// The entry whose key is component-equal to `root`; validation guarantees there is at most one.
    pub fn mapping(&self, root: &Path) -> Option<&Mapping> {
        self.mappings().find(|(key, _)| *key == root).map(|(_, mapping)| mapping)
    }

    /// Every entry, ordered by the stored key string.
    pub fn mappings(&self) -> impl Iterator<Item = (&Path, &Mapping)> {
        self.repositories.iter().map(|(key, mapping)| (Path::new(key.as_str()), mapping))
    }

    /// Every mapping field naming `profile`, ignoring ASCII case, so a case-only twin is never missed.
    pub fn mappings_referencing(&self, profile: &ProfileName) -> Vec<Reference> {
        let matches = |name: &ProfileName| name.as_str().eq_ignore_ascii_case(profile.as_str());
        let mut references = Vec::new();
        for (root, mapping) in self.mappings() {
            if let Some(name) = mapping.profile.as_ref().filter(|name| matches(name)) {
                references.push(Reference {
                    root: root.to_path_buf(),
                    agent: None,
                    profile: name.clone(),
                });
            }
            for (agent, name) in mapping.agents.iter().filter(|(_, name)| matches(name)) {
                references.push(Reference {
                    root: root.to_path_buf(),
                    agent: Some(agent.clone()),
                    profile: name.clone(),
                });
            }
        }
        references
    }
}

fn invalid(path: &Path, key: Option<String>, detail: String) -> Error {
    Error::ConfigInvalid { path: path.to_path_buf(), key, detail }
}

fn parse(path: &Path, bytes: &[u8]) -> Result<Config> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| invalid(path, None, "the file is not valid UTF-8".to_owned()))?;
    let table: toml::Table = text.parse().map_err(|error: toml::de::Error| {
        let line = error.span().map(|span| text[..span.start].matches('\n').count() + 1);
        let detail = match line {
            Some(line) => format!("TOML syntax error at line {line}: {}", error.message()),
            None => format!("TOML syntax error: {}", error.message()),
        };
        invalid(path, None, detail)
    })?;
    validate(path, &table)
}

/// Applies the strict schema (SP1 design §6.2, SP3 design §6.1) to a parsed document.
fn validate(path: &Path, table: &toml::Table) -> Result<Config> {
    let mut config = Config::default();
    for (key, value) in table {
        match key.as_str() {
            "agents" => validate_agents(path, key, value, &mut config)?,
            "default_profile" => {
                let name = value.as_str().ok_or_else(|| {
                    invalid(path, Some(key.clone()), "must be a string".to_owned())
                })?;
                config.default_profile = Some(profile_name(path, key, name)?);
            }
            "repositories" => validate_repositories(path, key, value, &mut config)?,
            _ => return Err(invalid(path, Some(key.clone()), "unknown key".to_owned())),
        }
    }
    Ok(config)
}

fn profile_name(path: &Path, key: &str, name: &str) -> Result<ProfileName> {
    ProfileName::parse(name, Platform::host()).map_err(|reason| {
        invalid(path, Some(key.to_owned()), format!("invalid profile name {name:?}: {reason}"))
    })
}

/// `repositories.<key>` with the key quoted as TOML would write it.
fn repository_key(root: &str) -> String {
    format!("repositories.{}", toml_edit::Key::new(root).display_repr())
}

/// Absolute in Unix form (`/…`) or Windows form (`C:\…`, `C:/…`, `\\…`), whatever the host (SP3 design §6.1).
fn is_absolute_key(root: &str) -> bool {
    let bytes = root.as_bytes();
    let drive = bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/');
    root.starts_with('/') || root.starts_with(r"\\") || drive
}

fn validate_repositories(
    path: &Path,
    key: &str,
    value: &toml::Value,
    config: &mut Config,
) -> Result<()> {
    let repositories = value
        .as_table()
        .ok_or_else(|| invalid(path, Some(key.to_owned()), "must be a table".to_owned()))?;
    for (root, entry) in repositories {
        let entry_key = repository_key(root);
        if !is_absolute_key(root) {
            return Err(invalid(path, Some(entry_key), "must be an absolute path".to_owned()));
        }
        let entry = entry
            .as_table()
            .ok_or_else(|| invalid(path, Some(entry_key.clone()), "must be a table".to_owned()))?;
        let mut mapping = Mapping::default();
        for (field, value) in entry {
            let field_key = format!("{entry_key}.{field}");
            match field.as_str() {
                "profile" => {
                    let name = value.as_str().ok_or_else(|| {
                        invalid(path, Some(field_key.clone()), "must be a string".to_owned())
                    })?;
                    mapping.profile = Some(profile_name(path, &field_key, name)?);
                }
                "agents" => {
                    let agents = value.as_table().ok_or_else(|| {
                        invalid(path, Some(field_key.clone()), "must be a table".to_owned())
                    })?;
                    for (id, name) in agents {
                        let agent_key = format!("{field_key}.{id}");
                        let Some(agent) = AgentId::parse(id) else {
                            return Err(invalid(
                                path,
                                Some(agent_key),
                                "agent ids must match [a-z][a-z0-9-]*".to_owned(),
                            ));
                        };
                        let name = name.as_str().ok_or_else(|| {
                            invalid(path, Some(agent_key.clone()), "must be a string".to_owned())
                        })?;
                        mapping.agents.insert(agent, profile_name(path, &agent_key, name)?);
                    }
                }
                _ => return Err(invalid(path, Some(field_key), "unknown key".to_owned())),
            }
        }
        config.repositories.insert(root.clone(), mapping);
    }
    let roots: Vec<&String> = config.repositories.keys().collect();
    for (index, first) in roots.iter().enumerate() {
        if let Some(second) =
            roots[index + 1..].iter().find(|second| Path::new(second) == Path::new(first))
        {
            return Err(invalid(
                path,
                Some(repository_key(first)),
                format!("names the same directory as {}", repository_key(second)),
            ));
        }
    }
    Ok(())
}

fn validate_agents(path: &Path, key: &str, value: &toml::Value, config: &mut Config) -> Result<()> {
    let agents = value
        .as_table()
        .ok_or_else(|| invalid(path, Some(key.to_owned()), "must be a table".to_owned()))?;
    for (id, agent) in agents {
        let agent_key = format!("agents.{id}");
        if AgentId::parse(id).is_none() {
            return Err(invalid(
                path,
                Some(agent_key),
                "agent ids must match [a-z][a-z0-9-]*".to_owned(),
            ));
        }
        let agent = agent
            .as_table()
            .ok_or_else(|| invalid(path, Some(agent_key.clone()), "must be a table".to_owned()))?;
        let mut executable = None;
        for (field, value) in agent {
            let field_key = format!("{agent_key}.{field}");
            if field != "executable" {
                return Err(invalid(path, Some(field_key), "unknown key".to_owned()));
            }
            let text = value.as_str().ok_or_else(|| {
                invalid(path, Some(field_key.clone()), "must be a string".to_owned())
            })?;
            let candidate = PathBuf::from(text);
            if !candidate.is_absolute() {
                return Err(invalid(path, Some(field_key), "must be an absolute path".to_owned()));
            }
            executable = Some(candidate);
        }
        config.agents.insert(id.clone(), executable);
    }
    Ok(())
}

/// Locked, atomic read-modify-write of `config.toml` (spec §18, design §6.4).
pub fn update(
    root: &AppRoot,
    edit: impl FnOnce(&mut toml_edit::DocumentMut) -> Result<()>,
) -> Result<()> {
    update_with(
        root,
        edit,
        || Ok(()),
        |temp, destination| temp.persist(destination).map(drop).map_err(|error| error.error),
    )
}

pub(crate) fn update_with(
    root: &AppRoot,
    edit: impl FnOnce(&mut toml_edit::DocumentMut) -> Result<()>,
    before_persist: impl FnOnce() -> Result<()>,
    replace: impl FnOnce(NamedTempFile, &Path) -> io::Result<()>,
) -> Result<()> {
    let config_path = root.config_path();
    let write_error =
        |path: &Path, source: io::Error| Error::ConfigWrite { path: path.to_path_buf(), source };

    // 1. The root.
    fs::create_dir_all(root.path()).map_err(|source| write_error(root.path(), source))?;

    // 2. The exclusive lock, bounded.
    let lock_path = root.lock_path();
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|source| write_error(&lock_path, source))?;
    acquire(&lock, &lock_path)?;

    // 3. Best-effort sweep of temp files left by crashed writers.
    if let Ok(entries) = fs::read_dir(root.path()) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(TEMP_PREFIX) && name.ends_with(TEMP_SUFFIX) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    // 4. The latest configuration, validated.
    let text = match fs::read(&config_path) {
        Ok(bytes) => {
            parse(&config_path, &bytes)?;
            String::from_utf8(bytes).expect("parse checked UTF-8")
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(invalid(&config_path, None, format!("cannot read the file: {error}")));
        }
    };
    let mut document: toml_edit::DocumentMut = text
        .parse()
        .map_err(|error| invalid(&config_path, None, format!("TOML syntax error: {error}")))?;

    // 5. The edit, re-validated. An edit that changes nothing writes nothing. The rendering before the edit is
    // the comparison, not the file text: rendering normalizes line endings and drops a byte-order mark.
    let unedited = document.to_string();
    edit(&mut document)?;
    let updated = document.to_string();
    if updated == unedited {
        return Ok(());
    }
    parse(&config_path, updated.as_bytes())?;

    // 6. The temp file in the same directory, synced.
    let mut temp = tempfile::Builder::new()
        .prefix(TEMP_PREFIX)
        .suffix(TEMP_SUFFIX)
        .tempfile_in(root.path())
        .map_err(|source| write_error(&config_path, source))?;
    temp.write_all(updated.as_bytes()).map_err(|source| write_error(&config_path, source))?;
    temp.as_file().sync_all().map_err(|source| write_error(&config_path, source))?;
    before_persist()?;

    // 7. The atomic replace. On failure `replace` drops the temp file, which deletes it.
    replace(temp, &config_path).map_err(|source| write_error(&config_path, source))?;

    // 7a. Durability of the rename on Unix.
    #[cfg(unix)]
    File::open(root.path()).and_then(|directory| directory.sync_all()).map_err(|error| {
        write_error(
            &config_path,
            io::Error::new(
                error.kind(),
                format!(
                    "configuration replaced, but the directory could not be synced; the change may \
                     not survive a power loss: {error}"
                ),
            ),
        )
    })?;

    // 8. The lock is released when `lock` drops.
    drop(lock);
    Ok(())
}

/// Maps `repository` (or only `agent` in it) to `profile` (SP3 design §6.2). The caller has discovered
/// `repository`.
pub fn link(
    root: &AppRoot,
    repository: &Path,
    agent: Option<&AgentId>,
    profile: &ProfileName,
) -> Result<LinkOutcome> {
    let Some(key) = repository.to_str() else {
        return Err(Error::Repository {
            path: repository.to_path_buf(),
            reason: "a repository path that is not valid UTF-8 cannot be linked".to_owned(),
        });
    };
    check_case_twins(root, profile)?;
    let mut outcome = LinkOutcome::Linked;
    update(root, |document| {
        let stored = stored_keys(document)
            .into_iter()
            .find(|stored| Path::new(stored) == repository)
            .unwrap_or_else(|| key.to_owned());
        let repositories = child_table(document.as_table_mut(), "repositories", false);
        let entry = child_table(repositories, &stored, false);
        outcome = link_outcome(mapped(entry, agent), profile);
        if outcome == LinkOutcome::AlreadyLinked {
            return Ok(());
        }
        let value = toml_edit::value(profile.as_str());
        match agent {
            None => entry.insert("profile", value),
            Some(agent) => child_table(entry, "agents", true).insert(agent.as_str(), value),
        };
        Ok(())
    })?;
    Ok(outcome)
}

fn link_outcome(current: Option<String>, profile: &ProfileName) -> LinkOutcome {
    match current {
        None => LinkOutcome::Linked,
        Some(current) if current == profile.as_str() => LinkOutcome::AlreadyLinked,
        Some(current) => LinkOutcome::Changed {
            old: ProfileName::parse(&current, Platform::host()).expect("validated profile name"),
        },
    }
}

/// Removes the mapping at the first of `keys` that has one at that field, under one lock (SP3 design §6.2).
/// `keys` is never empty.
pub fn unlink(root: &AppRoot, keys: &[PathBuf], agent: Option<&AgentId>) -> Result<UnlinkOutcome> {
    let mut outcome = UnlinkOutcome::NothingToRemove { shown: keys[0].clone() };
    update(root, |document| {
        let Some(repositories) =
            document.get_mut("repositories").and_then(toml_edit::Item::as_table_like_mut)
        else {
            return Ok(());
        };
        let found = keys.iter().find_map(|key| {
            repositories.iter().find_map(|(stored, entry)| {
                let entry = entry.as_table_like()?;
                (Path::new(stored) == key)
                    .then(|| mapped(entry, agent))
                    .flatten()
                    .map(|old| (stored.to_owned(), old))
            })
        });
        let Some((stored, old)) = found else {
            return Ok(());
        };
        let entry = repositories
            .get_mut(&stored)
            .and_then(toml_edit::Item::as_table_like_mut)
            .expect("validated entry");
        match agent {
            None => {
                entry.remove("profile");
            }
            Some(agent) => {
                let agents = entry
                    .get_mut("agents")
                    .and_then(toml_edit::Item::as_table_like_mut)
                    .expect("validated agents table");
                agents.remove(agent.as_str());
                if agents.is_empty() {
                    entry.remove("agents");
                }
            }
        }
        if entry.is_empty() {
            repositories.remove(&stored);
        }
        outcome = UnlinkOutcome::Unlinked {
            root: PathBuf::from(stored),
            old: ProfileName::parse(&old, Platform::host()).expect("validated profile name"),
        };
        Ok(())
    })?;
    Ok(outcome)
}

/// The stored `repositories` keys of a validated document.
fn stored_keys(document: &toml_edit::DocumentMut) -> Vec<String> {
    document
        .get("repositories")
        .and_then(toml_edit::Item::as_table_like)
        .map(|repositories| repositories.iter().map(|(key, _)| key.to_owned()).collect())
        .unwrap_or_default()
}

/// The profile an entry maps at the field `agent` selects.
fn mapped(entry: &dyn toml_edit::TableLike, agent: Option<&AgentId>) -> Option<String> {
    let value = match agent {
        None => entry.get("profile"),
        Some(agent) => entry
            .get("agents")
            .and_then(toml_edit::Item::as_table_like)
            .and_then(|agents| agents.get(agent.as_str())),
    };
    value.and_then(toml_edit::Item::as_str).map(str::to_owned)
}

/// The table at `key` in `parent`, created when missing: an inline table when `inline`, otherwise an
/// implicit standard table.
fn child_table<'a>(
    parent: &'a mut dyn toml_edit::TableLike,
    key: &str,
    inline: bool,
) -> &'a mut dyn toml_edit::TableLike {
    let item = parent.entry(key).or_insert_with(|| {
        if inline {
            toml_edit::Item::Value(toml_edit::Value::InlineTable(toml_edit::InlineTable::new()))
        } else {
            let mut table = toml_edit::Table::new();
            table.set_implicit(true);
            toml_edit::Item::Table(table)
        }
    });
    item.as_table_like_mut().expect("validated as a table")
}

/// Refuses a profile whose name differs from an existing `profiles/` entry only in ASCII case (SP1 design §7.3).
pub(crate) fn check_case_twins(root: &AppRoot, profile: &ProfileName) -> Result<()> {
    let Ok(entries) = fs::read_dir(root.profiles_dir()) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name != profile.as_str() && name.eq_ignore_ascii_case(profile.as_str()) {
            return Err(Error::ProfileCaseConflict {
                requested: profile.to_string(),
                existing: name.to_owned(),
            });
        }
    }
    Ok(())
}

fn acquire(lock: &File, lock_path: &Path) -> Result<()> {
    let deadline = Instant::now() + LOCK_TIMEOUT;
    loop {
        match lock.try_lock() {
            Ok(()) => return Ok(()),
            Err(std::fs::TryLockError::WouldBlock) if Instant::now() < deadline => {
                thread::sleep(LOCK_RETRY);
            }
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(Error::ConfigWrite {
                    path: lock_path.to_path_buf(),
                    source: io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "another agent-profile process holds the configuration lock; retry, or check \
                         for a stuck process",
                    ),
                });
            }
            Err(std::fs::TryLockError::Error(source)) => {
                return Err(Error::ConfigWrite { path: lock_path.to_path_buf(), source });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    fn temp_root() -> (tempfile::TempDir, AppRoot) {
        let dir = tempfile::tempdir().unwrap();
        let root = AppRoot::from_path(dir.path().to_path_buf());
        (dir, root)
    }

    fn absolute(name: &str) -> String {
        std::env::temp_dir().join(name).to_str().unwrap().to_owned()
    }

    fn set_agent(doc: &mut toml_edit::DocumentMut, id: &str, executable: &str) {
        let agents = doc
            .entry("agents")
            .or_insert_with(|| {
                let mut agents = toml_edit::Table::new();
                agents.set_implicit(true);
                toml_edit::Item::Table(agents)
            })
            .as_table_mut()
            .expect("agents is a table");
        let mut agent = toml_edit::Table::new();
        agent.insert("executable", toml_edit::value(executable));
        agents.insert(id, toml_edit::Item::Table(agent));
    }

    #[test]
    fn app_root_override_must_be_absolute_and_non_empty() {
        let abs = std::env::temp_dir();
        assert_eq!(
            AppRoot::resolve_from(Some(abs.clone().into_os_string()), None).unwrap().path(),
            abs
        );
        for bad in ["", "relative/dir"] {
            let error = AppRoot::resolve_from(Some(bad.into()), Some(abs.clone())).unwrap_err();
            assert!(matches!(error, Error::AppRoot { .. }), "{bad:?}: {error:?}");
        }
    }

    #[test]
    fn non_utf8_home_override_is_an_app_root_error() {
        #[cfg(unix)]
        let value = {
            use std::os::unix::ffi::OsStringExt;
            OsString::from_vec(vec![0x66, 0xff])
        };
        #[cfg(windows)]
        let value = {
            use std::os::windows::ffi::OsStringExt;
            OsString::from_wide(&[0x66, 0xD800])
        };
        let error = AppRoot::resolve_from(Some(value), None).unwrap_err();
        assert!(matches!(error, Error::AppRoot { .. }), "{error:?}");
    }

    #[test]
    fn app_root_defaults_to_dot_agent_profile_in_home() {
        let home = std::env::temp_dir();
        let root = AppRoot::resolve_from(None, Some(home.clone())).unwrap();
        assert_eq!(root.path(), home.join(".agent-profile"));
        assert!(matches!(AppRoot::resolve_from(None, None), Err(Error::AppRoot { .. })));
        for bad in ["", "relative/home"] {
            let error = AppRoot::resolve_from(None, Some(PathBuf::from(bad))).unwrap_err();
            assert!(matches!(error, Error::AppRoot { .. }), "{bad:?}: {error:?}");
        }
    }

    #[test]
    fn schema_accepts_the_sp1_forms() {
        let exe = absolute("fake-agent");
        let text = format!("# comment\n[agents.fake]\nexecutable = {exe:?}\n\n[agents.claude]\n");
        let config = parse(Path::new("c"), text.as_bytes()).unwrap();
        assert_eq!(config.agent_executable("fake"), Some(Path::new(&exe)));
        assert_eq!(config.agent_executable("claude"), None);
        assert_eq!(config.agent_executable("codex"), None);
        assert_eq!(config.configured_agents().collect::<Vec<_>>(), ["claude", "fake"]);
        assert_eq!(parse(Path::new("c"), b"").unwrap(), Config::default());
    }

    #[test]
    fn schema_rejects_every_error_class_naming_the_key() {
        let exe = absolute("x");
        let cases = [
            (b"\xff".to_vec(), None),
            (b"[agents\n".to_vec(), None),
            (b"default = \"work\"\n".to_vec(), Some("default")),
            (b"agents = 3\n".to_vec(), Some("agents")),
            (b"[agents]\nFake = {}\n".to_vec(), Some("agents.Fake")),
            (b"[agents]\nfake = 3\n".to_vec(), Some("agents.fake")),
            (format!("[agents.fake]\npath = {exe:?}\n").into_bytes(), Some("agents.fake.path")),
            (b"[agents.fake]\nexecutable = 3\n".to_vec(), Some("agents.fake.executable")),
            (
                b"[agents.fake]\nexecutable = \"relative/x\"\n".to_vec(),
                Some("agents.fake.executable"),
            ),
        ];
        for (bytes, key) in cases {
            match parse(Path::new("c"), &bytes) {
                Err(Error::ConfigInvalid { key: actual, .. }) => {
                    assert_eq!(actual.as_deref(), key, "{:?}", String::from_utf8_lossy(&bytes))
                }
                other => panic!("{:?}: {other:?}", String::from_utf8_lossy(&bytes)),
            }
        }
    }

    #[test]
    fn failed_replacement_preserves_previous_config_and_leaves_no_temp_file() {
        let (_dir, root) = temp_root();
        let exe = absolute("old");
        let previous = format!("[agents.fake]\nexecutable = {exe:?}\n");
        fs::write(root.config_path(), &previous).unwrap();
        let error = update_with(
            &root,
            |doc| {
                set_agent(doc, "fake", &absolute("new"));
                Ok(())
            },
            || Ok(()),
            |temp, _destination| {
                drop(temp);
                Err(io::Error::other("injected replace failure"))
            },
        )
        .unwrap_err();
        assert!(matches!(error, Error::ConfigWrite { .. }), "{error:?}");
        assert_eq!(fs::read_to_string(root.config_path()).unwrap(), previous);
        assert_eq!(temp_files(&root), Vec::<String>::new());
    }

    #[test]
    fn reader_during_write_sees_previous_then_new_complete_content() {
        let (_dir, root) = temp_root();
        let previous = format!("[agents.fake]\nexecutable = {:?}\n", absolute("old"));
        fs::write(root.config_path(), &previous).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let reader = {
            let barrier = Arc::clone(&barrier);
            let root = root.clone();
            thread::spawn(move || {
                barrier.wait();
                let seen = fs::read_to_string(root.config_path()).unwrap();
                barrier.wait();
                seen
            })
        };
        update_with(
            &root,
            |doc| {
                set_agent(doc, "fake", &absolute("new"));
                Ok(())
            },
            || {
                barrier.wait();
                barrier.wait();
                Ok(())
            },
            |temp, destination| temp.persist(destination).map(drop).map_err(|error| error.error),
        )
        .unwrap();
        assert_eq!(reader.join().unwrap(), previous);
        let config = Config::load(&root).unwrap();
        assert_eq!(config.agent_executable("fake"), Some(Path::new(&absolute("new"))));
    }

    #[test]
    fn update_sweeps_stale_temp_files_and_never_reads_them() {
        let (_dir, root) = temp_root();
        fs::write(root.path().join(".config.toml.stale.tmp"), "garbage = [").unwrap();
        update(&root, |doc| {
            set_agent(doc, "fake", &absolute("x"));
            Ok(())
        })
        .unwrap();
        assert_eq!(temp_files(&root), Vec::<String>::new());
        assert!(Config::load(&root).unwrap().agent_executable("fake").is_some());
    }

    /// The Unix spelling on Unix, the Windows spelling on Windows.
    fn host<'a>(unix: &'a str, windows: &'a str) -> &'a str {
        if cfg!(windows) { windows } else { unix }
    }

    fn name(text: &str) -> ProfileName {
        ProfileName::parse(text, Platform::host()).unwrap()
    }

    fn agent(id: &str) -> AgentId {
        AgentId::parse(id).unwrap()
    }

    fn key(root: &str) -> String {
        toml_edit::Key::new(root).display_repr().into_owned()
    }

    #[test]
    fn schema_accepts_the_sp3_forms() {
        let acme = host("/src/acme", r"C:\src\acme");
        let text = format!(
            "default_profile = \"work\"\n\n[repositories.{}]\nprofile = \"work\"\nagents = {{ claude = \"personal\" }}\n\n[repositories.{}]\n\n[repositories.{}]\nprofile = \"other\"\n",
            key(acme),
            key(host("/empty", r"C:\empty")),
            key(host(r"C:\elsewhere", "/elsewhere")),
        );
        let config = parse(Path::new("c"), text.as_bytes()).unwrap();
        assert_eq!(config.default_profile(), Some(&name("work")));
        let expected = Mapping {
            profile: Some(name("work")),
            agents: BTreeMap::from([(agent("claude"), name("personal"))]),
        };
        assert_eq!(config.mapping(Path::new(acme)), Some(&expected));
        let trailing = format!("{acme}{}", std::path::MAIN_SEPARATOR);
        assert_eq!(config.mapping(Path::new(&trailing)), Some(&expected));
        assert_eq!(
            config.mapping(Path::new(host("/empty", r"C:\empty"))),
            Some(&Mapping::default())
        );
        assert_eq!(config.mappings().count(), 3);
        assert_eq!(config.mapping(Path::new(host("/src", r"C:\src"))), None);
        #[cfg(windows)]
        assert_eq!(config.mapping(Path::new("C:/src/acme")), Some(&expected));
    }

    #[test]
    fn schema_rejects_every_sp3_error_class_naming_the_key() {
        let acme = key(host("/acme", r"C:\acme"));
        let entry = |body: &str| format!("[repositories.{acme}]\n{body}\n");
        let entry_key = format!("repositories.{acme}");
        let cases = [
            ("default_profile = 3\n".to_owned(), "default_profile".to_owned(), "must be a string"),
            (
                "default_profile = \"link\"\n".to_owned(),
                "default_profile".to_owned(),
                "invalid profile name \"link\": \"link\" is a reserved command word",
            ),
            ("repositories = 3\n".to_owned(), "repositories".to_owned(), "must be a table"),
            (
                "[repositories.relative]\n".to_owned(),
                "repositories.relative".to_owned(),
                "must be an absolute path",
            ),
            (format!("[repositories]\n{acme} = 3\n"), entry_key.clone(), "must be a table"),
            (entry("profile = 3"), format!("{entry_key}.profile"), "must be a string"),
            (entry("profile = \".x\""), format!("{entry_key}.profile"), "invalid profile name"),
            (entry("agents = 3"), format!("{entry_key}.agents"), "must be a table"),
            (
                entry("agents = { Claude = \"x\" }"),
                format!("{entry_key}.agents.Claude"),
                "agent ids must match",
            ),
            (
                entry("agents = { claude = 3 }"),
                format!("{entry_key}.agents.claude"),
                "must be a string",
            ),
            (
                entry("agents = { claude = \"a b\" }"),
                format!("{entry_key}.agents.claude"),
                "invalid profile name",
            ),
            (entry("path = \"x\""), format!("{entry_key}.path"), "unknown key"),
        ];
        for (text, expected_key, detail) in cases {
            match parse(Path::new("c"), text.as_bytes()) {
                Err(Error::ConfigInvalid { key: Some(actual), detail: actual_detail, .. }) => {
                    assert_eq!(actual, expected_key, "{text:?}");
                    assert!(actual_detail.starts_with(detail), "{text:?}: {actual_detail}");
                }
                other => panic!("{text:?}: {other:?}"),
            }
        }
    }

    #[test]
    fn schema_accepts_keys_absolute_in_either_platform_form() {
        for root in ["/a", r"C:\a", "C:/a", r"\\server\share\a", "z:/"] {
            let text = format!("[repositories.{}]\n", key(root));
            assert!(parse(Path::new("c"), text.as_bytes()).is_ok(), "{root}");
        }
        for root in ["a", "C:", "C:a", r"\a", "~/a", ""] {
            let text = format!("[repositories.{}]\n", key(root));
            assert!(
                matches!(parse(Path::new("c"), text.as_bytes()), Err(Error::ConfigInvalid { .. })),
                "{root:?}"
            );
        }
    }

    #[test]
    fn component_equal_keys_are_rejected_naming_both() {
        let keys: &[&str] =
            if cfg!(windows) { &[r"C:\x", "C:/x", r"c:\x\"] } else { &["/a", "/a/"] };
        let text: String =
            keys.iter().map(|root| format!("[repositories.{}]\n", key(root))).collect();
        match parse(Path::new("c"), text.as_bytes()) {
            Err(Error::ConfigInvalid { key: Some(first), detail, .. }) => {
                let mut sorted = keys.to_vec();
                sorted.sort();
                assert_eq!(first, format!("repositories.{}", key(sorted[0])));
                assert_eq!(
                    detail,
                    format!("names the same directory as repositories.{}", key(sorted[1]))
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn link_reports_every_outcome_and_writes_only_on_change() {
        let (_dir, root) = temp_root();
        let repo = PathBuf::from(host("/src/acme", r"C:\src\acme"));
        let claude = agent("claude");
        assert_eq!(link(&root, &repo, None, &name("work")).unwrap(), LinkOutcome::Linked);
        assert_eq!(
            link(&root, &repo, Some(&claude), &name("personal")).unwrap(),
            LinkOutcome::Linked
        );
        let written = fs::read(root.config_path()).unwrap();
        assert_eq!(link(&root, &repo, None, &name("work")).unwrap(), LinkOutcome::AlreadyLinked);
        assert_eq!(
            link(&root, &repo, Some(&claude), &name("personal")).unwrap(),
            LinkOutcome::AlreadyLinked
        );
        assert_eq!(fs::read(root.config_path()).unwrap(), written, "no write when already linked");
        assert_eq!(
            link(&root, &repo, None, &name("other")).unwrap(),
            LinkOutcome::Changed { old: name("work") }
        );
        assert_eq!(
            link(&root, &repo, Some(&claude), &name("work")).unwrap(),
            LinkOutcome::Changed { old: name("personal") }
        );
        let config = Config::load(&root).unwrap();
        assert_eq!(
            config.mapping(&repo),
            Some(&Mapping {
                profile: Some(name("other")),
                agents: BTreeMap::from([(claude, name("work"))]),
            })
        );
    }

    #[test]
    fn link_and_unlink_preserve_comments_and_formatting() {
        let (_dir, root) = temp_root();
        let repo = host("/src/acme", r"C:\src\acme");
        let other = host("/src/other", r"C:\src\other");
        let text = format!(
            "# my settings\ndefault_profile   =   \"work\" # trailing\n\n[repositories.{}]\nprofile = \"work\" # keep\n",
            key(other)
        );
        fs::write(root.config_path(), &text).unwrap();
        link(&root, Path::new(repo), Some(&agent("claude")), &name("personal")).unwrap();
        let linked = fs::read_to_string(root.config_path()).unwrap();
        assert!(linked.starts_with(&text), "{linked}");
        assert_eq!(
            unlink(&root, &[PathBuf::from(repo)], Some(&agent("claude"))).unwrap(),
            UnlinkOutcome::Unlinked { root: PathBuf::from(repo), old: name("personal") }
        );
        assert_eq!(fs::read_to_string(root.config_path()).unwrap(), text);
    }

    #[test]
    fn unlink_removes_empty_agents_and_empty_entries() {
        let (_dir, root) = temp_root();
        let repo = PathBuf::from(host("/src/acme", r"C:\src\acme"));
        let keys = [repo.clone()];
        link(&root, &repo, None, &name("work")).unwrap();
        link(&root, &repo, Some(&agent("claude")), &name("personal")).unwrap();
        assert_eq!(
            unlink(&root, &keys, None).unwrap(),
            UnlinkOutcome::Unlinked { root: repo.clone(), old: name("work") }
        );
        assert_eq!(
            unlink(&root, &keys, None).unwrap(),
            UnlinkOutcome::NothingToRemove { shown: repo.clone() }
        );
        assert_eq!(
            unlink(&root, &keys, Some(&agent("codex"))).unwrap(),
            UnlinkOutcome::NothingToRemove { shown: repo.clone() }
        );
        assert!(Config::load(&root).unwrap().mapping(&repo).is_some());
        assert_eq!(
            unlink(&root, &keys, Some(&agent("claude"))).unwrap(),
            UnlinkOutcome::Unlinked { root: repo.clone(), old: name("personal") }
        );
        assert_eq!(Config::load(&root).unwrap().mapping(&repo), None);
        let text = fs::read_to_string(root.config_path()).unwrap();
        assert!(!text.contains("acme"), "{text}");
    }

    #[test]
    fn link_and_unlink_find_a_component_equal_stored_key() {
        let (_dir, root) = temp_root();
        let stored = host("/src/acme/", "C:/src/acme/");
        fs::write(
            root.config_path(),
            format!("[repositories.{}]\nprofile = \"work\"\n", key(stored)),
        )
        .unwrap();
        let repo = PathBuf::from(host("/src/acme", r"C:\src\acme"));
        assert_eq!(
            link(&root, &repo, Some(&agent("claude")), &name("personal")).unwrap(),
            LinkOutcome::Linked
        );
        assert_eq!(Config::load(&root).unwrap().mappings().count(), 1);
        assert_eq!(
            unlink(&root, std::slice::from_ref(&repo), None).unwrap(),
            UnlinkOutcome::Unlinked { root: PathBuf::from(stored), old: name("work") }
        );
    }

    #[test]
    fn unlink_takes_the_first_key_with_a_mapping_at_that_field() {
        let (_dir, root) = temp_root();
        let first = PathBuf::from(host("/a", r"C:\a"));
        let second = PathBuf::from(host("/b", r"C:\b"));
        let third = PathBuf::from(host("/c", r"C:\c"));
        link(&root, &first, Some(&agent("claude")), &name("x")).unwrap();
        link(&root, &second, None, &name("y")).unwrap();
        link(&root, &third, None, &name("z")).unwrap();
        let keys = [first.clone(), second.clone(), third.clone()];
        assert_eq!(
            unlink(&root, &keys, None).unwrap(),
            UnlinkOutcome::Unlinked { root: second, old: name("y") }
        );
        let missing = [PathBuf::from(host("/gone", r"C:\gone")), first.clone()];
        assert_eq!(
            unlink(&root, &missing, None).unwrap(),
            UnlinkOutcome::NothingToRemove { shown: missing[0].clone() }
        );
        assert_eq!(
            unlink(&root, &missing, Some(&agent("claude"))).unwrap(),
            UnlinkOutcome::Unlinked { root: first, old: name("x") }
        );
    }

    #[test]
    fn unlink_of_nothing_creates_no_configuration_file() {
        let (_dir, root) = temp_root();
        let keys = [PathBuf::from(host("/a", r"C:\a"))];
        assert!(matches!(unlink(&root, &keys, None), Ok(UnlinkOutcome::NothingToRemove { .. })));
        assert!(!root.config_path().exists());
    }

    #[test]
    fn link_refuses_a_case_twin_and_a_non_utf8_root() {
        let (_dir, root) = temp_root();
        fs::create_dir_all(root.profiles_dir().join("work")).unwrap();
        let repo = PathBuf::from(host("/a", r"C:\a"));
        let error = link(&root, &repo, None, &name("WORK")).unwrap_err();
        assert!(
            matches!(error, Error::ProfileCaseConflict { ref existing, .. } if existing == "work"),
            "{error:?}"
        );
        assert!(!root.config_path().exists());
        #[cfg(unix)]
        let non_utf8 = {
            use std::os::unix::ffi::OsStringExt;
            PathBuf::from(OsString::from_vec(b"/a\xff".to_vec()))
        };
        #[cfg(windows)]
        let non_utf8 = {
            use std::os::windows::ffi::OsStringExt;
            PathBuf::from(OsString::from_wide(&[0x43, 0x3A, 0x5C, 0xD800]))
        };
        match link(&root, &non_utf8, None, &name("work")).unwrap_err() {
            Error::Repository { path, reason } => {
                assert_eq!(path, non_utf8);
                assert_eq!(reason, "a repository path that is not valid UTF-8 cannot be linked");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn mappings_referencing_finds_case_twins_non_canonical_keys_and_agent_fields() {
        let text = format!(
            "[repositories.{}]\nprofile = \"Work\"\n\n[repositories.{}]\nagents = {{ claude = \"work\", codex = \"other\" }}\n\n[repositories.{}]\nprofile = \"other\"\n",
            key(host("/a", r"C:\a")),
            key(host("/b/../c/", "c:/b/../c/")),
            key(host("/d", r"C:\d")),
        );
        let config = parse(Path::new("c"), text.as_bytes()).unwrap();
        assert_eq!(
            config.mappings_referencing(&name("work")),
            [
                Reference {
                    root: PathBuf::from(host("/a", r"C:\a")),
                    agent: None,
                    profile: name("Work"),
                },
                Reference {
                    root: PathBuf::from(host("/b/../c/", "c:/b/../c/")),
                    agent: Some(agent("claude")),
                    profile: name("work"),
                },
            ]
        );
        assert_eq!(config.mappings_referencing(&name("none")), []);
    }

    #[test]
    fn an_edit_that_changes_nothing_writes_nothing() {
        let (_dir, root) = temp_root();
        let text = "default_profile='work'   # odd spacing\n";
        fs::write(root.config_path(), text).unwrap();
        update_with(
            &root,
            |_| Ok(()),
            || panic!("nothing to persist"),
            |_, _| panic!("no replace"),
        )
        .unwrap();
        assert_eq!(fs::read_to_string(root.config_path()).unwrap(), text);
    }

    #[test]
    fn already_linked_and_nothing_to_remove_keep_crlf_and_bom_files_byte_identical() {
        let repo = host("/src/acme", r"C:\src\acme");
        let body = format!(
            "default_profile = \"work\"\r\n\r\n[repositories.{}]\r\nprofile = \"work\"\r\n",
            key(repo)
        );
        for bytes in
            [body.clone().into_bytes(), [b"\xef\xbb\xbf".as_slice(), body.as_bytes()].concat()]
        {
            let (_dir, root) = temp_root();
            fs::write(root.config_path(), &bytes).unwrap();
            assert_eq!(
                link(&root, Path::new(repo), None, &name("work")).unwrap(),
                LinkOutcome::AlreadyLinked
            );
            assert_eq!(
                fs::read(root.config_path()).unwrap(),
                bytes,
                "AlreadyLinked rewrote the file"
            );
            let missing = [PathBuf::from(host("/gone", r"C:\gone"))];
            assert!(matches!(
                unlink(&root, &missing, None).unwrap(),
                UnlinkOutcome::NothingToRemove { .. }
            ));
            assert_eq!(
                fs::read(root.config_path()).unwrap(),
                bytes,
                "NothingToRemove rewrote the file"
            );
        }
    }

    fn temp_files(root: &AppRoot) -> Vec<String> {
        fs::read_dir(root.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(TEMP_PREFIX) && name.ends_with(TEMP_SUFFIX))
            .collect()
    }
}
```

- [ ] **Step 2: Edit `crates/agent-profile/src/adapter/mod.rs`** (the adapters use the moved check). Replace exactly this text, which occurs once:

```rust
use crate::config::{AppRoot, Config};
```

with:

```rust
use crate::config::{AppRoot, Config, check_case_twins};
```

- [ ] **Step 3: Edit `crates/agent-profile/src/adapter/mod.rs`** (remove the old copy). Replace exactly this text, which occurs once:

```rust
/// Refuses a profile whose name differs from an existing `profiles/` entry only in ASCII case (SP1 design §7.3).
fn check_case_twins(root: &AppRoot, profile: &ProfileName) -> Result<()> {
    let Ok(entries) = fs::read_dir(root.profiles_dir()) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name != profile.as_str() && name.eq_ignore_ascii_case(profile.as_str()) {
            return Err(Error::ProfileCaseConflict {
                requested: profile.to_string(),
                existing: name.to_owned(),
            });
        }
    }
    Ok(())
}

```

with nothing (delete it).

- [ ] **Step 4: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib config:: && cargo nextest run -p agent-profile --test config
```

Expected: every test passes, including `schema_rejects_every_sp3_error_class_naming_the_key`, `component_equal_keys_are_rejected_naming_both`, `link_reports_every_outcome_and_writes_only_on_change`, `mappings_referencing_finds_case_twins_non_canonical_keys_and_agent_fields`, `already_linked_and_nothing_to_remove_keep_crlf_and_bom_files_byte_identical`.

- [ ] **Step 5: Run the gate** (see "Gate commands"). Expected on Windows: `191 tests run: 191 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 6: Commit**

```bash
git add crates/agent-profile/src/config.rs crates/agent-profile/src/adapter/mod.rs
git commit -m "feat: repository mappings in config.toml with link and unlink

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 7: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/config.rs` replace exactly:

```rust
let matches = |name: &ProfileName| name.as_str().eq_ignore_ascii_case(profile.as_str());
```

with:

```rust
let matches = |name: &ProfileName| name.as_str() == profile.as_str();
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib config::`. Expected: FAIL, and the failing test list includes `mappings_referencing_finds_case_twins_non_canonical_keys_and_agent_fields`. Then restore with `git checkout -- crates/agent-profile/src/config.rs` and confirm `git status --short` prints nothing.


### Task 4: The resolver

`resolve(agent, explicit, &Config, &Discovery)` with the full V3 §12 precedence and exact-root applicability (design §4.3). Callers pass `NotInRepository` until Task 6.

**Files:**
- Modify (whole file): `crates/agent-profile/src/resolve.rs` (the resolver and its precedence tests)
- Modify: `crates/agent-profile/src/cli.rs` (the launch keeps its SP2 behaviour; Task 6 replaces this code)
- Modify: `crates/agent-profile/src/output.rs` (the report test helper; Task 5 rewrites this file)
- Modify: `crates/agent-profile/src/adapter/mod.rs` (the redaction test)

**Before:** `crates/agent-profile/src/resolve.rs` contains `pub fn resolve(agent: AgentId, explicit: Option<ProfileName>) -> Resolution {`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/resolve.rs`** (the resolver and its precedence tests; the whole file, byte-exact)

```rust
//! The single profile resolver shared by every command (spec §12, SP3 design §4.3).

use std::path::PathBuf;

use crate::config::Config;
use crate::name::{AgentId, ProfileName};
use crate::repo::Discovery;

/// Where a resolved profile came from (spec §12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionSource {
    Explicit,
    RepositoryAgent,
    RepositoryDefault,
    GlobalDefault,
    None,
}

impl ResolutionSource {
    /// The lower-case label used in human output.
    pub fn label(self) -> &'static str {
        match self {
            ResolutionSource::Explicit => "explicit",
            ResolutionSource::RepositoryAgent => "repository agent mapping",
            ResolutionSource::RepositoryDefault => "repository mapping",
            ResolutionSource::GlobalDefault => "global default",
            ResolutionSource::None => "none",
        }
    }
}

/// The canonical resolution object (spec §12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub agent: AgentId,
    pub profile: Option<ProfileName>,
    pub source: ResolutionSource,
    pub repository: Option<PathBuf>,
}

/// Pure: callers run discovery first. A mapping applies only when its key is component-equal to the
/// discovered root, so a nested repository never inherits its parent's mapping (design D5).
pub fn resolve(
    agent: AgentId,
    explicit: Option<ProfileName>,
    config: &Config,
    discovery: &Discovery,
) -> Resolution {
    let repository = match discovery {
        Discovery::Repository(root) => Some(root.clone()),
        Discovery::NotInRepository => None,
    };
    let mapping = repository.as_deref().and_then(|root| config.mapping(root));
    let (profile, source) = if let Some(profile) = explicit {
        (Some(profile), ResolutionSource::Explicit)
    } else if let Some(profile) = mapping.and_then(|mapping| mapping.agents.get(&agent)) {
        (Some(profile.clone()), ResolutionSource::RepositoryAgent)
    } else if let Some(profile) = mapping.and_then(|mapping| mapping.profile.as_ref()) {
        (Some(profile.clone()), ResolutionSource::RepositoryDefault)
    } else if let Some(profile) = config.default_profile() {
        (Some(profile.clone()), ResolutionSource::GlobalDefault)
    } else {
        (None, ResolutionSource::None)
    };
    Resolution { agent, profile, source, repository }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppRoot;
    use crate::name::Platform;
    use std::path::Path;

    fn profile(name: &str) -> ProfileName {
        ProfileName::parse(name, Platform::host()).unwrap()
    }

    fn agent(id: &str) -> AgentId {
        AgentId::parse(id).unwrap()
    }

    /// `/r` on Unix and `C:\r` on Windows, so the key is absolute on the host.
    fn root(name: &str) -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(format!(r"C:\{name}"))
        } else {
            PathBuf::from(format!("/{name}"))
        }
    }

    fn config(text: &str) -> Config {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.toml"), text).unwrap();
        Config::load(&AppRoot::from_path(dir.path().to_path_buf())).unwrap()
    }

    fn key(path: &Path) -> String {
        toml_edit::Key::new(path.to_str().unwrap()).display_repr().into_owned()
    }

    fn resolved(
        config: &Config,
        explicit: Option<&str>,
        discovery: &Discovery,
        id: &str,
    ) -> (Option<String>, ResolutionSource) {
        let resolution = resolve(agent(id), explicit.map(profile), config, discovery);
        (resolution.profile.map(|profile| profile.to_string()), resolution.source)
    }

    #[test]
    fn each_precedence_step_wins_over_the_ones_below_it() {
        let repo = root("acme");
        let full = config(&format!(
            "default_profile = \"global\"\n[repositories.{}]\nprofile = \"repo\"\nagents = {{ claude = \"agent\" }}\n",
            key(&repo)
        ));
        let inside = Discovery::Repository(repo.clone());
        assert_eq!(
            resolved(&full, Some("explicit"), &inside, "claude"),
            (Some("explicit".to_owned()), ResolutionSource::Explicit)
        );
        assert_eq!(
            resolved(&full, None, &inside, "claude"),
            (Some("agent".to_owned()), ResolutionSource::RepositoryAgent)
        );
        assert_eq!(
            resolved(&full, None, &inside, "codex"),
            (Some("repo".to_owned()), ResolutionSource::RepositoryDefault)
        );
        let agents_only = config(&format!(
            "default_profile = \"global\"\n[repositories.{}]\nagents = {{ claude = \"agent\" }}\n",
            key(&repo)
        ));
        assert_eq!(
            resolved(&agents_only, None, &inside, "codex"),
            (Some("global".to_owned()), ResolutionSource::GlobalDefault)
        );
        assert_eq!(resolved(&config(""), None, &inside, "codex"), (None, ResolutionSource::None));
    }

    #[test]
    fn a_mapping_applies_only_to_its_exact_root() {
        let parent = root("acme");
        let text = format!("[repositories.{}]\nprofile = \"work\"\n", key(&parent));
        let config = config(&text);
        let nested = Discovery::Repository(parent.join("vendor").join("lib"));
        assert_eq!(resolved(&config, None, &nested, "claude"), (None, ResolutionSource::None));
        let trailing = Discovery::Repository(PathBuf::from(format!(
            "{}{}",
            parent.display(),
            std::path::MAIN_SEPARATOR
        )));
        assert_eq!(
            resolved(&config, None, &trailing, "claude"),
            (Some("work".to_owned()), ResolutionSource::RepositoryDefault)
        );
    }

    #[test]
    fn outside_a_repository_only_the_global_default_applies() {
        let text = format!(
            "default_profile = \"global\"\n[repositories.{}]\nprofile = \"work\"\n",
            key(&root("acme"))
        );
        let outside = Discovery::NotInRepository;
        assert_eq!(
            resolved(&config(&text), None, &outside, "claude"),
            (Some("global".to_owned()), ResolutionSource::GlobalDefault)
        );
        let text = format!("[repositories.{}]\nprofile = \"work\"\n", key(&root("acme")));
        assert_eq!(
            resolved(&config(&text), None, &outside, "claude"),
            (None, ResolutionSource::None)
        );
    }

    #[test]
    fn the_repository_is_reported_for_every_source() {
        let repo = root("acme");
        let text = format!(
            "default_profile = \"global\"\n[repositories.{}]\nprofile = \"repo\"\nagents = {{ claude = \"agent\" }}\n",
            key(&repo)
        );
        let full = config(&text);
        let inside = Discovery::Repository(repo.clone());
        let other = Discovery::Repository(root("other"));
        for (explicit, discovery, id, source) in [
            (Some("x"), &inside, "claude", ResolutionSource::Explicit),
            (None, &inside, "claude", ResolutionSource::RepositoryAgent),
            (None, &inside, "codex", ResolutionSource::RepositoryDefault),
            (None, &other, "codex", ResolutionSource::GlobalDefault),
        ] {
            let resolution = resolve(agent(id), explicit.map(profile), &full, discovery);
            assert_eq!(resolution.source, source);
            let expected = match discovery {
                Discovery::Repository(root) => Some(root.clone()),
                Discovery::NotInRepository => None,
            };
            assert_eq!(resolution.repository, expected, "{source:?}");
        }
        let none = resolve(agent("codex"), None, &config(""), &other);
        assert_eq!((none.source, none.repository), (ResolutionSource::None, Some(root("other"))));
        let outside =
            resolve(agent("codex"), Some(profile("x")), &full, &Discovery::NotInRepository);
        assert_eq!(outside.repository, None);
    }
}
```

- [ ] **Step 2: Edit `crates/agent-profile/src/cli.rs`** (the launch keeps its SP2 behaviour; Task 6 replaces this code). Replace exactly this text, which occurs once:

```rust
    let resolution = resolve::resolve(agent.clone(), profile);
```

with:

```rust
    let resolution =
        resolve::resolve(agent.clone(), profile, &config, &crate::repo::Discovery::NotInRepository);
```

- [ ] **Step 3: Edit `crates/agent-profile/src/output.rs`** (the report test helper; Task 5 rewrites this file). Replace exactly this text, which occurs once:

```rust
            Some(ProfileName::parse("work", Platform::Unix).unwrap()),
        )
```

with:

```rust
            Some(ProfileName::parse("work", Platform::Unix).unwrap()),
            &crate::config::Config::default(),
            &crate::repo::Discovery::NotInRepository,
        )
```

- [ ] **Step 4: Edit `crates/agent-profile/src/adapter/mod.rs`** (the redaction test). Replace exactly this text, which occurs once:

```rust
            Some(profile.clone()),
        );
```

with:

```rust
            Some(profile.clone()),
            &config,
            &crate::repo::Discovery::NotInRepository,
        );
```

- [ ] **Step 5: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib resolve::
```

Expected: every test passes, including `each_precedence_step_wins_over_the_ones_below_it`, `a_mapping_applies_only_to_its_exact_root`.

- [ ] **Step 6: Run the gate** (see "Gate commands"). Expected on Windows: `194 tests run: 194 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 7: Commit**

```bash
git add crates/agent-profile/src/resolve.rs crates/agent-profile/src/cli.rs crates/agent-profile/src/output.rs crates/agent-profile/src/adapter/mod.rs
git commit -m "feat: resolve profiles from repository mappings and the global default

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 8: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/resolve.rs` replace exactly:

```rust
    } else if let Some(profile) = mapping.and_then(|mapping| mapping.agents.get(&agent)) {
```

with:

```rust
    } else if let Some(profile) = mapping.and_then(|mapping| mapping.agents.get(&agent)).filter(|_| false) {
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib resolve::`. Expected: FAIL, and the failing test list includes `each_precedence_step_wins_over_the_ones_below_it`. Then restore with `git checkout -- crates/agent-profile/src/resolve.rs` and confirm `git status --short` prints nothing.


### Task 5: Command output

The `resolve`, `status`, `link` and `unlink` renderers (design §7.5).

**Files:**
- Modify (whole file): `crates/agent-profile/src/output.rs` (the new renderers and their tests)

**Before:** `crates/agent-profile/src/output.rs` does not contain `pub fn status_lines`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/output.rs`** (the new renderers and their tests; the whole file, byte-exact)

```rust
//! Human output: the dry-run and `--verbose` report (spec §26, SP1 design §7.4, SP2 design §7.3), the
//! repository command reports (SP3 design §7.5) and redaction (spec §22). JSON output (spec §32) arrives in
//! SP5.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::adapter::{PathKind, PlannedLaunch};
use crate::config::{Config, LinkOutcome, UnlinkOutcome};
use crate::exe::Origin;
use crate::name::{AgentId, ProfileName};
use crate::repo::Discovery;
use crate::resolve::Resolution;

/// Substrings that mark an override variable as secret-bearing, whatever the adapter declared.
const SENSITIVE_NAME_PARTS: [&str; 6] =
    ["TOKEN", "SECRET", "KEY", "PASSWORD", "CREDENTIAL", "AUTH"];

const LABEL_WIDTH: usize = 14;

/// Which report is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportMode {
    /// Before anything is created: marks and lists what a launch would create.
    DryRun,
    /// After initialization: nothing is left to create, so there are no markers and no `creates:` line.
    Verbose,
}

/// The report lines, without trailing newlines.
pub fn report_lines(
    planned: &PlannedLaunch,
    resolution: &Resolution,
    mode: ReportMode,
) -> Vec<String> {
    let mut lines = Vec::new();
    let line = |label: &str, value: String| format!("{:<LABEL_WIDTH$}{value}", format!("{label}:"));
    let continuation = |value: String| format!("{:LABEL_WIDTH$}{value}", "");
    lines.push(line("agent", resolution.agent.to_string()));
    lines.push(line("profile", format!("{} ({})", planned.profile, resolution.source.label())));
    let origin = match planned.executable_origin {
        Origin::Configured => "configured",
        Origin::Path => "PATH",
    };
    lines.push(line("executable", format!("{} ({origin})", planned.plan.executable.display())));
    let repository = match &resolution.repository {
        Some(path) => path.display().to_string(),
        None => "none".to_owned(),
    };
    lines.push(line("repository", repository));
    lines.push(line("mechanism", planned.mechanism.clone()));
    if planned.plan.env.is_empty() {
        lines.push(line("environment", "none".to_owned()));
    }
    let missing: Vec<_> = planned.paths.iter().filter(|entry| !entry.existed).collect();
    for (index, (key, value)) in planned.plan.env.iter().enumerate() {
        let shown = if is_sensitive(key, &planned.sensitive_env) {
            "<redacted>".to_owned()
        } else {
            let mut shown = value.to_string_lossy().into_owned();
            let will_create = missing.iter().any(|entry| {
                entry.kind == PathKind::Dir && entry.path.as_os_str() == value.as_os_str()
            });
            if mode == ReportMode::DryRun && will_create {
                shown.push_str(" (would be created)");
            }
            shown
        };
        let entry = format!("{}={shown}", key.to_string_lossy());
        lines.push(if index == 0 { line("environment", entry) } else { continuation(entry) });
    }
    if mode == ReportMode::DryRun {
        if missing.is_empty() {
            lines.push(line("creates", "none".to_owned()));
        }
        for (index, entry) in missing.iter().enumerate() {
            let value = format!("{} (would be created)", entry.path.display());
            lines.push(if index == 0 { line("creates", value) } else { continuation(value) });
        }
    }
    let args = render_args(&planned.plan.args);
    lines.push(line("arguments", format!("[{}]", args.join(", "))));
    for note in &planned.notes {
        lines.push(line("note", note.clone()));
    }
    lines
}

/// `label:` padded to the report column, then `value`.
fn labeled(label: &str, value: impl std::fmt::Display) -> String {
    format!("{:<LABEL_WIDTH$}{value}", format!("{label}:"))
}

/// A `note:` line.
pub fn note(text: &str) -> String {
    labeled("note", text)
}

fn or_none(value: Option<impl std::fmt::Display>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

/// `<agent> resolve` (SP3 design §7.5).
pub fn resolve_lines(resolution: &Resolution) -> Vec<String> {
    vec![
        labeled("agent", &resolution.agent),
        labeled("profile", or_none(resolution.profile.as_ref())),
        labeled("source", resolution.source.label()),
        labeled("repository", or_none(resolution.repository.as_deref().map(Path::display))),
    ]
}

/// `status` and `<agent> status`: `agents` holds one resolution per agent line and, for `<agent> status`,
/// its `presence:` value (SP3 design §7.5).
pub fn status_lines(
    discovery: &Discovery,
    config: &Config,
    agents: &[(Resolution, Option<String>)],
) -> Vec<String> {
    let (repository, mapping) = match discovery {
        Discovery::Repository(root) => (root.display().to_string(), config.mapping(root)),
        Discovery::NotInRepository => ("not in a repository".to_owned(), None),
    };
    let mut lines = vec![labeled("repository", repository)];
    lines.push(labeled("mapping", or_none(mapping.and_then(|mapping| mapping.profile.as_ref()))));
    let pairs = mapping
        .map(|mapping| {
            mapping.agents.iter().map(|(id, profile)| format!("{id}={profile}")).collect::<Vec<_>>()
        })
        .filter(|pairs| !pairs.is_empty())
        .map(|pairs| pairs.join(", "));
    lines.push(labeled("agents", or_none(pairs)));
    lines.push(labeled("default", or_none(config.default_profile())));
    for (resolution, presence) in agents {
        let value = match &resolution.profile {
            Some(profile) => format!("{profile} ({})", resolution.source.label()),
            None => "none".to_owned(),
        };
        lines.push(labeled(resolution.agent.as_str(), value));
        if let Some(presence) = presence {
            lines.push(labeled("presence", presence));
        }
    }
    if let Discovery::Repository(root) = discovery {
        for (key, _) in config.mappings().filter(|(key, _)| is_proper_ancestor(key, root)) {
            lines.push(note(&format!(
                "{} has a mapping that does not apply to this repository",
                key.display()
            )));
        }
    }
    lines
}

fn is_proper_ancestor(key: &Path, path: &Path) -> bool {
    path.starts_with(key) && key != path
}

/// `link` (SP3 design §7.5).
pub fn link_line(
    outcome: &LinkOutcome,
    agent: Option<&AgentId>,
    repository: &Path,
    profile: &ProfileName,
) -> String {
    let agent = agent.map(|agent| format!("{agent}: ")).unwrap_or_default();
    let repository = repository.display();
    match outcome {
        LinkOutcome::Linked => format!("linked {agent}{repository} -> {profile}"),
        LinkOutcome::Changed { old } => format!("changed {agent}{repository}: {old} -> {profile}"),
        LinkOutcome::AlreadyLinked => format!("already linked {agent}{repository} -> {profile}"),
    }
}

/// `unlink` (SP3 design §7.5). `config` is the configuration read before the command, `keys` the candidates.
pub fn unlink_lines(
    outcome: &UnlinkOutcome,
    agent: Option<&AgentId>,
    config: &Config,
    keys: &[PathBuf],
) -> Vec<String> {
    let prefix = agent.map(|agent| format!("{agent}: ")).unwrap_or_default();
    let shown = match outcome {
        UnlinkOutcome::Unlinked { root, old } => {
            return vec![format!("unlinked {prefix}{} (was {old})", root.display())];
        }
        UnlinkOutcome::NothingToRemove { shown } => shown,
    };
    let mut lines = vec![format!("no mapping to remove for {prefix}{}", shown.display())];
    for (key, _) in config.mappings().filter(|(key, _)| is_proper_ancestor(key, shown)) {
        lines.push(note(&format!(
            "{} has a mapping; remove it with agent-profile unlink --repo {}",
            key.display(),
            key.display()
        )));
    }
    if agent.is_none()
        && let Some(mapping) = keys.iter().find_map(|key| config.mapping(key))
        && mapping.profile.is_none()
        && !mapping.agents.is_empty()
    {
        let pairs: Vec<String> =
            mapping.agents.iter().map(|(id, profile)| format!("{id}={profile}")).collect();
        lines.push(note(&format!(
            "agent mappings remain: {}; remove them with agent-profile <agent> unlink",
            pairs.join(", ")
        )));
    }
    lines
}

fn is_sensitive(key: &OsStr, declared: &[std::ffi::OsString]) -> bool {
    if declared.iter().any(|name| name == key) {
        return true;
    }
    has_sensitive_part(&key.to_string_lossy())
}

fn has_sensitive_part(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    SENSITIVE_NAME_PARTS.iter().any(|part| upper.contains(part))
}

/// How one opaque argument is shown (spec §26 "Sensitive values must be redacted", §36).
enum Shown {
    Verbatim,
    /// The argument's value is replaced: `prefix<redacted>`.
    Redacted(String),
    /// The argument is a secret-named option without `=`: the next argument is its value.
    HidesNext,
}

/// Renders the opaque arguments for the report. Only the report is redacted; the launched arguments never
/// change. Redaction is a shallow name rule, never a parser: a `--option` whose name holds a sensitive part
/// hides its value (`--api-key=<redacted>`, or the next argument); a `NAME=value` whose (possibly dotted) NAME
/// holds one hides the value (`mcp_servers.gh.env.GITHUB_TOKEN=<redacted>`); a `Name: value` header whose name
/// holds one hides the value (`Authorization: <redacted>`). Every argument is scanned, including after a `--`,
/// and boolean-looking names get no exemption, because hiding too much only costs readability.
fn render_args(args: &[std::ffi::OsString]) -> Vec<String> {
    let mut rendered = Vec::with_capacity(args.len());
    let mut hide_next = false;
    for arg in args {
        let shown = classify(&arg.to_string_lossy());
        if hide_next {
            // A hidden secret-named option still hides its own value, so a chain never leaks.
            hide_next = matches!(shown, Shown::HidesNext);
            rendered.push(format!("{:?}", "<redacted>"));
            continue;
        }
        rendered.push(match shown {
            Shown::Verbatim => render_arg(arg),
            Shown::Redacted(prefix) => format!("{:?}", format!("{prefix}<redacted>")),
            Shown::HidesNext => {
                hide_next = true;
                render_arg(arg)
            }
        });
    }
    rendered
}

fn classify(text: &str) -> Shown {
    if let Some(option) = text.strip_prefix("--").filter(|option| !option.is_empty()) {
        let (name, value) = match option.split_once('=') {
            Some((name, value)) => (name, Some(value)),
            None => (option, None),
        };
        if has_sensitive_part(name) {
            return match value {
                Some(_) => Shown::Redacted(format!("--{name}=")),
                None => Shown::HidesNext,
            };
        }
        return match value.and_then(sensitive_value_prefix) {
            Some(prefix) => Shown::Redacted(format!("--{name}={prefix}")),
            None => Shown::Verbatim,
        };
    }
    match sensitive_value_prefix(text) {
        Some(prefix) => Shown::Redacted(prefix),
        None => Shown::Verbatim,
    }
}

/// The shown prefix when `text` is a `NAME=value` assignment or a `Name: value` header whose name holds a
/// sensitive part: `NAME=` or `Name: `.
fn sensitive_value_prefix(text: &str) -> Option<String> {
    if let Some((name, _)) = text.split_once('=')
        && is_dotted_identifier(name)
        && has_sensitive_part(name)
    {
        return Some(format!("{name}="));
    }
    let (name, _) = text.split_once(':')?;
    let header = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    (header && has_sensitive_part(name)).then(|| format!("{name}: "))
}

/// `A_1`, `mcp_servers.gh.env.GITHUB_TOKEN` or `mcp_servers.chrome-devtools.http_headers.X-Api-Key`: TOML
/// bare keys (letters, digits, `_`, `-`) joined by dots, as Codex `-c` dotted paths are.
fn is_dotted_identifier(name: &str) -> bool {
    name.split('.').all(|segment| {
        !segment.is_empty()
            && segment.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    })
}

fn render_arg(arg: &OsStr) -> String {
    match arg.to_str() {
        Some(text) => format!("{text:?}"),
        None => format!("{:?} (non-UTF-8)", arg.to_string_lossy()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::ProfilePath;
    use crate::launch::LaunchPlan;
    use crate::name::{AgentId, Platform, ProfileName};
    use crate::resolve::resolve;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn planned(
        env: Vec<(OsString, OsString)>,
        sensitive: Vec<OsString>,
        exists: bool,
    ) -> PlannedLaunch {
        let dir = PathBuf::from("/root/profiles/work/fake");
        PlannedLaunch {
            plan: LaunchPlan {
                executable: PathBuf::from("/bin/fake-agent"),
                args: vec!["--foo".into(), "a b".into()],
                env,
                cwd: None,
            },
            profile: ProfileName::parse("work", Platform::Unix).unwrap(),
            profile_dir: dir.clone(),
            paths: vec![ProfilePath { path: dir, kind: PathKind::Dir, existed: exists }],
            executable_origin: Origin::Configured,
            mechanism: "environment variable FAKE_AGENT_HOME".to_owned(),
            sensitive_env: sensitive,
            notes: Vec::new(),
        }
    }

    fn resolution() -> Resolution {
        resolve(
            AgentId::parse("fake").unwrap(),
            Some(ProfileName::parse("work", Platform::Unix).unwrap()),
            &crate::config::Config::default(),
            &Discovery::NotInRepository,
        )
    }

    fn home_env() -> Vec<(OsString, OsString)> {
        vec![("FAKE_AGENT_HOME".into(), "/root/profiles/work/fake".into())]
    }

    #[test]
    fn report_has_every_spec_26_field() {
        let lines =
            report_lines(&planned(home_env(), vec![], false), &resolution(), ReportMode::DryRun);
        let dir = PathBuf::from("/root/profiles/work/fake");
        assert_eq!(
            lines,
            [
                "agent:        fake".to_owned(),
                "profile:      work (explicit)".to_owned(),
                format!(
                    "executable:   {} (configured)",
                    PathBuf::from("/bin/fake-agent").display()
                ),
                "repository:   none".to_owned(),
                "mechanism:    environment variable FAKE_AGENT_HOME".to_owned(),
                "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake (would be created)"
                    .to_owned(),
                format!("creates:      {} (would be created)", dir.display()),
                "arguments:    [\"--foo\", \"a b\"]".to_owned(),
            ]
        );
        let existing =
            report_lines(&planned(home_env(), vec![], true), &resolution(), ReportMode::DryRun);
        assert_eq!(existing[5], "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake");
        assert_eq!(existing[6], "creates:      none");
    }

    #[test]
    fn verbose_has_no_markers_and_no_creates_line() {
        let lines =
            report_lines(&planned(home_env(), vec![], false), &resolution(), ReportMode::Verbose);
        assert_eq!(lines.len(), 7, "{lines:?}");
        let text = lines.join("\n");
        assert!(!text.contains("(would be created)"), "{text}");
        assert!(!text.contains("creates:"), "{text}");
    }

    #[test]
    fn several_missing_paths_and_notes_are_listed_in_order() {
        let mut planned = planned(Vec::new(), vec![], false);
        let file = PathBuf::from("/root/profiles/work/aider/.aider.conf.yml");
        planned.paths.push(ProfilePath {
            path: file.clone(),
            kind: PathKind::File { contents: b"{}\n" },
            existed: false,
        });
        planned.notes = vec!["first note".to_owned(), "second note".to_owned()];
        let lines = report_lines(&planned, &resolution(), ReportMode::DryRun);
        assert_eq!(lines[5], "environment:  none");
        assert_eq!(
            lines[6],
            format!("creates:      {} (would be created)", planned.paths[0].path.display())
        );
        assert_eq!(lines[7], format!("              {} (would be created)", file.display()));
        assert_eq!(&lines[9..], ["note:         first note", "note:         second note"]);
        let verbose = report_lines(&planned, &resolution(), ReportMode::Verbose);
        assert_eq!(verbose.last().unwrap(), "note:         second note");
    }

    #[test]
    fn sensitive_argument_values_are_redacted_in_the_report_only() {
        let args: Vec<std::ffi::OsString> = [
            "--api-key",
            "anthropic=sk-1",
            "--openai-api-key=sk-2",
            "--set-env",
            "ANTHROPIC_API_KEY=sk-3",
            "--set-env=OPENAI_API_KEY=sk-4",
            "--",
            "GITHUB_TOKEN=sk-5",
            "--no-op-key",
            "sk-6",
            "--model",
            "gpt",
            "path=a=b",
            "-c",
            "mcp_servers.gh.env.GITHUB_TOKEN=\"sk-7\"",
            "--config=model_providers.x.experimental_bearer_token=sk-8",
            "--header",
            "Authorization: Bearer sk-9",
            "--header=X-Api-Key:sk-10",
            "mcp_servers.chrome-devtools.env.GITHUB_TOKEN=sk-11",
            "mcp_servers.gh.http_headers.X-Api-Key=sk-12",
            "Proxy-Authorization: Basic sk-13:with-colon",
            "https://example.com/mcp",
            "a.b=c",
            "--Auth-Token",
            "--also-hidden",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        assert_eq!(
            render_args(&args),
            [
                r#""--api-key""#,
                r#""<redacted>""#,
                r#""--openai-api-key=<redacted>""#,
                r#""--set-env""#,
                r#""ANTHROPIC_API_KEY=<redacted>""#,
                r#""--set-env=OPENAI_API_KEY=<redacted>""#,
                r#""--""#,
                r#""GITHUB_TOKEN=<redacted>""#,
                r#""--no-op-key""#,
                r#""<redacted>""#,
                r#""--model""#,
                r#""gpt""#,
                r#""path=a=b""#,
                r#""-c""#,
                r#""mcp_servers.gh.env.GITHUB_TOKEN=<redacted>""#,
                r#""--config=model_providers.x.experimental_bearer_token=<redacted>""#,
                r#""--header""#,
                r#""Authorization: <redacted>""#,
                r#""--header=X-Api-Key: <redacted>""#,
                r#""mcp_servers.chrome-devtools.env.GITHUB_TOKEN=<redacted>""#,
                r#""mcp_servers.gh.http_headers.X-Api-Key=<redacted>""#,
                r#""Proxy-Authorization: <redacted>""#,
                r#""https://example.com/mcp""#,
                r#""a.b=c""#,
                r#""--Auth-Token""#,
                r#""<redacted>""#,
            ]
        );
        let mut planned = planned(Vec::new(), vec![], true);
        planned.plan.args = args.clone();
        let text = report_lines(&planned, &resolution(), ReportMode::Verbose).join("\n");
        for secret in ["sk-", "also-hidden"] {
            assert!(!text.contains(secret), "{text}");
        }
        assert_eq!(planned.plan.args, args, "the launched arguments never change");
    }

    #[test]
    fn every_sensitive_name_part_and_padded_headers_are_redacted() {
        let args: Vec<std::ffi::OsString> = [
            "--client-secret=a",
            "--db-password=b",
            "--credential-file=c",
            "--x-token=d",
            "--api-key=e",
            "--auth=f",
            "Authorization: Basic dXNlcjpwYXNz==",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        assert_eq!(
            render_args(&args),
            [
                r#""--client-secret=<redacted>""#,
                r#""--db-password=<redacted>""#,
                r#""--credential-file=<redacted>""#,
                r#""--x-token=<redacted>""#,
                r#""--api-key=<redacted>""#,
                r#""--auth=<redacted>""#,
                r#""Authorization: <redacted>""#,
            ]
        );
    }

    #[test]
    fn a_hidden_secret_option_still_hides_its_own_value() {
        let args: Vec<std::ffi::OsString> = [
            "--api-key",
            "--client-secret",
            "sk-live",
            "--model",
            "gpt",
            "--token",
            "GITHUB_TOKEN=x",
            "--shown",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        assert_eq!(
            render_args(&args),
            [
                r#""--api-key""#,
                r#""<redacted>""#,
                r#""<redacted>""#,
                r#""--model""#,
                r#""gpt""#,
                r#""--token""#,
                r#""<redacted>""#,
                r#""--shown""#,
            ]
        );
    }

    #[test]
    fn redaction_by_declaration_and_by_name() {
        let env: Vec<(OsString, OsString)> = vec![
            ("PLAIN".into(), "visible".into()),
            ("DECLARED".into(), "hidden-1".into()),
            ("MY_api_Token".into(), "hidden-2".into()),
            ("GITHUB_AUTH".into(), "hidden-3".into()),
        ];
        let lines = report_lines(
            &planned(env, vec!["DECLARED".into()], true),
            &resolution(),
            ReportMode::DryRun,
        );
        let text = lines.join("\n");
        assert!(text.contains("PLAIN=visible"), "{text}");
        for secret in ["hidden-1", "hidden-2", "hidden-3"] {
            assert!(!text.contains(secret), "{text}");
        }
        assert_eq!(text.matches("<redacted>").count(), 3, "{text}");
        assert!(lines[6].starts_with("              DECLARED="), "{text}");
    }

    /// The Unix spelling on Unix, the Windows spelling on Windows.
    fn host(unix: &str, windows: &str) -> PathBuf {
        PathBuf::from(if cfg!(windows) { windows } else { unix })
    }

    fn config(text: &str) -> Config {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.toml"), text).unwrap();
        Config::load(&crate::config::AppRoot::from_path(dir.path().to_path_buf())).unwrap()
    }

    fn key(path: &Path) -> String {
        toml_edit::Key::new(path.to_str().unwrap()).display_repr().into_owned()
    }

    fn name(text: &str) -> ProfileName {
        ProfileName::parse(text, Platform::host()).unwrap()
    }

    #[test]
    fn resolve_lines_show_the_source_and_none() {
        let acme = host("/src/acme", r"C:\src\acme");
        let text = format!("[repositories.{}]\nagents = {{ claude = \"personal\" }}\n", key(&acme));
        let config = config(&text);
        let claude = AgentId::parse("claude").unwrap();
        let inside = resolve(claude.clone(), None, &config, &Discovery::Repository(acme.clone()));
        assert_eq!(
            resolve_lines(&inside),
            [
                "agent:        claude".to_owned(),
                "profile:      personal".to_owned(),
                "source:       repository agent mapping".to_owned(),
                format!("repository:   {}", acme.display()),
            ]
        );
        let outside = resolve(claude, None, &config, &Discovery::NotInRepository);
        assert_eq!(
            resolve_lines(&outside),
            [
                "agent:        claude",
                "profile:      none",
                "source:       none",
                "repository:   none"
            ]
        );
    }

    #[test]
    fn status_lines_show_mappings_the_default_each_agent_and_ancestor_notes() {
        let src = host("/src", r"C:\src");
        let acme = src.join("acme");
        let text = format!(
            "default_profile = \"work\"\n[repositories.{}]\nprofile = \"work\"\nagents = {{ codex = \"b\", claude = \"personal\" }}\n[repositories.{}]\nprofile = \"outer\"\n[repositories.{}]\nprofile = \"root\"\n[repositories.{}]\nprofile = \"sibling\"\n",
            key(&acme),
            key(&src),
            key(&host("/", r"C:\")),
            key(&host("/src/acme-2", r"C:\src\acme-2")),
        );
        let config = config(&text);
        let discovery = Discovery::Repository(acme.clone());
        let rows: Vec<(Resolution, Option<String>)> = ["claude", "aider"]
            .into_iter()
            .map(|id| (resolve(AgentId::parse(id).unwrap(), None, &config, &discovery), None))
            .collect();
        assert_eq!(
            status_lines(&discovery, &config, &rows),
            [
                format!("repository:   {}", acme.display()),
                "mapping:      work".to_owned(),
                "agents:       claude=personal, codex=b".to_owned(),
                "default:      work".to_owned(),
                "claude:       personal (repository agent mapping)".to_owned(),
                "aider:        work (repository mapping)".to_owned(),
                format!(
                    "note:         {} has a mapping that does not apply to this repository",
                    host("/", r"C:\").display()
                ),
                format!(
                    "note:         {} has a mapping that does not apply to this repository",
                    src.display()
                ),
            ]
        );
    }

    #[test]
    fn agent_status_lines_add_presence_and_show_none_outside_a_repository() {
        let config = config("");
        let claude = AgentId::parse("claude").unwrap();
        let none = resolve(claude.clone(), None, &config, &Discovery::NotInRepository);
        assert_eq!(
            status_lines(&Discovery::NotInRepository, &config, &[(none, None)]),
            [
                "repository:   not in a repository",
                "mapping:      none",
                "agents:       none",
                "default:      none",
                "claude:       none",
            ]
        );
        let explicit = resolve(claude, Some(name("work")), &config, &Discovery::NotInRepository);
        let lines = status_lines(
            &Discovery::NotInRepository,
            &config,
            &[(explicit, Some("conflicts with Work".to_owned()))],
        );
        assert_eq!(
            &lines[4..],
            ["claude:       work (explicit)", "presence:     conflicts with Work"]
        );
    }

    #[test]
    fn link_lines_cover_every_outcome() {
        let acme = host("/src/acme", r"C:\src\acme");
        let claude = AgentId::parse("claude").unwrap();
        let shown = acme.display();
        for (outcome, agent, expected) in [
            (LinkOutcome::Linked, None, format!("linked {shown} -> work")),
            (LinkOutcome::Linked, Some(&claude), format!("linked claude: {shown} -> work")),
            (
                LinkOutcome::Changed { old: name("personal") },
                None,
                format!("changed {shown}: personal -> work"),
            ),
            (
                LinkOutcome::Changed { old: name("personal") },
                Some(&claude),
                format!("changed claude: {shown}: personal -> work"),
            ),
            (LinkOutcome::AlreadyLinked, None, format!("already linked {shown} -> work")),
            (
                LinkOutcome::AlreadyLinked,
                Some(&claude),
                format!("already linked claude: {shown} -> work"),
            ),
        ] {
            assert_eq!(link_line(&outcome, agent, &acme, &name("work")), expected);
        }
    }

    #[test]
    fn unlink_lines_cover_every_outcome_and_both_notes() {
        let src = host("/src", r"C:\src");
        let acme = src.join("acme");
        let claude = AgentId::parse("claude").unwrap();
        let text = format!(
            "[repositories.{}]\nprofile = \"outer\"\n[repositories.{}]\nagents = {{ codex = \"b\", claude = \"personal\" }}\n",
            key(&src),
            key(&acme)
        );
        let config = config(&text);
        let removed = UnlinkOutcome::Unlinked { root: acme.clone(), old: name("work") };
        assert_eq!(
            unlink_lines(&removed, None, &config, std::slice::from_ref(&acme)),
            [format!("unlinked {} (was work)", acme.display())]
        );
        assert_eq!(
            unlink_lines(&removed, Some(&claude), &config, std::slice::from_ref(&acme)),
            [format!("unlinked claude: {} (was work)", acme.display())]
        );
        let nothing = UnlinkOutcome::NothingToRemove { shown: acme.clone() };
        let ancestor = format!(
            "note:         {} has a mapping; remove it with agent-profile unlink --repo {}",
            src.display(),
            src.display()
        );
        assert_eq!(
            unlink_lines(&nothing, None, &config, std::slice::from_ref(&acme)),
            [
                format!("no mapping to remove for {}", acme.display()),
                ancestor.clone(),
                "note:         agent mappings remain: claude=personal, codex=b; remove them with \
                 agent-profile <agent> unlink"
                    .to_owned(),
            ]
        );
        assert_eq!(
            unlink_lines(&nothing, Some(&claude), &config, std::slice::from_ref(&acme)),
            [format!("no mapping to remove for claude: {}", acme.display()), ancestor]
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_argument_is_rendered_lossily_with_marker() {
        use std::os::unix::ffi::OsStringExt;
        assert_eq!(render_arg(&OsString::from_vec(vec![0x66, 0xff])), "\"f\u{fffd}\" (non-UTF-8)");
    }

    #[cfg(windows)]
    #[test]
    fn non_utf8_argument_is_rendered_lossily_with_marker() {
        use std::os::windows::ffi::OsStringExt;
        assert_eq!(render_arg(&OsString::from_wide(&[0x66, 0xD800])), "\"f\u{fffd}\" (non-UTF-8)");
    }
}
```

- [ ] **Step 2: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib output::
```

Expected: every test passes, including `status_lines_show_mappings_the_default_each_agent_and_ancestor_notes`, `unlink_lines_cover_every_outcome_and_both_notes`.

- [ ] **Step 3: Run the gate** (see "Gate commands"). Expected on Windows: `199 tests run: 199 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 4: Commit**

```bash
git add crates/agent-profile/src/output.rs
git commit -m "feat: resolve, status, link and unlink reports

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 5: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/output.rs` replace exactly:

```rust
    path.starts_with(key) && key != path
```

with:

```rust
    path.starts_with(key)
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib output::`. Expected: FAIL, and the failing test list includes `status_lines_show_mappings_the_default_each_agent_and_ancestor_notes`. Then restore with `git checkout -- crates/agent-profile/src/output.rs` and confirm `git status --short` prints nothing.


### Task 6: CLI routing, the repository commands and the resolved launch

Top-level dispatch before Clap, `--repo` binding, the command checks and usage texts, `current`, `resolve`, `status`, `link`, `unlink`, and discovery in the launch (design §7). Integration tests run from a guarded temp directory.

**Files:**
- Modify (whole file): `crates/agent-profile/src/cli.rs` (routing, commands, launch discovery and routing tests)
- Modify (whole file): `crates/agent-profile/tests/support/mod.rs` (the repository guard and the default working directory)
- Modify: `crates/agent-profile/tests/launch.rs` (design §7.2 changes this SP1 row)

**Before:** `crates/agent-profile/src/cli.rs` contains `Current(Rest),`; `crates/agent-profile/tests/support/mod.rs` does not contain `assert_outside_any_repository`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/cli.rs`** (routing, commands, launch discovery and routing tests; the whole file, byte-exact)

```rust
//! CLI grammar: launch syntax, wrapper options and reserved command words (spec §5; SP1 design §4) and the
//! repository commands (SP3 design §7).

use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::adapter::{self, PlanContext, ProfilePresence};
use crate::config::{self, AppRoot, Config};
use crate::error::{Error, Result};
use crate::launch::{self, LaunchOutcome};
use crate::name::{AgentId, Platform, ProfileName, RESERVED_WORDS, is_reserved_word};
use crate::output::{self, ReportMode};
use crate::repo::{self, Discovery};
use crate::resolve;

const LAUNCH_USAGE: &str = "\
Usage: agent-profile <agent> <profile> [--dry-run] [--verbose] [-- <agent args>...]

Launch <agent> with <profile>. Everything after `--` is passed to the agent unchanged.

Options:
  --dry-run   Show what would be launched, without launching or creating anything
  --verbose   Print the launch report to stderr before launching
  -h, --help  Print this help
  -V, --version  Print the version
";

const COMMANDS_HELP: &str = "\
Repository commands:
  agent-profile <agent> current [--repo <path>]
  agent-profile <agent> resolve [--repo <path>]
  agent-profile [<agent>] status [--repo <path>]
  agent-profile [<agent>] link <profile> [--repo <path>]
  agent-profile [<agent>] unlink [--repo <path>]";

/// Select and launch profiles for multiple coding agents.
#[derive(Parser)]
#[command(
    name = "agent-profile",
    version,
    disable_help_subcommand = true,
    arg_required_else_help = true,
    override_usage = "agent-profile <agent> <profile> [--dry-run] [--verbose] [-- <agent args>...]\n       \
                      agent-profile <command>",
    after_help = COMMANDS_HELP
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// The reserved words of spec §5.3 that are not implemented yet. `status`, `link`, `unlink`, `resolve` and
/// `current` are dispatched before Clap (SP3 design §7.2).
#[derive(Subcommand)]
enum Command {
    /// Not yet implemented.
    Agents(Rest),
    /// Not yet implemented.
    Profiles(Rest),
    /// Not yet implemented.
    List(Rest),
    /// Not yet implemented.
    Create(Rest),
    /// Not yet implemented.
    Delete(Rest),
    /// Not yet implemented.
    Doctor(Rest),
    /// Not yet implemented.
    Repositories(Rest),
    /// Not yet implemented.
    Completions(Rest),
    #[command(external_subcommand)]
    Agent(Vec<OsString>),
}

#[derive(Args)]
#[command(disable_help_flag = true)]
struct Rest {
    #[arg(hide = true, trailing_var_arg = true, allow_hyphen_values = true)]
    _rest: Vec<OsString>,
}

impl Command {
    fn reserved_name(&self) -> Option<&'static str> {
        Some(match self {
            Command::Agents(_) => "agents",
            Command::Profiles(_) => "profiles",
            Command::List(_) => "list",
            Command::Create(_) => "create",
            Command::Delete(_) => "delete",
            Command::Doctor(_) => "doctor",
            Command::Repositories(_) => "repositories",
            Command::Completions(_) => "completions",
            Command::Agent(_) => return None,
        })
    }
}

/// The repository commands (SP3 design §7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandWord {
    Current,
    Resolve,
    Status,
    Link,
    Unlink,
}

impl CommandWord {
    /// The exact lower-case spelling only.
    fn parse(word: &str) -> Option<CommandWord> {
        Some(match word {
            "current" => CommandWord::Current,
            "resolve" => CommandWord::Resolve,
            "status" => CommandWord::Status,
            "link" => CommandWord::Link,
            "unlink" => CommandWord::Unlink,
            _ => return None,
        })
    }

    fn as_str(self) -> &'static str {
        match self {
            CommandWord::Current => "current",
            CommandWord::Resolve => "resolve",
            CommandWord::Status => "status",
            CommandWord::Link => "link",
            CommandWord::Unlink => "unlink",
        }
    }

    /// The command's help text (SP3 design §7.3).
    fn usage(self) -> &'static str {
        match self {
            CommandWord::Current => {
                "Usage: agent-profile <agent> current [--repo <path>]\n\
                 Print the profile agent-profile would select for <agent> here.\n\n  \
                 --repo <path>  Use the repository at <path> instead of the current directory\n"
            }
            CommandWord::Resolve => {
                "Usage: agent-profile <agent> resolve [--repo <path>]\n\
                 Show the profile, where it comes from, and the repository.\n\n  \
                 --repo <path>  Use the repository at <path> instead of the current directory\n"
            }
            CommandWord::Status => {
                "Usage: agent-profile [<agent>] status [--repo <path>]\n\
                 Show the repository, its mappings, the default profile and what each agent resolves to.\n\n  \
                 --repo <path>  Use the repository at <path> instead of the current directory\n"
            }
            CommandWord::Link => {
                "Usage: agent-profile [<agent>] link <profile> [--repo <path>]\n\
                 Map this repository (or only <agent> in it) to <profile>.\n\n  \
                 --repo <path>  Use the repository at <path> instead of the current directory\n"
            }
            CommandWord::Unlink => {
                "Usage: agent-profile [<agent>] unlink [--repo <path>]\n\
                 Remove this repository's mapping (or only <agent>'s). With --repo, removes the mapping stored \
                 for that path.\n\n  \
                 --repo <path>  Use the repository at <path> instead of the current directory\n"
            }
        }
    }
}

/// Runs the CLI and returns the process exit code. This is the only place an exit code is decided.
pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let args: Vec<OsString> = args.into_iter().collect();
    // Clap drops a leading `--` from a subcommand's trailing arguments, so the repository commands are
    // dispatched on the raw first argument (SP3 design §7.2).
    if let Some(command) = args.get(1).and_then(|word| word.to_str()).and_then(CommandWord::parse) {
        let rest = args[2..].to_vec();
        return match split_top_level(command, rest).and_then(execute) {
            Ok(code) => code,
            Err(error) => report(error),
        };
    }
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return error.exit_code();
        }
    };
    if let Some(name) = cli.command.reserved_name() {
        return report(Error::NotYetImplemented { command: name.to_owned() });
    }
    let Command::Agent(argv) = cli.command else {
        unreachable!("reserved commands returned above")
    };
    let known = adapter::known_agents();
    match split(argv, &known)
        .map_err(|error| with_unknown_configured(error, None, &known))
        .and_then(execute)
    {
        Ok(code) => code,
        Err(error) => report(error),
    }
}

fn report(error: Error) -> i32 {
    let _ = writeln!(io::stderr(), "agent-profile: error: {error}");
    error.exit_code()
}

/// The outcome of the argument checks (SP1 design §4.2, SP3 design §7.2).
#[derive(Debug, PartialEq, Eq)]
enum Invocation {
    Help,
    CommandHelp(CommandWord),
    Version,
    Launch {
        agent: String,
        profile: Option<String>,
        dry_run: bool,
        verbose: bool,
        opaque: Vec<OsString>,
    },
    Command {
        agent: Option<String>,
        command: CommandWord,
        profile: Option<String>,
        repo: Option<OsString>,
    },
}

/// One token before the first `--`, after `--repo` binding (SP3 design §7.2 step 1).
#[derive(Debug)]
enum Token {
    Repo(OsString),
    Option(OsString),
    Bare(OsString),
}

fn usage(message: impl Into<String>) -> Error {
    Error::Usage { message: message.into() }
}

/// Splits at the first `--`: the tokens before it, and the opaque arguments when there is one.
fn cut(args: Vec<OsString>) -> (Vec<OsString>, Option<Vec<OsString>>) {
    match args.iter().position(|arg| arg == "--") {
        Some(index) => (args[..index].to_vec(), Some(args[index + 1..].to_vec())),
        None => (args, None),
    }
}

/// Step 1: `--repo=<v>` carries its value; a bare `--repo` consumes the next token whatever it is.
fn bind(pre: Vec<OsString>) -> Result<Vec<Token>> {
    let mut tokens = Vec::with_capacity(pre.len());
    let mut pre = pre.into_iter();
    while let Some(arg) = pre.next() {
        if arg == "--repo" {
            let value = pre.next().ok_or_else(|| usage("`--repo` needs a path"))?;
            tokens.push(Token::Repo(value));
        } else if let Some(value) = repo_value(&arg) {
            tokens.push(Token::Repo(value));
        } else if arg.as_encoded_bytes().first() == Some(&b'-') {
            tokens.push(Token::Option(arg));
        } else {
            tokens.push(Token::Bare(arg));
        }
    }
    Ok(tokens)
}

const REPO_EQUALS: &str = "--repo=";

/// The value of a `--repo=<v>` token, which may be non-UTF-8.
#[cfg(unix)]
fn repo_value(arg: &OsStr) -> Option<OsString> {
    use std::os::unix::ffi::OsStrExt;
    let bytes = arg.as_bytes();
    bytes
        .starts_with(REPO_EQUALS.as_bytes())
        .then(|| OsStr::from_bytes(&bytes[REPO_EQUALS.len()..]).to_owned())
}

/// The value of a `--repo=<v>` token, which may be non-UTF-8.
#[cfg(windows)]
fn repo_value(arg: &OsStr) -> Option<OsString> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    let units: Vec<u16> = arg.encode_wide().collect();
    let prefix: Vec<u16> = OsStr::new(REPO_EQUALS).encode_wide().collect();
    units.starts_with(&prefix).then(|| OsString::from_wide(&units[prefix.len()..]))
}

fn options(tokens: &[Token]) -> impl Iterator<Item = &OsString> {
    tokens.iter().filter_map(|token| match token {
        Token::Option(option) => Some(option),
        _ => None,
    })
}

fn bare_words(tokens: &[Token]) -> Vec<&OsString> {
    tokens
        .iter()
        .filter_map(|token| match token {
            Token::Bare(word) => Some(word),
            _ => None,
        })
        .collect()
}

fn has_help(tokens: &[Token]) -> bool {
    options(tokens).any(|option| option == "-h" || option == "--help")
}

fn has_version(tokens: &[Token]) -> bool {
    options(tokens).any(|option| option == "-V" || option == "--version")
}

/// Only the part before `=` is echoed: the value may be a secret meant for the agent (spec §36).
fn option_name(option: &OsStr) -> String {
    let shown = option.to_string_lossy();
    shown.split_once('=').map_or(&*shown, |(name, _)| name).to_owned()
}

/// `agent-profile status|link|unlink|resolve|current …` (SP3 design §7.2).
fn split_top_level(command: CommandWord, rest: Vec<OsString>) -> Result<Invocation> {
    let (pre, opaque) = cut(rest);
    let tokens = bind(pre)?;
    if has_help(&tokens) {
        return Ok(Invocation::CommandHelp(command));
    }
    if has_version(&tokens) {
        return Ok(Invocation::Version);
    }
    if matches!(command, CommandWord::Resolve | CommandWord::Current) {
        let name = command.as_str();
        return Err(usage(format!("`{name}` needs an agent: agent-profile <agent> {name}")));
    }
    let bare = bare_words(&tokens);
    validate_command(None, command, &tokens, &bare, opaque.is_some())
}

/// `agent-profile <agent> …`: SP1 design §4.2 rules 3-4 as replaced by SP3 design §7.2.
fn split(argv: Vec<OsString>, known: &[&str]) -> Result<Invocation> {
    let mut argv = argv.into_iter();
    let agent_word = argv.next().unwrap_or_default();
    let (pre, opaque) = cut(argv.collect());
    let tokens = bind(pre)?;
    let bare = bare_words(&tokens);
    let first = bare.first().and_then(|word| word.to_str());

    // 2. Help and version.
    if has_help(&tokens) {
        return Ok(match first.and_then(CommandWord::parse) {
            Some(command) => Invocation::CommandHelp(command),
            None => Invocation::Help,
        });
    }
    if has_version(&tokens) {
        return Ok(Invocation::Version);
    }
    // 3. Unknown agent (exact, case-sensitive).
    let agent = match agent_word.to_str() {
        Some(agent) if known.contains(&agent) => agent.to_owned(),
        _ => {
            return Err(Error::UnknownAgent {
                agent: agent_word.to_string_lossy().into_owned(),
                known: known.iter().map(|agent| (*agent).to_owned()).collect(),
                unknown_configured: Vec::new(),
            });
        }
    };
    // 4. A reserved first bare word.
    if let Some(word) = first
        && is_reserved_word(word)
    {
        if let Some(command) = CommandWord::parse(word) {
            return validate_command(Some(agent), command, &tokens, &bare[1..], opaque.is_some());
        }
        if RESERVED_WORDS.contains(&word) {
            return Err(Error::NotYetImplemented { command: format!("{agent} {word}") });
        }
        return Err(usage(format!(
            "command words are lower case: `{}`",
            word.to_ascii_lowercase()
        )));
    }
    // 6. A launch. A `--repo` binding is an unknown option here.
    let launch_options: Vec<OsString> = tokens
        .iter()
        .filter_map(|token| match token {
            Token::Option(option) => Some(option.clone()),
            Token::Repo(_) => Some(OsString::from("--repo")),
            Token::Bare(_) => None,
        })
        .collect();
    if let Some(bad) = launch_options
        .iter()
        .find(|arg| !matches!(arg.to_str(), Some("--dry-run" | "--verbose" | "--json")))
    {
        return Err(usage(format!(
            "unknown option {:?}; agent arguments must follow `--`",
            option_name(bad)
        )));
    }
    if bare.len() > 1 {
        return Err(usage("agent arguments must follow `--`"));
    }
    if launch_options.iter().any(|arg| arg == "--json") {
        return Err(Error::NotYetImplemented { command: "--json".to_owned() });
    }
    let profile = match bare.first() {
        Some(word) => Some(
            word.to_str().ok_or_else(|| usage("the profile name is not valid UTF-8"))?.to_owned(),
        ),
        None => None,
    };
    Ok(Invocation::Launch {
        agent,
        profile,
        dry_run: launch_options.iter().any(|arg| arg == "--dry-run"),
        verbose: launch_options.iter().any(|arg| arg == "--verbose"),
        opaque: opaque.unwrap_or_default(),
    })
}

/// Step 5: `bare` holds the bare words after the command word.
fn validate_command(
    agent: Option<String>,
    command: CommandWord,
    tokens: &[Token],
    bare: &[&OsString],
    has_cut: bool,
) -> Result<Invocation> {
    let name = command.as_str();
    if has_cut {
        return Err(usage(format!("`{name}` takes no agent arguments; remove `--`")));
    }
    let repos: Vec<&OsString> = tokens
        .iter()
        .filter_map(|token| match token {
            Token::Repo(repo) => Some(repo),
            _ => None,
        })
        .collect();
    if repos.len() > 1 {
        return Err(usage("`--repo` may be given only once"));
    }
    if repos.first().is_some_and(|repo| repo.is_empty()) {
        return Err(usage("`--repo` needs a non-empty path"));
    }
    if matches!(command, CommandWord::Status | CommandWord::Resolve)
        && options(tokens).any(|option| option == "--json")
    {
        return Err(Error::NotYetImplemented { command: "--json".to_owned() });
    }
    if let Some(option) = options(tokens).next() {
        return Err(usage(format!("unknown option {:?} for `{name}`", option_name(option))));
    }
    let allowed = usize::from(command == CommandWord::Link);
    if command == CommandWord::Link && bare.is_empty() {
        return Err(usage("`link` needs a profile: agent-profile [<agent>] link <profile>"));
    }
    if bare.len() > allowed {
        return Err(usage(match command {
            CommandWord::Link => "`link` takes one profile".to_owned(),
            _ => format!("`{name}` takes no arguments"),
        }));
    }
    let profile = match bare.first() {
        Some(word) => Some(
            word.to_str().ok_or_else(|| usage("the profile name is not valid UTF-8"))?.to_owned(),
        ),
        None => None,
    };
    Ok(Invocation::Command {
        agent,
        command,
        profile,
        repo: repos.first().map(|repo| (*repo).clone()),
    })
}

fn execute(invocation: Invocation) -> Result<i32> {
    match invocation {
        Invocation::Help => {
            write_out(LAUNCH_USAGE)?;
            Ok(0)
        }
        Invocation::CommandHelp(command) => {
            write_out(command.usage())?;
            Ok(0)
        }
        Invocation::Version => {
            write_out(&format!("agent-profile {}\n", env!("CARGO_PKG_VERSION")))?;
            Ok(0)
        }
        Invocation::Launch { agent, profile, dry_run, verbose, opaque } => {
            run_launch(agent, profile, dry_run, verbose, opaque)
        }
        Invocation::Command { agent, command, profile, repo } => {
            run_command(agent, command, profile, repo)
        }
    }
}

/// Whether a launch runs discovery (SP3 design §7.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaunchDiscovery {
    /// No profile word: discovery selects the profile, and its errors exit 4.
    Required,
    /// An explicit profile with a report: discovery only fills the report, and its errors are ignored.
    ReportOnly,
    /// An explicit profile without a report: nothing is discovered.
    Skipped,
}

fn launch_discovery(explicit: bool, dry_run: bool, verbose: bool) -> LaunchDiscovery {
    match (explicit, dry_run || verbose) {
        (false, _) => LaunchDiscovery::Required,
        (true, true) => LaunchDiscovery::ReportOnly,
        (true, false) => LaunchDiscovery::Skipped,
    }
}

fn parse_profile(name: String) -> Result<ProfileName> {
    ProfileName::parse(&name, Platform::host())
        .map_err(|reason| Error::InvalidProfileName { name, reason })
}

fn current_dir() -> Result<PathBuf> {
    std::env::current_dir().map_err(|error| Error::Repository {
        path: PathBuf::from("."),
        reason: format!("cannot resolve the directory: {error}"),
    })
}

fn no_profile(agent: &AgentId, root: &AppRoot, discovery: &Discovery) -> Error {
    Error::NoProfile {
        agent: agent.to_string(),
        config_file: root.config_path(),
        in_repository: matches!(discovery, Discovery::Repository(_)),
    }
}

fn run_launch(
    agent: String,
    profile: Option<String>,
    dry_run: bool,
    verbose: bool,
    opaque: Vec<OsString>,
) -> Result<i32> {
    // SP1 design §5.1 steps 2-4, SP3 design §7.6.
    let known = adapter::known_agents();
    let profile = profile.map(parse_profile).transpose()?;
    let root = AppRoot::resolve()?;
    let config = Config::load(&root)?;
    let agent = AgentId::parse(&agent).expect("known agents are valid agent ids");
    let discovery = match launch_discovery(profile.is_some(), dry_run, verbose) {
        LaunchDiscovery::Required => repo::discover(&current_dir()?)?,
        LaunchDiscovery::ReportOnly => std::env::current_dir()
            .ok()
            .and_then(|cwd| repo::discover(&cwd).ok())
            .unwrap_or(Discovery::NotInRepository),
        LaunchDiscovery::Skipped => Discovery::NotInRepository,
    };
    let resolution = resolve::resolve(agent.clone(), profile, &config, &discovery);
    let Some(profile) = resolution.profile.as_ref() else {
        return Err(no_profile(&agent, &root, &discovery));
    };

    // SP2 design §4.3 step 1: parsing already refused unknown agents.
    let adapter = adapter::lookup(agent.as_str()).expect("known agents have an adapter");
    // Step 2: a pure conflict scan, before executable discovery.
    adapter::check_conflicts(adapter.metadata(), &opaque)?;
    // Step 3: discovery, the case-only-twin check, the plan.
    let path_var = std::env::var_os("PATH");
    let ctx = PlanContext {
        profile,
        root: &root,
        config: &config,
        args: &opaque,
        path_var: path_var.as_deref(),
    };
    let planned = adapter
        .plan(&ctx)
        .map_err(|error| with_unknown_configured(error, Some(&config), &known))?;

    // Step 4.
    if dry_run {
        let lines = output::report_lines(&planned, &resolution, ReportMode::DryRun);
        write_out(&lines.iter().map(|line| format!("{line}\n")).collect::<String>())?;
        return Ok(0);
    }

    // Step 5. The verbose report is rendered after initialization, so it never says "(would be created)".
    adapter.initialize(&planned)?;
    if verbose {
        let mut stderr = io::stderr();
        for line in &output::report_lines(&planned, &resolution, ReportMode::Verbose) {
            let _ = writeln!(stderr, "agent-profile: {line}");
        }
        let _ = stderr.flush();
    }
    match launch::launch(&planned.plan, verbose)? {
        LaunchOutcome::Exited(code) => Ok(code),
        LaunchOutcome::ReplacedProcess => Ok(0),
    }
}

/// SP3 design §7.4 steps 2-5.
fn run_command(
    agent: Option<String>,
    command: CommandWord,
    profile: Option<String>,
    repo: Option<OsString>,
) -> Result<i32> {
    let profile = profile.map(parse_profile).transpose()?;
    let root = AppRoot::resolve()?;
    let config = Config::load(&root)?;
    let agent =
        agent.map(|agent| AgentId::parse(&agent).expect("known agents are valid agent ids"));
    let cwd = current_dir()?;
    if command == CommandWord::Unlink {
        return run_unlink(&root, &config, agent.as_ref(), &cwd, repo.as_deref());
    }
    let start = match &repo {
        Some(repo) => cwd.join(repo),
        None => cwd,
    };
    let discovery = repo::discover(&start)?;
    let lines = match command {
        CommandWord::Current => {
            let agent = agent.expect("`current` has an agent");
            let resolution = resolve::resolve(agent.clone(), None, &config, &discovery);
            let Some(profile) = resolution.profile else {
                return Err(no_profile(&agent, &root, &discovery));
            };
            vec![profile.to_string()]
        }
        CommandWord::Resolve => {
            let agent = agent.expect("`resolve` has an agent");
            let resolution = resolve::resolve(agent.clone(), None, &config, &discovery);
            write_lines(&output::resolve_lines(&resolution))?;
            if resolution.profile.is_none() {
                return Err(no_profile(&agent, &root, &discovery));
            }
            return Ok(0);
        }
        CommandWord::Status => {
            let agents: Vec<AgentId> = match &agent {
                Some(agent) => vec![agent.clone()],
                None => adapter::known_agents()
                    .into_iter()
                    .map(|id| AgentId::parse(id).expect("known agents are valid agent ids"))
                    .collect(),
            };
            let rows: Vec<_> = agents
                .into_iter()
                .map(|id| {
                    let resolution = resolve::resolve(id, None, &config, &discovery);
                    let presence = match (&agent, &resolution.profile) {
                        (Some(agent), Some(profile)) => Some(presence(&root, agent, profile)),
                        _ => None,
                    };
                    (resolution, presence)
                })
                .collect();
            output::status_lines(&discovery, &config, &rows)
        }
        CommandWord::Link => {
            let Discovery::Repository(repository) = discovery else {
                return Err(Error::Repository {
                    path: repo::canonical(&start).unwrap_or(start),
                    reason: "not inside a Git repository".to_owned(),
                });
            };
            let profile = profile.expect("`link` has a profile");
            let outcome = config::link(&root, &repository, agent.as_ref(), &profile)?;
            let mut lines =
                vec![output::link_line(&outcome, agent.as_ref(), &repository, &profile)];
            if let Some(agent) = &agent {
                let adapter =
                    adapter::lookup(agent.as_str()).expect("known agents have an adapter");
                if adapter.presence(&root, &profile) == ProfilePresence::Absent {
                    lines.push(output::note(&format!(
                        "profile {profile} has not been launched with {agent} yet"
                    )));
                }
            }
            lines
        }
        CommandWord::Unlink => unreachable!("unlink returned above"),
    };
    write_lines(&lines)?;
    Ok(0)
}

/// `unlink` chooses its keys without discovery when `--repo` is given (SP3 design §6.2).
fn run_unlink(
    root: &AppRoot,
    config: &Config,
    agent: Option<&AgentId>,
    cwd: &std::path::Path,
    repo: Option<&OsStr>,
) -> Result<i32> {
    let keys = match repo {
        Some(repo) => repo::unlink_keys(cwd, repo),
        None => match repo::discover(cwd)? {
            Discovery::Repository(repository) => vec![repository],
            Discovery::NotInRepository => {
                return Err(Error::Repository {
                    path: repo::canonical(cwd).unwrap_or_else(|_| cwd.to_path_buf()),
                    reason: "not inside a Git repository".to_owned(),
                });
            }
        },
    };
    let outcome = config::unlink(root, &keys, agent)?;
    write_lines(&output::unlink_lines(&outcome, agent, config, &keys))?;
    Ok(0)
}

/// The `presence:` value of `<agent> status` (SP3 design §7.5).
fn presence(root: &AppRoot, agent: &AgentId, profile: &ProfileName) -> String {
    if let Err(Error::ProfileCaseConflict { existing, .. }) =
        config::check_case_twins(root, profile)
    {
        return format!("conflicts with {existing}");
    }
    let adapter = adapter::lookup(agent.as_str()).expect("known agents have an adapter");
    match adapter.presence(root, profile) {
        ProfilePresence::Materialized => "materialized",
        ProfilePresence::Known => "known",
        ProfilePresence::Absent => "absent",
    }
    .to_owned()
}

fn write_lines(lines: &[String]) -> Result<()> {
    write_out(&lines.iter().map(|line| format!("{line}\n")).collect::<String>())
}

fn write_out(text: &str) -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .write_all(text.as_bytes())
        .and_then(|()| stdout.flush())
        .map_err(|source| Error::Io { context: "could not write to stdout".to_owned(), source })
}

/// Fills `unknown_configured` on `UnknownAgent` and `AgentNotInstalled`. With no configuration at hand,
/// one best-effort load is attempted; its failure leaves the list empty (design §4.2 check 3).
fn with_unknown_configured(error: Error, config: Option<&Config>, known: &[&str]) -> Error {
    let unknown = |config: Option<&Config>| -> Vec<String> {
        let loaded;
        let config = match config {
            Some(config) => Some(config),
            None => {
                loaded = AppRoot::resolve().ok().and_then(|root| Config::load(&root).ok());
                loaded.as_ref()
            }
        };
        config
            .map(|config| {
                config
                    .configured_agents()
                    .filter(|id| !known.contains(id))
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    };
    match error {
        Error::UnknownAgent { agent, known: names, .. } => {
            Error::UnknownAgent { agent, known: names, unknown_configured: unknown(config) }
        }
        Error::AgentNotInstalled { agent, reason, .. } => {
            Error::AgentNotInstalled { agent, reason, unknown_configured: unknown(config) }
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWN: &[&str] = &["fake", "claude"];

    fn args(items: &[&str]) -> Vec<OsString> {
        items.iter().map(OsString::from).collect()
    }

    fn launch(profile: Option<&str>, dry_run: bool, verbose: bool, opaque: &[&str]) -> Invocation {
        Invocation::Launch {
            agent: "fake".to_owned(),
            profile: profile.map(str::to_owned),
            dry_run,
            verbose,
            opaque: args(opaque),
        }
    }

    fn command(
        agent: Option<&str>,
        command: CommandWord,
        profile: Option<&str>,
        repo: Option<&str>,
    ) -> Invocation {
        Invocation::Command {
            agent: agent.map(str::to_owned),
            command,
            profile: profile.map(str::to_owned),
            repo: repo.map(OsString::from),
        }
    }

    fn split_ok(items: &[&str]) -> Invocation {
        split(args(items), KNOWN).unwrap()
    }

    fn split_err(items: &[&str]) -> Error {
        split(args(items), KNOWN).unwrap_err()
    }

    /// Runs the top-level dispatch the way `run` does: `items[0]` is the command word.
    fn top(items: &[&str]) -> Result<Invocation> {
        let command = CommandWord::parse(items[0]).expect("a top-level command word");
        split_top_level(command, args(&items[1..]))
    }

    fn usage_message(result: Result<Invocation>) -> String {
        match result {
            Err(Error::Usage { message }) => message,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn cut_is_at_the_first_double_dash_and_later_ones_are_opaque() {
        assert_eq!(
            split_ok(&["fake", "work", "--", "--dry-run", "--", "a b"]),
            launch(Some("work"), false, false, &["--dry-run", "--", "a b"])
        );
        assert_eq!(split_ok(&["fake", "--", "work"]), launch(None, false, false, &["work"]));
        assert_eq!(
            split_ok(&["fake", "work", "--", "--help"]),
            launch(Some("work"), false, false, &["--help"])
        );
    }

    #[test]
    fn options_may_appear_anywhere_before_the_cut() {
        assert_eq!(
            split_ok(&["fake", "--dry-run", "work", "--verbose"]),
            launch(Some("work"), true, true, &[])
        );
        assert_eq!(split_ok(&["fake"]), launch(None, false, false, &[]));
    }

    #[test]
    fn help_and_version_win_first() {
        assert_eq!(split_ok(&["zzz", "--help"]), Invocation::Help);
        assert_eq!(split_ok(&["fake", "create", "-h"]), Invocation::Help);
        assert_eq!(split_ok(&["fake", "-h", "--", "x"]), Invocation::Help);
        assert_eq!(split_ok(&["zzz", "--version"]), Invocation::Version);
        assert_eq!(split_ok(&["fake", "work", "-V"]), Invocation::Version);
    }

    #[test]
    fn unknown_agent_is_exact_and_precedes_reserved_words() {
        for items in [&["zzz", "work"][..], &["Fake", "work"], &["zzz", "create"], &["zzz", "link"]]
        {
            assert!(matches!(split_err(items), Error::UnknownAgent { .. }), "{items:?}");
        }
    }

    #[test]
    fn reserved_first_bare_word_routes_by_exact_spelling() {
        match split_err(&["fake", "create", "work"]) {
            Error::NotYetImplemented { command } => assert_eq!(command, "fake create"),
            other => panic!("{other:?}"),
        }
        for (items, word) in [
            (&["fake", "CREATE", "--bogus"][..], "create"),
            (&["fake", "Create", "x"], "create"),
            (&["claude", "LINK"], "link"),
        ] {
            assert_eq!(
                usage_message(split(args(items), KNOWN)),
                format!("command words are lower case: `{word}`"),
                "{items:?}"
            );
        }
        assert_eq!(
            usage_message(split(args(&["fake", "--bogus", "link"]), KNOWN)),
            "unknown option \"--bogus\" for `link`"
        );
    }

    #[test]
    fn agent_scoped_commands_bind_their_words_and_repo() {
        assert_eq!(
            split_ok(&["claude", "current"]),
            command(Some("claude"), CommandWord::Current, None, None)
        );
        assert_eq!(
            split_ok(&["claude", "resolve", "--repo", "../x"]),
            command(Some("claude"), CommandWord::Resolve, None, Some("../x"))
        );
        assert_eq!(
            split_ok(&["claude", "--repo=a=b", "status"]),
            command(Some("claude"), CommandWord::Status, None, Some("a=b"))
        );
        assert_eq!(
            split_ok(&["claude", "link", "work", "--repo", "-dir"]),
            command(Some("claude"), CommandWord::Link, Some("work"), Some("-dir"))
        );
        assert_eq!(
            split_ok(&["claude", "unlink", "--repo", "link"]),
            command(Some("claude"), CommandWord::Unlink, None, Some("link"))
        );
    }

    #[test]
    fn top_level_commands_take_every_raw_token() {
        assert_eq!(top(&["status"]).unwrap(), command(None, CommandWord::Status, None, None));
        assert_eq!(
            top(&["link", "work", "--repo", "r"]).unwrap(),
            command(None, CommandWord::Link, Some("work"), Some("r"))
        );
        assert_eq!(
            top(&["link", "status"]).unwrap(),
            command(None, CommandWord::Link, Some("status"), None)
        );
        assert_eq!(
            top(&["link", "Create"]).unwrap(),
            command(None, CommandWord::Link, Some("Create"), None)
        );
        assert_eq!(
            top(&["unlink", "--repo=/gone"]).unwrap(),
            command(None, CommandWord::Unlink, None, Some("/gone"))
        );
    }

    #[test]
    fn command_help_names_the_command_word() {
        assert_eq!(top(&["link", "-h"]).unwrap(), Invocation::CommandHelp(CommandWord::Link));
        assert_eq!(
            top(&["link", "status", "-h"]).unwrap(),
            Invocation::CommandHelp(CommandWord::Link)
        );
        assert_eq!(top(&["resolve", "-h"]).unwrap(), Invocation::CommandHelp(CommandWord::Resolve));
        assert_eq!(top(&["status", "-V"]).unwrap(), Invocation::Version);
        assert_eq!(
            split_ok(&["claude", "status", "--help"]),
            Invocation::CommandHelp(CommandWord::Status)
        );
        assert_eq!(split_ok(&["claude", "Status", "--help"]), Invocation::Help);
    }

    #[test]
    fn a_consumed_repo_value_is_never_help_an_option_or_a_command_word() {
        assert_eq!(
            split_ok(&["claude", "link", "work", "--repo", "-h"]),
            command(Some("claude"), CommandWord::Link, Some("work"), Some("-h"))
        );
        assert_eq!(
            usage_message(split(args(&["claude", "--repo"]), KNOWN)),
            "`--repo` needs a path"
        );
        assert_eq!(
            usage_message(split(args(&["fake", "-h", "--repo"]), KNOWN)),
            "`--repo` needs a path"
        );
        assert_eq!(
            usage_message(split(args(&["fake", "work", "--repo", "--help"]), KNOWN)),
            "unknown option \"--repo\"; agent arguments must follow `--`"
        );
        assert_eq!(
            usage_message(split(args(&["fake", "--repo", "create"]), KNOWN)),
            "unknown option \"--repo\"; agent arguments must follow `--`"
        );
    }

    #[test]
    fn command_usage_errors_in_order() {
        for (items, message) in [
            (&["status", "--"][..], "`status` takes no agent arguments; remove `--`"),
            (&["unlink", "--", "--repo", "x"], "`unlink` takes no agent arguments; remove `--`"),
            (&["link", "work", "--repo", "a", "--repo=b"], "`--repo` may be given only once"),
            (&["link", "work", "--repo="], "`--repo` needs a non-empty path"),
            (&["link", "work", "--repo", ""], "`--repo` needs a non-empty path"),
            (&["link", "work", "--json"], "unknown option \"--json\" for `link`"),
            (&["unlink", "--dry-run"], "unknown option \"--dry-run\" for `unlink`"),
            (&["status", "--verbose=yes"], "unknown option \"--verbose\" for `status`"),
            (&["link"], "`link` needs a profile: agent-profile [<agent>] link <profile>"),
            (
                &["link", "--repo", "r"],
                "`link` needs a profile: agent-profile [<agent>] link <profile>",
            ),
            (&["link", "work", "extra"], "`link` takes one profile"),
            (&["status", "extra"], "`status` takes no arguments"),
            (&["unlink", "extra"], "`unlink` takes no arguments"),
            (&["resolve"], "`resolve` needs an agent: agent-profile <agent> resolve"),
            (
                &["current", "--repo", "x"],
                "`current` needs an agent: agent-profile <agent> current",
            ),
        ] {
            assert_eq!(usage_message(top(items)), message, "{items:?}");
        }
        for (items, message) in [
            (&["claude", "current", "extra"][..], "`current` takes no arguments"),
            (&["claude", "resolve", "--json", "--bogus"], "`--json` is not yet implemented"),
            (&["claude", "current", "--json"], "unknown option \"--json\" for `current`"),
            (&["claude", "link", "a", "--", "x"], "`link` takes no agent arguments; remove `--`"),
        ] {
            let result = split(args(items), KNOWN);
            let text = match result {
                Err(error @ (Error::Usage { .. } | Error::NotYetImplemented { .. })) => {
                    error.to_string()
                }
                other => panic!("{items:?}: {other:?}"),
            };
            assert_eq!(text, message, "{items:?}");
        }
        assert!(matches!(top(&["status", "--json"]), Err(Error::NotYetImplemented { .. })));
    }

    #[test]
    fn launch_discovery_runs_only_when_it_can_matter() {
        assert_eq!(launch_discovery(false, false, false), LaunchDiscovery::Required);
        assert_eq!(launch_discovery(false, true, true), LaunchDiscovery::Required);
        assert_eq!(launch_discovery(true, true, false), LaunchDiscovery::ReportOnly);
        assert_eq!(launch_discovery(true, false, true), LaunchDiscovery::ReportOnly);
        assert_eq!(launch_discovery(true, false, false), LaunchDiscovery::Skipped);
    }

    #[test]
    fn usage_errors() {
        for items in [
            &["fake", "work", "--bogus"][..],
            &["fake", "-"],
            &["fake", "--dry-run=yes", "work"],
            &["fake", "work", "extra"],
            &["fake", "", "extra"],
            &["fake", "work", "--json", "extra"],
        ] {
            assert!(matches!(split_err(items), Error::Usage { .. }), "{items:?}");
        }
    }

    #[test]
    fn unknown_option_is_reported_before_extra_bare_words() {
        match split_err(&["fake", "work", "extra", "--bogus"]) {
            Error::Usage { message } => {
                assert!(message.starts_with("unknown option \"--bogus\""), "{message}")
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn unknown_option_error_never_echoes_its_value() {
        match split_err(&["fake", "work", "--openai-api-key=sk-secret=="]) {
            Error::Usage { message } => {
                assert!(message.starts_with("unknown option \"--openai-api-key\";"), "{message}");
                assert!(!message.contains("sk-secret"), "{message}");
            }
            other => panic!("{other:?}"),
        }
        let message = usage_message(top(&["status", "--api-key=sk-secret"]));
        assert_eq!(message, "unknown option \"--api-key\" for `status`");
    }

    #[test]
    fn json_is_not_yet_implemented() {
        for items in [&["fake", "work", "--json"][..], &["fake", "--json"]] {
            match split_err(items) {
                Error::NotYetImplemented { command } => assert_eq!(command, "--json"),
                other => panic!("{items:?}: {other:?}"),
            }
        }
    }

    #[test]
    fn empty_string_is_a_profile_word() {
        assert_eq!(split_ok(&["fake", ""]), launch(Some(""), false, false, &[]));
    }

    #[cfg(unix)]
    fn non_utf8() -> OsString {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(vec![0x66, 0xff])
    }

    #[cfg(windows)]
    fn non_utf8() -> OsString {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(&[0x66, 0xD800])
    }

    #[test]
    fn non_utf8_words() {
        let bare = vec![OsString::from("fake"), non_utf8()];
        assert!(matches!(split(bare, KNOWN), Err(Error::Usage { .. })));
        let agent = vec![non_utf8(), OsString::from("work")];
        assert!(matches!(split(agent, KNOWN), Err(Error::UnknownAgent { .. })));
        let opaque = vec![OsString::from("fake"), "work".into(), "--".into(), non_utf8()];
        match split(opaque, KNOWN).unwrap() {
            Invocation::Launch { opaque, .. } => assert_eq!(opaque, vec![non_utf8()]),
            other => panic!("{other:?}"),
        }
        let link = vec![OsString::from("fake"), "link".into(), non_utf8()];
        assert_eq!(usage_message(split(link, KNOWN)), "the profile name is not valid UTF-8");
        let mut equals = OsString::from("--repo=");
        equals.push(non_utf8());
        for repo in [vec![OsString::from("--repo"), non_utf8()], vec![equals]] {
            let mut items = vec![OsString::from("fake"), "status".into()];
            items.extend(repo);
            assert_eq!(
                split(items, KNOWN).unwrap(),
                Invocation::Command {
                    agent: Some("fake".to_owned()),
                    command: CommandWord::Status,
                    profile: None,
                    repo: Some(non_utf8()),
                }
            );
        }
    }
}
```

- [ ] **Step 2: Write `crates/agent-profile/tests/support/mod.rs`** (the repository guard and the default working directory; the whole file, byte-exact)

```rust
//! Shared helpers for integration tests (SP0 design §3.4, SP1 design §8).
//!
//! Each test file includes this module with `mod support;` and uses only part of it.
#![allow(dead_code)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every `fake-agent` control variable (SP1 design §8.1).
pub const FIXTURE_VARS: [&str; 8] = [
    "FAKE_AGENT_EXIT",
    "FAKE_AGENT_ECHO_ENV",
    "FAKE_AGENT_STDIN",
    "FAKE_AGENT_STDERR",
    "FAKE_AGENT_SLEEP_MS",
    "FAKE_AGENT_SPAWN_SLEEPER",
    "FAKE_AGENT_CTRL_C_EXIT",
    "FAKE_AGENT_BREAKAWAY",
];

/// Wrapper variables a developer's shell could leak into a test run.
pub const WRAPPER_VARS: [&str; 3] =
    ["AGENT_PROFILE_HOME", "FAKE_AGENT_HOME", "AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS"];

/// A `Command` for the `fake-agent` fixture with a clean fixture environment.
///
/// The fixture is a binary of this package, so cargo builds it before any test runs and
/// `CARGO_BIN_EXE_fake-agent` points at it; tests never invoke cargo themselves.
///
/// `Command` inherits the parent's environment, so a control variable exported in the shell running
/// the tests would silently change every fixture run. Tests set the variables they mean to test on top
/// of this baseline.
pub fn fake_agent() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fake-agent"));
    for name in FIXTURE_VARS {
        command.env_remove(name);
    }
    command
}

/// A temporary application root whose `config.toml` points `agents.fake.executable` at the fixture.
pub struct Root {
    pub dir: tempfile::TempDir,
}

impl Root {
    pub fn new() -> Root {
        let root = Root::empty();
        root.write_config(&format!(
            "[agents.fake]\nexecutable = {:?}\n",
            env!("CARGO_BIN_EXE_fake-agent")
        ));
        root
    }

    pub fn empty() -> Root {
        let dir = tempfile::tempdir().unwrap();
        assert_outside_any_repository(dir.path());
        Root { dir }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn write_config(&self, text: &str) {
        std::fs::write(self.path().join("config.toml"), text).unwrap();
    }

    pub fn profile_dir(&self, profile: &str) -> PathBuf {
        self.path().join("profiles").join(profile).join("fake")
    }

    /// `agent-profile` with this root, a clean fixture and wrapper environment, and the given args. The working
    /// directory is the root itself, which is outside any repository, so no test discovers this checkout.
    pub fn agent_profile<I, S>(&self, args: I) -> Command
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let mut command = Command::new(env!("CARGO_BIN_EXE_agent-profile"));
        for name in FIXTURE_VARS.iter().chain(WRAPPER_VARS.iter()) {
            command.env_remove(name);
        }
        command.env("AGENT_PROFILE_HOME", self.path());
        command.current_dir(self.path());
        command.args(args.into_iter().map(Into::into));
        command
    }
}

/// Fails unless no ancestor of `path` has a `.git` entry, so discovery from `path` finds no repository (SP3
/// design §8.5). It reads `.git` entries directly rather than trusting the discovery under test.
pub fn assert_outside_any_repository(path: &Path) {
    for ancestor in path.ancestors() {
        assert!(
            std::fs::symlink_metadata(ancestor.join(".git")).is_err(),
            "{} has a .git entry, so a test directory below it would be inside a repository",
            ancestor.display()
        );
    }
}

/// An argument that is a valid OS string but not valid UTF-8.
#[cfg(unix)]
pub fn non_utf8() -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(vec![0x66, 0xff])
}

/// An argument that is a valid OS string but not valid UTF-8: a lone surrogate.
#[cfg(windows)]
pub fn non_utf8() -> OsString {
    use std::os::windows::ffi::OsStringExt;
    OsString::from_wide(&[0x0066, 0xD800])
}

/// Parses the fixture's one-line JSON report.
pub fn report(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).unwrap_or_else(|e| {
        panic!("not a fake-agent report ({e}): {}", String::from_utf8_lossy(stdout))
    })
}
```

- [ ] **Step 3: Edit `crates/agent-profile/tests/launch.rs`** (design §7.2 changes this SP1 row). Replace exactly this text, which occurs once:

```rust
        (&["link", "work", "extra"], 2, "`link` is not yet implemented"),
```

with:

```rust
        (&["link", "work", "extra"], 2, "`link` takes one profile"),
```

- [ ] **Step 4: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib cli:: && cargo nextest run -p agent-profile --test launch
```

Expected: every test passes, including `command_usage_errors_in_order`, `a_consumed_repo_value_is_never_help_an_option_or_a_command_word`, `launch_discovery_runs_only_when_it_can_matter`, `behaviour_table_rows_with_non_zero_exits`.

- [ ] **Step 5: Run the gate** (see "Gate commands"). Expected on Windows: `205 tests run: 205 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 6: Commit**

```bash
git add crates/agent-profile/src/cli.rs crates/agent-profile/tests/support/mod.rs crates/agent-profile/tests/launch.rs
git commit -m "feat: link, unlink, current, resolve and status commands

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 7: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/cli.rs` replace exactly:

```rust
        (true, false) => LaunchDiscovery::Skipped,
```

with:

```rust
        (true, false) => LaunchDiscovery::ReportOnly,
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib cli::`. Expected: FAIL, and the failing test list includes `launch_discovery_runs_only_when_it_can_matter`. Then restore with `git checkout -- crates/agent-profile/src/cli.rs` and confirm `git status --short` prints nothing.


### Task 7: Resolution end to end

The design §8.4 scenarios through the binary with synthetic repositories.

**Files:**
- Create: `crates/agent-profile/tests/resolution.rs` (the end-to-end resolution suite)

**Before:** `crates/agent-profile/tests/resolution.rs` does not exist. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/tests/resolution.rs`** (the end-to-end resolution suite; the whole file, byte-exact)

```rust
//! Repository resolution end to end through the `agent-profile` binary (SP3 design §8.4; spec §34
//! "Resolution"). Repositories are synthetic `.git` layouts in guarded temp directories, so no test needs `git`
//! and none discovers this checkout.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use agent_profile::repo;
use support::Root;

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A guarded temp directory and its canonical path.
fn scratch() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    support::assert_outside_any_repository(dir.path());
    let path = repo::canonical(dir.path()).unwrap();
    (dir, path)
}

/// Makes `path` a repository root: a `.git` directory holding `HEAD`.
fn git_dir(path: &Path) {
    fs::create_dir_all(path.join(".git")).unwrap();
    fs::write(path.join(".git").join("HEAD"), "ref: refs/heads/main\n").unwrap();
}

fn key(path: &Path) -> String {
    toml_edit::Key::new(path.to_str().unwrap()).display_repr().into_owned()
}

/// `config.toml` pointing `fake` at the fixture, with an optional default and mapping tables.
fn configure(root: &Root, default: Option<&str>, tables: &str) {
    let default = default.map(|name| format!("default_profile = {name:?}\n")).unwrap_or_default();
    root.write_config(&format!(
        "{default}[agents.fake]\nexecutable = {:?}\n{tables}",
        env!("CARGO_BIN_EXE_fake-agent")
    ));
}

fn mapping(repository: &Path, body: &str) -> String {
    format!("\n[repositories.{}]\n{body}\n", key(repository))
}

fn run(root: &Root, cwd: &Path, args: &[&str]) -> Output {
    root.agent_profile(args).current_dir(cwd).output().unwrap()
}

/// Launches `fake` with no profile word from `cwd` and returns the profile directory it received.
fn launched_home(root: &Root, cwd: &Path) -> String {
    let output = root
        .agent_profile(["fake"])
        .current_dir(cwd)
        .env("FAKE_AGENT_ECHO_ENV", "FAKE_AGENT_HOME")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    support::report(&output.stdout)["env"]["FAKE_AGENT_HOME"].as_str().unwrap().to_owned()
}

fn home(root: &Root, profile: &str) -> String {
    root.profile_dir(profile).to_str().unwrap().to_owned()
}

#[test]
fn a_resolved_launch_uses_each_source_in_precedence_order() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let repository = base.join("acme");
    git_dir(&repository);
    let deep = repository.join("src").join("deep");
    fs::create_dir_all(&deep).unwrap();

    configure(
        &root,
        Some("global"),
        &mapping(&repository, "profile = \"repo\"\nagents = { fake = \"agent\" }"),
    );
    assert_eq!(launched_home(&root, &deep), home(&root, "agent"));
    configure(&root, Some("global"), &mapping(&repository, "profile = \"repo\""));
    assert_eq!(launched_home(&root, &deep), home(&root, "repo"));
    configure(&root, Some("global"), "");
    assert_eq!(launched_home(&root, &deep), home(&root, "global"));
    assert_eq!(launched_home(&root, &base), home(&root, "global"));

    configure(&root, None, "");
    let output = run(&root, &deep, &["fake"]);
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("link this repository (agent-profile link <profile>)"),
        "{}",
        stderr(&output)
    );
    let output = run(&root, &base, &["fake"]);
    assert_eq!(output.status.code(), Some(4));
    assert!(!stderr(&output).contains("link this repository"), "{}", stderr(&output));
    assert!(
        stderr(&output).contains(&format!(
            "or set default_profile in {}",
            root.path().join("config.toml").display()
        )),
        "{}",
        stderr(&output)
    );
}

#[test]
fn current_resolve_and_status_agree_for_every_source() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let repository = base.join("acme");
    git_dir(&repository);
    let explicit_launch = || {
        let output = run(&root, &repository, &["fake", "explicit", "--dry-run"]);
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
        assert!(
            stdout(&output).contains("profile:      explicit (explicit)"),
            "{}",
            stdout(&output)
        );
        assert!(
            stdout(&output).contains(&format!("repository:   {}", repository.display())),
            "{}",
            stdout(&output)
        );
    };
    configure(&root, None, "");
    explicit_launch();

    for (default, tables, profile, source) in [
        (
            Some("global"),
            mapping(&repository, "profile = \"repo\"\nagents = { fake = \"agent\" }"),
            "agent",
            "repository agent mapping",
        ),
        (Some("global"), mapping(&repository, "profile = \"repo\""), "repo", "repository mapping"),
        (Some("global"), String::new(), "global", "global default"),
    ] {
        configure(&root, default, &tables);
        let current = run(&root, &repository, &["fake", "current"]);
        assert_eq!(current.status.code(), Some(0), "{}", stderr(&current));
        assert_eq!(stdout(&current), format!("{profile}\n"));

        let resolve = run(&root, &repository, &["fake", "resolve"]);
        assert_eq!(resolve.status.code(), Some(0), "{}", stderr(&resolve));
        assert_eq!(
            stdout(&resolve),
            format!(
                "agent:        fake\nprofile:      {profile}\nsource:       {source}\nrepository:   {}\n",
                repository.display()
            )
        );

        let status = run(&root, &repository, &["status"]);
        assert_eq!(status.status.code(), Some(0), "{}", stderr(&status));
        assert!(
            stdout(&status).contains(&format!("\nfake:         {profile} ({source})\n")),
            "{}",
            stdout(&status)
        );
        explicit_launch();
    }

    configure(&root, None, "");
    let current = run(&root, &repository, &["fake", "current"]);
    assert_eq!(current.status.code(), Some(4));
    assert!(current.stdout.is_empty());
    assert!(stderr(&current).contains("no profile selected for `fake`"), "{}", stderr(&current));
    let resolve = run(&root, &repository, &["fake", "resolve"]);
    assert_eq!(resolve.status.code(), Some(4));
    assert_eq!(
        stdout(&resolve),
        format!(
            "agent:        fake\nprofile:      none\nsource:       none\nrepository:   {}\n",
            repository.display()
        )
    );
    assert!(stderr(&resolve).contains("no profile selected for `fake`"), "{}", stderr(&resolve));
    let status = run(&root, &repository, &["status"]);
    assert_eq!(status.status.code(), Some(0));
    assert!(stdout(&status).contains("\nfake:         none\n"), "{}", stdout(&status));
    explicit_launch();
}

#[test]
fn status_reports_the_repository_mappings_and_an_ancestor_note() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let outer = base.join("outer");
    git_dir(&outer);
    let inner = outer.join("vendor").join("inner");
    git_dir(&inner);
    configure(
        &root,
        Some("work"),
        &(mapping(&outer, "profile = \"outer\"")
            + &mapping(&inner, "profile = \"inner\"\nagents = { fake = \"mine\" }")),
    );
    let output = run(&root, &inner.join("."), &["status"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], format!("repository:   {}", inner.display()));
    assert_eq!(
        &lines[1..4],
        ["mapping:      inner", "agents:       fake=mine", "default:      work"]
    );
    assert_eq!(lines[4], "claude:       inner (repository mapping)");
    assert!(lines.contains(&"fake:         mine (repository agent mapping)"), "{text}");
    assert_eq!(
        *lines.last().unwrap(),
        format!(
            "note:         {} has a mapping that does not apply to this repository",
            outer.display()
        )
    );

    let outside = run(&root, &base, &["status"]);
    assert_eq!(outside.status.code(), Some(0));
    assert!(
        stdout(&outside).starts_with("repository:   not in a repository\nmapping:      none\n")
    );
}

#[test]
fn agent_status_shows_one_agent_and_its_presence() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let repository = base.join("acme");
    git_dir(&repository);
    configure(&root, None, &mapping(&repository, "profile = \"work\""));
    let presence = || {
        let output = run(&root, &repository, &["fake", "status"]);
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
        let text = stdout(&output);
        assert!(!text.contains("claude:"), "{text}");
        assert!(text.contains("fake:         work (repository mapping)\n"), "{text}");
        text.lines().find(|line| line.starts_with("presence:")).map(str::to_owned)
    };
    assert_eq!(presence().as_deref(), Some("presence:     absent"));
    let launch = run(&root, &repository, &["fake"]);
    assert_eq!(launch.status.code(), Some(0), "{}", stderr(&launch));
    assert_eq!(presence().as_deref(), Some("presence:     materialized"));
    configure(&root, None, "");
    let output = run(&root, &repository, &["fake", "status"]);
    assert!(!stdout(&output).contains("presence:"), "{}", stdout(&output));
}

#[test]
fn a_case_twin_profile_directory_is_reported_as_a_conflict() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let repository = base.join("acme");
    git_dir(&repository);
    configure(&root, None, &mapping(&repository, "profile = \"work\""));
    fs::create_dir_all(root.path().join("profiles").join("WORK")).unwrap();
    let output = run(&root, &repository, &["fake", "status"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(stdout(&output).contains("presence:     conflicts with WORK\n"), "{}", stdout(&output));
}

#[test]
fn link_and_unlink_with_and_without_repo() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let repository = base.join("acme");
    git_dir(&repository);
    let sub = repository.join("sub");
    fs::create_dir_all(&sub).unwrap();
    let shown = repository.display();

    for (cwd, args, expected) in [
        (&sub, &["link", "work"][..], format!("linked {shown} -> work\n")),
        (&sub, &["link", "work"], format!("already linked {shown} -> work\n")),
        (
            &base,
            &["link", "other", "--repo", "acme/sub"],
            format!("changed {shown}: work -> other\n"),
        ),
        (
            &base,
            &["fake", "link", "personal", "--repo=acme"],
            format!(
                "linked fake: {shown} -> personal\nnote:         profile personal has not been launched with fake yet\n"
            ),
        ),
        (&sub, &["fake", "current"], "personal\n".to_owned()),
        (&sub, &["fake", "unlink"], format!("unlinked fake: {shown} (was personal)\n")),
        (&sub, &["fake", "current"], "other\n".to_owned()),
        (&sub, &["fake", "unlink"], format!("no mapping to remove for fake: {shown}\n")),
        (&base, &["unlink", "--repo", "acme"], format!("unlinked {shown} (was other)\n")),
        (&base, &["unlink", "--repo", "acme"], format!("no mapping to remove for {shown}\n")),
    ] {
        let output = run(&root, cwd, args);
        assert_eq!(output.status.code(), Some(0), "{args:?}: {}", stderr(&output));
        assert_eq!(stdout(&output), expected, "{args:?}");
    }
    let launch = run(&root, &repository, &["fake", "work"]);
    assert_eq!(launch.status.code(), Some(0), "{}", stderr(&launch));
    let output = run(&root, &repository, &["fake", "link", "work"]);
    assert_eq!(stdout(&output), format!("linked fake: {shown} -> work\n"));
}

#[test]
fn link_and_unlink_outside_a_repository_exit_4() {
    let root = Root::new();
    let (_dir, base) = scratch();
    for args in
        [&["link", "work"][..], &["fake", "unlink"], &["unlink"], &["link", "work", "--repo", "."]]
    {
        let output = run(&root, &base, args);
        assert_eq!(output.status.code(), Some(4), "{args:?}: {}", stderr(&output));
        assert_eq!(
            stderr(&output),
            format!(
                "agent-profile: error: repository {}: not inside a Git repository\n",
                base.display()
            ),
            "{args:?}"
        );
        assert!(output.stdout.is_empty());
    }
    assert!(!root.path().join("config.toml.lock").exists());
}

#[test]
fn unlink_repo_removes_orphan_mappings_and_never_an_enclosing_one() {
    let root = Root::new();
    let (_dir, base) = scratch();

    let deleted = base.join("deleted");
    git_dir(&deleted);
    let enclosing = base.join("enclosing");
    git_dir(&enclosing);
    let recreated = enclosing.join("nested");
    git_dir(&recreated);
    let main = base.join("main");
    git_dir(&main);
    let worktree = base.join("wt");
    fs::create_dir_all(main.join(".git").join("worktrees").join("wt")).unwrap();
    fs::write(main.join(".git").join("worktrees").join("wt").join("HEAD"), "x\n").unwrap();
    fs::create_dir_all(&worktree).unwrap();
    fs::write(worktree.join(".git"), "gitdir: ../main/.git/worktrees/wt\n").unwrap();

    for (cwd, profile) in [(&deleted, "d"), (&enclosing, "e"), (&recreated, "r"), (&worktree, "w")]
    {
        let output = run(&root, cwd, &["link", profile]);
        assert_eq!(output.status.code(), Some(0), "{}: {}", cwd.display(), stderr(&output));
    }

    fs::remove_dir_all(&deleted).unwrap();
    fs::remove_dir_all(recreated.join(".git")).unwrap();
    fs::remove_dir_all(main.join(".git").join("worktrees")).unwrap();
    let broken = run(&root, &worktree, &["fake", "current"]);
    assert_eq!(broken.status.code(), Some(4), "the stale worktree is a discovery error");

    let sub = base.join("sub");
    fs::create_dir_all(&sub).unwrap();
    for (repo, expected) in [
        ("../deleted", format!("unlinked {} (was d)\n", deleted.display())),
        ("../enclosing/nested", format!("unlinked {} (was r)\n", recreated.display())),
        ("../wt", format!("unlinked {} (was w)\n", worktree.display())),
    ] {
        let output = run(&root, &sub, &["unlink", "--repo", repo]);
        assert_eq!(output.status.code(), Some(0), "{repo}: {}", stderr(&output));
        assert_eq!(stdout(&output), expected, "{repo}");
    }
    let text = fs::read_to_string(root.path().join("config.toml")).unwrap();
    assert_eq!(text.matches("[repositories.").count(), 1, "{text}");
    assert_eq!(run(&root, &enclosing, &["fake", "current"]).stdout, b"e\n");

    let output = run(&root, &recreated, &["fake", "unlink"]);
    assert_eq!(
        stdout(&output),
        format!("no mapping to remove for fake: {}\n", enclosing.display())
    );
}

#[test]
fn unlink_notes_an_ancestor_mapping_and_remaining_agent_mappings() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let outer = base.join("outer");
    git_dir(&outer);
    let inner = outer.join("inner");
    git_dir(&inner);
    configure(
        &root,
        None,
        &(mapping(&outer, "profile = \"o\"") + &mapping(&inner, "agents = { fake = \"f\" }")),
    );
    let output = run(&root, &inner, &["unlink"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        format!(
            "no mapping to remove for {inner}\nnote:         {outer} has a mapping; remove it with agent-profile unlink --repo {outer}\nnote:         agent mappings remain: fake=f; remove them with agent-profile <agent> unlink\n",
            inner = inner.display(),
            outer = outer.display()
        )
    );
}

#[test]
fn a_worktree_and_a_submodule_each_resolve_their_own_mapping() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let main = base.join("main");
    git_dir(&main);
    let admin = main.join(".git").join("worktrees").join("wt");
    fs::create_dir_all(&admin).unwrap();
    fs::write(admin.join("HEAD"), "x\n").unwrap();
    fs::write(admin.join("commondir"), "../..\n").unwrap();
    let worktree = base.join("wt");
    fs::create_dir_all(&worktree).unwrap();
    fs::write(worktree.join(".git"), format!("gitdir: {}\n", admin.to_str().unwrap())).unwrap();
    let modules = main.join(".git").join("modules").join("sub");
    fs::create_dir_all(&modules).unwrap();
    fs::write(modules.join("HEAD"), "x\n").unwrap();
    let submodule = main.join("sub");
    fs::create_dir_all(&submodule).unwrap();
    fs::write(submodule.join(".git"), "gitdir: ../.git/modules/sub\n").unwrap();

    configure(
        &root,
        None,
        &(mapping(&main, "profile = \"main\"")
            + &mapping(&worktree, "profile = \"tree\"")
            + &mapping(&submodule, "profile = \"module\"")),
    );
    assert_eq!(launched_home(&root, &main), home(&root, "main"));
    assert_eq!(launched_home(&root, &worktree), home(&root, "tree"));
    assert_eq!(launched_home(&root, &submodule), home(&root, "module"));
}

#[test]
fn an_explicit_launch_ignores_a_broken_repository_and_a_resolved_launch_does_not() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let broken = base.join("broken");
    fs::create_dir_all(broken.join(".git")).unwrap();

    let output = run(&root, &broken, &["fake", "work"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(support::report(&output.stdout).get("pid").is_some());
    let dry = run(&root, &broken, &["fake", "work", "--dry-run"]);
    assert_eq!(dry.status.code(), Some(0), "{}", stderr(&dry));
    assert!(stdout(&dry).contains("repository:   none\n"), "{}", stdout(&dry));

    let output = run(&root, &broken, &["fake"]);
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert_eq!(
        stderr(&output),
        format!(
            "agent-profile: error: repository {}: invalid .git directory: no HEAD file\n",
            broken.join(".git").display()
        )
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn every_command_usage_error_through_the_binary() {
    let root = Root::new();
    let (_dir, base) = scratch();
    for (args, code, message) in [
        (&["claude", "link", "work", "--repo"][..], 2, "`--repo` needs a path"),
        (&["link", "work", "--repo", "a", "--repo", "b"], 2, "`--repo` may be given only once"),
        (&["status", "--repo="], 2, "`--repo` needs a non-empty path"),
        (&["fake", "Link", "work"], 2, "command words are lower case: `link`"),
        (&["status", "--"], 2, "`status` takes no agent arguments; remove `--`"),
        (&["unlink", "--", "--repo", "x"], 2, "`unlink` takes no agent arguments; remove `--`"),
        (&["fake", "unlink", "--verbose"], 2, "unknown option \"--verbose\" for `unlink`"),
        (&["fake", "resolve", "--json"], 2, "`--json` is not yet implemented"),
        (&["link"], 2, "`link` needs a profile: agent-profile [<agent>] link <profile>"),
        (&["fake", "link", "a", "b"], 2, "`link` takes one profile"),
        (&["fake", "current", "x"], 2, "`current` takes no arguments"),
        (&["resolve"], 2, "`resolve` needs an agent: agent-profile <agent> resolve"),
        (&["current"], 2, "`current` needs an agent: agent-profile <agent> current"),
        (
            &["fake", "--repo", "x"],
            2,
            "unknown option \"--repo\"; agent arguments must follow `--`",
        ),
        (&["LINK", "work"], 2, "unknown agent `LINK`"),
        (&["link", "status"], 4, "\"status\" is a reserved command word"),
        (&["link", "Create"], 4, "\"create\" is a reserved command word"),
    ] {
        let output = run(&root, &base, args);
        assert_eq!(output.status.code(), Some(code), "{args:?}: {}", stderr(&output));
        assert_eq!(stderr(&output).lines().count(), 1, "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).contains(message), "{args:?}: {}", stderr(&output));
        assert!(output.stdout.is_empty(), "{args:?}");
    }
}

#[test]
fn command_help_and_the_top_level_help() {
    let root = Root::new();
    let (_dir, base) = scratch();
    for (args, first) in [
        (&["link", "-h"][..], "Usage: agent-profile [<agent>] link <profile> [--repo <path>]"),
        (
            &["link", "status", "-h"],
            "Usage: agent-profile [<agent>] link <profile> [--repo <path>]",
        ),
        (&["resolve", "-h"], "Usage: agent-profile <agent> resolve [--repo <path>]"),
        (&["fake", "unlink", "--help"], "Usage: agent-profile [<agent>] unlink [--repo <path>]"),
        (&["fake", "create", "-h"], "Usage: agent-profile <agent> <profile>"),
    ] {
        let output = run(&root, &base, args);
        assert_eq!(output.status.code(), Some(0), "{args:?}");
        assert!(stdout(&output).starts_with(first), "{args:?}: {}", stdout(&output));
    }
    let output = run(&root, &base, &["--help"]);
    assert_eq!(output.status.code(), Some(0));
    for word in ["current", "resolve", "status", "link", "unlink"] {
        assert!(
            stdout(&output).lines().any(|line| line.trim_start().starts_with("agent-profile")
                && line.split_whitespace().any(|part| part == word)),
            "{word}: {}",
            stdout(&output)
        );
    }
}

#[test]
fn missing_repo_paths_exit_4() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let missing = base.join("missing");
    for args in
        [&["link", "work", "--repo", "missing"][..], &["fake", "resolve", "--repo", "missing"]]
    {
        let output = run(&root, &base, args);
        assert_eq!(output.status.code(), Some(4), "{args:?}: {}", stderr(&output));
        assert!(
            stderr(&output).starts_with(&format!(
                "agent-profile: error: repository {}: cannot resolve the directory: ",
                missing.display()
            )),
            "{args:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn link_through_a_link_to_the_repository_stores_the_canonical_root() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let real = base.join("real");
    git_dir(&real);
    let alias = base.join("alias");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, &alias).unwrap();
    #[cfg(windows)]
    {
        let status = std::process::Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(&alias)
            .arg(&real)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());
    }
    let output = run(&root, &base, &["link", "work", "--repo", "alias"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), format!("linked {} -> work\n", real.display()));
    assert_eq!(launched_home(&root, &real), home(&root, "work"));
    assert_eq!(launched_home(&root, &alias), home(&root, "work"));
}

#[cfg(unix)]
#[test]
fn a_non_utf8_repo_value_is_a_path() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let output = root
        .agent_profile(["fake", "resolve", "--repo"])
        .arg(support::non_utf8())
        .current_dir(&base)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert!(stderr(&output).contains("cannot resolve the directory"), "{}", stderr(&output));
}
```

- [ ] **Step 2: Run the task checks**

```bash
cargo nextest run -p agent-profile --test resolution
```

Expected: every test passes, including `unlink_repo_removes_orphan_mappings_and_never_an_enclosing_one`, `every_command_usage_error_through_the_binary`, `an_explicit_launch_ignores_a_broken_repository_and_a_resolved_launch_does_not`.

- [ ] **Step 3: Run the gate** (see "Gate commands"). Expected on Windows: `220 tests run: 220 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 4: Commit**

```bash
git add crates/agent-profile/tests/resolution.rs
git commit -m "test: repository resolution end to end

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 5: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/cli.rs` replace exactly:

```rust
        Some(repo) => repo::unlink_keys(cwd, repo),
```

with:

```rust
        Some(repo) => vec![cwd.join(repo)],
```

Run `cargo nextest run --no-fail-fast -p agent-profile --test resolution`. Expected: FAIL, and the failing test list includes `unlink_repo_removes_orphan_mappings_and_never_an_enclosing_one`. Then restore with `git checkout -- crates/agent-profile/src/cli.rs` and confirm `git status --short` prints nothing.


### Task 8: Documentation

README resolution order, commands and a `config.toml` example; TODO SP3 known limits (design §9).

**Files:**
- Modify (whole file): `README.md` (usage, resolution and the configuration example)
- Modify (whole file): `TODO.md` (SP3 known limits replace the open decision)

**Before:** `TODO.md` contains `## SP3 open decisions`; `README.md` contains `**Status: architecture gate (SP2).**`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `README.md`** (usage, resolution and the configuration example; the whole file, byte-exact)

````markdown
# Agent Profile

A local, privacy-first Rust CLI for selecting and launching profiles for multiple coding agents.

> `agent-profile` owns profile selection. The coding agent owns authentication and agent-specific
> configuration. The evidence determines what `agent-profile` is allowed to claim.

Agent Profile never copies credentials, extracts tokens, sends telemetry or trusts
repository-controlled profile selection.

**Status: repository resolution (SP3).** Profile-name validation, configuration, the launch path (`exec` on
Unix, a supervised child on Windows), three evidence-backed adapters (Claude Code, Codex CLI, Aider) and
repository mappings exist; see [ROADMAP.md](ROADMAP.md). The authoritative design is
[`agent-profile-implementation-spec-v3.md`](agent-profile-implementation-spec-v3.md).

## Architecture

```text
CLI
  ↓
Configuration
  ↓
Repository Discovery
  ↓
Profile Resolution
  ↓
Agent Adapter
  ↓
LaunchPlan
  ↓
Process Launcher
  ↓
Coding Agent
```

Adapters produce a `LaunchPlan`; they never spawn processes.

## Usage

```text
agent-profile <agent> <profile> [WRAPPER OPTIONS] [-- <agent args...>]
agent-profile <agent> [WRAPPER OPTIONS] [-- <agent args...>]
agent-profile <agent> current|resolve|status [--repo <path>]
agent-profile [<agent>] link <profile> [--repo <path>]
agent-profile [<agent>] unlink [--repo <path>]
agent-profile status [--repo <path>]
```

Everything after `--` is passed to the agent untouched. `--dry-run` shows what would be launched without
launching or creating anything.

Configuration and profiles live in `~/.agent-profile/` (`%USERPROFILE%\.agent-profile\` on Windows). Set
`AGENT_PROFILE_HOME` to an absolute path to use another directory.

Without a profile word, the profile comes from the first of: the agent's mapping for the current Git repository,
the repository's mapping, `default_profile`, and otherwise an error. `link` stores a mapping for the repository
you are in (its canonical root; a subdirectory, symlink or junction resolves to it), `unlink` removes it, and
`unlink --repo <path>` removes the mapping stored for a path even after the repository was deleted or moved. A
mapping applies only to that repository: not to a nested repository, a submodule or a linked worktree, which are
repositories of their own. `<agent> resolve` and `status` show what would be selected and why.

```toml
default_profile = "work"

[repositories.'C:\src\acme']
profile = "work"
agents = { claude = "personal" }
```

The global default is set by editing `default_profile`. Discovery reads only the `.git` entry and Git's
`gitdir`/`commondir` files; it never runs Git, never reads Git configuration, and ignores `GIT_DIR` and
`core.worktree` (use `--repo`).

## Supported agents

Every profile lives under `<root>/profiles/<profile>/<agent>/` and is created on first launch. Each row is
backed by an evidence entry (verified 2026-09-15); a state weaker than `Supported` means exactly what its
reason says.

| Agent | Mechanism | Support | Config isolation | Credential isolation | State isolation |
|---|---|---|---|---|---|
| Claude Code 2.1.270 (`claude`) | `CLAUDE_CONFIG_DIR` | Proven | Supported (project `.claude/` settings still layer on top) | Conditional (`ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `CLAUDE_CODE_OAUTH_TOKEN` and similar variables bypass it) | NotGuaranteed (history and project state moving with the directory is community-sourced only) |
| Codex CLI 0.153.4 (`codex`) | `CODEX_HOME` | Proven | Supported (project-level configuration layering is not measured) | Conditional (`OPENAI_API_KEY`, `CODEX_API_KEY`, `CODEX_ACCESS_TOKEN` bypass it) | Conditional (`CODEX_SQLITE_HOME`) |
| Aider 0.86.2 (`aider`) | `--config <profile>/.aider.conf.yml` | Proven | NotGuaranteed (home, repository and working-directory `.aider.conf.yml`, `.env` and `AIDER_*` still apply) | NotSupported | NotSupported |

A new Codex profile starts logged out. Arguments that select the same mechanism (Aider's `-c`, `--config`
and its abbreviations) are refused before launch. On Windows an agent must be a native executable (`.exe`, or a configured `.com`): an npm or pnpm
`.cmd` shim is refused, and the error names the `[agents.<id>] executable` setting to use instead.

## Planned adapters — not yet implemented or evidence-verified

The mechanisms below are the specification's starting point, not a claim about isolation.

| Agent | Primary mechanism | Intended semantic tier |
|---|---|---|
| Gemini CLI | `GEMINI_CLI_HOME` | Home/state isolation |
| GitHub Copilot CLI | `COPILOT_HOME` | Home/config isolation |
| OpenCode | `OPENCODE_CONFIG_DIR` / `OPENCODE_CONFIG` | Config/home isolation |
| Cline CLI | `--config`, `--data-dir` | Config/state selection |
| Pi | `PI_CODING_AGENT_DIR` | Agent home/state isolation |
| Kiro CLI | `KIRO_HOME` | Independent home/profile |
| Cursor Agent CLI | `CURSOR_CONFIG_DIR` | Configuration selection |
| Continue CLI | `--config` | Configuration selection |
| Amp | `--settings-file` | Settings selection |

## Workspace

| Crate | Responsibility |
|---|---|
| `agent-profile` | Library (naming, configuration, repository discovery, resolution, adapters, launcher, CLI, output) and the `agent-profile` binary |
| `agent-profile` → `fake-agent` binary | Test-only stand-in for a coding agent (`src/bin/fake-agent.rs`); never shipped |

## Building

Requires Rust 1.98+ (edition 2024).

```
cargo build --workspace
just check              # the local gate: fmt + clippy + typos + test
```

Tooling is listed in [`docs/dev-tooling.md`](docs/dev-tooling.md); see
[`CONTRIBUTING.md`](CONTRIBUTING.md) to get set up.

## Roadmap

See [`ROADMAP.md`](ROADMAP.md) for the sub-project plan and [`TODO.md`](TODO.md) for what is
immediately next.

## Licence

[PolyForm Noncommercial License 1.0.0](LICENSE). Noncommercial use only.
````

- [ ] **Step 2: Write `TODO.md`** (SP3 known limits replace the open decision; the whole file, byte-exact)

```markdown
# TODO

Near-term work. Sub-project scope lives in [ROADMAP.md](ROADMAP.md).

## SP2 known limits

- [x] **Launching `.cmd`/`.bat` shims without shell mediation** (spec §20, §23.2). Decided in SP2 (design D2):
      shims are refused, never parsed; the error names the `[agents.<id>] executable` setting.
- [ ] Aider conflict scan is shallow (spec §21): option values are not parsed, so a value equal to a conflicting
      spelling (`--message --config`) is refused, and clustered short options (`-vc f`) are not detected.
- [ ] Credential variables that bypass Claude Code and Codex isolation (SP2 design D8) are passed through
      unchanged; decide in `doctor` (SP5) whether to warn.
- [ ] Setting `[agents.codex] executable` to the npm-vendored `codex.exe` bypasses the npm launcher, which may put
      bundled tools such as `rg` on `PATH`; measure in a sandbox before recommending it.
- [ ] Creating the Aider config file needs a no-replace rename (Linux falls back to hard links; macOS needs
      exclusive-rename support, with no fallback). A filesystem without it fails with exit 4; macOS smbfs, msdos and
      exfat are not measured.
- [ ] A `.com` beside a `.exe` in one `PATH` directory is ignored although `cmd.exe` would prefer it, and a `.com`
      alone on `PATH` is reported as needing a shell although it is a native program.
- [ ] The Unix directory-sync failure branch of the Aider file writer is untested (like SP1 `config.rs` step 7a),
      and so is its temp-file `sync_all` before the rename (durability needs crash injection to observe).
- [ ] Report argument redaction is a name rule: it misses secrets passed positionally, in inline JSON, in
      `key=value` or `Name: value` forms without a sensitive name part, or under an option abbreviation without
      the sensitive part, or in an option name whose sensitive part is split by an invalid UTF-8 byte (the rule
      matches the lossy text), and it hides harmless values such as `--map-tokens 1024`.
- [ ] `ArgumentConflict` echoes the whole matched argument; redact it before any conflict option can carry a secret.
- [ ] The Codex "new profile starts logged out" note keys on the home directory being absent, so a present but
      empty home gives no note.
- [ ] An agent with no native executable cannot be launched on Windows until its vendor ships one.
- [ ] Adapter evidence is static; mechanism drift detection belongs to `doctor` (SP5).

## SP3 known limits

Design: [docs/superpowers/specs/2026-09-15-sp3-resolution-design.md](docs/superpowers/specs/2026-09-15-sp3-resolution-design.md) §10.

- [ ] Git layouts that rely on `core.worktree`, `GIT_DIR` or `GIT_WORK_TREE` are not honoured; a bare repository is
      not a repository for resolution. `--repo` is the explicit override.
- [ ] A moved repository's mapping stays under the old path until it is linked again; the SP5 `repositories`
      report shows the orphan.
- [ ] A repository root that is not valid UTF-8 cannot be linked. On a Linux case-insensitive mount two letter-case
      spellings of one directory are two repository identities.
- [ ] Profile names in `config.toml` are validated with the host's rules, so a name Windows forbids makes a synced
      configuration invalid on Windows.
- [ ] An agent mapping for an agent this build does not know cannot be removed with `<agent> unlink`; SP5's
      `delete` refusal must name the key and field to edit.
- [ ] Shared multi-user machines (revisit with `doctor` in SP5, with a `safe.directory`-style escape hatch):
      discovery does not check who owns a `.git`, and a local user who swaps a checked file for a FIFO can block it.
- [ ] Unix automount paths and Windows mapped drive letters in a `gitdir`/`commondir` are not detected as network
      paths. On Windows the network refusal also refuses a repository on a volume without a drive letter, a local
      worktree of a repository on a share, and a share reached through two server spellings.
- [ ] A mapping made in the main checkout does not apply in its linked worktrees.
- [ ] A `link` or `unlink` that changes `config.toml` rewrites it with LF line endings and without a byte-order
      mark (`toml_edit` renders that way); a command that changes nothing leaves the file untouched.

## Housekeeping

- [ ] Enable GitHub private vulnerability reporting (see [SECURITY.md](SECURITY.md)). Branch
      protection is applied during SP0 (design §4.1).
- [ ] The MSRV (1.98) is not checked in CI; verify by hand with
      `cargo +1.98 check --workspace --all-targets`. Decide whether to add a CI job.
- [ ] `cargo install --path crates/agent-profile` also installs the `fake-agent` and `console-driver` test
      binaries. Not reachable from any gate or release (release.yml builds `--bin agent-profile`); revisit if
      source installs are ever documented.

- [ ] `console-driver` (test-only) drains the wrapper's stdout and stderr to EOF with no timeout; a descendant
      that inherits those pipes (for example a `FAKE_AGENT_SPAWN_SLEEPER` sleeper) keeps the driver alive past its
      60 s exit timeout. No current test combines a console event with a sleeper; bound the drains before adding one.

- [ ] `windows_console` `breakaway_follows_a_controlled_caller_job` fails under plain `cargo test` (exit 125,
      "cannot spawn the sleeper: Access is denied") but passes under `cargo nextest`, the gate: it assumes one process
      per test. Isolate it (for example a nextest-only marker or a child process) or document it.

- [ ] Pin the actions in `.github/workflows/ci.yml` (`actions/checkout@v7`, `dtolnay/rust-toolchain@stable`,
      `Swatinem/rust-cache@v2`, `EmbarkStudios/cargo-deny-action@v2`, `taiki-e/install-action@nextest`) to commit SHAs
      with version comments, as the release and sandbox workflows already are.

- [ ] `sandbox/run.sh` picks the nextest download from the host `uname -m`; a Docker Desktop configured to build
      for another platform (`DOCKER_DEFAULT_PLATFORM`, Rosetta x86_64 default) gets the wrong binary. Pass the
      engine's build architecture instead if that setup is needed.

## Scaffold follow-ups

- [ ] Run `lefthook install` in each clone
```

- [ ] **Step 3: Run the task checks**

```bash
typos
```

Expected: no output.

- [ ] **Step 4: Run the gate** (see "Gate commands"). Expected on Windows: `220 tests run: 220 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 5: Commit**

```bash
git add README.md TODO.md
git commit -m "docs: SP3 repository resolution usage and known limits

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```


## After the last task

- [ ] Push `sp3-resolution` and open a PR only with the owner's confirmation. CI (Windows, macOS, Linux) must be green.
- [ ] Run AGY-CAPSTONE (on subagents, owner-directed) over the committed range, then AGY-TEST-AUDIT, before declaring SP3 complete.
- [ ] Mark SP3 done in `ROADMAP.md` after the merge, as SP2 did.
