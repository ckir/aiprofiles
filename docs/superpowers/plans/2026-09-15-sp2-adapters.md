# SP2 Adapter Architecture Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver V3 §35 phase 4A: a `trait Adapter` with capability, support-level, presence and evidence metadata, the Claude Code (`CLAUDE_CONFIG_DIR`), Codex CLI (`CODEX_HOME`) and Aider (`--config`) adapters, shallow argument-conflict checks, Windows shim detection, and the common contract suite plus one end-to-end launch per adapter, with no real agent ever running in a test.

**Architecture:** `adapter/metadata.rs` holds the declarative types; `adapter/mod.rs` holds the trait, the registry (`REAL_ADAPTERS` plus a debug-only `fake`), the plan helpers, the conflict scan and a lock-free durable no-clobber file writer; one small file per adapter. The CLI looks up the adapter, scans conflicts, plans, reports (dry run) or initializes and launches. `exe::discover` skips relative `PATH` entries and refuses Windows shims with a hint.

**Tech Stack:** Rust 1.98 (edition 2024), clap 4, thiserror 2, tempfile 3.27 (`persist_noclobber`), cargo-nextest. No new dependencies.

**Design:** `docs/superpowers/specs/2026-09-15-sp2-adapters-design.md` (the oracle for behaviour; V3 `agent-profile-implementation-spec-v3.md` wins over both). SP1 design: `docs/superpowers/specs/2026-09-14-sp1-core-launch-design.md`.

---

## How this plan was produced, and how to execute it

Every code block below is copied byte-for-byte from a prototype of the whole design built against the SP1 code at `3281f61`. The prototype passed on Windows (`cargo nextest run --workspace`: 141/141) and on Linux (WSL Ubuntu 26.04: 128/128), with `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` (debug on both, release on Windows) and `typos` clean. The same prototype then passed every job of CI run 34926167302 (draft PR #10): Windows 141/141, Linux 128/128, macOS 128/128, Clippy, Format, Typos, Cargo deny and the docs build. The plan was then replayed task by task in a fresh worktree from `3281f61`, running the gate after every task, and every mutant below was run against the prototype and made its named test fail.

Rules for every task:

1. **Step 0 - state check.** On branch `sp2-adapters`, run `git status --short` (expect no output). `HEAD` must be the previous task's commit; for Task 1 it is the commit that added this plan or a later documentation commit. Then check the "Before" facts listed for the task. If any check fails, STOP and report `STATE_MISMATCH: <what>`.
2. **Byte-exact files.** Write each file with exactly the content shown: the whole file, not a merge. An "Edit" step replaces exactly the quoted text, which occurs once. The gate includes `cargo fmt --all -- --check`, so do not reformat. Do not "improve" any code or test.
3. **Shape-divergence stop.** If making the code compile would change the shape, type or encoding of anything shown, STOP and report `[original] -> [yours] because <reason>`. "It compiles" is not a justification.
4. **Oracle.** The named tests pin the behaviour, and the design is the oracle behind them. If a test fails, fix the code to match the test and the design; never edit a test to match the code.
5. **Gate.** Run the gate exactly as written; do not add flags.
6. **Mutant.** After committing, apply the mutant, run the command, confirm the named test FAILS, then restore with `git checkout -- <file>` and confirm `git status --short` prints nothing.
7. **Toolchain drift.** `rust-toolchain.toml` pins `stable`. If a gate fails with a clippy lint or compiler diagnostic in code copied from this plan, STOP and report the diagnostic; do not change the code to silence it.
8. **Windows desktop.** Every full-workspace test run on Windows opens short-lived console windows (SP1's console tests); do not type into them while tests run.
9. **No real agents.** Never install or run Claude Code, Codex or Aider for this plan. Behaviour claims were measured in a sandbox during design (CONTRIBUTING.md "Measuring agent behaviour").

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
| `crates/agent-profile/src/adapter/metadata.rs` | 1 | `SupportLevel`, `AdapterEvidence`, `Capability`, `CapabilityState`, `CapabilityClaim`, `EnvOverride`, `ConflictOption` (+ `matches`), `ProfilePresence`, `AdapterMetadata` (design §4.2, §6) |
| `crates/agent-profile/src/error.rs` | 2 | `ArgumentConflict` (exit 2), `NotOnPath { ignored_relative }`, `UnsupportedExecutable { agent, path, config_file }` with the hint, one not-installed text (design §6, §7) |
| `crates/agent-profile/src/exe.rs` | 2 | absolute `PATH` entries only; Windows `.exe/.com/.cmd/.bat/.ps1` first-directory rule; configured `.bat/.cmd/.ps1` refused (design §7.1, §7.2) |
| `crates/agent-profile/src/adapter/mod.rs` | 1, 2, 3 | trait `Adapter`, `PlanContext`, `PathKind`, `ProfilePath`, `PlannedLaunch`, `REAL_ADAPTERS`, `registry`, `known_agents`, `lookup`, `check_conflicts`, plan helpers, `ensure_paths`, `write_new_file_with` (design §4) |
| `crates/agent-profile/src/adapter/claude.rs` | 3 | Claude Code adapter (design §5.1) |
| `crates/agent-profile/src/adapter/codex.rs` | 3 | Codex CLI adapter (design §5.2) |
| `crates/agent-profile/src/adapter/aider.rs` | 3 | Aider adapter (design §5.3) |
| `crates/agent-profile/src/adapter/fake.rs` | 3 | debug-only test agent (design §5.4) |
| `crates/agent-profile/src/output.rs` | 3 | `ReportMode`, `creates:` and `note:` lines (design §7.3) |
| `crates/agent-profile/src/cli.rs` | 3 | launch flow and error precedence (design §4.3) |
| `crates/agent-profile/tests/launch.rs` | 2, 3 | two SP1 expectations updated (shim message, known agents) |
| `crates/agent-profile/tests/adapter_contract.rs` | 4 | the common contract suite (design §8.1) |
| `crates/agent-profile/tests/adapters_e2e.rs` | 5 | one launch per real adapter through the binary (design §8.2) |
| `README.md`, `TODO.md`, `CONTRIBUTING.md`, `.claude/recommended-tools.json` | 6 | supported agents, known limits, sandbox prerequisite (design §9) |

### Task 1: Adapter metadata types

**Files:**
- Create: `crates/agent-profile/src/adapter/metadata.rs`
- Modify: `crates/agent-profile/src/adapter/mod.rs` (declare the module)

**Before:** `crates/agent-profile/src/adapter/metadata.rs` does not exist; `crates/agent-profile/src/adapter/mod.rs` contains the line `use std::ffi::{OsStr, OsString};` exactly once and no `pub mod metadata;`.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/adapter/metadata.rs`** (the metadata types and `ConflictOption::matches` with its unit tests; the whole file, byte-exact)

```rust
//! Declarative adapter metadata: support level, capabilities, evidence, environment declarations, conflicting
//! options and profile presence (spec §3, §8, §21, §22, §28; SP2 design §4.2).

use std::ffi::OsStr;

/// How far an adapter is proven, separately from its capabilities (spec §3, §37).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportLevel {
    /// Evidence entry, every capability claimed, and the contract suite passes.
    Proven,
    Experimental,
}

/// Where an adapter's mechanism was verified (spec §28, extended with version and source).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterEvidence {
    pub mechanism_id: &'static str,
    /// ISO date, `YYYY-MM-DD`.
    pub verified_at: &'static str,
    pub upstream_version: &'static str,
    /// A URL, or `measured` with the method in `notes`.
    pub source_url: &'static str,
    pub notes: &'static str,
}

/// An isolation property a profile may provide (SP2 design §4.2 definitions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// The profile replaces the user-level configuration the agent reads from its default user location.
    ConfigIsolation,
    /// Stored credentials (files or OS keychain entries) are separated per profile.
    CredentialIsolation,
    /// History, sessions and caches are separated per profile.
    StateIsolation,
}

impl Capability {
    /// Every capability, so each adapter can be checked for claiming all of them.
    pub const ALL: [Capability; 3] =
        [Capability::ConfigIsolation, Capability::CredentialIsolation, Capability::StateIsolation];
}

/// How strongly a capability holds (spec §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityState {
    Supported,
    NotSupported,
    NotGuaranteed,
    Conditional,
    Unknown,
}

/// One capability claim and the evidence-scoped reason for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityClaim {
    pub capability: Capability,
    pub state: CapabilityState,
    pub basis: &'static str,
}

/// A variable an adapter's plan may set, and whether its value must never be printed (spec §22).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvOverride {
    pub name: &'static str,
    pub sensitive: bool,
}

/// An agent option proven to control the same mechanism the adapter uses (spec §21).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConflictOption {
    /// Every accepted spelling, each starting with `--`.
    pub long: &'static [&'static str],
    pub short: Option<char>,
}

impl ConflictOption {
    /// Whether `arg` selects this option. Works on encoded bytes, so non-UTF-8 arguments are scanned too;
    /// every spelling is ASCII, which makes the byte comparison exact on every platform.
    pub fn matches(&self, arg: &OsStr) -> bool {
        let bytes = arg.as_encoded_bytes();
        let long = self.long.iter().any(|spelling| {
            let spelling = spelling.as_bytes();
            bytes == spelling
                || (bytes.len() > spelling.len()
                    && bytes.starts_with(spelling)
                    && bytes[spelling.len()] == b'=')
        });
        let short = self.short.is_some_and(|letter| {
            let mut flag = [0u8; 4];
            let flag = format!("-{}", letter.encode_utf8(&mut flag));
            bytes.starts_with(flag.as_bytes())
        });
        long || short
    }
}

/// Whether a profile exists for an adapter (spec §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfilePresence {
    Absent,
    Materialized,
    Known,
}

/// Everything about an adapter that does not depend on a launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterMetadata {
    pub id: &'static str,
    /// Executable base name, without `.exe`.
    pub executable: &'static str,
    /// Path-free mechanism text used in messages, e.g. `argument --config <file>`.
    pub mechanism_summary: &'static str,
    pub support: SupportLevel,
    pub evidence: AdapterEvidence,
    pub capabilities: &'static [CapabilityClaim],
    /// Every variable `plan()` may set, with its sensitivity.
    pub env: &'static [EnvOverride],
    pub conflicts: &'static [ConflictOption],
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    const AIDER_CONFIG: ConflictOption =
        ConflictOption { long: &["--config", "--confi", "--conf", "--con"], short: Some('c') };

    #[test]
    fn long_spellings_match_exactly_or_with_an_equals_value() {
        for arg in ["--config", "--config=f", "--confi", "--conf=", "--con=a=b", "-c", "-cf", "-c="]
        {
            assert!(AIDER_CONFIG.matches(OsStr::new(arg)), "{arg}");
        }
        for arg in
            ["--co", "--code-theme", "--configx", "--conf-x", "config", "-", "--", "-x", "--c"]
        {
            assert!(!AIDER_CONFIG.matches(OsStr::new(arg)), "{arg}");
        }
    }

    #[test]
    fn an_option_without_a_short_form_matches_only_long_spellings() {
        let option = ConflictOption { long: &["--fake-profile"], short: None };
        assert!(option.matches(OsStr::new("--fake-profile=x")));
        assert!(!option.matches(OsStr::new("-f")));
    }

    #[cfg(unix)]
    fn non_utf8_value(prefix: &str) -> OsString {
        use std::os::unix::ffi::OsStringExt;
        let mut bytes = prefix.as_bytes().to_vec();
        bytes.push(0xff);
        OsString::from_vec(bytes)
    }

    #[cfg(windows)]
    fn non_utf8_value(prefix: &str) -> OsString {
        use std::os::windows::ffi::OsStringExt;
        let mut wide: Vec<u16> = prefix.encode_utf16().collect();
        wide.push(0xD800);
        OsString::from_wide(&wide)
    }

    #[test]
    fn non_utf8_arguments_are_scanned() {
        assert!(AIDER_CONFIG.matches(&non_utf8_value("--config=")));
        assert!(AIDER_CONFIG.matches(&non_utf8_value("-c")));
        assert!(!AIDER_CONFIG.matches(&non_utf8_value("--cod")));
    }
}
```

- [ ] **Step 2: Edit `crates/agent-profile/src/adapter/mod.rs`** (declare the module). Replace exactly this text, which occurs once:

```rust
use std::ffi::{OsStr, OsString};
```

with:

```rust
pub mod metadata;

use std::ffi::{OsStr, OsString};
```

- [ ] **Step 3: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib adapter::metadata
```

Expected: every listed test passes.

- [ ] **Step 4: Run the gate** (see "Gate commands"). Expected on Windows: `111 tests run: 111 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 5: Commit**

```bash
git add crates/agent-profile/src/adapter/metadata.rs crates/agent-profile/src/adapter/mod.rs
git commit -m "feat(adapter): declarative adapter metadata types

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 6: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/adapter/metadata.rs` replace exactly:

```rust
                    && bytes[spelling.len()] == b'=')
```

with:

```rust
                    && true)
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib adapter::metadata`. Expected: FAIL, and the failing test list includes `long_spellings_match_exactly_or_with_an_equals_value`. Then restore with `git checkout -- crates/agent-profile/src/adapter/metadata.rs` and confirm `git status --short` prints nothing.


### Task 2: Errors and executable discovery

**Files:**
- Modify (whole file): `crates/agent-profile/src/error.rs`
- Modify (whole file): `crates/agent-profile/src/exe.rs`
- Modify: `crates/agent-profile/src/adapter/mod.rs` (the SP1 `fake::plan` call site)
- Modify: `crates/agent-profile/tests/launch.rs` (the refused-shim message)

**Before:** `error.rs` contains `UnsupportedExecutable { path: PathBuf },`; `exe.rs` contains `pub fn discover(` with four parameters; `adapter/mod.rs` contains the `exe::discover("fake", ...)` line quoted in Step 3; `tests/launch.rs` contains `never through cmd.exe`.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/error.rs`** (new variants, fields, messages and exit codes, with unit tests; the whole file, byte-exact)

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
    format!("`{agent}` is not installed: {reason}{}", unknown_configured_suffix(unknown_configured))
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
            (Error::NoProfile { agent: "a".into() }, 4),
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
}
```

- [ ] **Step 2: Write `crates/agent-profile/src/exe.rs`** (relative `PATH` entries skipped, Windows shim rule, hint, with unit tests; the whole file, byte-exact)

```rust
//! Executable discovery (spec §20): an explicit configured override, then the absolute entries of `PATH`.

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

/// Extensions std would run through a shell or that need an interpreter: refused on every platform
/// (spec §23.2, SP2 design §7.2).
const SHELL_EXTENSIONS: [&str; 3] = ["bat", "cmd", "ps1"];

/// Windows `PATH` forms checked in each directory; the first directory holding any of them decides.
#[cfg(windows)]
const WINDOWS_FORMS: [&str; 5] = ["exe", "com", "cmd", "bat", "ps1"];

/// Finds `name` for `agent`: the `explicit` override when given, otherwise the first absolute `PATH`
/// directory that holds it. A shell script or a Windows shim is refused with a hint naming `config_file`.
pub fn discover(
    agent: &str,
    name: &str,
    explicit: Option<&Path>,
    path_var: Option<&OsStr>,
    config_file: &Path,
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
        None => search_path(agent, name, path_var, config_file)?,
    };
    let refused =
        found.path.extension().and_then(OsStr::to_str).is_some_and(|ext| {
            SHELL_EXTENSIONS.iter().any(|shell| ext.eq_ignore_ascii_case(shell))
        });
    if refused {
        return Err(unsupported(agent, found.path, config_file));
    }
    Ok(found)
}

fn search_path(
    agent: &str,
    name: &str,
    path_var: Option<&OsStr>,
    config_file: &Path,
) -> Result<Found> {
    // Only a Windows shim on `PATH` needs the hint.
    #[cfg(not(windows))]
    let _ = config_file;
    let mut ignored_relative = 0;
    for dir in path_var.into_iter().flat_map(std::env::split_paths) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        // A relative entry resolves against the working directory: repository-local discovery (spec §20, §36).
        if !dir.is_absolute() {
            ignored_relative += 1;
            continue;
        }
        match search_dir(&dir, name) {
            Some(Hit::Native(path)) => return Ok(Found { path, origin: Origin::Path }),
            #[cfg(windows)]
            Some(Hit::Unsupported(path)) => return Err(unsupported(agent, path, config_file)),
            None => {}
        }
    }
    Err(not_installed(agent, NotInstalledReason::NotOnPath { ignored_relative }))
}

/// What a `PATH` directory holds for an agent.
enum Hit {
    Native(PathBuf),
    /// The first Windows non-`.exe` form (`.com`, `.cmd`, `.bat`, `.ps1`) in a directory without an `.exe`;
    /// the user's shell would run it, so a later `.exe` is never chosen instead.
    #[cfg(windows)]
    Unsupported(PathBuf),
}

#[cfg(windows)]
fn search_dir(dir: &Path, name: &str) -> Option<Hit> {
    let mut forms = WINDOWS_FORMS
        .iter()
        .map(|ext| dir.join(format!("{name}.{ext}")))
        .filter(|candidate| is_executable(candidate));
    let first = forms.next()?;
    if first.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("exe")) {
        Some(Hit::Native(first))
    } else {
        Some(Hit::Unsupported(first))
    }
}

#[cfg(unix)]
fn search_dir(dir: &Path, name: &str) -> Option<Hit> {
    let candidate = dir.join(name);
    is_executable(&candidate).then_some(Hit::Native(candidate))
}

fn not_installed(agent: &str, reason: NotInstalledReason) -> Error {
    Error::AgentNotInstalled { agent: agent.to_owned(), reason, unknown_configured: Vec::new() }
}

fn unsupported(agent: &str, path: PathBuf, config_file: &Path) -> Error {
    Error::UnsupportedExecutable {
        agent: agent.to_owned(),
        path,
        config_file: config_file.to_path_buf(),
    }
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

    const CONFIG: &str = "/root/config.toml";

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

    fn native(name: &str) -> String {
        if cfg!(windows) { format!("{name}.exe") } else { name.to_owned() }
    }

    fn find(name: &str, explicit: Option<&Path>, path_var: Option<&OsStr>) -> Result<Found> {
        discover("fake", name, explicit, path_var, Path::new(CONFIG))
    }

    #[test]
    fn explicit_override_wins_and_must_be_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let exe = make_file(dir.path(), "agent-bin", true);
        let found = find("fake-agent", Some(&exe), None).unwrap();
        assert_eq!(found, Found { path: exe, origin: Origin::Configured });

        let missing = dir.path().join("missing");
        let error = find("fake-agent", Some(&missing), None).unwrap_err();
        assert!(matches!(
            error,
            Error::AgentNotInstalled { reason: NotInstalledReason::ExplicitMissing(ref p), .. } if *p == missing
        ));
        let error = find("fake-agent", Some(dir.path()), None).unwrap_err();
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
        make_file(first.path(), &native("tool"), true);
        let expected = make_file(second.path(), &native("tool"), true);
        let path_var = std::env::join_paths([empty.path(), second.path(), first.path()]).unwrap();
        let found = find("tool", None, Some(&path_var)).unwrap();
        assert_eq!(found, Found { path: expected, origin: Origin::Path });
    }

    #[test]
    fn missing_from_path_is_not_installed() {
        let empty = tempfile::tempdir().unwrap();
        let path_var = std::env::join_paths([empty.path()]).unwrap();
        for path_var in [Some(path_var.as_os_str()), Some(OsStr::new("")), None] {
            let error = find("tool", None, path_var).unwrap_err();
            assert!(matches!(
                error,
                Error::AgentNotInstalled {
                    reason: NotInstalledReason::NotOnPath { ignored_relative: 0 },
                    ..
                }
            ));
        }
    }

    #[test]
    fn relative_path_entries_are_ignored_and_counted() {
        let mut entries =
            vec![PathBuf::from("."), PathBuf::from("bin"), PathBuf::from("..").join("tools")];
        if cfg!(windows) {
            entries.push(PathBuf::from(r"\tools"));
            entries.push(PathBuf::from("C:tools"));
        }
        let expected = entries.len();
        let path_var = std::env::join_paths(&entries).unwrap();
        let error = find("tool", None, Some(&path_var)).unwrap_err();
        match error {
            Error::AgentNotInstalled {
                reason: NotInstalledReason::NotOnPath { ignored_relative },
                ..
            } => assert_eq!(ignored_relative, expected),
            other => panic!("{other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn path_search_skips_files_without_an_execute_bit() {
        let dir = tempfile::tempdir().unwrap();
        make_file(dir.path(), "tool", false);
        let path_var = std::env::join_paths([dir.path()]).unwrap();
        assert!(find("tool", None, Some(&path_var)).is_err());
    }

    #[test]
    fn shell_scripts_are_refused_when_configured() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["agent.cmd", "agent.BAT", "agent.Cmd", "agent.ps1", "agent.PS1"] {
            let path = make_file(dir.path(), name, true);
            match find("fake-agent", Some(&path), None).unwrap_err() {
                Error::UnsupportedExecutable { agent, path: refused, config_file } => {
                    assert_eq!((agent.as_str(), refused), ("fake", path));
                    assert_eq!(config_file, PathBuf::from(CONFIG));
                }
                other => panic!("{name}: {other:?}"),
            }
        }
    }

    #[cfg(windows)]
    fn refused_on_path(dirs: &[&Path]) -> PathBuf {
        let path_var = std::env::join_paths(dirs).unwrap();
        match find("codex", None, Some(&path_var)).unwrap_err() {
            Error::UnsupportedExecutable { agent, path, config_file } => {
                assert_eq!(agent, "fake");
                assert_eq!(config_file, PathBuf::from(CONFIG));
                path
            }
            other => panic!("{other:?}"),
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_shim_only_directories_are_refused() {
        for form in ["codex.cmd", "codex.bat", "codex.ps1", "codex.com"] {
            let dir = tempfile::tempdir().unwrap();
            let shim = make_file(dir.path(), form, true);
            assert_eq!(refused_on_path(&[dir.path()]), shim, "{form}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_first_directory_decides_and_names_the_first_form() {
        let both = tempfile::tempdir().unwrap();
        make_file(both.path(), "codex.ps1", true);
        let cmd = make_file(both.path(), "codex.cmd", true);
        assert_eq!(refused_on_path(&[both.path()]), cmd);

        let later = tempfile::tempdir().unwrap();
        make_file(later.path(), "codex.exe", true);
        assert_eq!(refused_on_path(&[both.path(), later.path()]), cmd);

        let beside = tempfile::tempdir().unwrap();
        make_file(beside.path(), "codex.cmd", true);
        let exe = make_file(beside.path(), "codex.exe", true);
        let path_var = std::env::join_paths([beside.path()]).unwrap();
        assert_eq!(
            find("codex", None, Some(&path_var)).unwrap(),
            Found { path: exe, origin: Origin::Path }
        );
    }
}
```

- [ ] **Step 3: Edit `crates/agent-profile/src/adapter/mod.rs`** (pass the configuration file to discovery). Replace exactly this text, which occurs once:

```rust
        let found = exe::discover("fake", "fake-agent", config.agent_executable("fake"), path_var)?;
```

with:

```rust
        let found = exe::discover(
            "fake",
            "fake-agent",
            config.agent_executable("fake"),
            path_var,
            &root.config_path(),
        )?;
```

- [ ] **Step 4: Edit `crates/agent-profile/tests/launch.rs`** (the new unsupported-executable message). Replace exactly this text, which occurs once:

```rust
    assert!(stderr(&output).contains("never through cmd.exe"), "{}", stderr(&output));
```

with:

```rust
    assert!(stderr(&output).contains("cannot launch without a shell"), "{}", stderr(&output));
```

- [ ] **Step 5: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib error:: exe::
cargo nextest run -p agent-profile --test launch batch_file_override_is_refused
```

Expected: every listed test passes.

- [ ] **Step 6: Run the gate** (see "Gate commands"). Expected on Windows: `116 tests run: 116 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 7: Commit**

```bash
git add crates/agent-profile/src/error.rs crates/agent-profile/src/exe.rs crates/agent-profile/src/adapter/mod.rs crates/agent-profile/tests/launch.rs
git commit -m "feat(exe): skip relative PATH entries and refuse Windows shims with a hint

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 8: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/exe.rs` replace exactly:

```rust
const SHELL_EXTENSIONS: [&str; 3] = ["bat", "cmd", "ps1"];
```

with:

```rust
const SHELL_EXTENSIONS: [&str; 3] = ["bat", "cmd", "bat"];
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib exe::tests::shell_scripts`. Expected: FAIL, and the failing test list includes `shell_scripts_are_refused_when_configured`. Then restore with `git checkout -- crates/agent-profile/src/exe.rs` and confirm `git status --short` prints nothing.


### Task 3: The adapter trait, the three adapters and the launch flow

**Files:**
- Modify (whole file): `crates/agent-profile/src/adapter/mod.rs`
- Create: `crates/agent-profile/src/adapter/claude.rs`, `codex.rs`, `aider.rs`, `fake.rs`
- Modify (whole file): `crates/agent-profile/src/output.rs`
- Modify (whole file): `crates/agent-profile/src/cli.rs`
- Modify: `crates/agent-profile/tests/launch.rs` (known agents)

**Before:** `adapter/mod.rs` contains `pub mod metadata;` and `pub fn ensure_profile_dir(`; `cli.rs` contains `adapter::ensure_profile_dir(&planned)?;`; `output.rs` contains `profile_dir_exists: exists,`; the four adapter files do not exist; `tests/launch.rs` contains the known-agents line quoted in Step 8.

Write all files of this task before building: they depend on each other.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/adapter/mod.rs`** (trait, registry, helpers, conflict scan, file writer, unit tests; the whole file, byte-exact)

```rust
//! Agent adapters: the adapter trait and registry (spec §2, §4, §38), capability and evidence metadata
//! (spec §3, §28), profile presence (spec §8), lazy initialization (spec §9) and argument conflicts
//! (spec §21). SP2 design §4-§6.
//!
//! Adapters never spawn processes: `plan()` returns a `LaunchPlan` and the launcher runs it (spec §4).

pub mod metadata;

mod aider;
mod claude;
mod codex;
#[cfg(debug_assertions)]
mod fake;

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::config::{AppRoot, Config};
use crate::error::{Error, Result};
use crate::exe::{self, Origin};
use crate::launch::LaunchPlan;
use crate::name::ProfileName;

pub use aider::Aider;
pub use claude::Claude;
pub use codex::Codex;
#[cfg(debug_assertions)]
pub use fake::Fake;
pub use metadata::{
    AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState, ConflictOption,
    EnvOverride, ProfilePresence, SupportLevel,
};

/// One supported coding agent (SP2 design §4.2).
pub trait Adapter: Sync {
    fn metadata(&self) -> &'static AdapterMetadata;

    /// Pure: no filesystem access. Every path the adapter owns for `profile`, in creation order. The first
    /// entry is always the adapter's profile directory.
    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)>;

    /// Reads the filesystem at most; never writes. Discovers the executable, then refuses case-only twins.
    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch>;

    /// Reads the filesystem at most; never writes; never consults `PATH` or the executable (spec §8).
    fn presence(&self, root: &AppRoot, profile: &ProfileName) -> ProfilePresence {
        if check_case_twins(root, profile).is_err() {
            return ProfilePresence::Absent;
        }
        let paths = self.paths(root, profile);
        if paths.iter().all(|(path, kind)| has_kind(path, *kind)) {
            ProfilePresence::Materialized
        } else {
            ProfilePresence::Absent
        }
    }

    /// Ensures every path in `planned.paths`, in order; idempotent; never overwrites a file (spec §9, §9.1).
    fn initialize(&self, planned: &PlannedLaunch) -> Result<()> {
        ensure_paths(&planned.paths)
    }
}

/// Everything `plan()` needs. The agent id is always `metadata().id`.
#[derive(Debug, Clone, Copy)]
pub struct PlanContext<'a> {
    pub profile: &'a ProfileName,
    pub root: &'a AppRoot,
    pub config: &'a Config,
    pub args: &'a [OsString],
    pub path_var: Option<&'a OsStr>,
}

/// What a profile path must be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathKind {
    Dir,
    /// A file created with `contents` when missing; an existing file is never modified.
    File {
        contents: &'static [u8],
    },
}

/// A path an adapter owns, and whether it already had the declared kind at plan time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePath {
    pub path: PathBuf,
    pub kind: PathKind,
    pub existed: bool,
}

/// A launch plus what the report shows but the launcher does not need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedLaunch {
    pub plan: LaunchPlan,
    pub profile: ProfileName,
    /// The adapter's profile directory, `<root>/profiles/<profile>/<agent>`.
    pub profile_dir: PathBuf,
    pub paths: Vec<ProfilePath>,
    pub executable_origin: Origin,
    /// The per-launch report text, e.g. `environment variable CODEX_HOME`.
    pub mechanism: String,
    /// Override variables whose values must never be printed (spec §22).
    pub sensitive_env: Vec<OsString>,
    /// Non-sensitive facts shown in the report.
    pub notes: Vec<String>,
}

/// The real adapters, in every build (SP2 design §4.2).
pub const REAL_ADAPTERS: &[&dyn Adapter] = &[&Claude, &Codex, &Aider];

/// The adapters this build knows: the real ones, plus `fake` when debug assertions are on.
pub fn registry() -> Vec<&'static dyn Adapter> {
    #[cfg_attr(not(debug_assertions), allow(unused_mut))]
    let mut adapters = REAL_ADAPTERS.to_vec();
    #[cfg(debug_assertions)]
    adapters.push(&Fake);
    adapters
}

/// The agent ids this build knows, in registry order.
pub fn known_agents() -> Vec<&'static str> {
    registry().iter().map(|adapter| adapter.metadata().id).collect()
}

/// The adapter for `id`, if this build knows it.
pub fn lookup(id: &str) -> Option<&'static dyn Adapter> {
    registry().into_iter().find(|adapter| adapter.metadata().id == id)
}

/// Refuses an opaque argument that selects the adapter's own mechanism (spec §21, SP2 design §6). The scan
/// stops at the first `--`, after which arguments are positional for the agent.
pub fn check_conflicts(metadata: &AdapterMetadata, args: &[OsString]) -> Result<()> {
    for arg in args.iter().take_while(|arg| arg.as_os_str() != "--") {
        if metadata.conflicts.iter().any(|option| option.matches(arg)) {
            return Err(Error::ArgumentConflict {
                agent: metadata.id.to_owned(),
                option: arg.to_string_lossy().into_owned(),
                mechanism: metadata.mechanism_summary,
            });
        }
    }
    Ok(())
}

/// `<root>/profiles/<profile>/<id>`.
pub(crate) fn profile_dir(root: &AppRoot, profile: &ProfileName, id: &str) -> PathBuf {
    root.profiles_dir().join(profile.as_str()).join(id)
}

/// Plans an adapter whose mechanism is one environment variable naming its profile directory.
pub(crate) fn env_dir_plan(
    adapter: &dyn Adapter,
    ctx: &PlanContext<'_>,
    var: &str,
) -> Result<PlannedLaunch> {
    let metadata = adapter.metadata();
    let found = discover(metadata, ctx)?;
    check_case_twins(ctx.root, ctx.profile)?;
    let paths = profile_paths(adapter, ctx);
    let dir = paths[0].path.clone();
    Ok(PlannedLaunch {
        plan: LaunchPlan {
            executable: found.path,
            args: ctx.args.to_vec(),
            env: vec![(var.into(), dir.clone().into_os_string())],
            cwd: None,
        },
        profile: ctx.profile.clone(),
        profile_dir: dir,
        paths,
        executable_origin: found.origin,
        mechanism: format!("environment variable {var}"),
        sensitive_env: sensitive_env(metadata),
        notes: Vec::new(),
    })
}

/// Plans an adapter whose mechanism is an argument naming its profile's configuration file.
pub(crate) fn config_file_arg_plan(
    adapter: &dyn Adapter,
    ctx: &PlanContext<'_>,
    flag: &str,
) -> Result<PlannedLaunch> {
    let metadata = adapter.metadata();
    let found = discover(metadata, ctx)?;
    check_case_twins(ctx.root, ctx.profile)?;
    let paths = profile_paths(adapter, ctx);
    let file = paths
        .iter()
        .find(|entry| matches!(entry.kind, PathKind::File { .. }))
        .expect("a configuration-file adapter owns a file")
        .path
        .clone();
    let mut args: Vec<OsString> = vec![flag.into(), file.clone().into_os_string()];
    args.extend(ctx.args.iter().cloned());
    Ok(PlannedLaunch {
        plan: LaunchPlan { executable: found.path, args, env: Vec::new(), cwd: None },
        profile: ctx.profile.clone(),
        profile_dir: paths[0].path.clone(),
        paths,
        executable_origin: found.origin,
        mechanism: format!("argument {flag} {}", file.display()),
        sensitive_env: sensitive_env(metadata),
        notes: Vec::new(),
    })
}

fn discover(metadata: &AdapterMetadata, ctx: &PlanContext<'_>) -> Result<exe::Found> {
    exe::discover(
        metadata.id,
        metadata.executable,
        ctx.config.agent_executable(metadata.id),
        ctx.path_var,
        &ctx.root.config_path(),
    )
}

fn profile_paths(adapter: &dyn Adapter, ctx: &PlanContext<'_>) -> Vec<ProfilePath> {
    adapter
        .paths(ctx.root, ctx.profile)
        .into_iter()
        .map(|(path, kind)| ProfilePath { existed: has_kind(&path, kind), path, kind })
        .collect()
}

fn sensitive_env(metadata: &AdapterMetadata) -> Vec<OsString> {
    metadata.env.iter().filter(|entry| entry.sensitive).map(|entry| entry.name.into()).collect()
}

/// Whether `fs::metadata` (which follows symlinks) reports the declared kind.
fn has_kind(path: &Path, kind: PathKind) -> bool {
    fs::metadata(path).is_ok_and(|metadata| match kind {
        PathKind::Dir => metadata.is_dir(),
        PathKind::File { .. } => metadata.is_file(),
    })
}

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

/// Ensures every path, ignoring `existed`, so a path created or removed after planning is still handled.
pub fn ensure_paths(paths: &[ProfilePath]) -> Result<()> {
    for entry in paths {
        let error = |source: io::Error| Error::ProfileDir { path: entry.path.clone(), source };
        match entry.kind {
            PathKind::Dir => {
                fs::create_dir_all(&entry.path).map_err(error)?;
                if !fs::metadata(&entry.path).map_err(error)?.is_dir() {
                    return Err(error(io::Error::other("the path exists but is not a directory")));
                }
            }
            PathKind::File { contents } => {
                if !has_kind(&entry.path, entry.kind) {
                    write_new_file(&entry.path, contents)?;
                }
            }
        }
    }
    Ok(())
}

fn write_new_file(path: &Path, contents: &[u8]) -> Result<()> {
    write_new_file_with(path, contents, || Ok(()))
}

/// A lock-free, durable, no-clobber file create (SP2 design §4.4). There is no sweep of leftover temp
/// files: without a lock a sweep could delete another launch's live temp file.
fn write_new_file_with(
    path: &Path,
    contents: &[u8],
    before_persist: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let error = |source: io::Error| Error::ProfileDir { path: path.to_path_buf(), source };
    let directory = path.parent().expect("profile file paths have a parent");
    let name = path.file_name().expect("profile file paths have a name").to_string_lossy();
    let mut temp = tempfile::Builder::new()
        .prefix(&format!("{name}."))
        .suffix(".tmp")
        .tempfile_in(directory)
        .map_err(error)?;
    temp.write_all(contents).and_then(|()| temp.as_file().sync_all()).map_err(error)?;
    before_persist()?;
    if let Err(persist) = temp.persist_noclobber(path) {
        // A concurrent launch won; its file is complete because it, too, was renamed into place.
        if fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
            return Ok(());
        }
        return Err(error(persist.error));
    }
    #[cfg(unix)]
    fs::File::open(directory).and_then(|directory| directory.sync_all()).map_err(|sync| {
        error(io::Error::new(
            sync.kind(),
            format!(
                "file created, but the directory could not be synced; it may not survive a power loss: {sync}"
            ),
        ))
    })?;
    Ok(())
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use crate::name::Platform;

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

    fn profile(name: &str) -> ProfileName {
        ProfileName::parse(name, Platform::host()).unwrap()
    }

    fn plan_fake(
        root: &AppRoot,
        config: &Config,
        name: &str,
        args: &[OsString],
    ) -> Result<PlannedLaunch> {
        let profile = profile(name);
        Fake.plan(&PlanContext { profile: &profile, root, config, args, path_var: None })
    }

    #[test]
    fn fake_plan_sets_home_override_and_passes_args_verbatim() {
        let (_dir, root, config, exe) = setup();
        let args: Vec<OsString> = vec!["--foo".into(), "a b".into()];
        let planned = plan_fake(&root, &config, "work", &args).unwrap();
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
        assert_eq!(
            planned.paths,
            vec![ProfilePath { path: dir, kind: PathKind::Dir, existed: false }]
        );
        assert_eq!(planned.executable_origin, Origin::Configured);
    }

    #[test]
    fn case_only_twin_is_refused_for_any_entry_type() {
        let (_dir, root, config, _exe) = setup();
        fs::create_dir_all(root.profiles_dir()).unwrap();
        fs::write(root.profiles_dir().join("work"), b"a file, not a directory").unwrap();
        let error = plan_fake(&root, &config, "WORK", &[]).unwrap_err();
        assert!(
            matches!(error, Error::ProfileCaseConflict { ref existing, .. } if existing == "work"),
            "{error:?}"
        );
        assert!(plan_fake(&root, &config, "work", &[]).is_ok());
    }

    #[test]
    fn initialize_is_idempotent_and_rejects_a_file_at_a_directory_path() {
        let (_dir, root, config, _exe) = setup();
        let planned = plan_fake(&root, &config, "work", &[]).unwrap();
        Fake.initialize(&planned).unwrap();
        Fake.initialize(&planned).unwrap();
        assert!(planned.profile_dir.is_dir());

        let blocked = plan_fake(&root, &config, "blocked", &[]).unwrap();
        fs::create_dir_all(blocked.profile_dir.parent().unwrap()).unwrap();
        fs::write(&blocked.profile_dir, b"file").unwrap();
        assert!(matches!(Fake.initialize(&blocked), Err(Error::ProfileDir { .. })));
    }

    #[test]
    fn conflict_scan_stops_at_double_dash_and_names_the_option() {
        let metadata = Aider.metadata();
        let args = |items: &[&str]| items.iter().map(OsString::from).collect::<Vec<_>>();
        match check_conflicts(metadata, &args(&["x", "--conf=f"])).unwrap_err() {
            Error::ArgumentConflict { agent, option, mechanism } => {
                assert_eq!((agent.as_str(), option.as_str()), ("aider", "--conf=f"));
                assert_eq!(mechanism, "argument --config <file>");
            }
            other => panic!("{other:?}"),
        }
        assert!(check_conflicts(metadata, &args(&["x", "--", "--config", "f"])).is_ok());
        assert!(check_conflicts(Codex.metadata(), &args(&["-p", "personal"])).is_ok());
    }

    fn new_file_path() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".aider.conf.yml");
        (dir, path)
    }

    #[test]
    fn new_file_does_not_exist_under_its_final_name_before_persist() {
        let (_dir, path) = new_file_path();
        write_new_file_with(&path, b"{}\n", || {
            assert!(!path.exists(), "the final name must appear only at persist");
            Ok(())
        })
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"{}\n");
    }

    #[test]
    fn a_file_created_before_persist_wins_and_is_never_overwritten() {
        let (_dir, path) = new_file_path();
        write_new_file_with(&path, b"{}\n", || {
            fs::write(&path, b"theirs").unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"theirs");
    }

    #[test]
    fn a_directory_created_before_persist_is_a_profile_error() {
        let (_dir, path) = new_file_path();
        let result = write_new_file_with(&path, b"{}\n", || {
            fs::create_dir(&path).unwrap();
            Ok(())
        });
        assert!(matches!(result, Err(Error::ProfileDir { .. })), "{result:?}");
    }

    #[test]
    fn concurrent_file_creation_never_exposes_a_partial_file() {
        let (dir, path) = new_file_path();
        fs::write(dir.path().join(".aider.conf.yml.leftover.tmp"), b"stale").unwrap();
        let entry = ProfilePath {
            path: path.clone(),
            kind: PathKind::File { contents: b"{}\n" },
            existed: false,
        };
        let done = std::sync::atomic::AtomicBool::new(false);
        std::thread::scope(|scope| {
            let reader = scope.spawn(|| {
                while !done.load(std::sync::atomic::Ordering::SeqCst) {
                    if let Ok(bytes) = fs::read(&path) {
                        assert_eq!(bytes, b"{}\n");
                    }
                }
            });
            let writers: Vec<_> = (0..8)
                .map(|_| scope.spawn(|| ensure_paths(std::slice::from_ref(&entry))))
                .collect();
            for writer in writers {
                writer.join().unwrap().unwrap();
            }
            done.store(true, std::sync::atomic::Ordering::SeqCst);
            reader.join().unwrap();
        });
        assert_eq!(fs::read(&path).unwrap(), b"{}\n");
    }

    struct Secretive;

    static SECRETIVE: AdapterMetadata = AdapterMetadata {
        id: "secretive",
        executable: "fake-agent",
        mechanism_summary: "environment variable PROFILE_SESSION_HANDLE",
        support: SupportLevel::Experimental,
        evidence: AdapterEvidence {
            mechanism_id: "secretive-v1",
            verified_at: "2026-09-15",
            upstream_version: "0.0.0",
            source_url: "measured",
            notes: "test-only adapter",
        },
        capabilities: &[],
        env: &[EnvOverride { name: "PROFILE_SESSION_HANDLE", sensitive: true }],
        conflicts: &[],
    };

    impl Adapter for Secretive {
        fn metadata(&self) -> &'static AdapterMetadata {
            &SECRETIVE
        }

        fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
            vec![(profile_dir(root, profile, SECRETIVE.id), PathKind::Dir)]
        }

        fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
            env_dir_plan(self, ctx, "PROFILE_SESSION_HANDLE")
        }
    }

    #[test]
    fn sensitive_declarations_fill_sensitive_env_and_are_redacted() {
        let (dir, root, _config, exe) = setup();
        fs::write(
            root.config_path(),
            format!("[agents.secretive]\nexecutable = {:?}\n", exe.to_str().unwrap()),
        )
        .unwrap();
        let config = Config::load(&root).unwrap();
        let profile = profile("work");
        let ctx = PlanContext {
            profile: &profile,
            root: &root,
            config: &config,
            args: &[],
            path_var: None,
        };
        let planned = Secretive.plan(&ctx).unwrap();
        assert_eq!(planned.sensitive_env, vec![OsString::from("PROFILE_SESSION_HANDLE")]);
        let resolution = crate::resolve::resolve(
            crate::name::AgentId::parse("secretive").unwrap(),
            Some(profile.clone()),
        );
        let text =
            crate::output::report_lines(&planned, &resolution, crate::output::ReportMode::DryRun)
                .join("\n");
        assert!(text.contains("PROFILE_SESSION_HANDLE=<redacted>"), "{text}");
        drop(dir);
    }
}
```

- [ ] **Step 2: Write `crates/agent-profile/src/adapter/claude.rs`** (Claude Code; the whole file, byte-exact)

```rust
//! Claude Code: `CLAUDE_CONFIG_DIR` per profile (SP2 design §5.1).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    EnvOverride, PathKind, PlanContext, PlannedLaunch, SupportLevel, env_dir_plan, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "CLAUDE_CONFIG_DIR";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "claude",
    executable: "claude",
    mechanism_summary: "environment variable CLAUDE_CONFIG_DIR",
    support: SupportLevel::Proven,
    evidence: AdapterEvidence {
        mechanism_id: "claude-config-dir-v1",
        verified_at: "2026-09-15",
        upstream_version: "2.1.270",
        source_url: "https://code.claude.com/docs/en/authentication",
        notes: "CLAUDE_CONFIG_DIR relocates .credentials.json and keys the macOS Keychain entry per directory; \
                ANTHROPIC_API_KEY, ANTHROPIC_AUTH_TOKEN, CLAUDE_CODE_OAUTH_TOKEN, CLAUDE_CODE_OAUTH_REFRESH_TOKEN, \
                ANTHROPIC_PROFILE and CLAUDE_CODE_USE_BEDROCK/VERTEX/FOUNDRY override it (binary strings measured)",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Supported,
            basis: "user settings live in the config directory; project .claude/settings*.json and .mcp.json \
                    still layer on top",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Conditional,
            basis: "per-directory .credentials.json and macOS Keychain entry (cited); credential environment \
                    variables override it (measured)",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::NotGuaranteed,
            basis: "history and project state moving with the directory is community-sourced only",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[],
};

/// The Claude Code adapter.
#[derive(Debug, Clone, Copy)]
pub struct Claude;

impl Adapter for Claude {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        env_dir_plan(self, ctx, VAR)
    }
}
```

- [ ] **Step 3: Write `crates/agent-profile/src/adapter/codex.rs`** (Codex CLI; the whole file, byte-exact)

```rust
//! Codex CLI: `CODEX_HOME` per profile (SP2 design §5.2).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    EnvOverride, PathKind, PlanContext, PlannedLaunch, SupportLevel, env_dir_plan, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "CODEX_HOME";

/// Shown when the profile's home did not exist at plan time.
pub const NEW_PROFILE_NOTE: &str =
    "new profile starts logged out; run codex login with this profile";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "codex",
    executable: "codex",
    mechanism_summary: "environment variable CODEX_HOME",
    support: SupportLevel::Proven,
    evidence: AdapterEvidence {
        mechanism_id: "codex-home-v1",
        verified_at: "2026-09-15",
        upstream_version: "0.153.4",
        source_url: "measured",
        notes: "codex --help: -p/--profile layers $CODEX_HOME/<name>.config.toml; binary strings OPENAI_API_KEY, \
                CODEX_API_KEY, CODEX_ACCESS_TOKEN, CODEX_SQLITE_HOME; CODEX_HOME semantics per the Codex CLI \
                configuration documentation",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Supported,
            basis: "config.toml and <name>.config.toml live in CODEX_HOME; project-level configuration layering \
                    is not measured",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Conditional,
            basis: "auth.json and the keyring key follow CODEX_HOME; OPENAI_API_KEY, CODEX_API_KEY and \
                    CODEX_ACCESS_TOKEN bypass it (measured)",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Conditional,
            basis: "CODEX_SQLITE_HOME relocates the state database (measured)",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[],
};

/// The Codex CLI adapter.
#[derive(Debug, Clone, Copy)]
pub struct Codex;

impl Adapter for Codex {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        let mut planned = env_dir_plan(self, ctx, VAR)?;
        if !planned.paths[0].existed {
            planned.notes.push(NEW_PROFILE_NOTE.to_owned());
        }
        Ok(planned)
    }
}
```

- [ ] **Step 4: Write `crates/agent-profile/src/adapter/aider.rs`** (Aider; the whole file, byte-exact)

```rust
//! Aider: `--config <profile file>` (SP2 design §5.3).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    ConflictOption, PathKind, PlanContext, PlannedLaunch, SupportLevel, config_file_arg_plan,
    profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const FLAG: &str = "--config";
const FILE_NAME: &str = ".aider.conf.yml";

/// An empty YAML mapping: Aider rejects an empty or comment-only file, and `{}` sets no option (design D5).
pub const INITIAL_CONFIG: &[u8] = b"{}\n";

/// Shown on every launch.
pub const LAYERING_NOTE: &str =
    "--config is layered over .aider.conf.yml in the working directory, git root and home";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "aider",
    executable: "aider",
    mechanism_summary: "argument --config <file>",
    support: SupportLevel::Proven,
    evidence: AdapterEvidence {
        mechanism_id: "aider-config-file-v1",
        verified_at: "2026-09-15",
        upstream_version: "0.86.2",
        source_url: "measured",
        notes: "measured in a sandbox: --config layers over default .aider.conf.yml files; -c, --con, --conf and \
                --confi select the config file; a missing, empty or comment-only file exits 2; {} is accepted",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::NotGuaranteed,
            basis: "the user-level ~/.aider.conf.yml, repository and cwd config files, .env files and AIDER_* \
                    variables still apply (measured)",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::NotSupported,
            basis: "API keys come from the environment, .env files and config files",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::NotSupported,
            basis: "history files are written in the working directory",
        },
    ],
    env: &[],
    conflicts: &[ConflictOption {
        long: &["--config", "--confi", "--conf", "--con"],
        short: Some('c'),
    }],
};

/// The Aider adapter.
#[derive(Debug, Clone, Copy)]
pub struct Aider;

impl Adapter for Aider {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        let dir = profile_dir(root, profile, METADATA.id);
        let file = dir.join(FILE_NAME);
        vec![(dir, PathKind::Dir), (file, PathKind::File { contents: INITIAL_CONFIG })]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        let mut planned = config_file_arg_plan(self, ctx, FLAG)?;
        planned.notes.push(LAYERING_NOTE.to_owned());
        Ok(planned)
    }
}
```

- [ ] **Step 5: Write `crates/agent-profile/src/adapter/fake.rs`** (the debug-only test agent; the whole file, byte-exact)

```rust
//! The debug-only test agent: `FAKE_AGENT_HOME` per profile (SP1 design D1, SP2 design §5.4).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    ConflictOption, EnvOverride, PathKind, PlanContext, PlannedLaunch, SupportLevel, env_dir_plan,
    profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "FAKE_AGENT_HOME";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "fake",
    executable: "fake-agent",
    mechanism_summary: "environment variable FAKE_AGENT_HOME",
    support: SupportLevel::Experimental,
    evidence: AdapterEvidence {
        mechanism_id: "fake-home-v1",
        verified_at: "2026-09-15",
        upstream_version: "0.0.0",
        source_url: "measured",
        notes: "test fixture",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Unknown,
            basis: "test fixture",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "test fixture",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Unknown,
            basis: "test fixture",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[ConflictOption { long: &["--fake-profile"], short: None }],
};

/// The test agent, launched through the `fake-agent` fixture.
#[derive(Debug, Clone, Copy)]
pub struct Fake;

impl Adapter for Fake {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        env_dir_plan(self, ctx, VAR)
    }
}
```

- [ ] **Step 6: Write `crates/agent-profile/src/output.rs`** (`ReportMode`, `creates:` and `note:` lines, unit tests; the whole file, byte-exact)

```rust
//! Human output: the dry-run and `--verbose` report (spec §26, SP1 design §7.4, SP2 design §7.3) and
//! redaction (spec §22). JSON output (spec §32) arrives in SP5.

use std::ffi::OsStr;

use crate::adapter::{PathKind, PlannedLaunch};
use crate::exe::Origin;
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
    let args: Vec<String> = planned.plan.args.iter().map(|arg| render_arg(arg)).collect();
    lines.push(line("arguments", format!("[{}]", args.join(", "))));
    for note in &planned.notes {
        lines.push(line("note", note.clone()));
    }
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

- [ ] **Step 7: Write `crates/agent-profile/src/cli.rs`** (lookup, conflict scan, plan, report, initialize, launch; the whole file, byte-exact)

```rust
//! CLI grammar: launch syntax, wrapper options and reserved command words (spec §5; design §4).

use std::ffi::{OsStr, OsString};
use std::io::{self, Write};

use clap::{Args, Parser, Subcommand};

use crate::adapter::{self, PlanContext};
use crate::config::{AppRoot, Config};
use crate::error::{Error, Result};
use crate::launch::{self, LaunchOutcome};
use crate::name::{AgentId, Platform, ProfileName, is_reserved_word};
use crate::output::{self, ReportMode};
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
    let Some(profile) = resolution.profile.as_ref() else {
        return Err(Error::NoProfile { agent: agent.to_string() });
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
    fn unknown_option_is_reported_before_extra_bare_words() {
        match split_err(&["fake", "work", "extra", "--bogus"]) {
            Error::Usage { message } => {
                assert!(message.starts_with("unknown option \"--bogus\""), "{message}")
            }
            other => panic!("{other:?}"),
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

- [ ] **Step 8: Edit `crates/agent-profile/tests/launch.rs`** (the build now knows four agents). Replace exactly this text, which occurs once:

```rust
        (&["zzz", "work"], 2, "unknown agent `zzz` (known agents: `fake`)"),
```

with:

```rust
        (
            &["zzz", "work"],
            2,
            "unknown agent `zzz` (known agents: `claude`, `codex`, `aider`, `fake`)",
        ),
```

- [ ] **Step 9: Run the task checks**

```bash
cargo nextest run -p agent-profile --lib adapter:: output:: cli::
cargo nextest run -p agent-profile --test launch
```

Expected: every listed test passes.

- [ ] **Step 10: Run the gate** (see "Gate commands"). Expected on Windows: `123 tests run: 123 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 11: Commit**

```bash
git add crates/agent-profile/src/adapter crates/agent-profile/src/output.rs crates/agent-profile/src/cli.rs crates/agent-profile/tests/launch.rs
git commit -m "feat(adapter): adapter trait with Claude Code, Codex CLI and Aider

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 12: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/adapter/mod.rs` replace exactly:

```rust
        if fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
            return Ok(());
```

with:

```rust
        if false {
            return Ok(());
```

Run `cargo nextest run --no-fail-fast -p agent-profile --lib adapter::tests::a_file_created`. Expected: FAIL, and the failing test list includes `a_file_created_before_persist_wins_and_is_never_overwritten`. Then restore with `git checkout -- crates/agent-profile/src/adapter/mod.rs` and confirm `git status --short` prints nothing.


### Task 4: The common adapter contract suite

**Files:**
- Create: `crates/agent-profile/tests/adapter_contract.rs`

**Before:** the file does not exist; `crates/agent-profile/src/adapter/mod.rs` contains `pub const REAL_ADAPTERS`.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/tests/adapter_contract.rs`** (plan, environment, conflict, metadata, paths, presence and initialization contracts; the whole file, byte-exact)

```rust
//! The common adapter contract suite (spec §34 "LaunchPlan", SP2 design §8.1): every registered adapter is
//! checked by the same assertions. Adding an adapter without a row here fails `plan_contract`.

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use agent_profile::adapter::{
    self, Adapter, Capability, PathKind, PlanContext, PlannedLaunch, ProfilePath, ProfilePresence,
    SupportLevel,
};
use agent_profile::config::{AppRoot, Config};
use agent_profile::error::{Error, Result};
use agent_profile::launch::LaunchPlan;
use agent_profile::name::{AgentId, Platform, ProfileName};

struct Fixture {
    _dir: tempfile::TempDir,
    root: AppRoot,
    exe: PathBuf,
}

/// An application root whose `config.toml` points every registered agent at one dummy executable.
fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = AppRoot::from_path(dir.path().join("root"));
    fs::create_dir_all(root.path()).unwrap();
    let exe = dir.path().join("agent-bin");
    fs::write(&exe, b"x").unwrap();
    let config: String = adapter::registry()
        .iter()
        .map(|adapter| {
            format!(
                "[agents.{}]\nexecutable = {:?}\n",
                adapter.metadata().id,
                exe.to_str().unwrap()
            )
        })
        .collect();
    fs::write(root.config_path(), config).unwrap();
    Fixture { _dir: dir, root, exe }
}

fn profile(name: &str) -> ProfileName {
    ProfileName::parse(name, Platform::host()).unwrap()
}

fn args(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

impl Fixture {
    fn plan(
        &self,
        adapter: &dyn Adapter,
        name: &str,
        opaque: &[OsString],
    ) -> Result<PlannedLaunch> {
        let config = Config::load(&self.root).unwrap();
        let profile = profile(name);
        adapter.plan(&PlanContext {
            profile: &profile,
            root: &self.root,
            config: &config,
            args: opaque,
            path_var: None,
        })
    }

    fn dir(&self, id: &str) -> PathBuf {
        self.root.profiles_dir().join("work").join(id)
    }
}

/// The exact plan design §5 specifies for profile `work` and opaque args `["x", "a b"]`.
fn expected(fixture: &Fixture, id: &str) -> PlannedLaunch {
    let dir = fixture.dir(id);
    let opaque = args(&["x", "a b"]);
    let env_dir = |var: &str, notes: Vec<String>| PlannedLaunch {
        plan: LaunchPlan {
            executable: fixture.exe.clone(),
            args: opaque.clone(),
            env: vec![(var.into(), dir.clone().into_os_string())],
            cwd: None,
        },
        profile: profile("work"),
        profile_dir: dir.clone(),
        paths: vec![ProfilePath { path: dir.clone(), kind: PathKind::Dir, existed: false }],
        executable_origin: agent_profile::exe::Origin::Configured,
        mechanism: format!("environment variable {var}"),
        sensitive_env: Vec::new(),
        notes,
    };
    match id {
        "claude" => env_dir("CLAUDE_CONFIG_DIR", Vec::new()),
        "codex" => env_dir(
            "CODEX_HOME",
            vec!["new profile starts logged out; run codex login with this profile".to_owned()],
        ),
        "fake" => env_dir("FAKE_AGENT_HOME", Vec::new()),
        "aider" => {
            let file = dir.join(".aider.conf.yml");
            let mut planned_args = vec![OsString::from("--config"), file.clone().into_os_string()];
            planned_args.extend(opaque.clone());
            PlannedLaunch {
                plan: LaunchPlan {
                    executable: fixture.exe.clone(),
                    args: planned_args,
                    env: Vec::new(),
                    cwd: None,
                },
                profile: profile("work"),
                profile_dir: dir.clone(),
                paths: vec![
                    ProfilePath { path: dir.clone(), kind: PathKind::Dir, existed: false },
                    ProfilePath {
                        path: file.clone(),
                        kind: PathKind::File { contents: b"{}\n" },
                        existed: false,
                    },
                ],
                executable_origin: agent_profile::exe::Origin::Configured,
                mechanism: format!("argument --config {}", file.display()),
                sensitive_env: Vec::new(),
                notes: vec![
                    "--config is layered over .aider.conf.yml in the working directory, git root and home"
                        .to_owned(),
                ],
            }
        }
        other => panic!("no contract row for adapter `{other}`; add one to this suite"),
    }
}

#[test]
fn plan_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        let planned = fixture.plan(adapter, "work", &args(&["x", "a b"])).unwrap();
        assert_eq!(planned, expected(&fixture, id), "{id}");
    }
}

#[test]
fn codex_new_profile_note_disappears_once_the_home_exists() {
    let fixture = fixture();
    let codex = adapter::lookup("codex").unwrap();
    let planned = fixture.plan(codex, "work", &[]).unwrap();
    codex.initialize(&planned).unwrap();
    assert_eq!(fixture.plan(codex, "work", &[]).unwrap().notes, Vec::<String>::new());
}

#[test]
fn declared_environment_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let metadata = adapter.metadata();
        let planned = fixture.plan(adapter, "work", &[]).unwrap();
        for (name, _) in &planned.plan.env {
            assert!(
                metadata.env.iter().any(|declared| name == declared.name),
                "{}: {name:?} is set but not declared",
                metadata.id
            );
        }
        let sensitive: Vec<OsString> = metadata
            .env
            .iter()
            .filter(|declared| declared.sensitive)
            .map(|declared| declared.name.into())
            .collect();
        assert_eq!(planned.sensitive_env, sensitive, "{}", metadata.id);
    }
}

#[test]
fn conflict_contract() {
    for adapter in adapter::registry() {
        let metadata = adapter.metadata();
        let (refused, accepted): (&[&[&str]], &[&[&str]]) = match metadata.id {
            "aider" => (
                &[
                    &["--config", "f"],
                    &["--config=f"],
                    &["--confi", "f"],
                    &["--conf", "f"],
                    &["--con=f"],
                    &["-c", "f"],
                    &["-cf"],
                ],
                &[&["--co"], &["--code-theme", "x"], &["--", "--config", "f"]],
            ),
            "codex" => (&[], &[&["-p", "personal"], &["--profile", "personal"]]),
            "claude" => (&[], &[&["--settings", "s"]]),
            "fake" => (&[&["--fake-profile", "x"]], &[&["--", "--fake-profile", "x"]]),
            other => panic!("no conflict row for adapter `{other}`"),
        };
        for items in refused {
            assert!(
                matches!(
                    adapter::check_conflicts(metadata, &args(items)),
                    Err(Error::ArgumentConflict { .. })
                ),
                "{}: {items:?} must be refused",
                metadata.id
            );
        }
        for items in accepted {
            assert!(
                adapter::check_conflicts(metadata, &args(items)).is_ok(),
                "{}: {items:?} must be accepted",
                metadata.id
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn a_non_utf8_config_value_is_still_a_conflict() {
    use std::os::unix::ffi::OsStringExt;
    let arg = OsString::from_vec(b"--config=\xff".to_vec());
    let metadata = adapter::lookup("aider").unwrap().metadata();
    assert!(matches!(
        adapter::check_conflicts(metadata, &[arg]),
        Err(Error::ArgumentConflict { .. })
    ));
}

fn is_iso_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        })
}

#[test]
fn metadata_invariants() {
    let registry = adapter::registry();
    let mut ids: Vec<&str> = registry.iter().map(|adapter| adapter.metadata().id).collect();
    for adapter in &registry {
        let metadata = adapter.metadata();
        let id = metadata.id;
        assert_eq!(
            AgentId::parse(id).map(|parsed| parsed.as_str().to_owned()),
            Some(id.to_owned())
        );
        assert_eq!(adapter::lookup(id).map(|found| found.metadata().id), Some(id));
        for capability in Capability::ALL {
            let claims: Vec<_> = metadata
                .capabilities
                .iter()
                .filter(|claim| claim.capability == capability)
                .collect();
            assert_eq!(claims.len(), 1, "{id}: {capability:?}");
            assert!(!claims[0].basis.is_empty(), "{id}: {capability:?}");
        }
        assert_eq!(metadata.capabilities.len(), Capability::ALL.len(), "{id}");
        let evidence = metadata.evidence;
        assert!(is_iso_date(evidence.verified_at), "{id}: {}", evidence.verified_at);
        for field in [evidence.mechanism_id, evidence.upstream_version, evidence.source_url] {
            assert!(!field.is_empty(), "{id}");
        }
        for option in metadata.conflicts {
            assert!(option.long.iter().all(|spelling| spelling.starts_with("--")), "{id}");
        }
    }
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), registry.len(), "adapter ids must be unique");

    let real: Vec<(&str, SupportLevel)> = adapter::REAL_ADAPTERS
        .iter()
        .map(|adapter| (adapter.metadata().id, adapter.metadata().support))
        .collect();
    assert_eq!(
        real,
        [
            ("claude", SupportLevel::Proven),
            ("codex", SupportLevel::Proven),
            ("aider", SupportLevel::Proven)
        ]
    );
}

#[test]
fn paths_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let planned = fixture.plan(adapter, "work", &[]).unwrap();
        let declared: Vec<(PathBuf, PathKind)> =
            planned.paths.iter().map(|entry| (entry.path.clone(), entry.kind)).collect();
        assert_eq!(
            adapter.paths(&fixture.root, &profile("work")),
            declared,
            "{}",
            adapter.metadata().id
        );
        assert_eq!(planned.profile_dir, declared[0].0, "{}", adapter.metadata().id);
        assert_eq!(declared[0].1, PathKind::Dir, "{}", adapter.metadata().id);
    }
}

#[test]
fn presence_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        let work = profile("work");
        assert_eq!(adapter.presence(&fixture.root, &work), ProfilePresence::Absent, "{id}");
        let planned = fixture.plan(adapter, "work", &[]).unwrap();
        adapter.initialize(&planned).unwrap();
        assert_eq!(adapter.presence(&fixture.root, &work), ProfilePresence::Materialized, "{id}");
        fs::remove_file(&fixture.exe).unwrap();
        fs::remove_file(fixture.root.config_path()).unwrap();
        assert_eq!(adapter.presence(&fixture.root, &work), ProfilePresence::Materialized, "{id}");
        // Materialize the twin's own paths too, so only the case-twin rule can make it Absent on a
        // case-sensitive filesystem (on a case-insensitive one they are the same entries).
        for (path, kind) in adapter.paths(&fixture.root, &profile("WORK")) {
            match kind {
                PathKind::Dir => fs::create_dir_all(&path).unwrap(),
                PathKind::File { contents } => fs::write(&path, contents).unwrap(),
            }
        }
        assert_eq!(
            adapter.presence(&fixture.root, &profile("WORK")),
            ProfilePresence::Absent,
            "{id}"
        );
    }
    let fixture = fixture();
    let aider = adapter::lookup("aider").unwrap();
    let planned = fixture.plan(aider, "work", &[]).unwrap();
    aider.initialize(&planned).unwrap();
    fs::remove_file(&planned.paths[1].path).unwrap();
    assert!(planned.profile_dir.is_dir());
    assert_eq!(aider.presence(&fixture.root, &profile("work")), ProfilePresence::Absent);
}

fn assert_profile_dir_error(result: Result<()>, context: &str) {
    assert!(matches!(result, Err(Error::ProfileDir { .. })), "{context}: {result:?}");
}

#[test]
fn initialization_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        let planned = fixture.plan(adapter, "work", &[]).unwrap();
        adapter.initialize(&planned).unwrap();
        adapter.initialize(&planned).unwrap();
        for entry in &planned.paths {
            match entry.kind {
                PathKind::Dir => assert!(entry.path.is_dir(), "{id}"),
                PathKind::File { contents } => {
                    assert_eq!(fs::read(&entry.path).unwrap(), contents, "{id}")
                }
            }
        }

        let blocked = fixture.plan(adapter, "blocked", &[]).unwrap();
        fs::create_dir_all(blocked.profile_dir.parent().unwrap()).unwrap();
        fs::write(&blocked.profile_dir, b"a file where a directory belongs").unwrap();
        assert_profile_dir_error(adapter.initialize(&blocked), id);
    }

    let fixture = fixture();
    let aider = adapter::lookup("aider").unwrap();
    let planned = fixture.plan(aider, "work", &[]).unwrap();
    assert_eq!(planned.paths[1].kind, PathKind::File { contents: b"{}\n" });
    fs::create_dir_all(&planned.profile_dir).unwrap();
    fs::write(&planned.paths[1].path, b"model: mine\n").unwrap();
    aider.initialize(&planned).unwrap();
    assert_eq!(fs::read(&planned.paths[1].path).unwrap(), b"model: mine\n");

    let directory = fixture.plan(aider, "directory", &[]).unwrap();
    fs::create_dir_all(&directory.paths[1].path).unwrap();
    assert_profile_dir_error(aider.initialize(&directory), "directory at the Aider file");

    #[cfg(unix)]
    {
        let dangling = fixture.plan(aider, "dangling", &[]).unwrap();
        fs::create_dir_all(&dangling.profile_dir).unwrap();
        std::os::unix::fs::symlink(dangling.profile_dir.join("nowhere"), &dangling.paths[1].path)
            .unwrap();
        assert_profile_dir_error(aider.initialize(&dangling), "dangling symlink at the Aider file");
    }
}
```

- [ ] **Step 2: Run the task checks**

```bash
cargo nextest run -p agent-profile --test adapter_contract
```

Expected: every listed test passes.

- [ ] **Step 3: Run the gate** (see "Gate commands"). Expected on Windows: `131 tests run: 131 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 4: Commit**

```bash
git add crates/agent-profile/tests/adapter_contract.rs
git commit -m "test(adapter): common adapter contract suite

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 5: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/adapter/mod.rs` replace exactly:

```rust
        if paths.iter().all(|(path, kind)| has_kind(path, *kind)) {
```

with:

```rust
        if paths.iter().any(|(path, kind)| has_kind(path, *kind)) {
```

Run `cargo nextest run --no-fail-fast -p agent-profile --test adapter_contract presence_contract`. Expected: FAIL, and the failing test list includes `presence_contract`. Then restore with `git checkout -- crates/agent-profile/src/adapter/mod.rs` and confirm `git status --short` prints nothing.


### Task 5: One end-to-end launch per adapter

**Files:**
- Create: `crates/agent-profile/tests/adapters_e2e.rs`

**Before:** the file does not exist.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/tests/adapters_e2e.rs`** (the fixture renamed as each agent on a one-directory `PATH`; the whole file, byte-exact)

```rust
//! One end-to-end launch per real adapter through the `agent-profile` binary (SP2 design §8.2). The
//! `fake-agent` fixture is copied under each agent's executable name onto a `PATH` holding only that
//! directory, so no real agent ever runs.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};

use support::Root;

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A directory holding the fixture under `name` (with `.exe` on Windows).
fn bin_with(name: &str) -> tempfile::TempDir {
    let bin = tempfile::tempdir().unwrap();
    let file = if cfg!(windows) { format!("{name}.exe") } else { name.to_owned() };
    fs::copy(env!("CARGO_BIN_EXE_fake-agent"), bin.path().join(file)).unwrap();
    bin
}

fn profile_dir(root: &Root, agent: &str) -> PathBuf {
    root.path().join("profiles").join("work").join(agent)
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap()
}

#[test]
fn env_dir_agents_receive_their_home_argv_cwd_and_exit_code() {
    for (agent, var) in [("claude", "CLAUDE_CONFIG_DIR"), ("codex", "CODEX_HOME")] {
        let root = Root::empty();
        let bin = bin_with(agent);
        let cwd = tempfile::tempdir().unwrap();
        let output = root
            .agent_profile([agent, "work", "--", "--flag", "a b"])
            .env("PATH", bin.path())
            .env(var, "/wrong")
            .env("FAKE_AGENT_ECHO_ENV", var)
            .env("FAKE_AGENT_EXIT", "5")
            .current_dir(cwd.path())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(5), "{agent}: {}", stderr(&output));
        let report = support::report(&output.stdout);
        assert_eq!(report["argv"], serde_json::json!(["--flag", "a b"]), "{agent}");
        let dir = profile_dir(&root, agent);
        assert_eq!(report["env"][var], serde_json::json!(dir.to_str().unwrap()), "{agent}");
        assert_eq!(
            canonical(Path::new(report["cwd"].as_str().unwrap())),
            canonical(cwd.path()),
            "{agent}"
        );
        assert!(dir.is_dir(), "{agent}");
    }
}

#[test]
fn aider_receives_its_config_file_first_and_the_file_is_created() {
    let root = Root::empty();
    let bin = bin_with("aider");
    let cwd = tempfile::tempdir().unwrap();
    let output = root
        .agent_profile(["aider", "work", "--", "--model", "x"])
        .env("PATH", bin.path())
        .env("FAKE_AGENT_EXIT", "5")
        .current_dir(cwd.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(5), "{}", stderr(&output));
    let file = profile_dir(&root, "aider").join(".aider.conf.yml");
    let report = support::report(&output.stdout);
    assert_eq!(
        report["argv"],
        serde_json::json!(["--config", file.to_str().unwrap(), "--model", "x"])
    );
    assert_eq!(canonical(Path::new(report["cwd"].as_str().unwrap())), canonical(cwd.path()));
    assert_eq!(fs::read(&file).unwrap(), b"{}\n");
}

#[test]
fn aider_concurrent_first_launches_all_succeed() {
    let root = Root::empty();
    let bin = bin_with("aider");
    let children: Vec<_> = (0..8)
        .map(|_| {
            root.agent_profile(["aider", "work"])
                .env("PATH", bin.path())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    for child in children {
        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    }
    assert_eq!(fs::read(profile_dir(&root, "aider").join(".aider.conf.yml")).unwrap(), b"{}\n");
}

#[test]
fn aider_conflict_is_refused_before_anything_launches_or_is_created() {
    let root = Root::empty();
    let bin = bin_with("aider");
    let output = root
        .agent_profile(["aider", "work", "--", "--conf", "f"])
        .env("PATH", bin.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(output.stdout.is_empty(), "the agent must not run");
    assert!(
        stderr(&output)
            .contains("`--conf` conflicts with how agent-profile selects the aider profile"),
        "{}",
        stderr(&output)
    );
    assert!(!profile_dir(&root, "aider").exists());
}

#[test]
fn aider_conflict_is_reported_even_when_aider_is_not_installed() {
    let root = Root::empty();
    let output = root
        .agent_profile(["aider", "work", "--", "--conf", "f"])
        .env("PATH", "")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
}

#[test]
fn an_invalid_profile_name_wins_over_a_conflict() {
    let root = Root::empty();
    let output = root
        .agent_profile(["aider", ".hidden", "--", "--conf", "f"])
        .env("PATH", "")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert!(stderr(&output).contains("invalid profile name"), "{}", stderr(&output));
}

#[test]
fn the_fake_agent_conflict_is_refused_end_to_end() {
    let root = Root::new();
    let output =
        root.agent_profile(["fake", "work", "--", "--fake-profile", "x"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(output.stdout.is_empty(), "the agent must not run");
    assert!(!root.profile_dir("work").exists());
}

#[test]
fn codex_dry_run_notes_a_new_profile_and_lists_what_would_be_created() {
    let root = Root::empty();
    let bin = bin_with("codex");
    let output = root
        .agent_profile(["codex", "work", "--dry-run"])
        .env("PATH", bin.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let dir = profile_dir(&root, "codex");
    assert!(
        text.contains(&format!("creates:      {} (would be created)", dir.display())),
        "{text}"
    );
    assert!(
        text.contains(
            "note:         new profile starts logged out; run codex login with this profile"
        ),
        "{text}"
    );
    assert!(!dir.exists());
}

#[test]
fn a_relative_path_entry_holding_the_agent_is_ignored() {
    let root = Root::empty();
    let bin = bin_with("codex");
    let output = root
        .agent_profile(["codex", "work"])
        .env("PATH", ".")
        .current_dir(bin.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("(1 relative PATH entries were ignored)"),
        "{}",
        stderr(&output)
    );
    assert!(output.stdout.is_empty());
}

#[cfg(windows)]
#[test]
fn a_cmd_shim_on_path_is_refused_with_the_config_hint() {
    let root = Root::empty();
    let bin = tempfile::tempdir().unwrap();
    fs::write(bin.path().join("codex.cmd"), b"@echo off\r\n").unwrap();
    let output = root.agent_profile(["codex", "work"]).env("PATH", bin.path()).output().unwrap();
    assert_eq!(output.status.code(), Some(6), "{}", stderr(&output));
    let text = stderr(&output);
    assert!(
        text.contains("codex.cmd, which agent-profile cannot launch without a shell"),
        "{text}"
    );
    assert!(text.contains(&root.path().join("config.toml").display().to_string()), "{text}");
}
```

- [ ] **Step 2: Run the task checks**

```bash
cargo nextest run -p agent-profile --test adapters_e2e
```

Expected: every listed test passes.

- [ ] **Step 3: Run the gate** (see "Gate commands"). Expected on Windows: `141 tests run: 141 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 4: Commit**

```bash
git add crates/agent-profile/tests/adapters_e2e.rs
git commit -m "test(adapter): end-to-end launch per adapter

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 5: Prove the tests are not vacuous** (rule 6). In `crates/agent-profile/src/cli.rs` replace exactly:

```rust
    adapter::check_conflicts(adapter.metadata(), &opaque)?;
```

with:

```rust

```

Run `cargo nextest run --no-fail-fast -p agent-profile --test adapters_e2e aider_conflict_is_reported_even`. Expected: FAIL, and the failing test list includes `aider_conflict_is_reported_even_when_aider_is_not_installed`. Then restore with `git checkout -- crates/agent-profile/src/cli.rs` and confirm `git status --short` prints nothing.


### Task 6: Documentation and the sandbox prerequisite

**Files:**
- Modify (whole file): `README.md`, `TODO.md`, `CONTRIBUTING.md`, `.claude/recommended-tools.json`

**Before:** `README.md` contains `**Status: core and launcher (SP1).**`; `TODO.md` contains `## SP2 open decisions`; `CONTRIBUTING.md` has no `## Measuring agent behaviour`; `.claude/recommended-tools.json` has no `Sandboxie-Plus` entry.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `README.md`** (status and supported agents; the whole file, byte-exact)

````markdown
# Agent Profile

A local, privacy-first Rust CLI for selecting and launching profiles for multiple coding agents.

> `agent-profile` owns profile selection. The coding agent owns authentication and agent-specific
> configuration. The evidence determines what `agent-profile` is allowed to claim.

Agent Profile never copies credentials, extracts tokens, sends telemetry or trusts
repository-controlled profile selection.

**Status: architecture gate (SP2).** Profile-name validation, configuration, the explicit launch path
(`exec` on Unix, a supervised child on Windows) and three evidence-backed adapters (Claude Code, Codex CLI,
Aider) exist; see [ROADMAP.md](ROADMAP.md). The authoritative design is
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
and its abbreviations) are refused before launch. On Windows an agent must be a native `.exe`: an npm or pnpm
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

- [ ] **Step 2: Write `TODO.md`** (SP2 decision closed, known limits; the whole file, byte-exact)

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
- [ ] The Unix directory-sync failure branch of the Aider file writer is untested (like SP1 `config.rs` step 7a).
- [ ] The Codex "new profile starts logged out" note keys on the home directory being absent, so a present but
      empty home gives no note.
- [ ] An agent with no native executable cannot be launched on Windows until its vendor ships one.
- [ ] Adapter evidence is static; mechanism drift detection belongs to `doctor` (SP5).

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

- [ ] `console-driver` (test-only) drains the wrapper's stdout and stderr to EOF with no timeout; a descendant
      that inherits those pipes (for example a `FAKE_AGENT_SPAWN_SLEEPER` sleeper) keeps the driver alive past its
      60 s exit timeout. No current test combines a console event with a sleeper; bound the drains before adding one.

## Scaffold follow-ups

- [ ] Run `lefthook install` in each clone
```

- [ ] **Step 3: Write `CONTRIBUTING.md`** (contract suite pointer and "Measuring agent behaviour"; the whole file, byte-exact)

````markdown
# Contributing to Agent Profile

Thanks for your interest. This document covers the basics.

Agent Profile is built against an authoritative implementation specification,
[`agent-profile-implementation-spec-v3.md`](agent-profile-implementation-spec-v3.md). **The spec is the
oracle.** If the code and the spec disagree, that is a bug in the code — or a change that needs to be
made to the spec first, deliberately. Please cite the relevant section (e.g. "spec §12") in issues and
pull requests.

## Getting started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/aiprofiles.git`
3. Install the tooling (see below) and run `lefthook install`
4. Create a branch: `git checkout -b my-feature`
5. Make your changes
6. Run the gate: `just check`
7. Commit and push: `git commit -m "feat: my feature" && git push origin my-feature`
8. Open a pull request

## Development setup

Requires Rust 1.98+ (edition 2024). The toolchain is pinned by `rust-toolchain.toml`.

```bash
# One-time: install the dev tools
cargo binstall -y cargo-nextest just lefthook cargo-deny typos-cli bacon git-cliff cargo-release cargo-mutants
lefthook install

# Everyday
just build      # cargo build --workspace
just check      # the gate: fmt-check + clippy + typos + test
just watch      # bacon, in the background, while you work
just test       # nextest + doctests
just doc        # build and open the API docs
```

Every tool, what it gates and which config file drives it is documented in
[`docs/dev-tooling.md`](docs/dev-tooling.md). The same list is machine-checked from
`.claude/recommended-tools.json`.

## The gate

`just check` is exactly what CI runs. It must be green before you open a PR:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
typos
cargo nextest run --workspace --no-tests=pass
cargo test --doc --workspace       # nextest does not run doctests
```

CI additionally runs `cargo deny check`, builds the docs with warnings as errors, and runs the test job
on Linux, macOS **and** Windows. Agent Profile launches processes, handles console signals and
canonicalizes repository paths — all of which behave differently per platform — so a change that
passes on one OS is not evidence it passes on the others.

## Code style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- `cargo fmt --all` before committing
- Fix every clippy warning; the gate treats them as errors
- All public items get `///` documentation
- Cite the spec section in doc comments for anything implementing a normative rule

## Testing

Spec §34 defines the mandatory contract suite: profile names, resolution precedence, configuration
corruption and atomicity, LaunchPlan per adapter, opaque passthrough, environment overrides and
process behaviour. New behaviour should land with the matching test from that list, and
platform-specific behaviour needs the test on the platform it concerns.

Tests that launch an "agent" use the `fake-agent` fixture (`crates/agent-profile/src/bin/fake-agent.rs`),
launched through `crates/agent-profile/tests/support`. Never launch a real coding agent from a test.

Every adapter has a row in `crates/agent-profile/tests/adapter_contract.rs` and an end-to-end launch in
`crates/agent-profile/tests/adapters_e2e.rs`; a new adapter without its row fails the suite.

## Measuring agent behaviour

Adapter evidence (`AdapterEvidence` in `crates/agent-profile/src/adapter/`) is refreshed only from measurements
of the real agent, and those measurements run **inside a disposable sandbox, never on your own machine**.
Nobody wants their machine filling up with coding agents they do not use, and a sandbox also behaves like a
clean install. Agents you already use may be probed read-only on the host (`--help`, `--version`).

**Windows: Sandboxie-Plus** (`winget install Sandboxie.Plus`).

```powershell
$sbie = 'C:\Program Files\Sandboxie-Plus'
& "$sbie\SbieIni.exe" set AgentProbe Enabled y
# Hide your real agent homes so the box behaves like a clean machine; add any other agent home you have.
& "$sbie\SbieIni.exe" append AgentProbe ClosedFilePath '%USERPROFILE%\.claude'
& "$sbie\SbieIni.exe" append AgentProbe ClosedFilePath '%USERPROFILE%\.codex'
& "$sbie\Start.exe" /reload
# Run a script inside the box; its exit code comes back, its writes stay in the box.
& "$sbie\Start.exe" /box:AgentProbe /wait cmd /c "C:\path\to\probe.cmd > C:\m\out.txt 2>&1"
Get-Content 'C:\Sandbox\<user>\AgentProbe\drive\C\m\out.txt'
# Delete everything the box installed, then remove the box.
& "$sbie\Start.exe" /box:AgentProbe /terminate
& "$sbie\Start.exe" /box:AgentProbe delete_sandbox_silent
& "$sbie\SbieIni.exe" set AgentProbe '*' ''
& "$sbie\Start.exe" /reload
```

Pin interpreter versions an agent supports (for example `uv tool install --python 3.12 aider-chat`): an
unsupported interpreter can start a long source build of native dependencies.

**Linux:** rootless Podman or Docker (`podman run --rm`) for installs; Bubblewrap (`bwrap`) or Firejail with a
private home for probing a host binary without exposing your real agent homes.

**macOS:** Tart disposable macOS virtual machines when the behaviour is macOS-specific (for example the
Keychain); OrbStack, Colima or Docker `--rm` for checks that do not depend on macOS. `sandbox-exec` is
deprecated and can only deny writes, so it is not a substitute.

Record each measurement in the adapter's `AdapterEvidence` (`verified_at`, `upstream_version`, `source_url`,
`notes`) and in the design document's decision table.

## Commit messages

We follow [Conventional Commits](https://www.conventionalcommits.org/), because `git-cliff` generates
the changelog from them:

```
type(scope): description

[optional body]

[optional footer(s)]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

Examples:
- `feat(resolve): prefer the agent-specific repository mapping`
- `fix(launch): return the child exit code on Windows`
- `docs: document the Claude Code credential caveat`

## Pull request process

1. CI must be green on all platforms
2. Update documentation if you changed behaviour
3. Add tests for new functionality
4. Reference the spec section and any related issues

## Reporting bugs

Open an issue with:
- Steps to reproduce
- Expected behaviour (cite the spec section if it defines one)
- Actual behaviour
- Environment: OS and version, the coding agent and its version, `agent-profile --version`
- The `--dry-run` output for the failing invocation, **with any secret values redacted**

For security issues, do **not** open a public issue; see [SECURITY.md](SECURITY.md).

## Releasing

Releases use `cargo release` and follow [Semantic Versioning](https://semver.org/). All crates are
versioned in lockstep.

```bash
just release patch    # bug fixes
just release minor    # new features
just release major    # breaking changes
```

Pushing the resulting `v*` tag triggers the cross-platform release build.

## Licence

By contributing, you agree that your contributions will be licensed under the
[PolyForm Noncommercial License 1.0.0](LICENSE).
````

- [ ] **Step 4: Write `.claude/recommended-tools.json`** (Sandboxie-Plus prerequisite; the whole file, byte-exact)

```json
[
  {
    "name": "cargo-nextest",
    "why": "Test runner used by CI (`cargo nextest run --workspace`). Runs each test in its own process, which matters here because agent-profile tests spawn the binary and the fake-agent fixture and set per-process environment. Does NOT run doctests — pair with `cargo test --doc`.",
    "install": "cargo binstall -y cargo-nextest",
    "in_path": "cargo-nextest"
  },
  {
    "name": "just",
    "why": "Task runner and the single entry point for the local gate. `just check` = fmt-check + clippy + typos + test. Configured by the root justfile.",
    "install": "cargo binstall -y just",
    "in_path": "just"
  },
  {
    "name": "lefthook",
    "why": "Git hooks. pre-push runs `just fmt-check`, `just clippy` and `just typos` in parallel; there is no pre-commit hook, so commits stay fast. Routing through just keeps the hook and CI from drifting apart. Configured by lefthook.yml; needs `lefthook install` once per clone.",
    "install": "cargo binstall -y lefthook",
    "in_path": "lefthook"
  },
  {
    "name": "cargo-deny",
    "why": "Advisory, licence, ban and source auditing across the dependency graph. Configured by deny.toml, which pins the allow-listed licences, bans openssl-sys (V3 spec §36: no hidden network activity) and lists the per-platform target triples.",
    "install": "cargo binstall -y cargo-deny",
    "in_path": "cargo-deny"
  },
  {
    "name": "typos",
    "why": "Spell-checks source and prose; runs as its own CI job and in `just check`. Configured by _typos.toml, which excludes the delivered V3 spec.",
    "install": "cargo binstall -y typos-cli",
    "in_path": "typos"
  },
  {
    "name": "bacon",
    "why": "Background check/clippy/test watcher during development (`just watch`). Configured by bacon.toml; default job is check-all.",
    "install": "cargo binstall -y bacon",
    "in_path": "bacon"
  },
  {
    "name": "git-cliff",
    "why": "Generates CHANGELOG.md from conventional commits (`just changelog`). Configured by cliff.toml.",
    "install": "cargo binstall -y git-cliff",
    "in_path": "git-cliff"
  },
  {
    "name": "cargo-release",
    "why": "Bumps every workspace crate in lockstep and pushes the single `v{version}` tag that .github/workflows/release.yml triggers on. Configured by [workspace.metadata.release] in the root Cargo.toml.",
    "install": "cargo binstall -y cargo-release",
    "in_path": "cargo-release"
  },
  {
    "name": "cargo-mutants",
    "why": "Mutation testing (`just mutants`) for the invariant-heavy library: profile-name validation, resolution precedence, config atomicity. .gitignore already excludes mutants.out*.",
    "install": "cargo binstall -y cargo-mutants",
    "in_path": "cargo-mutants"
  },
  {
    "name": "actionlint",
    "why": "Lints the GitHub Actions workflows. Runs the shell in each `run:` step through shellcheck when shellcheck is on PATH; without it that check is silently skipped (visible only with `actionlint -verbose`).",
    "install": "winget install rhysd.actionlint",
    "in_path": "actionlint"
  },
  {
    "name": "shellcheck",
    "why": "Lets actionlint check the shell inside workflow `run:` steps; actionlint disables that rule when shellcheck is not on PATH.",
    "install": "winget install koalaman.shellcheck",
    "in_path": "shellcheck"
  },
  {
    "name": "Sandboxie-Plus",
    "why": "The Windows sandbox for measuring real coding agents (CONTRIBUTING.md \"Measuring agent behaviour\"): adapter evidence is refreshed only inside a disposable box, never by installing agents on the host. Windows only; Linux and macOS contributors use the Podman/Bubblewrap or Tart/OrbStack recipes in CONTRIBUTING.md, and this check has no platform guard, so it lists Sandboxie-Plus as missing there.",
    "install": "winget install Sandboxie.Plus",
    "file_exists": "C:/Program Files/Sandboxie-Plus/Start.exe"
  }
]
```

- [ ] **Step 5: Run the task checks**

```bash
typos
```

Expected: every listed test passes.

- [ ] **Step 6: Run the gate** (see "Gate commands"). Expected on Windows: `141 tests run: 141 passed`. Linux and macOS run fewer tests (Windows-only tests are compiled out); every test must pass.

- [ ] **Step 7: Commit**

```bash
git add README.md TODO.md CONTRIBUTING.md .claude/recommended-tools.json
git commit -m "docs: SP2 supported agents, known limits and sandbox prerequisite

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```


## After the last task

- [ ] Push `sp2-adapters` and open a PR only with the owner's confirmation. CI (Windows, macOS, Linux) must be green.
- [ ] Run AGY-CAPSTONE (on subagents, owner-directed) over the committed range, then AGY-TEST-AUDIT, before declaring SP2 complete.
- [ ] Mark SP2 done in `ROADMAP.md` after the merge, as SP1 did.
