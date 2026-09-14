# SP1 Core and Explicit Launch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver V3 §35 phases 1 and 3: profile-name validation, the application root, strict and atomic configuration, structured errors, `LaunchPlan`, executable discovery, the Unix `exec` and Windows child launchers, passthrough, environment overrides and dry run, proven end-to-end through the real binary against a debug-only `fake` agent.

**Architecture:** One crate, `crates/agent-profile`, filling the SP0 empty modules and adding `error` and `exe`. The CLI splits `<agent> ...` with an ordered check list, the `fake` adapter builds a `LaunchPlan`, and the launcher `exec`s on Unix or supervises a child in a kill-on-close job on Windows.

**Tech Stack:** Rust 1.98 (edition 2024), clap 4.6, toml 1.1 (reads), toml_edit 0.25 (writes), thiserror 2, tempfile 3, windows-sys 0.61 (Windows only), cargo-nextest.

**Design:** `docs/superpowers/specs/2026-09-14-sp1-core-launch-design.md` (the oracle for behaviour; V3 `agent-profile-implementation-spec-v3.md` wins over both).

---

## How this plan was produced, and how to execute it

Every code block below is copied byte-for-byte from a prototype of the whole design that passed on all three GitHub runners (draft PR #7, run 34853308762: Windows 97/97 including the console-event tests, macOS and Linux 91/91), plus `just check`, `cargo deny check`, `cargo +1.98 check`, the `-D warnings` docs build and a clean release build. The plan was then replayed task by task in a fresh worktree: each task's gate and its logic mutant were run exactly as written here.

Rules for every task:

1. **Step 0 - state check.** On branch `sp1-core-launch`, run `git status --short` (expect no output). `HEAD` must be the previous task's commit; for Task 1 it is the commit that added this plan or a later documentation commit (`git log --oneline -1` shows a `docs:` subject). Then check the "Before" fact listed for every file the task modifies. If any check fails, STOP and report `STATE_MISMATCH: <what>`.
2. **Byte-exact files.** Write each file with exactly the content shown: the whole file, not a merge. The gate includes `cargo fmt --all -- --check`, so do not reformat. Do not "improve" any code or test.
3. **Shape-divergence stop.** If making the code compile would change the shape, type or encoding of anything shown, STOP and report `[original] -> [yours] because <reason>`. "It compiles" is not a justification.
4. **Oracle.** The named tests pin the behaviour. If a test fails, fix the code to match the test and the design; never edit a test to match the code.
5. **Gate.** Run the gate exactly as written; do not add flags.
6. **Mutant.** After committing, apply the mutant, run the command, confirm the named test FAILS, then restore with `git checkout -- <file>` and confirm `git status --short` prints nothing.
7. **Toolchain drift.** `rust-toolchain.toml` pins `stable`. If a gate fails with a clippy lint or compiler diagnostic in code copied from this plan (a newer stable than the verified one), STOP and report the diagnostic; do not change the code to silence it.
8. **Windows desktop.** From Task 10 on, every full-workspace test run on Windows opens short-lived console windows; do not type into them while tests run.

Gate commands (every task after its own checks):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
typos
cargo nextest run --workspace --no-tests=pass
```

Expected: `cargo fmt` and `typos` print nothing; clippy prints no warnings; nextest ends with `N tests run: N passed`, 0 failed.

## File structure

| File | Task | Responsibility |
|---|---|---|
| `Cargo.toml` | 1 | workspace pins: add `toml_edit`, `windows-sys` |
| `crates/agent-profile/Cargo.toml` | 1 | crate dependencies |
| `crates/agent-profile/src/name.rs` | 1 | `ProfileName`, `AgentId`, `Platform`, reserved words (V3 §5.3, §6) |
| `crates/agent-profile/src/error.rs` | 2 | `Error` and `exit_code()` (V3 §33) |
| `crates/agent-profile/src/lib.rs` | 2 | module list: add `error`, `exe` |
| `crates/agent-profile/src/config.rs` | 3 | `AppRoot`, `Config`, `update`/`update_with` (V3 §7, §17, §18) |
| `crates/agent-profile/tests/config.rs` | 3 | configuration integration tests |
| `crates/agent-profile/src/exe.rs` | 4 | executable discovery (V3 §20) |
| `crates/agent-profile/src/bin/fake-agent.rs` | 5 | test fixture: report + control variables |
| `crates/agent-profile/tests/support/mod.rs` | 5 | shared test helpers |
| `crates/agent-profile/tests/smoke.rs` | 5 | top-level CLI and fixture smoke tests |
| `crates/agent-profile/src/launch/mod.rs` | 6 | `LaunchPlan`, `LaunchOutcome`, `launch()` |
| `crates/agent-profile/src/launch/unix.rs` | 6 | `exec` launcher (V3 §23.1) |
| `crates/agent-profile/src/launch/windows.rs` | 6 | job + handler child launcher (V3 §23.2, §24) |
| `crates/agent-profile/tests/launch_plan.rs` | 6 | `LaunchPlan::command` tests |
| `crates/agent-profile/src/resolve.rs` | 7 | `Resolution`, `ResolutionSource`, stub `resolve` (V3 §12) |
| `crates/agent-profile/src/adapter/mod.rs` | 7 | `PlannedLaunch`, `plan`, debug-only `fake`, lazy profile dir |
| `crates/agent-profile/src/output.rs` | 8 | dry-run/verbose report, redaction (V3 §22, §26) |
| `crates/agent-profile/src/cli.rs` | 9 | Clap surface, splitter, dispatch (V3 §5) |
| `crates/agent-profile/src/main.rs` | 9 | single exit path |
| `crates/agent-profile/tests/launch.rs` | 9 | end-to-end launch tests |
| `crates/agent-profile/src/bin/console-driver.rs` | 10 | test-only Windows console driver |
| `crates/agent-profile/tests/windows_console.rs` | 10 | Windows control-event and job tests |
| `README.md` | 11 | status, configuration location |
| `TODO.md` | 11 | settled SP1 decisions out, SP2 shim decision in |

`crates/agent-profile/src/repo.rs` stays empty (SP3). `Cargo.lock` changes as cargo resolves the new dependencies; commit it with Task 1.

---

### Task 1: Dependencies and profile-name validation

**Design/spec:** design §5 (`name`), §6.2 note, §10 step 2; V3 §5.3, §6

Adds every SP1 dependency once, then the `name` module: `ProfileName`, `AgentId`, `Platform` and the reserved words. The Windows-only rules are a `Platform` parameter so every CI OS tests both rule sets.

**Files:**
- Modify (replace whole file): `Cargo.toml` - Before: contains `toml = "1.1"` and does not contain `toml_edit`
- Modify (replace whole file): `crates/agent-profile/Cargo.toml` - Before: its `[dependencies]` section lists only `clap` and `serde_json`
- Modify (replace whole file): `crates/agent-profile/src/name.rs` - Before: contains only `//!` doc-comment lines (the SP0 empty module)

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `Cargo.toml`**

```toml
# Agent Profile workspace root.
#
# Virtual workspace: no root package. The shipped binary is `agent-profile`, produced by
# crates/agent-profile. Its second binary, fake-agent, is a test-only fixture and never ships.

[workspace]
resolver = "3"
members = [
    "crates/agent-profile",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.98"
license = "PolyForm-Noncommercial-1.0.0"
repository = "https://github.com/ckir/aiprofiles"
publish = false

# Shared version registry. Listing a crate here pins ONE version for the whole workspace;
# it does NOT add it to any crate — members opt in with `foo = { workspace = true }`.
# Pins are copied from flux; later sub-projects add crates in their own specs.
[workspace.dependencies]
clap = { version = "4.6", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "1.1"
toml_edit = "0.25"
thiserror = "2"
tempfile = "3"
proptest = "1.11"
windows-sys = { version = "0.61", features = [
    "Win32_Foundation",
    "Win32_Security",
    "Win32_Storage_FileSystem",
    "Win32_System_Console",
    "Win32_System_JobObjects",
    "Win32_System_Threading",
] }

[workspace.metadata.release]
shared-version = true
tag-name = "v{{version}}"
tag-prefix = ""
pre-release-commit-message = "release: v{{version}}"
publish = false

[profile.ci]
inherits = "dev"
debug = 0
strip = "debuginfo"
```

- [ ] **Step 2: Write `crates/agent-profile/Cargo.toml`**

```toml
[package]
name = "agent-profile"
description = "Select and launch profiles for multiple coding agents"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
publish.workspace = true
# Two binaries: `agent-profile` (src/main.rs, the product) and `fake-agent` (src/bin/fake-agent.rs, a
# test-only fixture that tests launch through CARGO_BIN_EXE_fake-agent). Only agent-profile ships:
# release.yml builds `--bin agent-profile`.
default-run = "agent-profile"

[dependencies]
clap = { workspace = true }
serde_json = { workspace = true }
tempfile = { workspace = true }
thiserror = { workspace = true }
toml = { workspace = true }
toml_edit = { workspace = true }

[target.'cfg(windows)'.dependencies]
windows-sys = { workspace = true }
```

- [ ] **Step 3: Write `crates/agent-profile/src/name.rs`**

```rust
//! Profile-name validation (spec §6) and the reserved command words (spec §5.3).

use std::fmt;

/// The v0.1 reserved command words (spec §5.3). A profile name must not equal one, ignoring ASCII case.
pub const RESERVED_WORDS: [&str; 13] = [
    "agents",
    "profiles",
    "status",
    "list",
    "create",
    "delete",
    "current",
    "resolve",
    "doctor",
    "link",
    "unlink",
    "repositories",
    "completions",
];

/// Windows device names (spec §6), rejected with or without an extension.
const WINDOWS_DEVICE_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Whether `word` is a reserved command word, ignoring ASCII case.
pub fn is_reserved_word(word: &str) -> bool {
    RESERVED_WORDS.iter().any(|reserved| reserved.eq_ignore_ascii_case(word))
}

/// Which rule set applies. The Windows-only rules of spec §6 are a parameter so every OS tests both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Unix,
    Windows,
}

impl Platform {
    /// The platform this binary runs on.
    pub fn host() -> Platform {
        if cfg!(windows) { Platform::Windows } else { Platform::Unix }
    }
}

/// Why a profile name is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidReason {
    Empty,
    BadFirstChar(char),
    BadChar(char),
    Reserved(&'static str),
    WindowsDeviceName,
    WindowsTrailingDot,
}

impl fmt::Display for InvalidReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidReason::Empty => write!(f, "a profile name must not be empty"),
            InvalidReason::BadFirstChar(c) => {
                write!(f, "a profile name must begin with an ASCII letter or digit, not {c:?}")
            }
            InvalidReason::BadChar(c) => write!(
                f,
                "a profile name may contain only ASCII letters, digits, '.', '_' and '-', not {c:?}"
            ),
            InvalidReason::Reserved(word) => {
                write!(f, "\"{word}\" is a reserved command word and cannot be a profile name")
            }
            InvalidReason::WindowsDeviceName => {
                write!(f, "the name is a reserved Windows device name")
            }
            InvalidReason::WindowsTrailingDot => {
                write!(f, "on Windows a profile name must not end with a dot")
            }
        }
    }
}

/// A validated profile name (spec §6).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileName(String);

impl ProfileName {
    /// Validates `name` against every spec §6 rule for `platform`.
    pub fn parse(name: &str, platform: Platform) -> Result<ProfileName, InvalidReason> {
        let mut chars = name.chars();
        let first = chars.next().ok_or(InvalidReason::Empty)?;
        if !first.is_ascii_alphanumeric() {
            return Err(InvalidReason::BadFirstChar(first));
        }
        if let Some(bad) =
            chars.find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')))
        {
            return Err(InvalidReason::BadChar(bad));
        }
        if let Some(reserved) = RESERVED_WORDS.iter().find(|word| word.eq_ignore_ascii_case(name)) {
            return Err(InvalidReason::Reserved(reserved));
        }
        if platform == Platform::Windows {
            let stem = name.split('.').next().unwrap_or(name);
            if WINDOWS_DEVICE_NAMES.iter().any(|device| device.eq_ignore_ascii_case(stem)) {
                return Err(InvalidReason::WindowsDeviceName);
            }
            if name.ends_with('.') {
                return Err(InvalidReason::WindowsTrailingDot);
            }
        }
        Ok(ProfileName(name.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProfileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An agent identifier: `[a-z][a-z0-9-]*`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AgentId(String);

impl AgentId {
    /// Returns `None` unless `id` matches `[a-z][a-z0-9-]*`.
    pub fn parse(id: &str) -> Option<AgentId> {
        let mut chars = id.chars();
        let first = chars.next()?;
        let valid = first.is_ascii_lowercase()
            && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        valid.then(|| AgentId(id.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unix(name: &str) -> Result<ProfileName, InvalidReason> {
        ProfileName::parse(name, Platform::Unix)
    }

    fn windows(name: &str) -> Result<ProfileName, InvalidReason> {
        ProfileName::parse(name, Platform::Windows)
    }

    #[test]
    fn valid_names_are_accepted_on_both_platforms() {
        for name in ["work", "Work", "w", "7", "a.b", "a_b", "a-b", "personal-2", "WORK.v1_x-y"] {
            assert_eq!(unix(name).unwrap().as_str(), name);
            assert_eq!(windows(name).unwrap().as_str(), name);
        }
    }

    #[test]
    fn empty_name_is_rejected() {
        assert_eq!(unix(""), Err(InvalidReason::Empty));
        assert_eq!(windows(""), Err(InvalidReason::Empty));
    }

    #[test]
    fn name_must_begin_with_letter_or_digit() {
        for (name, first) in
            [(".a", '.'), ("_a", '_'), ("-a", '-'), (".", '.'), ("..", '.'), (" a", ' ')]
        {
            assert_eq!(unix(name), Err(InvalidReason::BadFirstChar(first)), "{name:?}");
            assert_eq!(windows(name), Err(InvalidReason::BadFirstChar(first)), "{name:?}");
        }
    }

    #[test]
    fn forbidden_characters_are_rejected() {
        for (name, bad) in [
            ("a/b", '/'),
            ("a\\b", '\\'),
            ("a b", ' '),
            ("a\tb", '\t'),
            ("a\0b", '\0'),
            ("a\u{7f}", '\u{7f}'),
            ("a:b", ':'),
            ("a*b", '*'),
            ("x\u{e9}", '\u{e9}'),
        ] {
            assert_eq!(unix(name), Err(InvalidReason::BadChar(bad)), "{name:?}");
            assert_eq!(windows(name), Err(InvalidReason::BadChar(bad)), "{name:?}");
        }
    }

    #[test]
    fn reserved_words_are_rejected_in_any_case() {
        for word in RESERVED_WORDS {
            let upper = word.to_ascii_uppercase();
            let mixed: String = word
                .chars()
                .enumerate()
                .map(|(i, c)| if i % 2 == 0 { c.to_ascii_uppercase() } else { c })
                .collect();
            for candidate in [word.to_string(), upper, mixed] {
                assert_eq!(unix(&candidate), Err(InvalidReason::Reserved(word)), "{candidate}");
                assert_eq!(windows(&candidate), Err(InvalidReason::Reserved(word)), "{candidate}");
            }
        }
    }

    #[test]
    fn windows_device_names_are_rejected_only_on_windows() {
        let mut names: Vec<String> =
            ["CON", "PRN", "AUX", "NUL"].iter().map(|s| s.to_string()).collect();
        for n in 1..=9 {
            names.push(format!("COM{n}"));
            names.push(format!("LPT{n}"));
        }
        for name in names {
            for candidate in [
                name.clone(),
                name.to_ascii_lowercase(),
                format!("{name}.txt"),
                format!("{name}.a.b"),
            ] {
                assert_eq!(
                    windows(&candidate),
                    Err(InvalidReason::WindowsDeviceName),
                    "{candidate}"
                );
                assert!(unix(&candidate).is_ok(), "{candidate}");
            }
        }
        for not_device in ["CONSOLE", "COM10", "LPT0", "NULL", "AUXX"] {
            assert!(windows(not_device).is_ok(), "{not_device}");
        }
    }

    #[test]
    fn trailing_dot_is_rejected_only_on_windows() {
        assert_eq!(windows("work."), Err(InvalidReason::WindowsTrailingDot));
        assert!(unix("work.").is_ok());
    }

    #[test]
    fn agent_id_syntax() {
        for id in ["fake", "claude", "a", "gemini-cli", "a1-2"] {
            assert_eq!(AgentId::parse(id).unwrap().as_str(), id);
        }
        for id in ["", "Fake", "1a", "-a", "a_b", "a.b", "a b", "\u{e9}"] {
            assert!(AgentId::parse(id).is_none(), "{id:?}");
        }
    }

    #[test]
    fn is_reserved_word_ignores_ascii_case() {
        assert!(is_reserved_word("CREATE"));
        assert!(is_reserved_word("Link"));
        assert!(!is_reserved_word("work"));
    }
}
```

- [ ] **Step 4: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib name::
```

Expected: every `name::tests::*` test passes (9 tests), 0 failed.

- [ ] **Step 5: Run the gate** (see "Gate commands").

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/agent-profile/Cargo.toml crates/agent-profile/src/name.rs Cargo.lock
git commit -m "feat: add profile-name validation and the SP1 dependencies" -m "Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 7: Prove the tests are not vacuous**

In `crates/agent-profile/src/name.rs`, replace

```rust
        if platform == Platform::Windows {
```

with

```rust
        if platform != Platform::Windows {
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib name::`. Expected: FAIL, and the failing tests include `windows_device_names_are_rejected_only_on_windows`. Then restore:

```bash
git checkout -- crates/agent-profile/src/name.rs
git status --short
```

Expected: `git status --short` prints nothing.

---

### Task 2: Structured errors and exit codes

**Design/spec:** design §6.3; V3 §33

One `Error` enum whose `exit_code()` is the single mapping to the V3 §33 exit codes.

**Files:**
- Create: `crates/agent-profile/src/error.rs` - Before: the file does not exist
- Modify (replace whole file): `crates/agent-profile/src/lib.rs` - Before: contains `pub mod config;` and does not contain `pub mod error;`

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/error.rs`**

```rust
//! Structured errors and their exit codes (spec §33).

use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::name::InvalidReason;

/// Why an agent's executable was not found (spec §20).
#[derive(Debug)]
pub enum NotInstalledReason {
    NotOnPath,
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

    #[error("{}", not_installed_message(agent, reason, unknown_configured))]
    AgentNotInstalled { agent: String, reason: NotInstalledReason, unknown_configured: Vec<String> },

    #[error("invalid profile name {name:?}: {reason}")]
    InvalidProfileName { name: String, reason: InvalidReason },

    #[error("no profile selected for `{agent}`; name one: agent-profile {agent} <profile>")]
    NoProfile { agent: String },

    #[error("{message}")]
    AppRoot { message: String },

    #[error("{}", config_invalid_message(path, key.as_deref(), detail))]
    ConfigInvalid { path: PathBuf, key: Option<String>, detail: String },

    #[error("could not write {}: {source}", path.display())]
    ConfigWrite { path: PathBuf, source: io::Error },

    #[error("profile directory {}: {source}", path.display())]
    ProfileDir { path: PathBuf, source: io::Error },

    #[error(
        "profile `{requested}` differs only in letter case from the existing profile entry `{existing}`; \
         use `{existing}` or choose a different name"
    )]
    ProfileCaseConflict { requested: String, existing: String },

    #[error(
        "{} is a batch file; agent-profile launches agents directly and never through cmd.exe",
        path.display()
    )]
    UnsupportedExecutable { path: PathBuf },

    #[error("could not launch {}: {source}", executable.display())]
    Launch { executable: PathBuf, source: io::Error },

    #[error("{context}: {source}")]
    Io { context: String, source: io::Error },
}

impl Error {
    /// The spec §33 exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Usage { .. } | Error::NotYetImplemented { .. } | Error::UnknownAgent { .. } => 2,
            Error::AgentNotInstalled { .. } => 3,
            Error::InvalidProfileName { .. }
            | Error::NoProfile { .. }
            | Error::AppRoot { .. }
            | Error::ConfigInvalid { .. }
            | Error::ConfigWrite { .. }
            | Error::ProfileDir { .. }
            | Error::ProfileCaseConflict { .. } => 4,
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
    let what = match reason {
        NotInstalledReason::NotOnPath => format!("`{agent}` is not installed: not found in PATH"),
        NotInstalledReason::ExplicitMissing(path) => format!(
            "`{agent}` is not installed: the configured executable {} does not exist or is not a file",
            path.display()
        ),
    };
    format!("{what}{}", unknown_configured_suffix(unknown_configured))
}

fn config_invalid_message(path: &std::path::Path, key: Option<&str>, detail: &str) -> String {
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
            NotInstalledReason::NotOnPath => write!(f, "not found in PATH"),
            NotInstalledReason::ExplicitMissing(path) => {
                write!(f, "configured executable {} is missing", path.display())
            }
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
                Error::AgentNotInstalled {
                    agent: "a".into(),
                    reason: NotInstalledReason::NotOnPath,
                    unknown_configured: vec![],
                },
                3,
            ),
            (Error::InvalidProfileName { name: "".into(), reason: InvalidReason::Empty }, 4),
            (Error::NoProfile { agent: "a".into() }, 4),
            (Error::AppRoot { message: "m".into() }, 4),
            (Error::ConfigInvalid { path: "c".into(), key: None, detail: "d".into() }, 4),
            (Error::ConfigWrite { path: "c".into(), source: io() }, 4),
            (Error::ProfileDir { path: "p".into(), source: io() }, 4),
            (Error::ProfileCaseConflict { requested: "WORK".into(), existing: "work".into() }, 4),
            (Error::UnsupportedExecutable { path: "a.cmd".into() }, 6),
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
}
```

- [ ] **Step 2: Write `crates/agent-profile/src/lib.rs`**

```rust
//! `agent-profile` — select and launch profiles for multiple coding agents.
//!
//! The authoritative design is `agent-profile-implementation-spec-v3.md` at the repository root.
//! Each module below is one layer of the spec §4 architecture. SP0 (the scaffold) creates them
//! empty; later sub-projects fill them in.
//!
//! > `agent-profile` owns profile selection. The coding agent owns authentication and
//! > agent-specific configuration. (spec §1)

pub mod adapter;
pub mod cli;
pub mod config;
pub mod error;
pub mod launch;
pub mod name;
pub mod output;
pub mod repo;
pub mod resolve;
```

- [ ] **Step 3: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib error::
```

Expected: every `error::tests::*` test passes (3 tests), 0 failed.

- [ ] **Step 4: Run the gate** (see "Gate commands").

- [ ] **Step 5: Commit**

```bash
git add crates/agent-profile/src/error.rs crates/agent-profile/src/lib.rs
git commit -m "feat: add structured errors mapped to the spec exit codes" -m "Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 6: Prove the tests are not vacuous**

In `crates/agent-profile/src/error.rs`, replace

```rust
            Error::AgentNotInstalled { .. } => 3,
```

with

```rust
            Error::AgentNotInstalled { .. } => 4,
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib error::`. Expected: FAIL, and the failing tests include `exit_codes_follow_spec_33`. Then restore:

```bash
git checkout -- crates/agent-profile/src/error.rs
git status --short
```

Expected: `git status --short` prints nothing.

---

### Task 3: Application root, strict configuration reads and the locked atomic writer

**Design/spec:** design §6.1, §6.2, §6.4, §8.3; V3 §7, §17, §18, §18.1

`AppRoot` (the `AGENT_PROFILE_HOME` override or `~/.agent-profile`), `Config::load` with the strict schema, and `config::update` over the crate-private `update_with` (bounded lock wait, best-effort temp sweep, validate before and after the edit, sync, persist, Unix directory sync). The unit tests inside `config` cover the injected failed replacement and the reader-during-write barrier; `tests/config.rs` covers the public API, including the Windows sharing-violation replace failure.

**Files:**
- Modify (replace whole file): `crates/agent-profile/src/config.rs` - Before: contains only `//!` doc-comment lines (the SP0 empty module)
- Create: `crates/agent-profile/tests/config.rs` - Before: the file does not exist

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/config.rs`**

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
use crate::name::AgentId;

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
                Some(home) if !home.as_os_str().is_empty() => {
                    Ok(AppRoot(home.join(".agent-profile")))
                }
                _ => Err(Error::AppRoot {
                    message: format!(
                        "cannot determine the home directory; set {HOME_ENV} to an absolute path"
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

/// Applies the strict SP1 schema (design §6.2) to a parsed document.
fn validate(path: &Path, table: &toml::Table) -> Result<Config> {
    let mut config = Config::default();
    for (key, value) in table {
        if key != "agents" {
            return Err(invalid(path, Some(key.clone()), "unknown key".to_owned()));
        }
        let agents = value
            .as_table()
            .ok_or_else(|| invalid(path, Some(key.clone()), "must be a table".to_owned()))?;
        for (id, agent) in agents {
            let agent_key = format!("agents.{id}");
            if AgentId::parse(id).is_none() {
                return Err(invalid(
                    path,
                    Some(agent_key),
                    "agent ids must match [a-z][a-z0-9-]*".to_owned(),
                ));
            }
            let agent = agent.as_table().ok_or_else(|| {
                invalid(path, Some(agent_key.clone()), "must be a table".to_owned())
            })?;
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
                    return Err(invalid(
                        path,
                        Some(field_key),
                        "must be an absolute path".to_owned(),
                    ));
                }
                executable = Some(candidate);
            }
            config.agents.insert(id.clone(), executable);
        }
    }
    Ok(config)
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

    // 5. The edit, re-validated.
    edit(&mut document)?;
    let updated = document.to_string();
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
    fn app_root_defaults_to_dot_agent_profile_in_home() {
        let home = std::env::temp_dir();
        let root = AppRoot::resolve_from(None, Some(home.clone())).unwrap();
        assert_eq!(root.path(), home.join(".agent-profile"));
        assert!(matches!(AppRoot::resolve_from(None, None), Err(Error::AppRoot { .. })));
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

    fn temp_files(root: &AppRoot) -> Vec<String> {
        fs::read_dir(root.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(TEMP_PREFIX) && name.ends_with(TEMP_SUFFIX))
            .collect()
    }
}
```

- [ ] **Step 2: Write `crates/agent-profile/tests/config.rs`**

```rust
//! Configuration reads and locked atomic writes through the public API (SP1 design §8.3; spec §34
//! "Configuration"). The failed-replacement and reader-during-writes cases that need the crate-private
//! `update_with` hook are unit tests inside `config`.

use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use agent_profile::config::{self, AppRoot, Config};
use agent_profile::error::Error;

fn root() -> (tempfile::TempDir, AppRoot) {
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
fn valid_toml_reads_the_configured_executable() {
    let (_dir, root) = root();
    let exe = absolute("fake-agent");
    std::fs::write(root.config_path(), format!("[agents.fake]\nexecutable = {exe:?}\n")).unwrap();
    let config = Config::load(&root).unwrap();
    assert_eq!(config.agent_executable("fake"), Some(Path::new(&exe)));
}

#[test]
fn missing_file_is_an_empty_configuration() {
    let (_dir, root) = root();
    assert_eq!(Config::load(&root).unwrap(), Config::default());
}

#[test]
fn invalid_configurations_are_errors() {
    let (_dir, root) = root();
    for text in [
        "[agents\n",
        "[agents.fake]\nexecutable = 3\n",
        "[agents.fake]\nunknown = \"x\"\n",
        "[agents.fake]\nexecutable = \"relative/path\"\n",
    ] {
        std::fs::write(root.config_path(), text).unwrap();
        assert!(matches!(Config::load(&root), Err(Error::ConfigInvalid { .. })), "{text:?}");
    }
}

#[test]
fn update_refuses_a_corrupt_file_and_leaves_it_byte_identical() {
    let (_dir, root) = root();
    let corrupt = b"[agents.fake\n\xffexecutable".to_vec();
    std::fs::write(root.config_path(), &corrupt).unwrap();
    let error = config::update(&root, |doc| {
        set_agent(doc, "fake", &absolute("x"));
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(error, Error::ConfigInvalid { .. }), "{error:?}");
    assert_eq!(std::fs::read(root.config_path()).unwrap(), corrupt);
}

#[test]
fn update_preserves_comments_and_key_order() {
    let (_dir, root) = root();
    let original = format!(
        "# my agents\n[agents.zeta]\nexecutable = {:?} # trailing\n\n[agents.alpha]\n",
        absolute("zeta")
    );
    std::fs::write(root.config_path(), &original).unwrap();
    config::update(&root, |doc| {
        set_agent(doc, "fake", &absolute("fake"));
        Ok(())
    })
    .unwrap();
    let updated = std::fs::read_to_string(root.config_path()).unwrap();
    assert!(updated.starts_with(&original), "{updated}");
    assert!(updated.contains("# trailing"), "{updated}");
    let config = Config::load(&root).unwrap();
    assert_eq!(config.configured_agents().collect::<Vec<_>>(), ["alpha", "fake", "zeta"]);
}

#[test]
fn concurrent_writers_lose_no_update() {
    let (_dir, root) = root();
    let writers: Vec<_> = (0..8)
        .map(|index| {
            let root = root.clone();
            thread::spawn(move || {
                config::update(&root, |doc| {
                    set_agent(doc, &format!("agent{index}"), &absolute(&format!("a{index}")));
                    Ok(())
                })
                .unwrap();
            })
        })
        .collect();
    for writer in writers {
        writer.join().unwrap();
    }
    let config = Config::load(&root).unwrap();
    assert_eq!(config.configured_agents().count(), 8);
}

#[test]
fn a_waiting_writer_edits_the_first_writers_result() {
    let (_dir, root) = root();
    let (locked_tx, locked_rx) = mpsc::channel();
    let first = {
        let root = root.clone();
        thread::spawn(move || {
            config::update(&root, |doc| {
                set_agent(doc, "first", &absolute("first"));
                locked_tx.send(()).unwrap();
                thread::sleep(Duration::from_millis(300));
                Ok(())
            })
            .unwrap();
        })
    };
    locked_rx.recv().unwrap();
    config::update(&root, |doc| {
        assert!(doc.get("agents").and_then(|agents| agents.get("first")).is_some(), "{doc}");
        set_agent(doc, "second", &absolute("second"));
        Ok(())
    })
    .unwrap();
    first.join().unwrap();
    let config = Config::load(&root).unwrap();
    assert_eq!(config.configured_agents().collect::<Vec<_>>(), ["first", "second"]);
}

#[test]
fn an_edit_that_breaks_the_schema_is_refused() {
    let (_dir, root) = root();
    let error = config::update(&root, |doc| {
        doc["default"] = toml_edit::value("work");
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(error, Error::ConfigInvalid { .. }), "{error:?}");
    assert!(!root.config_path().exists());
}

#[cfg(windows)]
#[test]
fn windows_replace_blocked_by_an_open_handle_preserves_the_previous_file() {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;

    let (_dir, root) = root();
    let previous = format!("[agents.fake]\nexecutable = {:?}\n", absolute("old"));
    std::fs::write(root.config_path(), &previous).unwrap();
    let _holder = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(root.config_path())
        .unwrap();
    let error = config::update(&root, |doc| {
        set_agent(doc, "fake", &absolute("new"));
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(error, Error::ConfigWrite { .. }), "{error:?}");
    assert_eq!(std::fs::read_to_string(root.config_path()).unwrap(), previous);
    let leftovers: Vec<_> = std::fs::read_dir(root.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(".config.toml.") && name.ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}
```

- [ ] **Step 3: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib config::
cargo nextest run -p agent-profile --test config
```

Expected: every `config::tests::*` unit test passes (7 tests) and every `tests/config.rs` test passes (9 tests on Windows, 8 elsewhere), 0 failed.

- [ ] **Step 4: Run the gate** (see "Gate commands").

- [ ] **Step 5: Commit**

```bash
git add crates/agent-profile/src/config.rs crates/agent-profile/tests/config.rs
git commit -m "feat: add the application root and locked atomic configuration" -m "Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 6: Prove the tests are not vacuous**

In `crates/agent-profile/src/config.rs`, replace

```rust
                if !candidate.is_absolute() {
```

with

```rust
                if candidate.is_absolute() {
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib config::`. Expected: FAIL, and the failing tests include `schema_rejects_every_error_class_naming_the_key`. Then restore:

```bash
git checkout -- crates/agent-profile/src/config.rs
git status --short
```

Expected: `git status --short` prints nothing.

---

### Task 4: Executable discovery

**Design/spec:** design §7.2; V3 §20, §23.2

The explicit override, then a hand-rolled `PATH` search; `.bat` and `.cmd` are refused because std runs batch files through `cmd.exe`.

**Files:**
- Create: `crates/agent-profile/src/exe.rs` - Before: the file does not exist
- Modify (replace whole file): `crates/agent-profile/src/lib.rs` - Before: contains `pub mod error;` and does not contain `pub mod exe;`

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/exe.rs`**

```rust
//! Executable discovery (spec §20): an explicit configured override, then `PATH`.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, NotInstalledReason, Result};

/// Where a discovered executable came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Configured,
    Path,
}

/// A discovered executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub path: PathBuf,
    pub origin: Origin,
}

/// Finds `name` for `agent`: the `explicit` override when given, otherwise the first match on `path_var`.
/// A `.bat` or `.cmd` result is refused, because std would run it through `cmd.exe` (spec §23.2).
pub fn discover(
    agent: &str,
    name: &str,
    explicit: Option<&Path>,
    path_var: Option<&OsStr>,
) -> Result<Found> {
    let found = match explicit {
        Some(path) => {
            if !fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
                return Err(not_installed(
                    agent,
                    NotInstalledReason::ExplicitMissing(path.to_path_buf()),
                ));
            }
            Found { path: path.to_path_buf(), origin: Origin::Configured }
        }
        None => {
            let path = path_var
                .into_iter()
                .flat_map(std::env::split_paths)
                .filter(|dir| !dir.as_os_str().is_empty())
                .map(|dir| dir.join(file_name(name)))
                .find(|candidate| is_executable(candidate))
                .ok_or_else(|| not_installed(agent, NotInstalledReason::NotOnPath))?;
            Found { path, origin: Origin::Path }
        }
    };
    let is_batch = found
        .path
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|ext| ext.eq_ignore_ascii_case("bat") || ext.eq_ignore_ascii_case("cmd"));
    if is_batch {
        return Err(Error::UnsupportedExecutable { path: found.path });
    }
    Ok(found)
}

fn not_installed(agent: &str, reason: NotInstalledReason) -> Error {
    Error::AgentNotInstalled { agent: agent.to_owned(), reason, unknown_configured: Vec::new() }
}

fn file_name(name: &str) -> String {
    if cfg!(windows) { format!("{name}.exe") } else { name.to_owned() }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_file(dir: &Path, name: &str, executable: bool) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, b"x").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = if executable { 0o755 } else { 0o644 };
            fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        }
        #[cfg(windows)]
        let _ = executable;
        path
    }

    #[test]
    fn explicit_override_wins_and_must_be_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let exe = make_file(dir.path(), "agent-bin", true);
        let found = discover("fake", "fake-agent", Some(&exe), None).unwrap();
        assert_eq!(found, Found { path: exe, origin: Origin::Configured });

        let missing = dir.path().join("missing");
        let error = discover("fake", "fake-agent", Some(&missing), None).unwrap_err();
        assert!(matches!(
            error,
            Error::AgentNotInstalled { reason: NotInstalledReason::ExplicitMissing(ref p), .. } if *p == missing
        ));
        let error = discover("fake", "fake-agent", Some(dir.path()), None).unwrap_err();
        assert!(
            matches!(error, Error::AgentNotInstalled { .. }),
            "a directory is not an executable"
        );
    }

    #[test]
    fn path_search_takes_the_first_match_in_order() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let empty = tempfile::tempdir().unwrap();
        let name = file_name("tool");
        make_file(first.path(), &name, true);
        let expected = make_file(second.path(), &name, true);
        let path_var = std::env::join_paths([empty.path(), second.path(), first.path()]).unwrap();
        let found = discover("fake", "tool", None, Some(&path_var)).unwrap();
        assert_eq!(found, Found { path: expected, origin: Origin::Path });
    }

    #[test]
    fn missing_from_path_is_not_installed() {
        let empty = tempfile::tempdir().unwrap();
        let path_var = std::env::join_paths([empty.path()]).unwrap();
        for path_var in [Some(path_var.as_os_str()), Some(OsStr::new("")), None] {
            let error = discover("fake", "tool", None, path_var).unwrap_err();
            assert!(matches!(
                error,
                Error::AgentNotInstalled { reason: NotInstalledReason::NotOnPath, .. }
            ));
        }
    }

    #[cfg(unix)]
    #[test]
    fn path_search_skips_files_without_an_execute_bit() {
        let dir = tempfile::tempdir().unwrap();
        make_file(dir.path(), "tool", false);
        let path_var = std::env::join_paths([dir.path()]).unwrap();
        assert!(discover("fake", "tool", None, Some(&path_var)).is_err());
    }

    #[test]
    fn batch_files_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["agent.cmd", "agent.BAT", "agent.Cmd"] {
            let path = make_file(dir.path(), name, true);
            let error = discover("fake", "fake-agent", Some(&path), None).unwrap_err();
            assert!(matches!(error, Error::UnsupportedExecutable { .. }), "{name}");
        }
    }
}
```

- [ ] **Step 2: Write `crates/agent-profile/src/lib.rs`**

```rust
//! `agent-profile` — select and launch profiles for multiple coding agents.
//!
//! The authoritative design is `agent-profile-implementation-spec-v3.md` at the repository root.
//! Each module below is one layer of the spec §4 architecture. SP0 (the scaffold) creates them
//! empty; later sub-projects fill them in.
//!
//! > `agent-profile` owns profile selection. The coding agent owns authentication and
//! > agent-specific configuration. (spec §1)

pub mod adapter;
pub mod cli;
pub mod config;
pub mod error;
pub mod exe;
pub mod launch;
pub mod name;
pub mod output;
pub mod repo;
pub mod resolve;
```

- [ ] **Step 3: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib exe::
```

Expected: every `exe::tests::*` test passes (4 tests on Windows, 5 on Unix), 0 failed.

- [ ] **Step 4: Run the gate** (see "Gate commands").

- [ ] **Step 5: Commit**

```bash
git add crates/agent-profile/src/exe.rs crates/agent-profile/src/lib.rs
git commit -m "feat: add executable discovery" -m "Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 6: Prove the tests are not vacuous**

In `crates/agent-profile/src/exe.rs`, replace

```rust
    if is_batch {
```

with

```rust
    if false && is_batch {
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib exe::`. Expected: FAIL, and the failing tests include `batch_files_are_refused`. Then restore:

```bash
git checkout -- crates/agent-profile/src/exe.rs
git status --short
```

Expected: `git status --short` prints nothing.

---

### Task 5: fake-agent fixture extensions and shared test helpers

**Design/spec:** design §8.1; SP0 design §3.3, §3.4

Adds the `pid` report key and the `FAKE_AGENT_STDIN`, `FAKE_AGENT_STDERR`, `FAKE_AGENT_SLEEP_MS`, `FAKE_AGENT_SPAWN_SLEEPER` and `FAKE_AGENT_CTRL_C_EXIT` control variables; grows `tests/support/mod.rs` with `FIXTURE_VARS`, `WRAPPER_VARS`, `Root`, `non_utf8` and `report`; and adds the fixture smoke tests. The two SP0 stub-CLI smoke tests stay unchanged in this task; Task 9 replaces them together with the CLI.

**Files:**
- Modify (replace whole file): `crates/agent-profile/src/bin/fake-agent.rs` - Before: does not contain `FAKE_AGENT_SLEEP_MS`
- Modify (replace whole file): `crates/agent-profile/tests/support/mod.rs` - Before: does not contain `FIXTURE_VARS`
- Modify (replace whole file): `crates/agent-profile/tests/smoke.rs` - Before: does not contain `fake_agent_reports_its_pid`

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/bin/fake-agent.rs`**

```rust
//! Test-only stand-in for a coding agent (SP0 design §3.3, SP1 design §8.1).
//!
//! Prints one JSON object `{"argv": [...], "cwd": "...", "env": {...}, "pid": N}` to stdout and exits with
//! the code in `FAKE_AGENT_EXIT` (default 0). `env` holds only the variables named in the
//! comma-separated `FAKE_AGENT_ECHO_ENV`. Any fixture error — a non-UTF-8 argument, cwd or echoed
//! value, or an invalid control variable — prints nothing to stdout and exits 125.
//!
//! Control variables added in SP1:
//! - `FAKE_AGENT_STDIN=1`: read all of stdin first and report it as `"stdin"`.
//! - `FAKE_AGENT_STDERR=<text>`: write `<text>` to stderr after the report.
//! - `FAKE_AGENT_SLEEP_MS=<u64>`: after the report, sleep that long before exiting.
//! - `FAKE_AGENT_SPAWN_SLEEPER=<u64>`: spawn a detached copy of itself that only sleeps that long, and
//!   report its PID as `"sleeper_pid"`.
//! - `FAKE_AGENT_CTRL_C_EXIT=<u8>` (Windows only): on Ctrl-C or Ctrl-Break, sleep 300 ms, then exit with
//!   `<u8>`. The handler is installed before the report is printed.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

/// Reserved for fixture errors, so a test can tell them apart from a requested exit code.
const FIXTURE_ERROR: u8 = 125;

/// Every control variable, so a spawned sleeper can be given a clean environment.
const CONTROL_VARS: [&str; 7] = [
    "FAKE_AGENT_EXIT",
    "FAKE_AGENT_ECHO_ENV",
    "FAKE_AGENT_STDIN",
    "FAKE_AGENT_STDERR",
    "FAKE_AGENT_SLEEP_MS",
    "FAKE_AGENT_SPAWN_SLEEPER",
    "FAKE_AGENT_CTRL_C_EXIT",
];

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("fake-agent: {message}");
            ExitCode::from(FIXTURE_ERROR)
        }
    }
}

/// Builds the whole report before printing, so an error never leaves partial output.
fn run() -> Result<u8, String> {
    let code = parsed::<u8>("FAKE_AGENT_EXIT")?.unwrap_or(0);
    let sleep_ms = parsed::<u64>("FAKE_AGENT_SLEEP_MS")?;
    let sleeper_ms = parsed::<u64>("FAKE_AGENT_SPAWN_SLEEPER")?;
    let ctrl_c_exit = parsed::<u8>("FAKE_AGENT_CTRL_C_EXIT")?;
    let read_stdin = match std::env::var_os("FAKE_AGENT_STDIN") {
        None => false,
        Some(value) if value == "1" => true,
        Some(value) => return Err(format!("FAKE_AGENT_STDIN must be 1, got {value:?}")),
    };
    let stderr_text =
        std::env::var_os("FAKE_AGENT_STDERR").map(|v| utf8(v, "FAKE_AGENT_STDERR")).transpose()?;

    let argv = std::env::args_os()
        .skip(1)
        .map(|arg| utf8(arg, "argument"))
        .collect::<Result<Vec<_>, _>>()?;

    let cwd =
        std::env::current_dir().map_err(|e| format!("cannot read the current directory: {e}"))?;
    let cwd = utf8(cwd.into_os_string(), "current directory")?;

    let mut env = BTreeMap::new();
    if let Some(list) = std::env::var_os("FAKE_AGENT_ECHO_ENV") {
        let list = utf8(list, "FAKE_AGENT_ECHO_ENV")?;
        for name in list.split(',').filter(|name| !name.is_empty()) {
            if let Some(value) = std::env::var_os(name) {
                env.insert(name.to_owned(), utf8(value, name)?);
            }
        }
    }

    if let Some(exit) = ctrl_c_exit {
        install_ctrl_c_exit(exit)?;
    }

    let stdin = if read_stdin {
        let mut bytes = Vec::new();
        std::io::stdin().read_to_end(&mut bytes).map_err(|e| format!("cannot read stdin: {e}"))?;
        Some(String::from_utf8(bytes).map_err(|_| "stdin is not valid UTF-8".to_owned())?)
    } else {
        None
    };

    let mut report = serde_json::json!({
        "argv": argv,
        "cwd": cwd,
        "env": env,
        "pid": std::process::id(),
    });
    if let Some(stdin) = stdin {
        report["stdin"] = stdin.into();
    }
    if let Some(ms) = sleeper_ms {
        report["sleeper_pid"] = spawn_sleeper(ms)?.into();
    }

    let mut stdout = std::io::stdout();
    writeln!(stdout, "{report}")
        .and_then(|()| stdout.flush())
        .map_err(|e| format!("stdout: {e}"))?;
    if let Some(text) = stderr_text {
        let mut stderr = std::io::stderr();
        let _ = stderr.write_all(text.as_bytes());
        let _ = stderr.flush();
    }
    if let Some(ms) = sleep_ms {
        std::thread::sleep(Duration::from_millis(ms));
    }
    Ok(code)
}

/// Parses an optional control variable. Unset means `None`; anything unparsable is a fixture error.
fn parsed<T: std::str::FromStr>(name: &str) -> Result<Option<T>, String> {
    match std::env::var_os(name) {
        None => Ok(None),
        Some(value) => value
            .to_str()
            .and_then(|s| s.parse::<T>().ok())
            .map(Some)
            .ok_or_else(|| format!("{name} has an invalid value {value:?}")),
    }
}

fn utf8(value: OsString, what: &str) -> Result<String, String> {
    value.into_string().map_err(|raw| format!("{what} is not valid UTF-8: {raw:?}"))
}

fn spawn_sleeper(ms: u64) -> Result<u32, String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate fake-agent: {e}"))?;
    let child =
        sleeper_command(exe, ms).spawn().map_err(|e| format!("cannot spawn the sleeper: {e}"))?;
    Ok(child.id())
}

/// The sleeper gets no control variable except its sleep, so it never spawns another sleeper.
fn sleeper_command(exe: std::path::PathBuf, ms: u64) -> Command {
    let mut command = Command::new(exe);
    for name in CONTROL_VARS {
        command.env_remove(name);
    }
    command
        .env("FAKE_AGENT_SLEEP_MS", ms.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

#[cfg(windows)]
fn install_ctrl_c_exit(exit: u8) -> Result<(), String> {
    use std::sync::atomic::{AtomicU8, Ordering};
    use windows_sys::Win32::Foundation::TRUE;
    use windows_sys::Win32::System::Console::{
        CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler,
    };
    use windows_sys::core::BOOL;

    static EXIT: AtomicU8 = AtomicU8::new(0);

    unsafe extern "system" fn handler(event: u32) -> BOOL {
        if event == CTRL_C_EVENT || event == CTRL_BREAK_EVENT {
            std::thread::sleep(Duration::from_millis(300));
            std::process::exit(i32::from(EXIT.load(Ordering::SeqCst)));
        }
        0
    }

    EXIT.store(exit, Ordering::SeqCst);
    // SAFETY: `handler` is a valid `extern "system"` routine for the life of the process.
    if unsafe { SetConsoleCtrlHandler(Some(handler), TRUE) } == 0 {
        return Err(format!(
            "cannot install the console handler: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
fn install_ctrl_c_exit(_exit: u8) -> Result<(), String> {
    Err("FAKE_AGENT_CTRL_C_EXIT is supported only on Windows".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn sleeper_environment_removes_every_control_variable_except_its_sleep() {
        let command = sleeper_command("fake-agent".into(), 250);
        let envs: BTreeMap<&OsStr, Option<&OsStr>> = command.get_envs().collect();
        for name in CONTROL_VARS {
            let expected =
                if name == "FAKE_AGENT_SLEEP_MS" { Some(OsStr::new("250")) } else { None };
            assert_eq!(envs.get(OsStr::new(name)), Some(&expected), "{name}");
        }
    }
}
```

- [ ] **Step 2: Write `crates/agent-profile/tests/support/mod.rs`**

```rust
//! Shared helpers for integration tests (SP0 design §3.4, SP1 design §8).
//!
//! Each test file includes this module with `mod support;` and uses only part of it.
#![allow(dead_code)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every `fake-agent` control variable (SP1 design §8.1).
pub const FIXTURE_VARS: [&str; 7] = [
    "FAKE_AGENT_EXIT",
    "FAKE_AGENT_ECHO_ENV",
    "FAKE_AGENT_STDIN",
    "FAKE_AGENT_STDERR",
    "FAKE_AGENT_SLEEP_MS",
    "FAKE_AGENT_SPAWN_SLEEPER",
    "FAKE_AGENT_CTRL_C_EXIT",
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
        Root { dir: tempfile::tempdir().unwrap() }
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

    /// `agent-profile` with this root, a clean fixture and wrapper environment, and the given args.
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
        command.args(args.into_iter().map(Into::into));
        command
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

- [ ] **Step 3: Write `crates/agent-profile/tests/smoke.rs`**

```rust
//! Smoke tests: the binary's top-level contract and the fake-agent fixture's contract.

mod support;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn agent_profile(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-profile")).args(args).output().unwrap()
}

fn stdout_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

/// Asserts the fake-agent fixture-error contract: exit 125, nothing on stdout, a message on stderr.
fn assert_fixture_error(output: &Output, context: &str) {
    assert_eq!(output.status.code(), Some(125), "{context}");
    assert!(output.stdout.is_empty(), "{context}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("fake-agent: "), "{context}: {stderr}");
}

#[test]
fn version_exits_zero() {
    let output = agent_profile(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "{stdout}");
}

#[test]
fn help_exits_zero_and_says_scaffold() {
    let output = agent_profile(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("SP0 scaffold"), "{stdout}");
}

#[test]
fn unimplemented_invocation_is_usage_error() {
    for args in [&["doctor"][..], &["claude", "work"], &["zzz-unknown"]] {
        let output = agent_profile(args);
        assert_eq!(output.status.code(), Some(2), "args {args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("not yet implemented"), "args {args:?}: {stderr}");
    }
}

#[test]
fn fake_agent_echoes_argv_exactly() {
    let args = ["--foo", "bar", "--", "a b"];
    let output = support::fake_agent().args(args).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["argv"], serde_json::json!(args));
}

#[test]
fn fake_agent_exit_code_is_controllable() {
    let output = support::fake_agent().env("FAKE_AGENT_EXIT", "7").output().unwrap();
    assert_eq!(output.status.code(), Some(7));
    // A requested non-zero exit still prints the full report.
    assert_eq!(stdout_json(&output)["argv"], serde_json::json!([]));
}

#[test]
fn fake_agent_echoes_cwd() {
    // Not the package directory, which is where the test runner already starts the child.
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let output = support::fake_agent().current_dir(&dir).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    let cwd = stdout_json(&output)["cwd"].as_str().map(PathBuf::from).unwrap();
    assert_eq!(cwd.canonicalize().unwrap(), dir.canonicalize().unwrap());
}

#[test]
fn fake_agent_echoes_only_listed_env() {
    let output = support::fake_agent()
        .env("FAKE_AGENT_ECHO_ENV", "AP_TEST_SET,,AP_TEST_UNSET")
        .env("AP_TEST_SET", "yes")
        .env("AP_TEST_NOT_LISTED", "no")
        .env_remove("AP_TEST_UNSET")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["env"], serde_json::json!({ "AP_TEST_SET": "yes" }));
}

#[test]
fn fake_agent_env_is_empty_without_echo_list() {
    let output = support::fake_agent().env_remove("FAKE_AGENT_ECHO_ENV").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["env"], serde_json::json!({}));
}

#[test]
fn fake_agent_invalid_exit_code_is_fixture_error() {
    for value in ["", "256", "-1", "seven"] {
        let output = support::fake_agent().env("FAKE_AGENT_EXIT", value).output().unwrap();
        assert_fixture_error(&output, &format!("FAKE_AGENT_EXIT={value:?}"));
    }
}

#[test]
fn fake_agent_non_utf8_argument_is_fixture_error() {
    let output = support::fake_agent().arg(support::non_utf8()).output().unwrap();
    assert_fixture_error(&output, "non-UTF-8 argument");
}

#[test]
fn fake_agent_non_utf8_env_value_is_fixture_error() {
    let output = support::fake_agent()
        .env("FAKE_AGENT_ECHO_ENV", "AP_TEST_NON_UTF8")
        .env("AP_TEST_NON_UTF8", support::non_utf8())
        .output()
        .unwrap();
    assert_fixture_error(&output, "non-UTF-8 echoed env value");
}

#[test]
fn fake_agent_helper_clears_fixture_env() {
    let command = support::fake_agent();
    let envs: BTreeMap<&OsStr, Option<&OsStr>> = command.get_envs().collect();
    for name in support::FIXTURE_VARS {
        assert_eq!(envs.get(OsStr::new(name)), Some(&None), "{name} must be removed");
    }
}

#[test]
fn fake_agent_reports_its_pid() {
    let child = support::fake_agent().stdout(std::process::Stdio::piped()).spawn().unwrap();
    let pid = child.id();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["pid"], serde_json::json!(pid));
}

#[test]
fn fake_agent_reads_stdin_when_asked() {
    use std::io::Write;
    let mut child = support::fake_agent()
        .env("FAKE_AGENT_STDIN", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"line one\nline two").unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["stdin"], serde_json::json!("line one\nline two"));
}

#[test]
fn fake_agent_writes_requested_stderr_after_the_report() {
    let output = support::fake_agent().env("FAKE_AGENT_STDERR", "to stderr").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"to stderr");
    assert!(stdout_json(&output).get("argv").is_some());
}

#[test]
fn fake_agent_sleeps_after_reporting() {
    let started = std::time::Instant::now();
    let output = support::fake_agent().env("FAKE_AGENT_SLEEP_MS", "300").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(started.elapsed() >= std::time::Duration::from_millis(300));
    assert!(stdout_json(&output).get("pid").is_some());
}

#[test]
fn fake_agent_reports_a_distinct_sleeper_pid() {
    let output = support::fake_agent().env("FAKE_AGENT_SPAWN_SLEEPER", "200").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    let report = stdout_json(&output);
    let sleeper = report["sleeper_pid"].as_u64().unwrap();
    assert_ne!(Some(sleeper), report["pid"].as_u64());
}

#[test]
fn fake_agent_invalid_control_values_are_fixture_errors() {
    for (name, value) in [
        ("FAKE_AGENT_STDIN", "yes"),
        ("FAKE_AGENT_SLEEP_MS", "-1"),
        ("FAKE_AGENT_SLEEP_MS", "soon"),
        ("FAKE_AGENT_SPAWN_SLEEPER", ""),
        ("FAKE_AGENT_CTRL_C_EXIT", "256"),
    ] {
        let output = support::fake_agent().env(name, value).output().unwrap();
        assert_fixture_error(&output, &format!("{name}={value:?}"));
    }
}

#[cfg(unix)]
#[test]
fn fake_agent_ctrl_c_exit_is_windows_only() {
    let output = support::fake_agent().env("FAKE_AGENT_CTRL_C_EXIT", "42").output().unwrap();
    assert_fixture_error(&output, "FAKE_AGENT_CTRL_C_EXIT on Unix");
}
```

- [ ] **Step 4: Run the task checks**

```bash
cargo nextest run -p agent-profile --bin fake-agent
cargo nextest run -p agent-profile --test smoke
```

Expected: the `fake-agent` unit test passes (1 test) and every smoke test passes (18 tests on Windows, 19 on Unix), 0 failed.

- [ ] **Step 5: Run the gate** (see "Gate commands").

- [ ] **Step 6: Commit**

```bash
git add crates/agent-profile/src/bin/fake-agent.rs crates/agent-profile/tests/support/mod.rs crates/agent-profile/tests/smoke.rs
git commit -m "test: extend the fake-agent fixture and the shared test helpers" -m "Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 7: Prove the tests are not vacuous**

In `crates/agent-profile/src/bin/fake-agent.rs`, replace

```rust
    for name in CONTROL_VARS {
        command.env_remove(name);
    }
```

with

```rust
    for name in CONTROL_VARS.iter().take(0) {
        command.env_remove(name);
    }
```

Run `cargo nextest run --no-fail-fast -p agent-profile --bin fake-agent`. Expected: FAIL, and the failing tests include `sleeper_environment_removes_every_control_variable_except_its_sleep`. Then restore:

```bash
git checkout -- crates/agent-profile/src/bin/fake-agent.rs
git status --short
```

Expected: `git status --short` prints nothing.

---

### Task 6: LaunchPlan and the Unix and Windows launchers

**Design/spec:** design §7.1, §7.5, §7.6, §8.4 (`tests/launch_plan.rs`); V3 §4, §22, §23, §25

`LaunchPlan::command` is the one conversion both launchers use. Unix replaces the process with `exec`; Windows joins a kill-on-close job, installs the control handler, spawns, publishes the job and a `SYNCHRONIZE` duplicate of the child handle, waits, releases the job and returns the full exit code. The debug-only pause hook is compiled in only under `debug_assertions`. Its behaviour tests arrive in Task 10, once the CLI can launch.

**Files:**
- Modify (replace whole file): `crates/agent-profile/src/launch/mod.rs` - Before: contains only `//!` doc-comment lines (the SP0 empty module)
- Create: `crates/agent-profile/src/launch/unix.rs` - Before: the file does not exist
- Create: `crates/agent-profile/src/launch/windows.rs` - Before: the file does not exist
- Create: `crates/agent-profile/tests/launch_plan.rs` - Before: the file does not exist

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/launch/mod.rs`**

```rust
//! `LaunchPlan` (spec §4), environment overrides (spec §22), launch semantics (spec §23, §24)
//! and the launcher API (spec §25).

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

use crate::error::Result;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

/// What an adapter asks the launcher to run (spec §4). `env` is an override set, not a full environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    pub executable: PathBuf,
    pub args: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
    pub cwd: Option<PathBuf>,
}

impl LaunchPlan {
    /// The single conversion both launchers use: the inherited environment plus the overrides, the
    /// arguments verbatim, the working directory only when set, and inherited stdio.
    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command.args(&self.args);
        for (key, value) in &self.env {
            command.env(key, value);
        }
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        command
    }
}

/// How a launch ended (spec §25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchOutcome {
    /// Unix: `exec` never returns on success, so this is never constructed there. Kept for spec §25.
    ReplacedProcess,
    /// Windows: the child's full 32-bit exit code.
    Exited(i32),
}

/// Launches the plan: `exec` on Unix, a waited child on Windows. `verbose` enables diagnostics on stderr.
pub fn launch(plan: &LaunchPlan, verbose: bool) -> Result<LaunchOutcome> {
    #[cfg(unix)]
    {
        let _ = verbose;
        unix::launch(plan)
    }
    #[cfg(windows)]
    {
        windows::launch(plan, verbose)
    }
}
```

- [ ] **Step 2: Write `crates/agent-profile/src/launch/unix.rs`**

```rust
//! Unix launcher: process replacement (spec §23.1).

use std::os::unix::process::CommandExt;

use super::{LaunchOutcome, LaunchPlan};
use crate::error::{Error, Result};

/// Replaces this process with the agent. Returns only when `exec` fails.
pub(super) fn launch(plan: &LaunchPlan) -> Result<LaunchOutcome> {
    let source = plan.command().exec();
    Err(Error::Launch { executable: plan.executable.clone(), source })
}
```

- [ ] **Step 3: Write `crates/agent-profile/src/launch/windows.rs`**

```rust
//! Windows launcher: a direct child process with console control-event handling and a job object so an
//! interrupted wrapper leaves no orphan (spec §23.2, §24; design §7.6).

use std::io::{self, Write};
use std::mem::{size_of, zeroed};
use std::os::windows::io::AsRawHandle;
use std::ptr::null;
use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{
    DUPLICATE_HANDLE_OPTIONS, DuplicateHandle, FALSE, HANDLE, TRUE,
};
use windows_sys::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows_sys::Win32::System::Console::{
    CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT,
    SetConsoleCtrlHandler,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, INFINITE, WaitForSingleObject};
use windows_sys::core::BOOL;

use super::{LaunchOutcome, LaunchPlan};
use crate::error::{Error, Result};

/// Handles the console control handler needs, as raw values (raw handles are not `Send`/`Sync`).
struct Published {
    job: usize,
    child: usize,
}

static PUBLISHED: OnceLock<Published> = OnceLock::new();

/// Debug-only hook: sleep this many milliseconds immediately before spawning (design §8.4).
#[cfg(debug_assertions)]
const PAUSE_ENV: &str = "AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS";

pub(super) fn launch(plan: &LaunchPlan, verbose: bool) -> Result<LaunchOutcome> {
    let launch_error =
        |source: io::Error| Error::Launch { executable: plan.executable.clone(), source };

    // 1. Job: the wrapper joins a kill-on-close job, so its children die if it is killed.
    // SAFETY: plain Win32 calls with valid arguments; the job handle stays open until process exit.
    let job = unsafe { CreateJobObjectW(null(), null()) };
    if job.is_null() {
        return Err(launch_error(io::Error::last_os_error()));
    }
    if !set_kill_on_close(job, true) {
        return Err(launch_error(io::Error::last_os_error()));
    }
    // SAFETY: `job` is a valid job handle and `GetCurrentProcess` is always valid.
    if unsafe { AssignProcessToJobObject(job, GetCurrentProcess()) } == FALSE {
        return Err(launch_error(io::Error::last_os_error()));
    }

    // 2. Handler: survive Ctrl-C and Ctrl-Break; the agent shares the console and handles them itself.
    // SAFETY: `handler` is a valid `extern "system"` routine for the life of the process.
    if unsafe { SetConsoleCtrlHandler(Some(handler), TRUE) } == FALSE {
        return Err(launch_error(io::Error::last_os_error()));
    }

    #[cfg(debug_assertions)]
    if let Some(ms) = std::env::var_os(PAUSE_ENV).and_then(|v| v.to_str()?.parse::<u64>().ok()) {
        let mut stderr = io::stderr();
        let _ = writeln!(stderr, "agent-profile: debug: paused before spawn");
        let _ = stderr.flush();
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }

    // 3. Spawn directly and publish the handles for the handler.
    let mut child = plan.command().spawn().map_err(launch_error)?;
    let mut duplicate: HANDLE = std::ptr::null_mut();
    // SAFETY: duplicates the live child handle into this process with SYNCHRONIZE access only.
    let duplicated = unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            child.as_raw_handle() as HANDLE,
            GetCurrentProcess(),
            &mut duplicate,
            SYNCHRONIZE,
            FALSE,
            0 as DUPLICATE_HANDLE_OPTIONS,
        )
    };
    if duplicated != FALSE {
        let _ = PUBLISHED.set(Published { job: job as usize, child: duplicate as usize });
    }

    // 4. Wait.
    let status = child.wait().map_err(launch_error)?;

    // 5. Release the job so processes the agent left running survive the wrapper's exit.
    if !set_kill_on_close(job, false) && verbose {
        let _ = writeln!(
            io::stderr(),
            "agent-profile: could not release the job object: {}",
            io::Error::last_os_error()
        );
    }

    // 6. The child's full exit code; `main` exits with it.
    Ok(LaunchOutcome::Exited(status.code().unwrap_or(1)))
}

fn set_kill_on_close(job: HANDLE, kill: bool) -> bool {
    // SAFETY: an all-zero JOBOBJECT_EXTENDED_LIMIT_INFORMATION is a valid "no limits" value.
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    if kill {
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    }
    // SAFETY: `info` is a correctly sized, initialised structure for this information class.
    unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) != FALSE
    }
}

unsafe extern "system" fn handler(event: u32) -> BOOL {
    match event {
        CTRL_C_EVENT | CTRL_BREAK_EVENT => TRUE,
        CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT => match PUBLISHED.get() {
            Some(published) => {
                // SAFETY: `child` is an owned SYNCHRONIZE duplicate that is never closed.
                unsafe { WaitForSingleObject(published.child as HANDLE, INFINITE) };
                set_kill_on_close(published.job as HANDLE, false);
                // Never return: returning lets Windows terminate the wrapper before `main` exits with the
                // agent's code. Windows' own close timeout remains the backstop.
                loop {
                    std::thread::park();
                }
            }
            None => FALSE,
        },
        _ => FALSE,
    }
}
```

- [ ] **Step 4: Write `crates/agent-profile/tests/launch_plan.rs`**

```rust
//! `LaunchPlan::command` pinned independently of any adapter (SP1 design §8.4; spec §34 "LaunchPlan").

mod support;

use std::path::PathBuf;

use agent_profile::launch::LaunchPlan;

fn plan(cwd: Option<PathBuf>) -> LaunchPlan {
    LaunchPlan {
        executable: PathBuf::from(env!("CARGO_BIN_EXE_fake-agent")),
        args: vec!["--".into(), "a b".into(), "--flag".into()],
        env: vec![
            // Cargo and nextest set both variables in every test process; the plan overrides one.
            ("CARGO_MANIFEST_DIR".into(), "from-plan".into()),
            ("FAKE_AGENT_ECHO_ENV".into(), "CARGO_MANIFEST_DIR,CARGO_PKG_NAME".into()),
        ],
        cwd,
    }
}

fn run(plan: &LaunchPlan) -> serde_json::Value {
    let mut command = plan.command();
    for name in support::FIXTURE_VARS {
        if name != "FAKE_AGENT_ECHO_ENV" {
            command.env_remove(name);
        }
    }
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    support::report(&output.stdout)
}

#[test]
fn command_applies_cwd_env_overrides_and_args() {
    assert!(
        std::env::var_os("CARGO_MANIFEST_DIR").is_some()
            && std::env::var_os("CARGO_PKG_NAME").is_some()
    );
    let dir = tempfile::tempdir().unwrap();
    let report = run(&plan(Some(dir.path().to_path_buf())));
    assert_eq!(report["argv"], serde_json::json!(["--", "a b", "--flag"]));
    assert_eq!(
        report["env"],
        serde_json::json!({ "CARGO_MANIFEST_DIR": "from-plan", "CARGO_PKG_NAME": env!("CARGO_PKG_NAME") })
    );
    let cwd = PathBuf::from(report["cwd"].as_str().unwrap());
    assert_eq!(cwd.canonicalize().unwrap(), dir.path().canonicalize().unwrap());
}

#[test]
fn command_without_cwd_inherits_the_working_directory() {
    let report = run(&plan(None));
    let cwd = PathBuf::from(report["cwd"].as_str().unwrap());
    assert_eq!(
        cwd.canonicalize().unwrap(),
        std::env::current_dir().unwrap().canonicalize().unwrap()
    );
}
```

- [ ] **Step 5: Run the task checks**

```bash
cargo nextest run -p agent-profile --test launch_plan
```

Expected: both `tests/launch_plan.rs` tests pass, 0 failed.

- [ ] **Step 6: Run the gate** (see "Gate commands").

- [ ] **Step 7: Commit**

```bash
git add crates/agent-profile/src/launch/mod.rs crates/agent-profile/src/launch/unix.rs crates/agent-profile/src/launch/windows.rs crates/agent-profile/tests/launch_plan.rs
git commit -m "feat: add LaunchPlan and the Unix and Windows launchers" -m "Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 8: Prove the tests are not vacuous**

In `crates/agent-profile/src/launch/mod.rs`, replace

```rust
        if let Some(cwd) = &self.cwd {
```

with

```rust
        if let Some(cwd) = self.cwd.as_ref().filter(|_| false) {
```

Run `cargo nextest run --no-fail-fast -p agent-profile --test launch_plan`. Expected: FAIL, and the failing tests include `command_applies_cwd_env_overrides_and_args`. Then restore:

```bash
git checkout -- crates/agent-profile/src/launch/mod.rs
git status --short
```

Expected: `git status --short` prints nothing.

---

### Task 7: Resolution stub and the debug-only fake adapter

**Design/spec:** design §5 (`resolve`, `adapter`), §5.1 step 5, §7.3; V3 §8, §9, §9.1, §12

The V3 §12 `Resolution` types with an explicit-only stub, and `adapter::plan` with one arm, `fake`, compiled only under `debug_assertions`: discovery, then the case-only-twin check, then the plan with the `FAKE_AGENT_HOME` override. `ensure_profile_dir` does the §9.1 lazy creation.

**Files:**
- Modify (replace whole file): `crates/agent-profile/src/resolve.rs` - Before: contains only `//!` doc-comment lines (the SP0 empty module)
- Modify (replace whole file): `crates/agent-profile/src/adapter/mod.rs` - Before: contains only `//!` doc-comment lines (the SP0 empty module)

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/resolve.rs`**

```rust
//! The single profile resolver shared by every command (spec §12). SP1 resolves only an explicit
//! profile; SP3 replaces the body of `resolve`, not the types.

use std::path::PathBuf;

use crate::name::{AgentId, ProfileName};

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

/// SP1 stub: an explicit profile resolves as `Explicit`; anything else resolves to `None`.
pub fn resolve(agent: AgentId, explicit: Option<ProfileName>) -> Resolution {
    let source =
        if explicit.is_some() { ResolutionSource::Explicit } else { ResolutionSource::None };
    Resolution { agent, profile: explicit, source, repository: None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::name::Platform;

    #[test]
    fn explicit_profile_resolves_as_explicit() {
        let agent = AgentId::parse("fake").unwrap();
        let work = ProfileName::parse("work", Platform::host()).unwrap();
        let resolution = resolve(agent.clone(), Some(work.clone()));
        assert_eq!(
            resolution,
            Resolution {
                agent: agent.clone(),
                profile: Some(work),
                source: ResolutionSource::Explicit,
                repository: None
            }
        );
        let none = resolve(agent.clone(), None);
        assert_eq!(none.source, ResolutionSource::None);
        assert_eq!(none.profile, None);
    }
}
```

- [ ] **Step 2: Write `crates/agent-profile/src/adapter/mod.rs`**

```rust
//! Agent adapters: supported agents (spec §2), capability semantics (spec §3), argument
//! conflicts (spec §21) and evidence metadata (spec §28).
//!
//! SP1 has no adapter trait (SP2 designs it) and one test-only agent, `fake`, compiled only when debug
//! assertions are on, so release builds contain no agent at all (design D1).

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::config::{AppRoot, Config};
use crate::error::{Error, Result};
use crate::exe::Origin;
use crate::launch::LaunchPlan;
use crate::name::{AgentId, ProfileName};
use crate::resolve::Resolution;

/// A launch plus what dry run reports but the launcher does not need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedLaunch {
    pub plan: LaunchPlan,
    pub profile: ProfileName,
    pub profile_dir: PathBuf,
    pub profile_dir_exists: bool,
    pub executable_origin: Origin,
    pub mechanism: String,
    /// Override variables whose values must never be printed (spec §22).
    pub sensitive_env: Vec<OsString>,
}

/// The agents this build knows.
pub fn known_agents() -> Vec<&'static str> {
    #[cfg(debug_assertions)]
    let agents = vec!["fake"];
    #[cfg(not(debug_assertions))]
    let agents = Vec::new();
    agents
}

/// Builds the launch for an explicitly resolved profile (design §5.1 step 5).
pub fn plan(
    agent: &AgentId,
    resolution: &Resolution,
    root: &AppRoot,
    config: &Config,
    args: Vec<OsString>,
    path_var: Option<&OsStr>,
) -> Result<PlannedLaunch> {
    let profile =
        resolution.profile.clone().ok_or_else(|| Error::NoProfile { agent: agent.to_string() })?;
    match agent.as_str() {
        #[cfg(debug_assertions)]
        "fake" => fake::plan(profile, root, config, args, path_var),
        other => {
            // Release builds know no agent, so these inputs are unused there.
            let _ = (profile, root, config, args, path_var);
            Err(Error::UnknownAgent {
                agent: other.to_owned(),
                known: known_agents().into_iter().map(String::from).collect(),
                unknown_configured: Vec::new(),
            })
        }
    }
}

/// Refuses a profile whose name differs from an existing `profiles/` entry only in ASCII case (design §7.3).
#[cfg_attr(not(debug_assertions), allow(dead_code))]
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

/// Lazily creates the profile directory (spec §9, §9.1): idempotent, and verified to be a directory.
pub fn ensure_profile_dir(planned: &PlannedLaunch) -> Result<()> {
    let dir = &planned.profile_dir;
    let error = |source: io::Error| Error::ProfileDir { path: dir.clone(), source };
    fs::create_dir_all(dir).map_err(error)?;
    if fs::metadata(dir).map_err(error)?.is_dir() {
        Ok(())
    } else {
        Err(error(io::Error::other("the path exists but is not a directory")))
    }
}

#[cfg(debug_assertions)]
mod fake {
    use super::*;
    use crate::exe;

    pub(super) const HOME_VAR: &str = "FAKE_AGENT_HOME";

    pub(super) fn plan(
        profile: ProfileName,
        root: &AppRoot,
        config: &Config,
        args: Vec<OsString>,
        path_var: Option<&OsStr>,
    ) -> Result<PlannedLaunch> {
        let found = exe::discover("fake", "fake-agent", config.agent_executable("fake"), path_var)?;
        check_case_twins(root, &profile)?;
        let profile_dir = root.profiles_dir().join(profile.as_str()).join("fake");
        let profile_dir_exists = profile_dir.is_dir();
        Ok(PlannedLaunch {
            plan: LaunchPlan {
                executable: found.path,
                args,
                env: vec![(HOME_VAR.into(), profile_dir.clone().into_os_string())],
                cwd: None,
            },
            profile,
            profile_dir,
            profile_dir_exists,
            executable_origin: found.origin,
            mechanism: format!("environment variable {HOME_VAR}"),
            sensitive_env: Vec::new(),
        })
    }
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use crate::name::Platform;
    use crate::resolve::resolve;

    fn setup() -> (tempfile::TempDir, AppRoot, Config, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = AppRoot::from_path(dir.path().join("root"));
        let exe = dir.path().join("fake-agent-bin");
        fs::write(&exe, b"x").unwrap();
        fs::create_dir_all(root.path()).unwrap();
        fs::write(
            root.config_path(),
            format!("[agents.fake]\nexecutable = {:?}\n", exe.to_str().unwrap()),
        )
        .unwrap();
        let config = Config::load(&root).unwrap();
        (dir, root, config, exe)
    }

    fn resolution(profile: &str) -> Resolution {
        resolve(
            AgentId::parse("fake").unwrap(),
            Some(ProfileName::parse(profile, Platform::host()).unwrap()),
        )
    }

    #[test]
    fn fake_plan_sets_home_override_and_passes_args_verbatim() {
        let (_dir, root, config, exe) = setup();
        let args: Vec<OsString> = vec!["--foo".into(), "a b".into()];
        let planned = plan(
            &AgentId::parse("fake").unwrap(),
            &resolution("work"),
            &root,
            &config,
            args.clone(),
            None,
        )
        .unwrap();
        let dir = root.profiles_dir().join("work").join("fake");
        assert_eq!(
            planned.plan,
            LaunchPlan {
                executable: exe,
                args,
                env: vec![("FAKE_AGENT_HOME".into(), dir.clone().into_os_string())],
                cwd: None
            }
        );
        assert_eq!(planned.profile_dir, dir);
        assert!(!planned.profile_dir_exists);
        assert_eq!(planned.executable_origin, Origin::Configured);
    }

    #[test]
    fn no_profile_is_an_error() {
        let (_dir, root, config, _exe) = setup();
        let agent = AgentId::parse("fake").unwrap();
        let error =
            plan(&agent, &resolve(agent.clone(), None), &root, &config, vec![], None).unwrap_err();
        assert!(matches!(error, Error::NoProfile { .. }));
    }

    #[test]
    fn case_only_twin_is_refused_for_any_entry_type() {
        let (_dir, root, config, _exe) = setup();
        let agent = AgentId::parse("fake").unwrap();
        fs::create_dir_all(root.profiles_dir()).unwrap();
        fs::write(root.profiles_dir().join("work"), b"a file, not a directory").unwrap();
        let error = plan(&agent, &resolution("WORK"), &root, &config, vec![], None).unwrap_err();
        assert!(
            matches!(error, Error::ProfileCaseConflict { ref existing, .. } if existing == "work"),
            "{error:?}"
        );
        assert!(plan(&agent, &resolution("work"), &root, &config, vec![], None).is_ok());
    }

    #[test]
    fn ensure_profile_dir_is_idempotent_and_rejects_a_file() {
        let (_dir, root, config, _exe) = setup();
        let agent = AgentId::parse("fake").unwrap();
        let planned = plan(&agent, &resolution("work"), &root, &config, vec![], None).unwrap();
        ensure_profile_dir(&planned).unwrap();
        ensure_profile_dir(&planned).unwrap();
        assert!(planned.profile_dir.is_dir());

        let blocked = plan(&agent, &resolution("blocked"), &root, &config, vec![], None).unwrap();
        fs::create_dir_all(blocked.profile_dir.parent().unwrap()).unwrap();
        fs::write(&blocked.profile_dir, b"file").unwrap();
        assert!(matches!(ensure_profile_dir(&blocked), Err(Error::ProfileDir { .. })));
    }
}
```

- [ ] **Step 3: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib resolve:: adapter::
```

Expected: every `resolve::tests::*` and `adapter::tests::*` test passes (5 tests), 0 failed.

- [ ] **Step 4: Run the gate** (see "Gate commands").

- [ ] **Step 5: Commit**

```bash
git add crates/agent-profile/src/resolve.rs crates/agent-profile/src/adapter/mod.rs
git commit -m "feat: add the resolution stub and the debug-only fake adapter" -m "Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 6: Prove the tests are not vacuous**

In `crates/agent-profile/src/adapter/mod.rs`, replace

```rust
        check_case_twins(root, &profile)?;
```

with

```rust
        let _ = check_case_twins;
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib adapter::`. Expected: FAIL, and the failing tests include `case_only_twin_is_refused_for_any_entry_type`. Then restore:

```bash
git checkout -- crates/agent-profile/src/adapter/mod.rs
git status --short
```

Expected: `git status --short` prints nothing.

---

### Task 8: Dry-run report and redaction

**Design/spec:** design §7.4; V3 §22, §26

The report lines shared by `--dry-run` (stdout) and `--verbose` (stderr), with adapter-declared and name-based redaction.

**Files:**
- Modify (replace whole file): `crates/agent-profile/src/output.rs` - Before: contains only `//!` doc-comment lines (the SP0 empty module)

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/output.rs`**

```rust
//! Human output: the dry-run and `--verbose` report (spec §26, design §7.4) and redaction (spec §22).
//! JSON output (spec §32) arrives in SP5.

use std::ffi::OsStr;

use crate::adapter::PlannedLaunch;
use crate::exe::Origin;
use crate::resolve::Resolution;

/// Substrings that mark an override variable as secret-bearing, whatever the adapter declared.
const SENSITIVE_NAME_PARTS: [&str; 6] =
    ["TOKEN", "SECRET", "KEY", "PASSWORD", "CREDENTIAL", "AUTH"];

const LABEL_WIDTH: usize = 14;

/// The report lines, without trailing newlines.
pub fn report_lines(planned: &PlannedLaunch, resolution: &Resolution) -> Vec<String> {
    let mut lines = Vec::new();
    let line = |label: &str, value: String| format!("{:<LABEL_WIDTH$}{value}", format!("{label}:"));
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
    for (index, (key, value)) in planned.plan.env.iter().enumerate() {
        let shown = if is_sensitive(key, &planned.sensitive_env) {
            "<redacted>".to_owned()
        } else {
            let mut shown = value.to_string_lossy().into_owned();
            if value.as_os_str() == planned.profile_dir.as_os_str() && !planned.profile_dir_exists {
                shown.push_str(" (would be created)");
            }
            shown
        };
        let entry = format!("{}={shown}", key.to_string_lossy());
        if index == 0 {
            lines.push(line("environment", entry));
        } else {
            lines.push(format!("{:LABEL_WIDTH$}{entry}", ""));
        }
    }
    let args: Vec<String> = planned.plan.args.iter().map(|arg| render_arg(arg)).collect();
    lines.push(line("arguments", format!("[{}]", args.join(", "))));
    lines
}

fn is_sensitive(key: &OsStr, declared: &[std::ffi::OsString]) -> bool {
    if declared.iter().any(|name| name == key) {
        return true;
    }
    let upper = key.to_string_lossy().to_ascii_uppercase();
    SENSITIVE_NAME_PARTS.iter().any(|part| upper.contains(part))
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
            profile_dir: dir,
            profile_dir_exists: exists,
            executable_origin: Origin::Configured,
            mechanism: "environment variable FAKE_AGENT_HOME".to_owned(),
            sensitive_env: sensitive,
        }
    }

    fn resolution() -> Resolution {
        resolve(
            AgentId::parse("fake").unwrap(),
            Some(ProfileName::parse("work", Platform::Unix).unwrap()),
        )
    }

    #[test]
    fn report_has_every_spec_26_field() {
        let env = vec![("FAKE_AGENT_HOME".into(), "/root/profiles/work/fake".into())];
        let lines = report_lines(&planned(env, vec![], false), &resolution());
        assert_eq!(
            lines,
            [
                "agent:        fake",
                "profile:      work (explicit)",
                &format!(
                    "executable:   {} (configured)",
                    PathBuf::from("/bin/fake-agent").display()
                ),
                "repository:   none",
                "mechanism:    environment variable FAKE_AGENT_HOME",
                "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake (would be created)",
                "arguments:    [\"--foo\", \"a b\"]",
            ]
        );
        let env = vec![("FAKE_AGENT_HOME".into(), "/root/profiles/work/fake".into())];
        let existing = report_lines(&planned(env, vec![], true), &resolution());
        assert_eq!(existing[5], "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake");
    }

    #[test]
    fn redaction_by_declaration_and_by_name() {
        let env: Vec<(OsString, OsString)> = vec![
            ("PLAIN".into(), "visible".into()),
            ("DECLARED".into(), "hidden-1".into()),
            ("MY_api_Token".into(), "hidden-2".into()),
            ("GITHUB_AUTH".into(), "hidden-3".into()),
        ];
        let lines = report_lines(&planned(env, vec!["DECLARED".into()], true), &resolution());
        let text = lines.join("\n");
        assert!(text.contains("PLAIN=visible"), "{text}");
        for secret in ["hidden-1", "hidden-2", "hidden-3"] {
            assert!(!text.contains(secret), "{text}");
        }
        assert_eq!(text.matches("<redacted>").count(), 3, "{text}");
        assert!(lines[6].starts_with("              DECLARED="), "{text}");
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

Expected: every `output::tests::*` test passes (3 tests), 0 failed.

- [ ] **Step 3: Run the gate** (see "Gate commands").

- [ ] **Step 4: Commit**

```bash
git add crates/agent-profile/src/output.rs
git commit -m "feat: add the dry-run report and redaction" -m "Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 5: Prove the tests are not vacuous**

In `crates/agent-profile/src/output.rs`, replace

```rust
    SENSITIVE_NAME_PARTS.iter().any(|part| upper.contains(part))
```

with

```rust
    SENSITIVE_NAME_PARTS.iter().any(|part| upper.contains(part) && false)
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib output::`. Expected: FAIL, and the failing tests include `redaction_by_declaration_and_by_name`. Then restore:

```bash
git checkout -- crates/agent-profile/src/output.rs
git status --short
```

Expected: `git status --short` prints nothing.

---

### Task 9: CLI grammar, dispatch and the end-to-end launch tests

**Design/spec:** design §4, §5.1, §8.2 (`cli`), §8.4 (`tests/launch.rs`); V3 §5, §11, §26, §33, §34

The Clap surface (reserved words as not-yet-implemented subcommands, `external_subcommand` for `<agent> ...`), the ordered nine-check splitter, the §5.1 data flow and the single exit path in `main.rs`. The SP0 stub-CLI smoke tests are replaced: `help_exits_zero_and_shows_launch_usage` replaces the scaffold help test, and `tests/launch.rs` covers every non-zero behaviour row.

**Files:**
- Modify (replace whole file): `crates/agent-profile/src/cli.rs` - Before: contains only `//!` doc-comment lines (the SP0 empty module)
- Modify (replace whole file): `crates/agent-profile/src/main.rs` - Before: contains `SP0 scaffold`
- Modify (replace whole file): `crates/agent-profile/tests/smoke.rs` - Before: contains `help_exits_zero_and_says_scaffold`
- Create: `crates/agent-profile/tests/launch.rs` - Before: the file does not exist

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/cli.rs`**

```rust
//! CLI grammar: launch syntax, wrapper options and reserved command words (spec §5; design §4).

use std::ffi::{OsStr, OsString};
use std::io::{self, Write};

use clap::{Args, Parser, Subcommand};

use crate::adapter::{self, PlannedLaunch};
use crate::config::{AppRoot, Config};
use crate::error::{Error, Result};
use crate::launch::{self, LaunchOutcome};
use crate::name::{AgentId, Platform, ProfileName, is_reserved_word};
use crate::output;
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

/// Select and launch profiles for multiple coding agents.
#[derive(Parser)]
#[command(
    name = "agent-profile",
    version,
    disable_help_subcommand = true,
    arg_required_else_help = true,
    override_usage = "agent-profile <agent> <profile> [--dry-run] [--verbose] [-- <agent args>...]\n       \
                      agent-profile <command>"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Every reserved word of spec §5.3 is a subcommand that is not implemented in SP1.
#[derive(Subcommand)]
enum Command {
    /// Not yet implemented.
    Agents(Rest),
    /// Not yet implemented.
    Profiles(Rest),
    /// Not yet implemented.
    Status(Rest),
    /// Not yet implemented.
    List(Rest),
    /// Not yet implemented.
    Create(Rest),
    /// Not yet implemented.
    Delete(Rest),
    /// Not yet implemented.
    Current(Rest),
    /// Not yet implemented.
    Resolve(Rest),
    /// Not yet implemented.
    Doctor(Rest),
    /// Not yet implemented.
    Link(Rest),
    /// Not yet implemented.
    Unlink(Rest),
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
            Command::Status(_) => "status",
            Command::List(_) => "list",
            Command::Create(_) => "create",
            Command::Delete(_) => "delete",
            Command::Current(_) => "current",
            Command::Resolve(_) => "resolve",
            Command::Doctor(_) => "doctor",
            Command::Link(_) => "link",
            Command::Unlink(_) => "unlink",
            Command::Repositories(_) => "repositories",
            Command::Completions(_) => "completions",
            Command::Agent(_) => return None,
        })
    }
}

/// Runs the CLI and returns the process exit code. This is the only place an exit code is decided.
pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
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
    match run_agent(argv) {
        Ok(code) => code,
        Err(error) => report(error),
    }
}

fn report(error: Error) -> i32 {
    let _ = writeln!(io::stderr(), "agent-profile: error: {error}");
    error.exit_code()
}

/// The outcome of the design §4.2 rule 4 checks.
#[derive(Debug, PartialEq, Eq)]
enum Invocation {
    Help,
    Version,
    Launch {
        agent: String,
        profile: Option<String>,
        dry_run: bool,
        verbose: bool,
        opaque: Vec<OsString>,
    },
}

/// Design §4.2 rules 3-4: the first `--` cut, then the ordered checks.
fn split(argv: Vec<OsString>, known: &[&str]) -> Result<Invocation> {
    let mut argv = argv.into_iter();
    let agent_word = argv.next().unwrap_or_default();
    let rest: Vec<OsString> = argv.collect();
    let cut = rest.iter().position(|arg| arg == "--");
    let (pre, opaque) = match cut {
        Some(index) => (rest[..index].to_vec(), rest[index + 1..].to_vec()),
        None => (rest, Vec::new()),
    };
    let is_option = |arg: &OsStr| arg.as_encoded_bytes().first() == Some(&b'-');
    let options: Vec<&OsString> = pre.iter().filter(|arg| is_option(arg)).collect();
    let bare: Vec<&OsString> = pre.iter().filter(|arg| !is_option(arg)).collect();

    // 1-2. Help and version.
    if options.iter().any(|arg| *arg == "-h" || *arg == "--help") {
        return Ok(Invocation::Help);
    }
    if options.iter().any(|arg| *arg == "-V" || *arg == "--version") {
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
    // 4. A reserved first bare word routes to a not-yet-implemented agent-scoped command.
    if let Some(word) = bare.first().and_then(|word| word.to_str())
        && is_reserved_word(word)
    {
        return Err(Error::NotYetImplemented { command: format!("{agent} {word}") });
    }
    // 5. Unknown options.
    if let Some(bad) = options
        .iter()
        .find(|arg| !matches!(arg.to_str(), Some("--dry-run" | "--verbose" | "--json")))
    {
        return Err(Error::Usage {
            message: format!(
                "unknown option {:?}; agent arguments must follow `--`",
                bad.to_string_lossy()
            ),
        });
    }
    // 6. At most one bare word.
    if bare.len() > 1 {
        return Err(Error::Usage { message: "agent arguments must follow `--`".to_owned() });
    }
    // 7. JSON output is not implemented yet.
    if options.iter().any(|arg| *arg == "--json") {
        return Err(Error::NotYetImplemented { command: "--json".to_owned() });
    }
    // 8. The profile word must be UTF-8.
    let profile = match bare.first() {
        Some(word) => Some(
            word.to_str()
                .ok_or_else(|| Error::Usage {
                    message: "the profile name is not valid UTF-8".to_owned(),
                })?
                .to_owned(),
        ),
        None => None,
    };
    // 9. A launch.
    Ok(Invocation::Launch {
        agent,
        profile,
        dry_run: options.iter().any(|arg| *arg == "--dry-run"),
        verbose: options.iter().any(|arg| *arg == "--verbose"),
        opaque,
    })
}

fn run_agent(argv: Vec<OsString>) -> Result<i32> {
    let known = adapter::known_agents();
    let invocation =
        split(argv, &known).map_err(|error| with_unknown_configured(error, None, &known))?;
    let (agent, profile, dry_run, verbose, opaque) = match invocation {
        Invocation::Help => {
            write_out(LAUNCH_USAGE)?;
            return Ok(0);
        }
        Invocation::Version => {
            write_out(&format!("agent-profile {}\n", env!("CARGO_PKG_VERSION")))?;
            return Ok(0);
        }
        Invocation::Launch { agent, profile, dry_run, verbose, opaque } => {
            (agent, profile, dry_run, verbose, opaque)
        }
    };

    // Design §5.1 steps 2-4.
    let profile = profile
        .map(|name| {
            ProfileName::parse(&name, Platform::host())
                .map_err(|reason| Error::InvalidProfileName { name, reason })
        })
        .transpose()?;
    let root = AppRoot::resolve()?;
    let config = Config::load(&root)?;
    let agent = AgentId::parse(&agent).expect("known agents are valid agent ids");
    let resolution = resolve::resolve(agent.clone(), profile);
    if resolution.profile.is_none() {
        return Err(Error::NoProfile { agent: agent.to_string() });
    }

    // Step 5.
    let path_var = std::env::var_os("PATH");
    let planned = adapter::plan(&agent, &resolution, &root, &config, opaque, path_var.as_deref())
        .map_err(|error| with_unknown_configured(error, Some(&config), &known))?;

    // Step 6.
    if dry_run {
        let lines = output::report_lines(&planned, &resolution);
        write_out(&lines.iter().map(|line| format!("{line}\n")).collect::<String>())?;
        return Ok(0);
    }

    // Step 7. The verbose report is built after lazy creation, so it never says "(would be created)".
    adapter::ensure_profile_dir(&planned)?;
    let planned = PlannedLaunch { profile_dir_exists: true, ..planned };
    if verbose {
        let mut stderr = io::stderr();
        for line in &output::report_lines(&planned, &resolution) {
            let _ = writeln!(stderr, "agent-profile: {line}");
        }
        let _ = stderr.flush();
    }
    match launch::launch(&planned.plan, verbose)? {
        LaunchOutcome::Exited(code) => Ok(code),
        LaunchOutcome::ReplacedProcess => Ok(0),
    }
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

    const KNOWN: &[&str] = &["fake"];

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

    fn split_ok(items: &[&str]) -> Invocation {
        split(args(items), KNOWN).unwrap()
    }

    fn split_err(items: &[&str]) -> Error {
        split(args(items), KNOWN).unwrap_err()
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
        for items in [&["zzz", "work"][..], &["Fake", "work"], &["zzz", "create"]] {
            assert!(matches!(split_err(items), Error::UnknownAgent { .. }), "{items:?}");
        }
    }

    #[test]
    fn reserved_first_bare_word_ignores_everything_else() {
        for items in [
            &["fake", "create", "work"][..],
            &["fake", "CREATE", "--bogus"],
            &["fake", "--bogus", "link"],
        ] {
            assert!(matches!(split_err(items), Error::NotYetImplemented { .. }), "{items:?}");
        }
        match split_err(&["fake", "Create", "x"]) {
            Error::NotYetImplemented { command } => assert_eq!(command, "fake Create"),
            other => panic!("{other:?}"),
        }
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
    }
}
```

- [ ] **Step 2: Write `crates/agent-profile/src/main.rs`**

```rust
//! The `agent-profile` binary. All behaviour lives in the library; this is the single exit path.

fn main() {
    std::process::exit(agent_profile::cli::run(std::env::args_os()));
}
```

- [ ] **Step 3: Write `crates/agent-profile/tests/smoke.rs`**

```rust
//! Smoke tests: the binary's top-level contract and the fake-agent fixture's contract.

mod support;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn agent_profile(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-profile")).args(args).output().unwrap()
}

fn stdout_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

/// Asserts the fake-agent fixture-error contract: exit 125, nothing on stdout, a message on stderr.
fn assert_fixture_error(output: &Output, context: &str) {
    assert_eq!(output.status.code(), Some(125), "{context}");
    assert!(output.stdout.is_empty(), "{context}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("fake-agent: "), "{context}: {stderr}");
}

#[test]
fn version_exits_zero() {
    let output = agent_profile(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "{stdout}");
}

#[test]
fn help_exits_zero_and_shows_launch_usage() {
    let output = agent_profile(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("agent-profile <agent> <profile>"), "{stdout}");
}

#[test]
fn fake_agent_echoes_argv_exactly() {
    let args = ["--foo", "bar", "--", "a b"];
    let output = support::fake_agent().args(args).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["argv"], serde_json::json!(args));
}

#[test]
fn fake_agent_exit_code_is_controllable() {
    let output = support::fake_agent().env("FAKE_AGENT_EXIT", "7").output().unwrap();
    assert_eq!(output.status.code(), Some(7));
    // A requested non-zero exit still prints the full report.
    assert_eq!(stdout_json(&output)["argv"], serde_json::json!([]));
}

#[test]
fn fake_agent_echoes_cwd() {
    // Not the package directory, which is where the test runner already starts the child.
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let output = support::fake_agent().current_dir(&dir).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    let cwd = stdout_json(&output)["cwd"].as_str().map(PathBuf::from).unwrap();
    assert_eq!(cwd.canonicalize().unwrap(), dir.canonicalize().unwrap());
}

#[test]
fn fake_agent_echoes_only_listed_env() {
    let output = support::fake_agent()
        .env("FAKE_AGENT_ECHO_ENV", "AP_TEST_SET,,AP_TEST_UNSET")
        .env("AP_TEST_SET", "yes")
        .env("AP_TEST_NOT_LISTED", "no")
        .env_remove("AP_TEST_UNSET")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["env"], serde_json::json!({ "AP_TEST_SET": "yes" }));
}

#[test]
fn fake_agent_env_is_empty_without_echo_list() {
    let output = support::fake_agent().env_remove("FAKE_AGENT_ECHO_ENV").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["env"], serde_json::json!({}));
}

#[test]
fn fake_agent_invalid_exit_code_is_fixture_error() {
    for value in ["", "256", "-1", "seven"] {
        let output = support::fake_agent().env("FAKE_AGENT_EXIT", value).output().unwrap();
        assert_fixture_error(&output, &format!("FAKE_AGENT_EXIT={value:?}"));
    }
}

#[test]
fn fake_agent_non_utf8_argument_is_fixture_error() {
    let output = support::fake_agent().arg(support::non_utf8()).output().unwrap();
    assert_fixture_error(&output, "non-UTF-8 argument");
}

#[test]
fn fake_agent_non_utf8_env_value_is_fixture_error() {
    let output = support::fake_agent()
        .env("FAKE_AGENT_ECHO_ENV", "AP_TEST_NON_UTF8")
        .env("AP_TEST_NON_UTF8", support::non_utf8())
        .output()
        .unwrap();
    assert_fixture_error(&output, "non-UTF-8 echoed env value");
}

#[test]
fn fake_agent_helper_clears_fixture_env() {
    let command = support::fake_agent();
    let envs: BTreeMap<&OsStr, Option<&OsStr>> = command.get_envs().collect();
    for name in support::FIXTURE_VARS {
        assert_eq!(envs.get(OsStr::new(name)), Some(&None), "{name} must be removed");
    }
}

#[test]
fn fake_agent_reports_its_pid() {
    let child = support::fake_agent().stdout(std::process::Stdio::piped()).spawn().unwrap();
    let pid = child.id();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["pid"], serde_json::json!(pid));
}

#[test]
fn fake_agent_reads_stdin_when_asked() {
    use std::io::Write;
    let mut child = support::fake_agent()
        .env("FAKE_AGENT_STDIN", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"line one\nline two").unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["stdin"], serde_json::json!("line one\nline two"));
}

#[test]
fn fake_agent_writes_requested_stderr_after_the_report() {
    let output = support::fake_agent().env("FAKE_AGENT_STDERR", "to stderr").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"to stderr");
    assert!(stdout_json(&output).get("argv").is_some());
}

#[test]
fn fake_agent_sleeps_after_reporting() {
    let started = std::time::Instant::now();
    let output = support::fake_agent().env("FAKE_AGENT_SLEEP_MS", "300").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(started.elapsed() >= std::time::Duration::from_millis(300));
    assert!(stdout_json(&output).get("pid").is_some());
}

#[test]
fn fake_agent_reports_a_distinct_sleeper_pid() {
    let output = support::fake_agent().env("FAKE_AGENT_SPAWN_SLEEPER", "200").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    let report = stdout_json(&output);
    let sleeper = report["sleeper_pid"].as_u64().unwrap();
    assert_ne!(Some(sleeper), report["pid"].as_u64());
}

#[test]
fn fake_agent_invalid_control_values_are_fixture_errors() {
    for (name, value) in [
        ("FAKE_AGENT_STDIN", "yes"),
        ("FAKE_AGENT_SLEEP_MS", "-1"),
        ("FAKE_AGENT_SLEEP_MS", "soon"),
        ("FAKE_AGENT_SPAWN_SLEEPER", ""),
        ("FAKE_AGENT_CTRL_C_EXIT", "256"),
    ] {
        let output = support::fake_agent().env(name, value).output().unwrap();
        assert_fixture_error(&output, &format!("{name}={value:?}"));
    }
}

#[cfg(unix)]
#[test]
fn fake_agent_ctrl_c_exit_is_windows_only() {
    let output = support::fake_agent().env("FAKE_AGENT_CTRL_C_EXIT", "42").output().unwrap();
    assert_fixture_error(&output, "FAKE_AGENT_CTRL_C_EXIT on Unix");
}
```

- [ ] **Step 4: Write `crates/agent-profile/tests/launch.rs`**

```rust
//! End-to-end launch tests through the real `agent-profile` binary (SP1 design §8.4; spec §34
//! "Passthrough", "Environment", "Process behavior").

mod support;

use std::io::Write;
use std::process::{Output, Stdio};

use support::Root;

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn passthrough_arguments_arrive_verbatim() {
    let root = Root::new();
    let output =
        root.agent_profile(["fake", "work", "--", "--foo", "bar", "a b"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(support::report(&output.stdout)["argv"], serde_json::json!(["--foo", "bar", "a b"]));

    let output =
        root.agent_profile(["fake", "work", "--", "--dry-run", "--", "--help"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        support::report(&output.stdout)["argv"],
        serde_json::json!(["--dry-run", "--", "--help"])
    );
}

#[test]
fn non_utf8_opaque_argument_arrives_unconverted() {
    let root = Root::new();
    let mut command = root.agent_profile(["fake", "work", "--"]);
    command.arg(support::non_utf8());
    let output = command.output().unwrap();
    // The fixture rejects a non-UTF-8 argument (exit 125), which proves it arrived without lossy conversion.
    assert_eq!(output.status.code(), Some(125), "{}", stderr(&output));
    assert!(stderr(&output).contains("argument is not valid UTF-8"), "{}", stderr(&output));
}

#[test]
fn profile_override_replaces_the_inherited_value() {
    let root = Root::new();
    let before = std::env::var_os("FAKE_AGENT_HOME");
    let output = root
        .agent_profile(["fake", "work"])
        .env("FAKE_AGENT_HOME", "/wrong")
        .env("FAKE_AGENT_ECHO_ENV", "FAKE_AGENT_HOME")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        support::report(&output.stdout)["env"]["FAKE_AGENT_HOME"],
        serde_json::json!(root.profile_dir("work").to_str().unwrap())
    );
    assert_eq!(std::env::var_os("FAKE_AGENT_HOME"), before);
}

#[test]
fn agent_exit_status_is_the_wrapper_exit_status() {
    let root = Root::new();
    let output = root.agent_profile(["fake", "work"]).env("FAKE_AGENT_EXIT", "7").output().unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert!(support::report(&output.stdout).get("pid").is_some());
}

#[test]
fn dry_run_reports_every_field_and_launches_nothing() {
    let root = Root::new();
    let output = root.agent_profile(["fake", "work", "--dry-run", "--", "--foo"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    for prefix in [
        "agent:",
        "profile:",
        "executable:",
        "repository:",
        "mechanism:",
        "environment:",
        "arguments:",
    ] {
        assert_eq!(
            text.lines().filter(|line| line.starts_with(prefix)).count(),
            1,
            "{prefix}\n{text}"
        );
    }
    assert!(text.contains("work (explicit)"), "{text}");
    assert!(text.contains("(would be created)"), "{text}");
    assert!(text.contains("[\"--foo\"]"), "{text}");
    assert!(!text.contains("\"pid\""), "the agent must not run: {text}");
    assert!(!root.profile_dir("work").exists());
}

#[test]
fn dry_run_of_an_agent_that_is_not_installed_fails_like_a_launch() {
    let root = Root::empty();
    let output =
        root.agent_profile(["fake", "work", "--dry-run"]).env("PATH", "").output().unwrap();
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    assert!(output.stdout.is_empty());
}

#[test]
fn verbose_launch_reports_to_stderr_after_creating_the_profile_directory() {
    let root = Root::new();
    let output = root.agent_profile(["fake", "work", "--verbose"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stderr(&output);
    let report: Vec<&str> =
        text.lines().filter(|line| line.starts_with("agent-profile: ")).collect();
    assert_eq!(report.len(), 7, "{text}");
    assert!(report[0].starts_with("agent-profile: agent:"), "{text}");
    assert!(text.contains("agent-profile: environment:  FAKE_AGENT_HOME="), "{text}");
    assert!(!text.contains("(would be created)"), "{text}");
    assert!(support::report(&output.stdout).get("pid").is_some());
}

#[test]
fn launch_creates_the_profile_directory_lazily() {
    let root = Root::new();
    assert!(!root.profile_dir("work").exists());
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(root.profile_dir("work").is_dir());
}

#[test]
fn a_file_at_the_profile_path_is_a_profile_error_and_nothing_launches() {
    let root = Root::new();
    let dir = root.profile_dir("work");
    std::fs::create_dir_all(dir.parent().unwrap()).unwrap();
    std::fs::write(&dir, b"not a directory").unwrap();
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert!(output.stdout.is_empty());
}

#[test]
fn concurrent_first_launches_of_one_profile_all_succeed() {
    let root = Root::new();
    let children: Vec<_> = (0..8)
        .map(|_| root.agent_profile(["fake", "work"]).stdout(Stdio::piped()).spawn().unwrap())
        .collect();
    for child in children {
        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    }
    assert!(root.profile_dir("work").is_dir());
}

#[test]
fn stdin_and_stderr_are_inherited() {
    let root = Root::new();
    let mut child = root
        .agent_profile(["fake", "work"])
        .env("FAKE_AGENT_STDIN", "1")
        .env("FAKE_AGENT_STDERR", "agent stderr")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"typed input").unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(support::report(&output.stdout)["stdin"], serde_json::json!("typed input"));
    assert_eq!(stderr(&output), "agent stderr");
}

#[test]
fn help_and_version_after_the_agent_word() {
    let root = Root::new();
    for args in [&["fake", "--help"][..], &["zzz", "-h"], &["fake", "work", "--help", "--", "x"]] {
        let output = root.agent_profile(args).output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{args:?}");
        assert!(stdout(&output).contains("Usage: agent-profile <agent> <profile>"), "{args:?}");
    }
    let output = root.agent_profile(["fake", "work", "-V"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), format!("agent-profile {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn behaviour_table_rows_with_non_zero_exits() {
    let root = Root::new();
    let cases: &[(&[&str], i32, &str)] = &[
        (&["doctor"], 2, "`doctor` is not yet implemented"),
        (&["link", "work", "extra"], 2, "`link` is not yet implemented"),
        (&["fake", "create", "work"], 2, "`fake create` is not yet implemented"),
        (&["fake", "work", "--json"], 2, "`--json` is not yet implemented"),
        (&["zzz", "work"], 2, "unknown agent `zzz` (known agents: `fake`)"),
        (&["Fake", "work"], 2, "unknown agent `Fake`"),
        (&["fake", "work", "--bogus"], 2, "unknown option"),
        (&["fake", "work", "extra"], 2, "agent arguments must follow `--`"),
        (&["fake"], 4, "no profile selected for `fake`"),
        (&["fake", ".hidden"], 4, "invalid profile name"),
        (&["fake", ""], 4, "must not be empty"),
    ];
    for (args, code, message) in cases {
        let output = root.agent_profile(*args).output().unwrap();
        assert_eq!(output.status.code(), Some(*code), "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).contains(message), "{args:?}: {}", stderr(&output));
        assert!(output.stdout.is_empty(), "{args:?}");
    }
}

#[test]
fn bare_invocation_is_a_usage_error() {
    let root = Root::new();
    let output = root.agent_profile(Vec::<&str>::new()).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn explicit_executable_missing_is_not_installed() {
    let root = Root::empty();
    let missing = root.path().join("missing-agent");
    root.write_config(&format!("[agents.fake]\nexecutable = {:?}\n", missing.to_str().unwrap()));
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    assert!(stderr(&output).contains("does not exist or is not a file"), "{}", stderr(&output));
}

#[test]
fn not_on_path_is_not_installed_and_lists_unknown_configured_agents() {
    let root = Root::empty();
    root.write_config("[agents.fakr]\n");
    let output = root.agent_profile(["fake", "work"]).env("PATH", "").output().unwrap();
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    let text = stderr(&output);
    assert!(text.contains("not found in PATH"), "{text}");
    assert!(text.contains("config.toml also configures unknown agents: `fakr`"), "{text}");
}

#[test]
fn unknown_agent_lists_unknown_configured_agents() {
    let root = Root::new();
    root.write_config("[agents.fakr]\n");
    let output = root.agent_profile(["fakr", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("config.toml also configures unknown agents: `fakr`"));
}

#[test]
fn corrupt_configuration_is_an_error_and_stays_untouched() {
    let root = Root::empty();
    let corrupt = "[agents.fake\nexecutable = ";
    root.write_config(corrupt);
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("never rewrites an invalid configuration"),
        "{}",
        stderr(&output)
    );
    assert_eq!(std::fs::read_to_string(root.path().join("config.toml")).unwrap(), corrupt);
}

#[test]
fn relative_app_root_is_an_error() {
    let root = Root::new();
    let output = root
        .agent_profile(["fake", "work"])
        .env("AGENT_PROFILE_HOME", "relative")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
}

#[test]
fn batch_file_override_is_refused() {
    let root = Root::empty();
    let batch = root.path().join("agent.cmd");
    std::fs::write(&batch, b"@echo off\r\n").unwrap();
    root.write_config(&format!("[agents.fake]\nexecutable = {:?}\n", batch.to_str().unwrap()));
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(6), "{}", stderr(&output));
    assert!(stderr(&output).contains("never through cmd.exe"), "{}", stderr(&output));
}

#[test]
fn garbage_executable_is_a_launch_failure() {
    let root = Root::empty();
    let name = if cfg!(windows) { "garbage.exe" } else { "garbage" };
    let garbage = root.path().join(name);
    std::fs::write(&garbage, b"this is not a program").unwrap();
    root.write_config(&format!("[agents.fake]\nexecutable = {:?}\n", garbage.to_str().unwrap()));
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(6), "{}", stderr(&output));
    assert!(stderr(&output).contains("could not launch"), "{}", stderr(&output));
}

#[test]
fn case_only_twin_is_refused_on_every_os() {
    let root = Root::new();
    std::fs::create_dir_all(root.path().join("profiles").join("work")).unwrap();
    for args in [&["fake", "WORK"][..], &["fake", "WORK", "--dry-run"]] {
        let output = root.agent_profile(args).output().unwrap();
        assert_eq!(output.status.code(), Some(4), "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).contains("existing profile entry `work`"), "{}", stderr(&output));
        assert!(output.stdout.is_empty());
    }
    let names: Vec<String> = std::fs::read_dir(root.path().join("profiles"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["work"]);
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[cfg(unix)]
#[test]
fn unix_launch_replaces_the_wrapper_process() {
    let root = Root::new();
    let child = root.agent_profile(["fake", "work"]).stdout(Stdio::piped()).spawn().unwrap();
    let wrapper_pid = child.id();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::report(&output.stdout)["pid"], serde_json::json!(wrapper_pid));
}
```

- [ ] **Step 5: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib cli::
cargo nextest run -p agent-profile --test smoke --test launch
```

Expected: every `cli::tests::*` test passes (9 tests); every smoke test (17 on Windows, 18 on Unix) and every `tests/launch.rs` test (22 on Windows, 23 on Unix) passes, 0 failed.

- [ ] **Step 6: Run the gate** (see "Gate commands").

- [ ] **Step 7: Commit**

```bash
git add crates/agent-profile/src/cli.rs crates/agent-profile/src/main.rs crates/agent-profile/tests/smoke.rs crates/agent-profile/tests/launch.rs
git commit -m "feat: add the CLI grammar and the explicit launch path" -m "Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 8: Prove the tests are not vacuous**

In `crates/agent-profile/src/cli.rs`, replace

```rust
        Some(agent) if known.contains(&agent) => agent.to_owned(),
```

with

```rust
        Some(agent) if known.iter().any(|k| k.eq_ignore_ascii_case(agent)) => agent.to_owned(),
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib cli::`. Expected: FAIL, and the failing tests include `unknown_agent_is_exact_and_precedes_reserved_words`. Then restore:

```bash
git checkout -- crates/agent-profile/src/cli.rs
git status --short
```

Expected: `git status --short` prints nothing.

---

### Task 10: Windows console-event and job-object tests

**Design/spec:** design §7.6, §8.4 (`tests/windows_console.rs`), §10 step 1; V3 §24

`console-driver` runs the wrapper in a fresh console and sends Ctrl-C or Ctrl-Break after a readiness signal; the tests prove the handled-agent exit codes 42 and 43, the default `0xC000013A`, the swallowed pre-spawn window, no orphan after the wrapper is killed, background survival after a normal exit, and no agent after a kill before spawn. These tests passed on GitHub's Windows runner in prototype run 34853308762. If any of them fails only on CI, STOP and ask the owner (design §10 step 1).

**Files:**
- Create: `crates/agent-profile/src/bin/console-driver.rs` - Before: the file does not exist
- Create: `crates/agent-profile/tests/windows_console.rs` - Before: the file does not exist

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/bin/console-driver.rs`**

```rust
//! Test-only Windows console driver (SP1 design §8.4). Never ships.
//!
//! Usage: `console-driver <result-file> <none|ctrl-c|ctrl-break> <none|report|pause-marker> <program> [args...]`
//!
//! A test starts this driver with `CREATE_NEW_CONSOLE`, so the driver, the program it runs and the
//! program's children share a console that the test process is not attached to. The driver re-enables
//! Ctrl-C processing (an "ignore" attribute is inherited from the test runner's process chain), swallows
//! events itself, runs the program with piped stdout and stderr drained on separate threads, waits for the
//! readiness signal, sends the event to its whole console, waits for the program, and writes
//! `{"exit_code": N, "stdout": "...", "stderr": "..."}` (or `{"error": "...", ...}`) to the result file.

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("console-driver: Windows only");
    std::process::ExitCode::from(125)
}

#[cfg(windows)]
fn main() -> std::process::ExitCode {
    match windows::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("console-driver: {message}");
            std::process::ExitCode::from(125)
        }
    }
}

#[cfg(windows)]
mod windows {
    use std::io::{BufRead, BufReader, Read};
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    use windows_sys::Win32::Foundation::{FALSE, TRUE};
    use windows_sys::Win32::System::Console::{
        CTRL_BREAK_EVENT, CTRL_C_EVENT, GenerateConsoleCtrlEvent, SetConsoleCtrlHandler,
    };
    use windows_sys::core::BOOL;

    const PAUSE_MARKER: &str = "agent-profile: debug: paused before spawn";
    const READY_TIMEOUT: Duration = Duration::from_secs(30);
    const EXIT_TIMEOUT: Duration = Duration::from_secs(60);

    unsafe extern "system" fn swallow(_event: u32) -> BOOL {
        TRUE
    }

    pub fn run() -> Result<(), String> {
        let mut args = std::env::args_os().skip(1);
        let usage = "usage: console-driver <result-file> <event> <readiness> <program> [args...]";
        let result_file = args.next().ok_or(usage)?;
        let event = args.next().and_then(|v| v.into_string().ok()).ok_or(usage)?;
        let readiness = args.next().and_then(|v| v.into_string().ok()).ok_or(usage)?;
        let program = args.next().ok_or(usage)?;
        let event = match event.as_str() {
            "none" => None,
            "ctrl-c" => Some(CTRL_C_EVENT),
            "ctrl-break" => Some(CTRL_BREAK_EVENT),
            other => return Err(format!("unknown event {other:?}")),
        };
        if !matches!(readiness.as_str(), "none" | "report" | "pause-marker") {
            return Err(format!("unknown readiness {readiness:?}"));
        }

        // SAFETY: plain Win32 calls; `swallow` is a valid routine for the life of the process.
        unsafe {
            if SetConsoleCtrlHandler(None, FALSE) == FALSE
                || SetConsoleCtrlHandler(Some(swallow), TRUE) == FALSE
            {
                return Err(format!("SetConsoleCtrlHandler: {}", std::io::Error::last_os_error()));
            }
        }

        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("cannot spawn the program: {e}"))?;

        let (ready_tx, ready_rx) = mpsc::channel::<()>();
        let stdout = drain(
            child.stdout.take().unwrap(),
            (readiness == "report").then(|| ready_tx.clone()),
            |line| line.starts_with('{'),
        );
        let stderr = drain(
            child.stderr.take().unwrap(),
            (readiness == "pause-marker").then(|| ready_tx.clone()),
            |line| line == PAUSE_MARKER,
        );
        drop(ready_tx);

        let mut error = None;
        if let Some(event) = event {
            if readiness != "none" && ready_rx.recv_timeout(READY_TIMEOUT).is_err() {
                error = Some("readiness signal not seen".to_owned());
            } else {
                // SAFETY: group 0 targets every process attached to this driver's own console.
                if unsafe { GenerateConsoleCtrlEvent(event, 0) } == FALSE {
                    error = Some(format!(
                        "GenerateConsoleCtrlEvent: {}",
                        std::io::Error::last_os_error()
                    ));
                }
            }
        }

        let deadline = Instant::now() + EXIT_TIMEOUT;
        let code = loop {
            match child.try_wait().map_err(|e| format!("wait: {e}"))? {
                Some(status) => break status.code(),
                None if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    error.get_or_insert_with(|| "the program did not exit in time".to_owned());
                    break None;
                }
                None => thread::sleep(Duration::from_millis(20)),
            }
        };
        let stdout = stdout.join().unwrap_or_default();
        let stderr = stderr.join().unwrap_or_default();

        let mut result =
            serde_json::json!({ "exit_code": code, "stdout": stdout, "stderr": stderr });
        if let Some(error) = error {
            result["error"] = error.into();
        }
        std::fs::write(&result_file, result.to_string())
            .map_err(|e| format!("cannot write the result: {e}"))
    }

    /// Reads a pipe to the end on its own thread, signalling once when `is_ready` matches a line.
    fn drain(
        pipe: impl Read + Send + 'static,
        ready: Option<mpsc::Sender<()>>,
        is_ready: fn(&str) -> bool,
    ) -> thread::JoinHandle<String> {
        thread::spawn(move || {
            let mut text = String::new();
            let mut ready = ready;
            let mut reader = BufReader::new(pipe);
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                if ready.is_some() && is_ready(line.trim_end()) {
                    let _ = ready.take().unwrap().send(());
                }
                text.push_str(&line);
                line.clear();
            }
            text
        })
    }
}
```

- [ ] **Step 2: Write `crates/agent-profile/tests/windows_console.rs`**

```rust
//! Windows control-event and job-object behaviour (spec §24; SP1 design §7.6, §8.4).
#![cfg(windows)]

mod support;

use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use support::Root;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows_sys::Win32::System::Threading::{
    CREATE_NEW_CONSOLE, OpenProcess, PROCESS_TERMINATE, TerminateProcess, WaitForSingleObject,
};

/// `STATUS_CONTROL_C_EXIT`, the exit code of a process ended by default Ctrl-C or Ctrl-Break processing.
const STATUS_CONTROL_C_EXIT: i32 = 0xC000013A_u32 as i32;

/// Kills a child process if it is still running when dropped, so a failed assertion leaks nothing.
struct ChildGuard(Option<Child>);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// An owned process handle that terminates the process on drop if it is still running.
struct ProcessGuard(HANDLE);

impl ProcessGuard {
    fn open(pid: u32) -> ProcessGuard {
        // SAFETY: plain Win32 call; a null result is checked.
        let handle = unsafe { OpenProcess(SYNCHRONIZE | PROCESS_TERMINATE, 0, pid) };
        assert!(
            !handle.is_null(),
            "cannot open process {pid}: {}",
            std::io::Error::last_os_error()
        );
        ProcessGuard(handle)
    }

    fn has_exited_within(&self, timeout: Duration) -> bool {
        // SAFETY: `self.0` is a valid process handle with SYNCHRONIZE access.
        let wait = unsafe { WaitForSingleObject(self.0, timeout.as_millis() as u32) };
        assert!(
            wait == WAIT_OBJECT_0 || wait == WAIT_TIMEOUT,
            "WaitForSingleObject returned {wait}"
        );
        wait == WAIT_OBJECT_0
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        // SAFETY: `self.0` is a valid handle owned by this guard.
        unsafe {
            if WaitForSingleObject(self.0, 0) != WAIT_OBJECT_0 {
                TerminateProcess(self.0, 1);
            }
            CloseHandle(self.0);
        }
    }
}

/// Runs `agent-profile fake work` under `console-driver` in a new console and returns its result.
fn drive(event: &str, readiness: &str, env: &[(&str, &str)]) -> serde_json::Value {
    let root = Root::new();
    let result = root.path().join("result.json");
    let mut command = Command::new(env!("CARGO_BIN_EXE_console-driver"));
    command
        .arg(&result)
        .arg(event)
        .arg(readiness)
        .arg(env!("CARGO_BIN_EXE_agent-profile"))
        .args(["fake", "work"]);
    for name in support::FIXTURE_VARS.iter().chain(support::WRAPPER_VARS.iter()) {
        command.env_remove(name);
    }
    command.env("AGENT_PROFILE_HOME", root.path());
    for (name, value) in env {
        command.env(name, value);
    }
    let status = command
        .creation_flags(CREATE_NEW_CONSOLE)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(0), "console-driver failed");
    let result: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&result).unwrap()).unwrap();
    assert!(result.get("error").is_none(), "{result}");
    result
}

fn exit_code(result: &serde_json::Value) -> i64 {
    result["exit_code"].as_i64().unwrap_or_else(|| panic!("no exit code: {result}"))
}

#[test]
fn ctrl_c_reaches_the_agent_and_the_wrapper_waits_for_it() {
    let result = drive(
        "ctrl-c",
        "report",
        &[("FAKE_AGENT_CTRL_C_EXIT", "42"), ("FAKE_AGENT_SLEEP_MS", "30000")],
    );
    assert_eq!(exit_code(&result), 42, "{result}");
}

#[test]
fn ctrl_break_reaches_the_agent_and_the_wrapper_waits_for_it() {
    let result = drive(
        "ctrl-break",
        "report",
        &[("FAKE_AGENT_CTRL_C_EXIT", "43"), ("FAKE_AGENT_SLEEP_MS", "30000")],
    );
    assert_eq!(exit_code(&result), 43, "{result}");
}

#[test]
fn ctrl_c_default_agent_exit_code_is_propagated() {
    let result = drive("ctrl-c", "report", &[("FAKE_AGENT_SLEEP_MS", "30000")]);
    assert_eq!(exit_code(&result), i64::from(STATUS_CONTROL_C_EXIT), "{result}");
}

#[test]
fn ctrl_c_between_handler_install_and_spawn_is_swallowed() {
    let result = drive(
        "ctrl-c",
        "pause-marker",
        &[("AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS", "2000"), ("FAKE_AGENT_EXIT", "7")],
    );
    assert_eq!(exit_code(&result), 7, "{result}");
    assert!(result["stdout"].as_str().unwrap().contains("\"pid\""), "{result}");
}

#[test]
fn normal_and_non_zero_exits_through_the_job_path() {
    assert_eq!(exit_code(&drive("none", "none", &[])), 0);
    assert_eq!(exit_code(&drive("none", "none", &[("FAKE_AGENT_EXIT", "3")])), 3);
}

/// Spawns the wrapper directly (no new console) with piped stdout and stderr.
fn spawn_wrapper(root: &Root, env: &[(&str, &str)]) -> ChildGuard {
    let mut command = root.agent_profile(["fake", "work"]);
    for (name, value) in env {
        command.env(name, value);
    }
    ChildGuard(Some(
        command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap(),
    ))
}

/// The first line of a pipe, read on a helper thread with a timeout.
fn first_line(pipe: impl std::io::Read + Send + 'static) -> String {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let _ = BufReader::new(pipe).read_line(&mut line);
        let _ = tx.send(line);
    });
    rx.recv_timeout(Duration::from_secs(30)).expect("no line within 30 s")
}

#[test]
fn killing_the_wrapper_leaves_no_orphaned_agent() {
    let root = Root::new();
    let mut wrapper = spawn_wrapper(&root, &[("FAKE_AGENT_SLEEP_MS", "30000")]);
    let child = wrapper.0.as_mut().unwrap();
    let report = support::report(first_line(child.stdout.take().unwrap()).as_bytes());
    let agent = ProcessGuard::open(report["pid"].as_u64().unwrap() as u32);
    assert!(!agent.has_exited_within(Duration::ZERO));
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(agent.has_exited_within(Duration::from_secs(10)), "the agent outlived its wrapper");
}

#[test]
fn processes_the_agent_left_running_survive_a_normal_exit() {
    let root = Root::new();
    // Read the report and wait for the wrapper, not for the pipes: on Windows the sleeper inherits every
    // inheritable handle, including the stdout pipe, so the pipes stay open until the sleeper exits.
    let mut wrapper = spawn_wrapper(&root, &[("FAKE_AGENT_SPAWN_SLEEPER", "20000")]);
    let child = wrapper.0.as_mut().unwrap();
    let report = support::report(first_line(child.stdout.take().unwrap()).as_bytes());
    let sleeper = ProcessGuard::open(report["sleeper_pid"].as_u64().unwrap() as u32);
    assert_eq!(child.wait().unwrap().code(), Some(0));
    std::thread::sleep(Duration::from_millis(500));
    assert!(!sleeper.has_exited_within(Duration::ZERO), "the job killed a background process");
}

#[test]
fn killing_the_wrapper_before_spawn_starts_no_agent() {
    let root = Root::new();
    let mut wrapper =
        spawn_wrapper(&root, &[("AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS", "10000")]);
    let child = wrapper.0.as_mut().unwrap();
    let marker = first_line(child.stderr.take().unwrap());
    assert_eq!(marker.trim_end(), "agent-profile: debug: paused before spawn");
    child.kill().unwrap();
    child.wait().unwrap();
    let mut stdout = String::new();
    std::io::Read::read_to_string(&mut child.stdout.take().unwrap(), &mut stdout).unwrap();
    assert_eq!(stdout, "", "an agent started after the wrapper was killed");
}
```

- [ ] **Step 3: Run the task checks**

```bash
cargo nextest run --no-tests=pass -p agent-profile --test windows_console
```

Expected: Windows: all 8 `tests/windows_console.rs` tests pass; they open short-lived console windows on the desktop, so do not type into them. Unix: the file is `cfg(windows)`, so 0 tests run and `--no-tests=pass` makes the command succeed.

- [ ] **Step 4: Run the gate** (see "Gate commands").

- [ ] **Step 5: Commit**

```bash
git add crates/agent-profile/src/bin/console-driver.rs crates/agent-profile/tests/windows_console.rs
git commit -m "test: add the Windows console-event and job-object tests" -m "Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 6: Prove the tests are not vacuous (Windows only; on other OSes skip this step and say so)**

In `crates/agent-profile/src/launch/windows.rs`, replace

```rust
        CTRL_C_EVENT | CTRL_BREAK_EVENT => TRUE,
```

with

```rust
        CTRL_C_EVENT | CTRL_BREAK_EVENT => FALSE,
```

Run `cargo nextest run --no-fail-fast -p agent-profile --test windows_console`. Expected: FAIL, and the failing tests include `ctrl_c_reaches_the_agent_and_the_wrapper_waits_for_it`. Then restore:

```bash
git checkout -- crates/agent-profile/src/launch/windows.rs
git status --short
```

Expected: `git status --short` prints nothing.

---

### Task 11: Documentation

**Design/spec:** design §10 steps 3-4

README status and configuration location; TODO.md trades the four settled SP1 decisions for the SP2 shim decision and names `console-driver` in the install debt.

**Files:**
- Modify (replace whole file): `README.md` - Before: contains `**Status: scaffold.**`
- Modify (replace whole file): `TODO.md` - Before: contains `## SP1 open decisions`

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `README.md`**

````markdown
# Agent Profile

A local, privacy-first Rust CLI for selecting and launching profiles for multiple coding agents.

> `agent-profile` owns profile selection. The coding agent owns authentication and agent-specific
> configuration. The evidence determines what `agent-profile` is allowed to claim.

Agent Profile never copies credentials, extracts tokens, sends telemetry or trusts
repository-controlled profile selection.

**Status: core and launcher (SP1).** Profile-name validation, configuration, and the explicit launch path
(`exec` on Unix, a supervised child on Windows) exist. No real agent adapter ships yet, so release builds
know no agents; see [ROADMAP.md](ROADMAP.md). The authoritative design is
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
```

Everything after `--` is passed to the agent untouched. `--dry-run` shows what would be launched without
launching or creating anything.

Configuration and profiles live in `~/.agent-profile/` (`%USERPROFILE%\.agent-profile\` on Windows). Set
`AGENT_PROFILE_HOME` to an absolute path to use another directory. Profile resolution order: explicit profile >
agent-specific repository mapping > repository-wide mapping > global default > none.

## Planned adapters — not yet implemented or evidence-verified

Each adapter ships only with a verified evidence entry and capability declaration. Until then, the
mechanisms below are the specification's starting point, not a claim about isolation.

| Agent | Primary mechanism | Intended semantic tier |
|---|---|---|
| Claude Code | `CLAUDE_CONFIG_DIR` | Profile/environment isolation, with credential caveats |
| Codex CLI | native `--profile` / `CODEX_HOME` | Native profile |
| Gemini CLI | `GEMINI_CLI_HOME` | Home/state isolation |
| GitHub Copilot CLI | `COPILOT_HOME` | Home/config isolation |
| OpenCode | `OPENCODE_CONFIG_DIR` / `OPENCODE_CONFIG` | Config/home isolation |
| Cline CLI | `--config`, `--data-dir` | Config/state selection |
| Pi | `PI_CODING_AGENT_DIR` | Agent home/state isolation |
| Kiro CLI | `KIRO_HOME` | Independent home/profile |
| Cursor Agent CLI | `CURSOR_CONFIG_DIR` | Configuration selection |
| Continue CLI | `--config` | Configuration selection |
| Aider | `--config` | Configuration selection |
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

- [ ] **Step 2: Write `TODO.md`**

```markdown
# TODO

Near-term work. Sub-project scope lives in [ROADMAP.md](ROADMAP.md).

## SP2 open decisions

- [ ] **Launching `.cmd`/`.bat` shims without shell mediation** (spec §20, §23.2). npm installs agents on
      Windows as `<name>.cmd` shims, and Rust's `std::process::Command` runs batch files through `cmd.exe`.
      SP1 refuses `.bat` and `.cmd` executables outright (SP1 design D8); SP2 must decide how a real adapter
      launches a shim-installed agent directly.

## SP3 open decisions

- [ ] **Git repository discovery** (spec §13, §14). §14.2 requires Git's worktree metadata, so a plain
      upward walk for `.git` that ignores that metadata is not enough. Cover submodules, worktrees, nested
      repositories, symlinks and canonicalization failure.

## Housekeeping

- [ ] Enable GitHub private vulnerability reporting (see [SECURITY.md](SECURITY.md)). Branch
      protection is applied during SP0 (design §4.1).
- [ ] The MSRV (1.98) is not checked in CI; verify by hand with
      `cargo +1.98 check --workspace --all-targets`. Decide whether to add a CI job.
- [ ] `cargo install --path crates/agent-profile` also installs the `fake-agent` and `console-driver` test
      binaries. Not reachable from any gate or release (release.yml builds `--bin agent-profile`); revisit if
      source installs are ever documented.

## Scaffold follow-ups

- [ ] Run `lefthook install` in each clone
```

- [ ] **Step 3: Run the task checks**

```bash
typos
```

Expected: no output, exit 0.

- [ ] **Step 4: Run the gate** (see "Gate commands").

- [ ] **Step 5: Commit**

```bash
git add README.md TODO.md
git commit -m "docs: describe SP1 in the README and settle its TODO items" -m "Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 12: Whole-branch verification

No code changes. Run each command from the repository root on `sp1-core-launch`, in bash (Git Bash on Windows; the `VAR=value command` form does not work in PowerShell or cmd):

```bash
just check
cargo deny check
cargo +1.98 check --workspace --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items
cargo build --release --locked --bin agent-profile
EMPTY_HOME="$(mktemp -d)"; AGENT_PROFILE_HOME="$EMPTY_HOME" target/release/agent-profile fake work; echo "exit=$?"
```

Expected: the first five exit 0. The release binary knows no agents: with an empty temporary `AGENT_PROFILE_HOME` (so the owner's real configuration is never read), the last command prints exactly `agent-profile: error: unknown agent `fake` (no agents are available in this build)` and `exit=2`. Report any deviation; do not fix it inside this task.

After Task 12 the branch goes through the owner-directed reviews (capstone and test audit on subagent reviewers), then a PR to `main` with each outward step confirmed by the owner.
