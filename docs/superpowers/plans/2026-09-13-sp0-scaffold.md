# SP0 Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create the `agent-profile` Cargo workspace — stub binary, `fake-agent` test fixture, flux's development tooling, CI, licence and community docs — green locally and on GitHub on Linux, macOS and Windows.

**Architecture:**
- A virtual workspace with two crates:
  - `crates/agent-profile`: lib with empty, spec-cited modules, plus a Clap stub binary.
  - `crates/fake-agent`: a JSON-echo fixture that later sub-projects launch in contract tests.
- Integration tests live in `crates/agent-profile/tests/`; a helper builds `fake-agent` on demand.
- All tooling runs through `just` recipes shared by lefthook and CI.
- The GitHub rollout is a gated, user-confirmed sequence at the end.

**Tech Stack:** Rust 2024 (MSRV 1.85, stable toolchain), clap 4.6, serde_json 1, cargo-nextest, just, lefthook, cargo-deny, typos, bacon, git-cliff, cargo-release, cargo-mutants, actionlint + shellcheck, GitHub Actions, `gh` CLI.

**Oracle:** `docs/superpowers/specs/2026-09-13-sp0-scaffold-design.md` (SP0 design), which cites `agent-profile-implementation-spec-v3.md`. If a step here disagrees with the design, the design wins — stop and report.

**Provenance:**
- Every Rust file, manifest, tooling config and workflow below was built and run in a scratch copy on
  2026-09-13 (Windows 11, Rust 1.98 stable).
- Results there:
  - `cargo fmt --check` clean
  - `cargo clippy --workspace --all-targets -- -D warnings` clean
  - `cargo nextest run --workspace` 8/8 passed
  - `cargo +1.85 check --workspace --all-targets` clean
  - `just check` green
  - `cargo deny check` → `advisories ok, bans ok, licenses ok, sources ok`
  - `typos` clean
  - `actionlint` → `Found 0 errors in 4 files`
- Community-doc prose (Tasks 6–7) was not pre-run; its gate is `typos` plus review.

**Repository state at start:**
- Branch `sp0-scaffold` at `b1f05ee` (or a later docs-only commit), containing:
  - `agent-profile-implementation-spec-v3.md`
  - `docs/superpowers/specs/2026-09-13-sp0-scaffold-design.md`
  - `docs/superpowers/plans/2026-09-13-sp0-scaffold.md`
  - `.gitignore` (the Cargo template) and a 12-byte `README.md`
- `E:\Rust\flux` is the template repo and is read-only for this plan.

**Shell:** all commands are bash (Git Bash on Windows), run from the repository root `E:\Rust\aiprofiles`.

**Commit trailer:** every commit message in this plan ends with these two lines (shown once here, required on each):

```
Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01UTYq45Pr9gdUcm94AU2z3a
```

---

## File map

| Path | Responsibility | Task |
|---|---|---|
| `Cargo.toml` | Virtual workspace, shared package keys, dependency pins, release metadata | 1 |
| `Cargo.lock` | Locked graph (committed: the workspace ships a binary; `release.yml` builds `--locked`) | 1 |
| `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` | Toolchain + lint config (verbatim from flux) | 1 |
| `.gitignore` | Ignores (flux's, minus Python) | 1 |
| `crates/agent-profile/Cargo.toml` | Main crate manifest | 1 |
| `crates/agent-profile/src/lib.rs` + `name.rs`, `config.rs`, `repo.rs`, `resolve.rs`, `cli.rs`, `output.rs`, `adapter/mod.rs`, `launch/mod.rs` | Empty, spec-cited layer modules | 1 |
| `crates/agent-profile/src/main.rs` | Stub CLI | 1 (placeholder), 2 |
| `crates/fake-agent/Cargo.toml`, `crates/fake-agent/src/main.rs` | Test fixture | 1 (placeholder), 3 |
| `crates/agent-profile/tests/smoke.rs` | SP0 contract tests | 2, 3 |
| `crates/agent-profile/tests/support/mod.rs` | `fake_agent_path()` helper | 3 |
| `justfile`, `bacon.toml`, `cliff.toml`, `lefthook.yml`, `_typos.toml`, `deny.toml`, `.claude/recommended-tools.json` | Dev tooling | 4 |
| `.github/workflows/{ci,docs,release,dependabot-automerge}.yml`, `.github/dependabot.yml` | CI/CD | 5 |
| `LICENSE`, `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md` | Licence + community | 6 |
| `README.md`, `ROADMAP.md`, `TODO.md`, `docs/dev-tooling.md` | Project docs | 7 |

---

### Task 1: Workspace skeleton

**Files:**
- Create: `Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`
- Modify: `.gitignore` (full replacement)
- Create: `crates/agent-profile/Cargo.toml`
- Create: `crates/agent-profile/src/lib.rs`, `crates/agent-profile/src/name.rs`, `crates/agent-profile/src/config.rs`, `crates/agent-profile/src/repo.rs`, `crates/agent-profile/src/resolve.rs`, `crates/agent-profile/src/cli.rs`, `crates/agent-profile/src/output.rs`, `crates/agent-profile/src/adapter/mod.rs`, `crates/agent-profile/src/launch/mod.rs`
- Create: `crates/agent-profile/src/main.rs` (placeholder)
- Create: `crates/fake-agent/Cargo.toml`, `crates/fake-agent/src/main.rs` (placeholder)

- [ ] **Step 0: Verify starting state**

Run: `git branch --show-current && git status --short && ls`
Expected:
- The branch is `sp0-scaffold`.
- No tracked modifications (untracked `.serena/` is fine).
- The listing shows `README.md`, `agent-profile-implementation-spec-v3.md` and `docs`, with no `Cargo.toml` and no `crates/`.

If anything differs, STOP and report `STATE_MISMATCH: <what>`.

- [ ] **Step 1: Copy the verbatim toolchain files from flux**

```bash
cp /e/Rust/flux/rust-toolchain.toml /e/Rust/flux/rustfmt.toml /e/Rust/flux/clippy.toml .
```

Expected contents — check with `cat`. If they differ, STOP and report:
- `rust-toolchain.toml` sets `channel = "stable"` and components `rustfmt`, `clippy`.
- `rustfmt.toml` sets `edition = "2024"`, `max_width = 100`, `use_small_heuristics = "Max"`, `reorder_imports = true`.
- `clippy.toml` sets `msrv = "1.85"`, `allow-dbg-in-tests = true`, `allow-unwrap-in-tests = true`.

- [ ] **Step 2: Create `Cargo.toml`**

```toml
# Agent Profile workspace root.
#
# Virtual workspace: no root package. The shipped binary is `agent-profile`, produced by
# crates/agent-profile. crates/fake-agent is a test-only fixture and never ships.

[workspace]
resolver = "3"
members = [
    "crates/agent-profile",
    "crates/fake-agent",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
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
thiserror = "2"
tempfile = "3"
proptest = "1.11"

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

- [ ] **Step 3: Replace `.gitignore`**

```gitignore
# Generated by Cargo
# will have compiled files and executables
debug
target

# These are backup files generated by rustfmt
**/*.rs.bk

# MSVC Windows builds of rustc generate these, which store debugging information
*.pdb

# Generated by cargo mutants
# Contains mutation testing data
**/mutants.out*/

# RustRover
#  JetBrains specific template is maintained in a separate JetBrains.gitignore that can
#  be found at https://github.com/github/gitignore/blob/main/Global/JetBrains.gitignore
#  and can be added to the global gitignore or merged into this file.  For a more nuclear
#  option (not recommended) you can uncomment the following to ignore the entire idea folder.
#.idea/

# Local agent/tool state (machine-specific, not project config)
.serena/
.clavity/
```

- [ ] **Step 4: Create `crates/agent-profile/Cargo.toml`**

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

[dependencies]
clap = { workspace = true }

[dev-dependencies]
serde_json = { workspace = true }
```

- [ ] **Step 5: Create the library and its layer modules**

`crates/agent-profile/src/lib.rs`:

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
pub mod launch;
pub mod name;
pub mod output;
pub mod repo;
pub mod resolve;
```

`crates/agent-profile/src/name.rs`:

```rust
//! Profile-name validation (spec §6) and the reserved command words (spec §5.3).
```

`crates/agent-profile/src/config.rs`:

```rust
//! `config.toml`: corruption handling (spec §17) and locked, atomic writes (spec §18).
```

`crates/agent-profile/src/repo.rs`:

```rust
//! Repository discovery (spec §13) and canonical repository identity (spec §14).
```

`crates/agent-profile/src/resolve.rs`:

```rust
//! The single profile resolver shared by every command (spec §12).
```

`crates/agent-profile/src/cli.rs`:

```rust
//! CLI grammar: launch syntax, wrapper options and reserved command words (spec §5).
```

`crates/agent-profile/src/output.rs`:

```rust
//! Human and JSON output (spec §32) and exit codes (spec §33).
```

`crates/agent-profile/src/adapter/mod.rs`:

```rust
//! Agent adapters: supported agents (spec §2), capability semantics (spec §3), argument
//! conflicts (spec §21) and evidence metadata (spec §28).
```

`crates/agent-profile/src/launch/mod.rs`:

```rust
//! `LaunchPlan` (spec §4), environment overrides (spec §22), launch semantics (spec §23, §24)
//! and the launcher API (spec §25).
```

- [ ] **Step 6: Create the placeholder binary for `agent-profile`**

`crates/agent-profile/src/main.rs` (replaced in Task 2):

```rust
//! The `agent-profile` binary. Placeholder until Task 2.

fn main() {}
```

- [ ] **Step 7: Create `crates/fake-agent`**

`crates/fake-agent/Cargo.toml`:

```toml
[package]
name = "fake-agent"
description = "Test-only stand-in for a coding agent: echoes argv, cwd and selected env as JSON"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
publish.workspace = true

[dependencies]
serde_json = { workspace = true }
```

`crates/fake-agent/src/main.rs` (replaced in Task 3):

```rust
//! Test-only stand-in for a coding agent. Placeholder until Task 3.

fn main() {}
```

- [ ] **Step 8: Build and check the workspace metadata**

Run: `cargo build --workspace`
Expected: ends with `Finished \`dev\` profile`.

Run: `cargo metadata --no-deps --format-version 1 | jq -c '.packages[] | {name, publish, license, rust_version}'`
Expected (order may vary):
```
{"name":"agent-profile","publish":[],"license":"PolyForm-Noncommercial-1.0.0","rust_version":"1.85"}
{"name":"fake-agent","publish":[],"license":"PolyForm-Noncommercial-1.0.0","rust_version":"1.85"}
```
`"publish":[]` is how cargo reports `publish = false`. If `publish` is `null`, a member failed to opt in: STOP.

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml clippy.toml .gitignore crates
git commit -m "build: scaffold the agent-profile workspace

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01UTYq45Pr9gdUcm94AU2z3a"
```

---

### Task 2: `agent-profile` stub CLI

**Oracle:** SP0 design §3.2 and §4 item 3.
- `--version` exits 0 and prints the crate version.
- Every other invocation prints `not yet implemented` to stderr and exits 2 (V3 §33 usage error).
- No subcommands are declared.

**Files:**
- Create: `crates/agent-profile/tests/smoke.rs`
- Modify: `crates/agent-profile/src/main.rs` (full replacement)

- [ ] **Step 0: Verify state**

Run: `cat crates/agent-profile/src/main.rs`
Expected: the Task 1 placeholder (`fn main() {}`). Otherwise STOP with `STATE_MISMATCH`.

- [ ] **Step 1: Write the failing tests**

Create `crates/agent-profile/tests/smoke.rs`:

```rust
//! SP0 smoke tests: the stub binary's contract and the fake-agent fixture's contract.

use std::process::{Command, Output};

fn agent_profile(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-profile")).args(args).output().unwrap()
}

#[test]
fn version_exits_zero() {
    let output = agent_profile(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "{stdout}");
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo nextest run -p agent-profile --test smoke`
Expected: 2 tests run, both FAIL.
- `version_exits_zero`: the placeholder prints nothing, so the `contains` assertion fails.
- `unimplemented_invocation_is_usage_error`: `left: Some(0)`, `right: Some(2)`.

- [ ] **Step 3: Implement the stub**

Replace `crates/agent-profile/src/main.rs`:

```rust
//! The `agent-profile` binary. SP0 scaffold: only `--version` and `--help` work.

use std::ffi::OsString;
use std::process::ExitCode;

use clap::Parser;

/// Spec §33: exit code for a CLI usage error.
const USAGE_ERROR: u8 = 2;

/// Select and launch profiles for multiple coding agents.
///
/// This is the SP0 scaffold: nothing is implemented yet. See ROADMAP.md.
#[derive(Parser)]
#[command(name = "agent-profile", version)]
struct Cli {
    /// Everything else is accepted and rejected as not yet implemented.
    #[arg(hide = true, trailing_var_arg = true, allow_hyphen_values = true)]
    rest: Vec<OsString>,
}

fn main() -> ExitCode {
    let _cli = Cli::parse();
    eprintln!(
        "agent-profile: not yet implemented (this build is the SP0 scaffold; see ROADMAP.md)"
    );
    ExitCode::from(USAGE_ERROR)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p agent-profile --test smoke`
Expected: `2 tests run: 2 passed, 0 skipped`.

Run: `cargo run -q -p agent-profile -- --help`
Expected: output contains `Select and launch profiles for multiple coding agents.` and `This is the SP0 scaffold: nothing is implemented yet. See ROADMAP.md.`, lists only `-h, --help` and `-V, --version`, and exits 0.

- [ ] **Step 5: Lint**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: no diff output; clippy ends with `Finished`, no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/agent-profile/src/main.rs crates/agent-profile/tests/smoke.rs Cargo.lock
git commit -m "feat(cli): stub agent-profile binary with --version and usage-error exit

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01UTYq45Pr9gdUcm94AU2z3a"
```

---

### Task 3: `fake-agent` fixture and test helper

**Oracle:** SP0 design §3.3 (fixture contract), §3.4 (helper) and §4 item 3 (test list).
- stdout carries exactly one JSON object `{"argv","cwd","env"}`.
- `env` includes only variables listed in `FAKE_AGENT_ECHO_ENV` that are actually set; empty list entries are ignored; unset or empty `FAKE_AGENT_ECHO_ENV` gives `{}`.
- `FAKE_AGENT_EXIT`: unset means 0; otherwise it must be an integer `0..=255`.
- Fixture errors print nothing to stdout and exit 125: an empty, non-numeric or out-of-range `FAKE_AGENT_EXIT`, or any non-UTF-8 argument, cwd or echoed value.

**Files:**
- Create: `crates/agent-profile/tests/support/mod.rs`
- Modify: `crates/agent-profile/tests/smoke.rs` (full replacement)
- Modify: `crates/fake-agent/src/main.rs` (full replacement)

- [ ] **Step 0: Verify state**

Run: `cat crates/fake-agent/src/main.rs && grep -c '#\[test\]' crates/agent-profile/tests/smoke.rs`
Expected: the Task 1 placeholder and `2`. Otherwise STOP with `STATE_MISMATCH`.

- [ ] **Step 1: Write the helper**

Create `crates/agent-profile/tests/support/mod.rs`:

```rust
//! Shared helpers for integration tests (SP0 design §3.4).

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

/// Path to the `fake-agent` fixture binary, built on first use.
///
/// `CARGO_BIN_EXE_*` only covers binaries of this package, so the fixture — a separate package —
/// is built here with `cargo build -p fake-agent`. The cache is per test process; concurrent
/// builds from parallel test processes are serialized by cargo's build-directory lock.
pub fn fake_agent_path() -> PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(build_fake_agent).clone()
}

fn build_fake_agent() -> PathBuf {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(&cargo)
        .args(["build", "-p", "fake-agent", "--message-format=json"])
        .output()
        .unwrap_or_else(|e| panic!("failed to run {cargo:?}: {e}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "cargo build -p fake-agent failed\n{stdout}\n{stderr}");

    let mut executables = Vec::new();
    for line in stdout.lines() {
        let message: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("unparsable cargo message {line:?}: {e}\n{stderr}"));
        let is_bin = message["target"]["kind"]
            .as_array()
            .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"));
        let selected = message["reason"] == "compiler-artifact"
            && message["target"]["name"] == "fake-agent"
            && is_bin;
        // No let-chains here: they need Rust 1.88 and the workspace MSRV is 1.85.
        if let (true, Some(executable)) = (selected, message["executable"].as_str()) {
            executables.push(PathBuf::from(executable));
        }
    }
    assert_eq!(
        executables.len(),
        1,
        "expected exactly one fake-agent executable\n{stdout}\n{stderr}"
    );
    executables.remove(0)
}
```

- [ ] **Step 2: Write the failing fixture tests**

Replace `crates/agent-profile/tests/smoke.rs`. It keeps Task 2's two tests unchanged and adds `mod support`, `stdout_json` and six fixture tests:

```rust
//! SP0 smoke tests: the stub binary's contract and the fake-agent fixture's contract.

mod support;

use std::process::{Command, Output};

fn agent_profile(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-profile")).args(args).output().unwrap()
}

fn stdout_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn version_exits_zero() {
    let output = agent_profile(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "{stdout}");
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
    let output = Command::new(support::fake_agent_path()).args(args).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["argv"], serde_json::json!(args));
}

#[test]
fn fake_agent_exit_code_is_controllable() {
    let output =
        Command::new(support::fake_agent_path()).env("FAKE_AGENT_EXIT", "7").output().unwrap();
    assert_eq!(output.status.code(), Some(7));
}

#[test]
fn fake_agent_echoes_only_listed_env() {
    let output = Command::new(support::fake_agent_path())
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
    let output = Command::new(support::fake_agent_path())
        .env_remove("FAKE_AGENT_ECHO_ENV")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["env"], serde_json::json!({}));
}

#[test]
fn fake_agent_invalid_exit_code_is_fixture_error() {
    for value in ["", "256", "-1", "seven"] {
        let output = Command::new(support::fake_agent_path())
            .env("FAKE_AGENT_EXIT", value)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(125), "FAKE_AGENT_EXIT={value:?}");
        assert!(output.stdout.is_empty(), "FAKE_AGENT_EXIT={value:?}");
    }
}

#[test]
fn fake_agent_non_utf8_argument_is_fixture_error() {
    let output = Command::new(support::fake_agent_path()).arg(non_utf8()).output().unwrap();
    assert_eq!(output.status.code(), Some(125));
    assert!(output.stdout.is_empty());
}

#[cfg(unix)]
fn non_utf8() -> std::ffi::OsString {
    use std::os::unix::ffi::OsStringExt;
    std::ffi::OsString::from_vec(vec![0x66, 0xff])
}

#[cfg(windows)]
fn non_utf8() -> std::ffi::OsString {
    use std::os::windows::ffi::OsStringExt;
    // A lone surrogate: valid in an OS string, not valid UTF-8.
    std::ffi::OsString::from_wide(&[0x0066, 0xD800])
}
```

- [ ] **Step 3: Run the tests to verify the fixture tests fail**

Run: `cargo nextest run -p agent-profile --test smoke --no-fail-fast`
Expected: 8 tests run.
- PASS: `version_exits_zero`, `unimplemented_invocation_is_usage_error`.
- FAIL (6): all `fake_agent_*` tests. The placeholder prints nothing and exits 0, so JSON parsing panics or the exit-code assertions fail.

If a `fake_agent_*` test passes here, STOP: the test is not exercising the fixture.

- [ ] **Step 4: Implement the fixture**

Replace `crates/fake-agent/src/main.rs`:

```rust
//! Test-only stand-in for a coding agent (SP0 design §3.3).
//!
//! Prints one JSON object `{"argv": [...], "cwd": "...", "env": {...}}` to stdout and exits with
//! the code in `FAKE_AGENT_EXIT` (default 0). `env` holds only the variables named in the
//! comma-separated `FAKE_AGENT_ECHO_ENV`. Any fixture error — a non-UTF-8 argument, cwd or echoed
//! value, or an invalid `FAKE_AGENT_EXIT` — prints nothing to stdout and exits 125.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::process::ExitCode;

/// Reserved for fixture errors, so a test can tell them apart from a requested exit code.
const FIXTURE_ERROR: u8 = 125;

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
    let code = requested_exit_code()?;

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

    let report = serde_json::json!({ "argv": argv, "cwd": cwd, "env": env });
    println!("{report}");
    Ok(code)
}

/// `FAKE_AGENT_EXIT`: unset means 0; otherwise an integer in `0..=255`.
fn requested_exit_code() -> Result<u8, String> {
    match std::env::var_os("FAKE_AGENT_EXIT") {
        None => Ok(0),
        Some(value) => value
            .to_str()
            .and_then(|s| s.parse::<u8>().ok())
            .ok_or_else(|| format!("FAKE_AGENT_EXIT must be an integer in 0..=255, got {value:?}")),
    }
}

fn utf8(value: OsString, what: &str) -> Result<String, String> {
    value.into_string().map_err(|raw| format!("{what} is not valid UTF-8: {raw:?}"))
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo nextest run --workspace --no-tests=pass`
Expected: `8 tests run: 8 passed, 0 skipped`.

Run: `cargo test -p agent-profile --test smoke`
Expected: `test result: ok. 8 passed; 0 failed`. This proves the helper also works under plain `cargo test`, not only nextest.

- [ ] **Step 6: Lint and check the MSRV**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: clean.

Run: `rustup toolchain install 1.85 --profile minimal && cargo +1.85 check --workspace --all-targets`
Expected: ends with `Finished`. A failure here means code newer than the MSRV slipped in (e.g. let-chains): fix the code, do not bump `rust-version`.

- [ ] **Step 7: Commit**

```bash
git add crates/fake-agent/src/main.rs crates/agent-profile/tests Cargo.lock
git commit -m "test: add fake-agent fixture and its contract tests

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01UTYq45Pr9gdUcm94AU2z3a"
```

---

### Task 4: Development tooling

**Oracle:** SP0 design §5 and §5.1.

**Files:**
- Create: `bacon.toml`, `cliff.toml`, `lefthook.yml` (verbatim from flux)
- Create: `justfile`, `_typos.toml`, `deny.toml`, `.claude/recommended-tools.json`

- [ ] **Step 1: Copy the verbatim tooling configs**

```bash
cp /e/Rust/flux/bacon.toml /e/Rust/flux/cliff.toml /e/Rust/flux/lefthook.yml .
# grep exits 0 on a match, 1 on no match, 2 on an error such as a missing file; only 1 is clean.
grep -i -n flux bacon.toml cliff.toml lefthook.yml; test $? -eq 1 && echo NO-FLUX-REFERENCES
```
Expected: exactly `NO-FLUX-REFERENCES`. STOP and report if instead:
- a `file:line` match prints (flux's copy changed since this plan was written), or
- a "No such file" error prints (a copy failed).

- [ ] **Step 2: Create `justfile`**

```just
# Justfile for Agent Profile

# Default: run the local gate
default:
    just check

# Build the workspace
build:
    cargo build --workspace

# Run all tests
test:
    cargo nextest run --workspace --no-tests=pass
    cargo test --doc --workspace

# Run tests without stopping at the first failure
test-verbose:
    cargo nextest run --workspace --no-tests=pass --no-fail-fast

# Clippy, warnings are errors
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Format check
fmt-check:
    cargo fmt --check

# Format all code
fmt:
    cargo fmt --all

# Spell check
typos:
    typos

# Dependency advisories, licences, bans and sources
deny:
    cargo deny check

# The local gate: fmt + clippy + typos + test
check: fmt-check clippy typos test

# Background watcher
watch:
    bacon

# Build the docs
doc:
    cargo doc --workspace --no-deps --open

# Install the git hooks (once per clone)
hooks:
    lefthook install

# Generate the changelog
changelog:
    git-cliff --output CHANGELOG.md

# Mutation testing over the library
mutants:
    cargo mutants --package agent-profile

# Clean build artifacts
clean:
    cargo clean

# Release: bump every crate in lockstep, tag, commit
# Usage: just release <patch|minor|major>
release VERSION_BUMP:
    cargo release {{VERSION_BUMP}} --workspace --execute
```

- [ ] **Step 3: Create `_typos.toml`**

```toml
# https://github.com/crate-ci/typos

[files]
extend-exclude = [
    "target/",
    "Cargo.lock",
    # The V3 specification is a delivered document; do not rewrite it.
    "agent-profile-implementation-spec-v3.md",
]

[default]
extend-ignore-re = [
    # Long hex digests (commit SHAs, hashes)
    "[0-9a-fA-F]{32,}",
]

[default.extend-words]
# Domain terms and names that look like typos
ckir = "ckir"
aiprofiles = "aiprofiles"
```

- [ ] **Step 4: Create `deny.toml`**

This is flux's file with only three comment/reason edits, at lines 5, 37 and 49–51.

```toml
# cargo-deny configuration
# https://embarkstudios.github.io/cargo-deny/

[graph]
# agent-profile ships on Linux, macOS and Windows (V3 spec §37) — audit all of them.
targets = [
    { triple = "x86_64-unknown-linux-gnu" },
    { triple = "aarch64-unknown-linux-gnu" },
    { triple = "x86_64-unknown-linux-musl" },
    { triple = "x86_64-apple-darwin" },
    { triple = "aarch64-apple-darwin" },
    { triple = "x86_64-pc-windows-msvc" },
    { triple = "aarch64-pc-windows-msvc" },
]

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
yanked = "deny"
ignore = []

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "MPL-2.0",
    "Zlib",
    "CC0-1.0",
]
confidence-threshold = 0.8
# This workspace's crates are PolyForm-Noncommercial-1.0.0 and publish = false. Exempt them
# here rather than adding PolyForm to `allow` — that would also silently permit a
# noncommercial *dependency*, which we must never ship.
private = { ignore = true }
exceptions = []

[bans]
multiple-versions = "warn"
wildcards = "allow"
highlight = "all"
allow = []
deny = [
    # V3 spec §36: agent-profile makes no hidden network requests and sends no telemetry.
    # A TLS stack appearing in the graph means something pulled in more than we intended.
    { name = "openssl-sys", reason = "agent-profile does no networking (V3 §36); prefer rustls if TLS is ever needed" },
]
skip = []
skip-tree = []

[sources]
unknown-registry = "warn"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

- [ ] **Step 5: Create `.claude/recommended-tools.json`**

```json
[
  {
    "name": "cargo-nextest",
    "why": "Test runner used by CI (`cargo nextest run --workspace`). Runs each test in its own process, which matters here because agent-profile tests spawn the binary and the fake-agent fixture and set per-process environment. Does NOT run doctests \u2014 pair with `cargo test --doc`.",
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
    "why": "Advisory, licence, ban and source auditing across the dependency graph. Configured by deny.toml, which pins the allow-listed licences, bans openssl-sys (V3 spec \u00a736: no hidden network activity) and lists the per-platform target triples.",
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
  }
]
```

Run: `jq length .claude/recommended-tools.json`
Expected: `11`.

- [ ] **Step 6: Run the local gate and the dependency audit**

Run: `just check`
Expected:
- fmt-check prints nothing.
- clippy ends with `Finished`.
- typos prints nothing.
- nextest reports `8 tests run: 8 passed`.
- The doctest run ends `test result: ok.`
- Exit status 0.

If `typos` flags a word, fix the word in the offending file. Add it to `_typos.toml` `extend-words` only if it is a genuine domain term, and name it in your report.

Run: `just deny`
Expected: ends with `advisories ok, bans ok, licenses ok, sources ok`. `unmatched license allowance` warnings are expected and not failures.

- [ ] **Step 7: Commit**

```bash
git add justfile bacon.toml cliff.toml lefthook.yml _typos.toml deny.toml .claude/recommended-tools.json
git commit -m "build: port flux's development tooling

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01UTYq45Pr9gdUcm94AU2z3a"
```

---

### Task 5: GitHub workflows

**Oracle:** SP0 design §5.1 (workflow adaptations, `Docs build` job) and §4.1 (check names).
- The required check names are exactly `Format`, `Typos`, `Clippy`, `Cargo deny`, `Docs build`, `Test (ubuntu-latest)`, `Test (macos-latest)`, `Test (windows-latest)`.
- A job `name:` that doesn't produce these names breaks §4.1: STOP rather than rename.

**Files:**
- Create: `.github/dependabot.yml` (verbatim from flux)
- Create: `.github/workflows/ci.yml`, `.github/workflows/docs.yml`, `.github/workflows/release.yml`, `.github/workflows/dependabot-automerge.yml`

- [ ] **Step 1: Copy `dependabot.yml` verbatim**

```bash
mkdir -p .github/workflows
cp /e/Rust/flux/.github/dependabot.yml .github/dependabot.yml
# grep exits 0 on a match, 1 on no match, 2 on an error such as a missing file; only 1 is clean.
grep -i -n flux .github/dependabot.yml; test $? -eq 1 && echo NO-FLUX-REFERENCES
```
Expected: exactly `NO-FLUX-REFERENCES`. Any match or "No such file" error: STOP and report.

- [ ] **Step 2: Create `.github/workflows/ci.yml`**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  typos:
    name: Typos
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: crate-ci/typos@master

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --workspace --all-targets -- -D warnings

  deny:
    name: Cargo deny
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: EmbarkStudios/cargo-deny-action@v2

  docs-build:
    # Not in flux: docs.yml only runs after merge, so without this job a PR could merge with a
    # broken docs build (SP0 design §5.1). Same command as docs.yml's build step.
    name: Docs build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Build docs
        env:
          RUSTDOCFLAGS: -D warnings
        run: cargo doc --workspace --no-deps --document-private-items

  test:
    # agent-profile launches processes, handles console signals and canonicalizes paths; all of
    # that differs per platform, so every OS is a first-class gate (V3 spec §37).
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@nextest
      - name: Run tests
        run: cargo nextest run --workspace --no-tests=pass --no-fail-fast
      - name: Run doctests
        run: cargo test --doc --workspace
```

- [ ] **Step 3: Create `.github/workflows/docs.yml`**

```yaml
name: Docs

on:
  push:
    branches: [main]
  workflow_dispatch:

# Least privilege, workflow-wide. `pages: write` + `id-token: write` are granted
# per-job, on `deploy` only — the `build` job runs `cargo doc`, which executes
# dependency build scripts, and that job has no business holding a Pages OIDC
# token. A job-level permissions block overrides this default for that job.
permissions:
  contents: read

# One deployment at a time. Do NOT cancel a run in progress: cancelling a
# half-finished Pages deploy can leave the site in a broken state.
concurrency:
  group: pages
  cancel-in-progress: false

jobs:
  build:
    name: Build docs
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Build docs
        # -D warnings catches broken intra-doc links, which are otherwise
        # invisible until someone clicks a dead link on the published site.
        # --document-private-items because every crate here is publish = false:
        # the audience is contributors, not downstream API consumers.
        env:
          RUSTDOCFLAGS: -D warnings
        run: cargo doc --workspace --no-deps --document-private-items

      - name: Generate workspace index
        # `cargo doc` on a workspace produces no root page, so without this the
        # Pages root is a 404.
        run: |
          cat << 'EOF' > target/doc/index.html
          <!DOCTYPE html>
          <html lang="en">
          <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Agent Profile Workspace Documentation</title>
            <style>
              :root { color-scheme: light dark; }
              body {
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                max-width: 820px; margin: 40px auto; padding: 0 20px;
                line-height: 1.6;
              }
              h1 { border-bottom: 1px solid #8884; padding-bottom: 10px; }
              ul { list-style: none; padding: 0; }
              li { margin: 12px 0; }
              a { color: #0366d6; text-decoration: none; }
              @media (prefers-color-scheme: dark) { a { color: #58a6ff; } }
              a:hover { text-decoration: underline; }
              .crate {
                font-weight: bold; font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
                background: #8882; padding: 2px 6px; border-radius: 4px;
              }
              .note { font-size: .92em; opacity: .75; }
            </style>
          </head>
          <body>
            <h1>Agent Profile Workspace Documentation</h1>
            <p>
              A local, privacy-first CLI for selecting and launching profiles for multiple coding agents.
              Module responsibilities follow &sect;4 of the V3 implementation specification.
            </p>
            <ul>
              <li><a href="agent_profile/index.html"><span class="crate">agent-profile</span></a> &mdash; library and the <code>agent-profile</code> binary: naming, configuration, repository discovery, resolution, adapters, launcher, CLI, output</li>
              <li><a href="fake_agent/index.html"><span class="crate">fake-agent</span></a> &mdash; test-only stand-in for a coding agent; never shipped</li>
            </ul>
            <p class="note">
              Private items are documented: every crate is <code>publish = false</code>,
              so the audience is contributors rather than downstream consumers.
            </p>
          </body>
          </html>
          EOF

      # No actions/configure-pages: its only job is computing base_url for static
      # site generators, which rustdoc does not consume, and `enablement` defaults
      # to false so it never creates the site either. Pages is enabled with
      # build_type=workflow (SP0 design §4.1 step 3), so the step is dead weight — and
      # dropping it keeps the build job honestly at contents:read.
      - uses: actions/upload-pages-artifact@v3
        with:
          path: target/doc

  deploy:
    name: Deploy to Pages
    needs: build
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 4: Create `.github/workflows/release.yml`**

```yaml
name: Release

on:
  push:
    tags: ["v*"]

env:
  CARGO_TERM_COLOR: always

jobs:
  release:
    name: ${{ matrix.target }}
    runs-on: ${{ matrix.runner }}
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - runner: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            archive: tar.gz
          - runner: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            archive: tar.gz
            cross: true
          - runner: macos-latest
            target: x86_64-apple-darwin
            archive: tar.gz
          - runner: macos-latest
            target: aarch64-apple-darwin
            archive: tar.gz
          - runner: windows-latest
            target: x86_64-pc-windows-msvc
            archive: zip
    steps:
      - uses: actions/checkout@v7

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: Swatinem/rust-cache@v2

      - name: Install cross linker
        if: matrix.cross
        run: |
          sudo apt-get update
          sudo apt-get install -y gcc-aarch64-linux-gnu
          echo "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc" >> "$GITHUB_ENV"

      - name: Build
        # --bin agent-profile only: the fake-agent test fixture never ships.
        run: cargo build --release --locked --target ${{ matrix.target }} --bin agent-profile

      - name: Package (tar.gz)
        if: matrix.archive == 'tar.gz'
        run: |
          tar -czf "agent-profile-${{ matrix.target }}.tar.gz" -C "target/${{ matrix.target }}/release" agent-profile

      - name: Package (zip)
        if: matrix.archive == 'zip'
        shell: pwsh
        run: |
          Compress-Archive -Path "target/${{ matrix.target }}/release/agent-profile.exe" -DestinationPath "agent-profile-${{ matrix.target }}.zip"

      - name: Upload to release
        uses: softprops/action-gh-release@v2
        with:
          files: agent-profile-${{ matrix.target }}.*
```

- [ ] **Step 5: Create `.github/workflows/dependabot-automerge.yml`**

```yaml
name: Dependabot auto-merge

# Turns on GitHub's native auto-merge for Dependabot PRs carrying only patch or
# minor bumps. Auto-merge does NOT merge immediately — it waits for the required
# status checks configured in main's branch protection (Format, Typos, Clippy,
# Cargo deny, Docs build, and Test on all three OSes). If any check fails, the PR just sits
# there. Branch protection is therefore load-bearing: without required checks,
# auto-merge would merge as soon as the PR is mergeable, which is immediately.
#
# Trigger is `pull_request`, not `pull_request_target`, per GitHub's documented
# recipe. Dependabot runs get a read-only token by default, so the permissions
# block below is required for `gh pr merge` to work. Nothing here checks out or
# executes the PR's code.

on: pull_request

permissions:
  contents: write
  pull-requests: write

jobs:
  automerge:
    name: Enable auto-merge
    runs-on: ubuntu-latest
    if: github.event.pull_request.user.login == 'dependabot[bot]'
    steps:
      - name: Fetch Dependabot metadata
        id: meta
        uses: dependabot/fetch-metadata@v2
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}

      - name: Enable auto-merge for patch and minor updates
        if: |
          steps.meta.outputs.update-type == 'version-update:semver-patch' ||
          steps.meta.outputs.update-type == 'version-update:semver-minor'
        run: gh pr merge --auto --squash "$PR_URL"
        env:
          PR_URL: ${{ github.event.pull_request.html_url }}
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Leave majors for review
        if: |
          steps.meta.outputs.update-type != 'version-update:semver-patch' &&
          steps.meta.outputs.update-type != 'version-update:semver-minor'
        run: |
          echo "::notice::${{ steps.meta.outputs.dependency-names }} is a ${{ steps.meta.outputs.update-type }} update — not auto-merging. Review it by hand."
```

- [ ] **Step 6: Lint the workflows with shellcheck active**

`actionlint` and `shellcheck` are installed by winget but may not be on the Git Bash PATH. Add them for this shell:

```bash
export PATH="$PATH:/c/Users/user/AppData/Local/Microsoft/WinGet/Packages/koalaman.shellcheck_Microsoft.Winget.Source_8wekyb3d8bbwe:/c/Users/user/AppData/Local/Microsoft/WinGet/Packages/rhysd.actionlint_Microsoft.Winget.Source_8wekyb3d8bbwe"
command -v actionlint shellcheck
```
Expected: two paths printed. If either is missing, run `winget install rhysd.actionlint` / `winget install koalaman.shellcheck` and retry.

Run: `actionlint -verbose 2>&1 | grep -E 'Found|disabled'`
Expected:
- A line `Found 0 errors in 4 files`.
- No line saying the `shellcheck` rule was disabled. A `pyflakes` disabled line is fine.

- [ ] **Step 7: Check the job names that §4.1 will require**

Run: `grep -E '^\s+name: (Format|Typos|Clippy|Cargo deny|Docs build|Test \(\$\{\{ matrix.os \}\}\))$' .github/workflows/ci.yml`
Expected: exactly 6 lines — `Format`, `Typos`, `Clippy`, `Cargo deny`, `Docs build`, `Test (${{ matrix.os }})`. The test matrix expands the last one into the three `Test (...)` checks.

- [ ] **Step 8: Commit**

```bash
git add .github
git commit -m "ci: add CI, docs, release and Dependabot workflows

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01UTYq45Pr9gdUcm94AU2z3a"
```

---

### Task 6: Licence and community policy docs

**Oracle:** SP0 design §6 — `LICENSE` and `CODE_OF_CONDUCT.md` are verbatim; `CONTRIBUTING.md` and `SECURITY.md` are adapted, with the threat model taken from V3 §1/§36.

**Files:**
- Create: `LICENSE`, `CODE_OF_CONDUCT.md` (verbatim from flux)
- Create: `CONTRIBUTING.md`, `SECURITY.md`

- [ ] **Step 1: Copy the verbatim files**

```bash
cp /e/Rust/flux/LICENSE /e/Rust/flux/CODE_OF_CONDUCT.md .
head -1 LICENSE
# grep exits 0 on a match, 1 on no match, 2 on an error such as a missing file; only 1 is clean.
grep -i -n flux LICENSE CODE_OF_CONDUCT.md; test $? -eq 1 && echo NO-FLUX-REFERENCES
```
Expected: `# PolyForm Noncommercial License 1.0.0`, then exactly `NO-FLUX-REFERENCES`. Any match or "No such file" error: STOP and report.

- [ ] **Step 2: Create `CONTRIBUTING.md`**

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

Requires Rust 1.85+ (edition 2024). The toolchain is pinned by `rust-toolchain.toml`.

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

Tests that launch an "agent" use the `fake-agent` fixture (`crates/fake-agent`), located through
`crates/agent-profile/tests/support`. Never launch a real coding agent from a test.

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

- [ ] **Step 3: Create `SECURITY.md`**

```markdown
# Security Policy

## Supported versions

Agent Profile is pre-release. Until 0.1.0 ships there is no supported version and no backported
fixes — security work lands on `main`.

| Version | Supported |
|---|---|
| `main` (pre-0.1) | ✅ |

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Report privately through GitHub's private vulnerability reporting: go to the
[Security tab](https://github.com/ckir/aiprofiles/security/advisories) of this repository and choose
**Report a vulnerability**. That opens a private advisory visible only to the maintainers.

Please include:

- A description of the issue and the impact you believe it has
- Steps to reproduce, ideally a minimal one
- The OS, the coding agent and its version, and the Agent Profile version
- Any relevant `--json` or `--dry-run` output, with secrets and private paths redacted

You can expect an acknowledgement within a few days. Please give us a reasonable window to ship a fix
before disclosing publicly.

## What counts as a vulnerability in Agent Profile

Agent Profile selects and launches coding-agent profiles. Its security boundary is set by spec §1 and
§36: it must never handle credentials, leak secrets, or let a repository choose a profile on its own.
Reports we especially want:

- **Credential handling.** Any path where Agent Profile copies OAuth credentials, extracts tokens,
  creates or stores credentials, or migrates credentials between profiles.
- **Secret disclosure.** Tokens, authorization headers, secret environment values or credential
  contents appearing in human output, `--json` output, `--dry-run`, `--verbose`, `doctor` or logs.
- **Repository-controlled selection.** A file or setting inside a repository that causes a profile to
  be selected or launched without the user explicitly linking it.
- **Escaping the profile root.** `delete` (or any other operation) removing or writing files outside
  `<application-root>/profiles/`, via crafted profile names, symlinks, junctions, reparse points or
  Windows device names.
- **Configuration corruption.** A crash, race or interrupted write that leaves `config.toml` partially
  written, or that silently replaces invalid configuration with defaults.
- **Shell mediation.** An agent launch that passes through `cmd.exe`, PowerShell or a Unix shell, so
  that arguments or profile values can be interpreted as shell syntax.
- **Hidden network activity.** Any network request or telemetry made by Agent Profile itself.

Crashes and panics on malformed input are bugs — please file them as normal issues unless they lead
to one of the outcomes above.

## Scope

Out of scope:
- The coding agents' own authentication, credential storage and network activity. That is the
  agent's responsibility (spec §36).
- Vulnerabilities in dependencies (report those upstream, though please tell us so we can pin or drop
  them).
- Issues that require an attacker who already has write access to the user's Agent Profile
  application root.
```

- [ ] **Step 4: Spell-check and commit**

Run: `typos`
Expected: no output, exit 0. Fix any flagged word in the prose itself.

```bash
git add LICENSE CODE_OF_CONDUCT.md CONTRIBUTING.md SECURITY.md
git commit -m "docs: add licence, code of conduct, contributing guide and security policy

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01UTYq45Pr9gdUcm94AU2z3a"
```

---

### Task 7: README, roadmap, TODO and tooling doc

**Oracle:**
- SP0 design §1 (SP table), §5 (tool table) and §6 (doc treatments).
- The README agent table MUST be headed as planned and not evidence-verified (V3 §2/§3).

**Files:**
- Modify: `README.md` (full replacement)
- Create: `ROADMAP.md`, `TODO.md`, `docs/dev-tooling.md`

- [ ] **Step 1: Replace `README.md`**

````markdown
# Agent Profile

A local, privacy-first Rust CLI for selecting and launching profiles for multiple coding agents.

> `agent-profile` owns profile selection. The coding agent owns authentication and agent-specific
> configuration. The evidence determines what `agent-profile` is allowed to claim.

Agent Profile never copies credentials, extracts tokens, sends telemetry or trusts
repository-controlled profile selection.

**Status: scaffold.** The workspace, tooling and CI exist; no profile behaviour is implemented yet. The
authoritative design is
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

## Usage (planned)

```text
agent-profile <agent> <profile> [WRAPPER OPTIONS] [-- <agent args...>]
agent-profile <agent> [WRAPPER OPTIONS] [-- <agent args...>]
```

Everything after `--` is passed to the agent untouched. Profile resolution order: explicit profile >
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
| `fake-agent` | Test-only stand-in for a coding agent; never shipped |

## Building

Requires Rust 1.85+ (edition 2024).

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

- [ ] **Step 2: Create `ROADMAP.md`**

```markdown
# Agent Profile Roadmap

v0.1 scope is fixed by the V3 specification. The build order follows spec §35, split into sub-projects.
Each sub-project has its own design spec and implementation plan under `docs/superpowers/`, and ends
green on Linux, macOS and Windows.

| SP | Deliverable | V3 §35 phases | State |
|---|---|---|---|
| SP0 | Scaffold — workspace, tooling, licence, community docs, CI, `fake-agent` fixture | — | **in progress** |
| SP1 | Core + explicit launch — profile-name validation, application root, TOML configuration with locked atomic writes, structured errors, `LaunchPlan`, Unix `exec` / Windows child launcher, passthrough, environment overrides, dry run, exit codes | 1, 3 | not started |
| SP2 | Architecture gate — adapter model, capability and evidence metadata, common contract suite, Claude Code, Codex CLI, Aider | 4A | not started |
| SP3 | Repository resolution — discovery, canonical identity, mapping storage, precedence, `resolve`, `current`, `status`, `link`, `unlink` | 2 + part of 5 | not started |
| SP4 | Remaining adapters — Gemini CLI, GitHub Copilot CLI, OpenCode, Cline CLI, Pi, Kiro CLI, Cursor Agent CLI, Continue CLI, Amp | 4B | not started |
| SP5 | Lifecycle and quality — `create`, `list`, `delete`, `repositories`, `doctor`, JSON output, shell completions, docs | rest of 5, 6 | not started |

`link`/`unlink` land with resolution (SP3), earlier than §35's Phase 5, so resolution never ships
without a way to create the mappings it resolves. All adapters still wait for the SP2 architecture
gate.

v0.1 is done when every item of spec §37 (Final Definition of Done) is checked.
```

- [ ] **Step 3: Create `TODO.md`**

```markdown
# TODO

Near-term work. Sub-project scope lives in [ROADMAP.md](ROADMAP.md).

## SP1 open decisions

- [ ] **Windows Ctrl-C mechanism** (spec §23.2, §24). Verify against Microsoft's documentation whether
      `SetConsoleCtrlHandler(NULL, TRUE)` is inherited by child processes. If it is, using it would make
      the launched agent ignore Ctrl-C. Candidate mechanism: a handler routine in the wrapper plus a job
      object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` for the no-orphan requirement. The observable
      behaviour in §24 is the oracle.
- [ ] **Application root** (spec §7). Choose the platform-appropriate location (and whether to use a
      crate for it), keeping it injectable for tests.
- [ ] **Configuration lock and atomic replace** (spec §18). Choose the file-lock mechanism, and how to get
      a guaranteed atomic replace on Windows; §18.1 forbids a silent non-atomic fallback.
- [ ] **Structured error types** and their mapping to the §33 exit codes.

## SP3 open decisions

- [ ] **Git repository discovery** (spec §13, §14). §14.2 requires Git's worktree metadata, so a plain
      upward walk for `.git` that ignores that metadata is not enough. Cover submodules, worktrees, nested
      repositories, symlinks and canonicalization failure.

## Housekeeping

- [ ] Enable GitHub private vulnerability reporting (see [SECURITY.md](SECURITY.md)). Branch
      protection is applied during SP0 (design §4.1).
- [ ] The MSRV (1.85) is not checked in CI. SP0 verified it by hand with
      `cargo +1.85 check --workspace --all-targets`. Decide whether to add a CI job.

## Scaffold follow-ups

- [ ] Run `lefthook install` in each clone
```

- [ ] **Step 4: Create `docs/dev-tooling.md`**

````markdown
# Agent Profile dev tooling

The toolchain is carried over from `flux` (`E:\Rust\flux`) and audited against what Agent Profile
needs. Agent Profile is an offline cross-platform CLI wrapper, so flux's TLA+ model-checking and
benchmarking tooling is dropped (see "Deliberately not carried over").

Every tool runs through a `just` recipe, so the local gate, the git hook and CI cannot drift apart.
The same list is machine-readable in `.claude/recommended-tools.json`.

## Tools

| Tool | Config | `just` recipe | Gates |
|---|---|---|---|
| `rustup` / `rustc` / `cargo` | `rust-toolchain.toml` (`stable`, with rustfmt + clippy) | `build` | everything |
| `cargo-nextest` | — | `test`, `test-verbose` (`cargo nextest run --workspace --no-tests=pass` + `cargo test --doc --workspace`) | `just check`; CI Test job on ubuntu, macos, windows |
| `lefthook` | `lefthook.yml` (pre-push: fmt-check, clippy, typos in parallel; no pre-commit) | `hooks` | local pre-push |
| `git-cliff` | `cliff.toml` (conventional commits; Dependabot uses `chore`/`ci` prefixes so its commits are kept) | `changelog` | release time |
| `cargo-release` | `[workspace.metadata.release]` in `Cargo.toml` (lockstep, `v{{version}}` tag, `publish = false`) | `release <patch\|minor\|major>` | pushed tag triggers `release.yml` |
| `rustfmt` | `rustfmt.toml` (edition 2024, width 100) | `fmt`, `fmt-check` | `just check`, pre-push, CI Format |
| `clippy` | `clippy.toml` (msrv 1.85) | `clippy` (`--workspace --all-targets -- -D warnings`) | `just check`, pre-push, CI Clippy |
| `typos` (typos-cli) | `_typos.toml` (excludes the V3 spec) | `typos` | `just check`, pre-push, CI Typos |
| `cargo-deny` | `deny.toml` (licence allow-list, `openssl-sys` ban per V3 §36, 7 target triples) | `deny` | CI Cargo deny |
| `bacon` | `bacon.toml` (default job `check-all`) | `watch` | local |
| `cargo-mutants` | — | `mutants` (`--package agent-profile`) | on demand |
| `actionlint` + `shellcheck` | — | — | workflow linting before pushing `.github/` changes |
| `cargo-binstall` | — | — | installs the cargo tools above |
| GitHub Actions | `.github/workflows/ci.yml` | — | Format, Typos, Clippy, Cargo deny, Docs build, Test ×3 OS — all required checks on `main` |
| GitHub Actions | `.github/workflows/docs.yml` | — | publishes rustdoc to Pages after merge |
| GitHub Actions | `.github/workflows/release.yml` | — | `v*` tag → cross-platform `agent-profile` binaries |
| Dependabot | `.github/dependabot.yml`, `.github/workflows/dependabot-automerge.yml` | — | weekly grouped minor/patch PRs, auto-merged once required checks pass |

## Install

```bash
cargo binstall -y cargo-nextest just lefthook cargo-deny typos-cli bacon git-cliff cargo-release cargo-mutants
winget install rhysd.actionlint
winget install koalaman.shellcheck
lefthook install
```

On Windows, winget installs `actionlint` and `shellcheck` under
`%LOCALAPPDATA%\Microsoft\WinGet\Packages\`. If Git Bash cannot find them, add those package
directories to `PATH`.

## Gate commands (the contract)

```
just check          # fmt-check + clippy + typos + test — the local gate
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
typos
cargo nextest run --workspace --no-tests=pass
cargo test --doc --workspace    # nextest does not run doctests
cargo deny check
```

## Deliberately not carried over from flux

- `models/`, `.github/workflows/model.yml`, and the `java` and `python3` tool entries: flux model-checks
  a lock protocol with TLA+. Nothing comparable exists here.
- `benches/` and `criterion`: the V3 spec asks for no performance work.

## Added beyond flux

- The `Docs build` CI job: flux builds docs only after merge, so a PR could merge with a broken docs
  build. Here it is a required PR check.
````

- [ ] **Step 5: Spell-check, lint and commit**

Run: `typos && just fmt-check`
Expected: no output, exit 0.

Run: `grep -n "not yet implemented or evidence-verified" README.md`
Expected: one match (the adapter table heading).

```bash
git add README.md ROADMAP.md TODO.md docs/dev-tooling.md
git commit -m "docs: add README, roadmap, TODO and dev-tooling guide

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01UTYq45Pr9gdUcm94AU2z3a"
```

---

### Task 8: Full local verification

**Oracle:** SP0 design §4 items 1–4 and 7.

**Files:** none created. Fix-ups only if a check fails; commit them separately.

- [ ] **Step 1: Run the gate on a clean build**

Run: `cargo clean && just check`
Expected:
- Exit 0.
- nextest: `8 tests run: 8 passed, 0 skipped`.
- Doctests: `test result: ok.`

- [ ] **Step 2: Dependency audit**

Run: `just deny`
Expected: `advisories ok, bans ok, licenses ok, sources ok`.

- [ ] **Step 3: MSRV**

Run: `cargo +1.85 check --workspace --all-targets`
Expected: `Finished`.

- [ ] **Step 4: Docs build exactly as CI runs it**

Run: `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items`
Expected:
- `Finished` with no warnings.
- `ls target/doc` lists `agent_profile` and `fake_agent`.

- [ ] **Step 5: Release build exactly as the release workflow runs it (host target)**

Run: `cargo build --release --locked --bin agent-profile && ./target/release/agent-profile --version`
Expected: `agent-profile 0.1.0`.

- [ ] **Step 6: Workflow lint**

Run the PATH export from Task 5 Step 6, then `actionlint`.
Expected: exit 0, no output.

- [ ] **Step 7: Install the git hooks and prove pre-push runs**

Run: `lefthook install && lefthook run pre-push`
Expected: `fmt`, `clippy` and `typos` each reported as passed.

- [ ] **Step 8: Tree is clean**

Run: `git status --short`
Expected: only `?? .serena/` if `.gitignore` did not yet apply to it; otherwise empty. Any other entry is uncommitted work: STOP and report it.

---

### Task 9: GitHub rollout (outward-facing — the user confirms each step)

**Oracle:** SP0 design §4 items 5–6 and §4.1 (order, gates, exact commands).

**Rule:** before EACH numbered step below, show the user the exact command(s) and wait for explicit confirmation. If the user declines a step, stop the sequence and record the gap in `TODO.md`; the design says the dependent workflow is then not added in SP0. If any API response contradicts the shape written here, STOP and report instead of adapting.

- [ ] **Step 0: Verify remote state**

Run:
```bash
gh api repos/ckir/aiprofiles --jq '{allow_auto_merge, has_pages, visibility}'
gh api repos/ckir/aiprofiles/branches/main/protection 2>&1 | head -1
git ls-remote --heads origin
```
Expected:
- `{"allow_auto_merge":false,"has_pages":false,"visibility":"public"}`
- `{"message":"Branch not protected",...}`
- Only `refs/heads/main` exists.

Anything else: STOP with `STATE_MISMATCH`.

- [ ] **Step 1 (confirm): Push the branch, open the PR, wait for all 8 checks to pass**

```bash
git push -u origin sp0-scaffold
gh pr create --base main --head sp0-scaffold --fill
gate='[.[] | select(.name == "Format" or .name == "Typos" or .name == "Clippy" or .name == "Cargo deny" or .name == "Docs build" or (.name | startswith("Test (")))]'
# Checks register asynchronously. Wait (at most 15 min) until all 8 gate checks exist before watching,
# so --watch cannot return early on an empty set.
n=0
until [ "$(gh pr checks sp0-scaffold --json name --jq "$gate | length")" -ge 8 ]; do
  n=$((n+1)); [ "$n" -le 90 ] || { echo "TIMEOUT: gate checks never registered"; break; }; sleep 10
done
gh pr checks sp0-scaffold --watch --fail-fast
# Final gate: independent of the wait/watch exit codes, all 8 gate checks must be in the pass bucket.
test "$(gh pr checks sp0-scaffold --json name,bucket --jq "$gate | map(select(.bucket == \"pass\")) | length")" -eq 8 && echo PR-GREEN
```
Expected: the last line prints `PR-GREEN`. If it does not, whether after a `TIMEOUT` or a failed check, the gate does not hold.

Nothing on GitHub has changed yet except the new branch and PR, so a failure here is fully recoverable.
- A failing check is a real defect.
- Diagnose it locally (reproduce with the same command CI runs), fix it, commit, push.
- Then repeat the watch.
- Do not iterate blindly against CI.

- [ ] **Step 2 (confirm): Protect `main`**

```bash
gh api -X PUT repos/ckir/aiprofiles/branches/main/protection --input - <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["Format", "Typos", "Clippy", "Cargo deny", "Docs build",
                 "Test (ubuntu-latest)", "Test (macos-latest)", "Test (windows-latest)"]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null
}
EOF
gh api repos/ckir/aiprofiles/branches/main/protection --jq '.required_status_checks | {strict, contexts}'
```
Expected read-back: `strict` is `true`, and `contexts` holds exactly the 8 names above.

- [ ] **Step 3 (confirm): Enable Pages with the workflow build type**

```bash
gh api -X POST repos/ckir/aiprofiles/pages -f build_type=workflow
gh api repos/ckir/aiprofiles/pages --jq .build_type
```
Expected read-back: `workflow`.

- [ ] **Step 4 (confirm): Merge and wait for `main`'s CI and Docs runs**

Tell the user before confirming: this merge enables Dependabot and publishes the docs site publicly.

```bash
gh pr merge sp0-scaffold --squash --delete-branch
git fetch origin main
sha="$(git rev-parse origin/main)"
# Runs for the merge commit spawn asynchronously; wait for both CI and Docs to exist for THIS sha,
# so a watch can never latch onto an older green run.
n=0
until [ "$(gh run list --commit "$sha" --event push --json workflowName --jq '[.[] | select(.workflowName == "CI" or .workflowName == "Docs")] | length')" -ge 2 ]; do
  n=$((n+1)); [ "$n" -le 90 ] || { echo "TIMEOUT: CI/Docs runs never appeared"; break; }; sleep 10
done
# A for loop's status is only its LAST iteration's, so collect failures explicitly.
# Fewer than 2 runs found (after a TIMEOUT) is also a failure.
failed=0
[ "$(gh run list --commit "$sha" --event push --json workflowName --jq '[.[] | select(.workflowName == "CI" or .workflowName == "Docs")] | length')" -ge 2 ] || failed=1
for id in $(gh run list --commit "$sha" --event push --json databaseId,workflowName --jq '.[] | select(.workflowName == "CI" or .workflowName == "Docs") | .databaseId'); do
  gh run watch "$id" --exit-status || failed=1
done
test "$failed" -eq 0 && echo MAIN-GREEN
```
Expected: the final line prints `MAIN-GREEN`. If it does not print — because a run failed, or because a
`TIMEOUT: CI/Docs runs never appeared` line was printed — `main` is not verified green: follow the
procedure below. For a TIMEOUT, first look at `gh run list --branch main --limit 5` and report to the user
before changing anything.

**If a `main` run fails:**
- Do NOT run Step 5. Auto-merge must never be enabled while `main` is red.
- `main` is protected, so do not push to it directly. Reproduce the failure locally with the command
  the failing job runs.
- Fix it on a new branch `sp0-fix-<short-name>` from `origin/main`.
- Open a PR, then re-run the **whole** Step 1 check block with every `sp0-scaffold` replaced by the fix
  branch name. That block runs from the `gate=` assignment through the final `PR-GREEN` test; do not copy
  only the loop. The fix PR's gate holds only when `PR-GREEN` prints.
- Merge it, then repeat this Step 4 wait-and-watch for the new merge commit.
- If the failure is `Deploy to Pages` (not a code defect), re-read the Step 3 read-back and report to
  the user rather than guessing at Pages settings.

- [ ] **Step 5 (confirm): Allow auto-merge — last**

```bash
gh api -X PATCH repos/ckir/aiprofiles -F allow_auto_merge=true
gh api repos/ckir/aiprofiles --jq .allow_auto_merge
```
Expected read-back: `true`.

- [ ] **Step 6: Sync the local checkout and mark SP0 done**

`gh pr merge --delete-branch` in Step 4 already deleted both the remote and the local `sp0-scaffold`
branch and switched to `main`. Do not run `git branch -d`: after a squash merge it would refuse or find
nothing.

```bash
git switch main
git pull --ff-only
git branch --list sp0-scaffold
git ls-remote --heads origin sp0-scaffold
```
Expected:
- `git pull` fast-forwards to the squash commit.
- Both `git branch --list` and `git ls-remote` print nothing.

If the local branch still exists, check that the PR is merged (`gh pr view sp0-scaffold --json state --jq .state` prints `MERGED`), then delete it with `git branch -D sp0-scaffold`.

In `ROADMAP.md`, change the SP0 row's state from `**in progress**` to `**done**`. This goes to `main` through a PR, because `main` is now protected:

```bash
git switch -c sp0-done
git add ROADMAP.md
git commit -m "docs: mark SP0 done

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01UTYq45Pr9gdUcm94AU2z3a"
```
Confirm with the user before pushing and opening that PR (same watch procedure as Step 1).

---

## Spec coverage map

| Design requirement | Task |
|---|---|
| §3.1 root manifest, key opt-in, pins, no root package | 1 |
| §3.2 layer modules citing spec sections; stub CLI contract | 1, 2 |
| §3.3 fake-agent contract (JSON, echo list, exit, 125, UTF-8) | 3 |
| §3.4 helper (CARGO fallback, selection rule, panics) | 3 |
| §4 items 1–4, 7 (gate, deny, tests, actionlint, MSRV) | 2, 3, 4, 5, 8 |
| §4 items 5–6, §4.1 (PR, protection, Pages, merge, auto-merge last) | 9 |
| §5 / §5.1 tools and files (verbatim, adapted, dropped) | 1, 4, 5 |
| §5.1 `Docs build` job | 5 |
| §6 licence and community docs | 6, 7 |
| §7 out of scope (no domain logic, no release trigger) | — (nothing in this plan triggers a release or adds domain logic) |
