# SP4a Adapter Evidence Infrastructure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Everything SP4 needs that can be built before the probe run: make an adapter's support level and capability states visible on every surface, turn the evidence rules into predicates with negative fixtures, repair the probe harness and write all twelve probe scripts, assemble evidence transcripts, probe several agents in one bounded matrix dispatch, and bind every committed transcript to the CI run that produced it.

**Architecture:** The report gains a capability matrix that is never hidden, because a support level alone overclaims. The evidence rules live in `src/adapter/gate.rs` as functions rather than as assertions over statics, so a negative fixture can prove each one rejects something. The probe harness records each step under a timeout, accumulates failures instead of aborting, and applies each agent's isolation mechanism directly — never through `agent-profile`, which rejects agent words it does not know. A workflow step assembles `/out` into `docs/evidence/<id>-<version>.md`, and a pull-request job downloads that run's artifact and requires the committed bytes to match it exactly.

**Tech Stack:** Rust 1.98 (edition 2024), POSIX shell, GitHub Actions. No new dependencies, and no new action: the verification job uses `gh`, which every runner already has.

**Design:** `docs/superpowers/specs/2026-09-16-sp4-adapters-design.md` is the oracle for behaviour; `agent-profile-implementation-spec-v3.md` wins over it, and §1.1 of the design records every divergence. SP2 design: `docs/superpowers/specs/2026-09-15-sp2-adapters-design.md`.

**Not in this plan:** the nine adapters themselves, and Gate A's transcript clauses. Both wait for the probe run, because an adapter written before its measurement is a guess with a version number on it.

---

## How this plan was produced, and how to execute it

This plan is generated, not written. Every file below was read out of a working prototype, and every
"Before" fact and edit anchor was asserted against a replay tree that had actually reached that state —
so no line number, no quoted anchor and no test name here is a recollection.

The route was: build the whole design in a throwaway worktree until the gate was green; split it into
eight tasks; replay those tasks from scratch onto the base commit, one commit each, gating every one of
them; and only then emit this document from the replay. The replay's final tree is byte-identical to the
prototype's — `git diff` between them is empty — which is what makes the ordering below a fact rather
than a plausible sequence.

That process earned its cost twice. It found that the twelve probe scripts had to arrive in two tasks
rather than one, because the harness suite asserts the six-step order over every script in the directory
and the three shipped probes still called a function this work deletes. And it found, through the mutant
run, that Task 1's verification step named the wrong test: the `--verbose` path is the one place the
design deliberately suppresses the launch hedge, so the verbose test could not observe that mutant at
all. Both would have shipped inside this plan as instructions an implementer follows and watches pass for
the wrong reason.

Seven of the eight tasks carry a mutant, and all seven were run: each one turns its named check red and
restores cleanly. The eighth task is documentation and carries none, because a test for prose is a test
that cannot fail.

Rules for every task:

1. **Step 0 — state check.** On branch `sp4-adapters`, run `git status --short` (expect no output). `HEAD` must be the previous task's commit; for Task 1 it is the commit that added this plan or a later documentation commit. Then check the "Before" facts listed for the task. If any check fails, STOP and report `STATE_MISMATCH: <what>`.
2. **Byte-exact files.** Write each file with exactly the content shown: the whole file, not a merge. An "Edit" step replaces exactly the quoted text, which occurs once. The gate includes `cargo fmt --all -- --check`, so do not reformat. Do not "improve" any code, test or comment.
3. **Shape-divergence stop.** If making the code compile or run would change the shape, type or encoding of anything shown, STOP and report `[original] -> [yours] because <reason>`. "It compiles" is not a justification. This applies with particular force to the transcript format: those bytes are compared against a CI artifact, so a changed field name or a dropped trailing newline is a broken contract, not a style choice.
4. **Oracle.** The named checks pin the behaviour, and the design is the oracle behind them. If a check fails, fix the code to match the check and the design; never edit a check to match the code.
5. **Gate.** Run the gate exactly as written; do not add flags.
6. **Mutant.** After committing, apply the mutant, run the command, confirm the named check FAILS, then restore with `git checkout -- <file>` and confirm `git status --short` prints nothing.
7. **Toolchain drift.** `rust-toolchain.toml` pins `stable`. If a gate fails with a clippy lint or compiler diagnostic in code copied from this plan, STOP and report the diagnostic; do not change the code to silence it.
8. **Shell files are LF.** `.gitattributes` pins `*.sh text eol=lf`. If you edit a shell file with a tool that rewrites line endings, it will still commit as LF — but `shellcheck` and the suites will disagree with what you see. Never "fix" `sandbox/run.sh`'s line endings; its working-tree CRLF predates the attribute and its index copy is already LF.
9. **No real agents.** Never install or run a coding agent while implementing this plan. Every probe script here is written, not executed: they run in CI, in a disposable container, and nowhere else. The shell suites need no agent and no container.
10. **No secrets reach a probe.** `permissions: contents: read` on the sandbox workflow is a security boundary, not a default. A probe executes third-party installers. Do not add a token, a secret, or a write permission to any job in `sandbox.yml`.

Gate commands (every task, after its own checks):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
typos
find sandbox -name '*.sh' -exec shellcheck -s sh {} +
cargo nextest run --workspace --no-tests=pass
just probe-tests
```

Expected: `cargo fmt`, `typos` and `shellcheck` print nothing; clippy prints no warnings; nextest ends with `N tests run: N passed`, 0 failed; every shell suite ends with `all ... checks passed`. `just probe-tests` and the `shellcheck` line do not exist until Task 3 adds them — skip both for Tasks 1 and 2, which touch no shell file. The `find` form matters: an unmatched glob passes through literally, and `sandbox/tests/` does not exist until Task 3.

## File structure

| File | Task | Responsibility |
|---|---|---|
| `crates/agent-profile/src/output.rs` | 1 | the isolation block, the label helpers and `support_hedge` |
| `crates/agent-profile/src/cli.rs` | 1 | both report call sites, and the hedge on the silent launch path |
| `crates/agent-profile/src/adapter/mod.rs` | 1, 2 | the `SECRETIVE` call site; declaring the gate module |
| `crates/agent-profile/src/adapter/gate.rs` | 2 | gates A, B and C as predicates, with one `GateFailure` per rule |
| `crates/agent-profile/src/adapter/*.rs` | 2 | provenance prefixes on every `basis` |
| `crates/agent-profile/src/adapter/metadata.rs` | 8 | `ProfilePresence::Known` documented as reserved |
| `crates/agent-profile/examples/` | 7 | the registry manifest the verification job reads |
| `crates/agent-profile/tests/` | 1, 2 | re-baselined report assertions; the gate fixtures |
| `sandbox/probes/text.sh` | 3 | control-sequence stripping and bounded excerpts |
| `sandbox/probes/common.sh` | 3 | the six step helpers and the mechanism vocabulary |
| `sandbox/probes/*.sh` | 3, 4 | three reworked probes, then nine new ones |
| `sandbox/transcript.sh` | 5 | assembles one probe run into a transcript |
| `sandbox/resolve-agents.sh` | 6 | the matrix rules, as something testable |
| `sandbox/verify-transcripts.sh` | 7 | binds a committed transcript to its run |
| `sandbox/tests/` | 3, 5, 6, 7 | each suite arrives with the code it covers |
| `.github/workflows/` | 6, 7 | the three sandbox jobs; the evidence job |
| `docs/evidence/README.md` | 8 | what a transcript is for, and what it cannot tell you |
| `README.md, CONTRIBUTING.md, ROADMAP.md, TODO.md` | 8 | the documentation SP4a owes |

### Task 1: Report the support level and the capability matrix, and hedge on a real launch

Make the support level and the three capability states visible wherever a launch is described, and warn once on a real launch of an adapter that is not Proven.

**Files:**

- Modify (whole file): `crates/agent-profile/src/output.rs` — The isolation block, the three label helpers and support_hedge, plus the re-baselined report tests and the position-independent field() helper they now use.
- Modify (whole file): `crates/agent-profile/src/cli.rs` — Both report call sites gain the metadata argument, and the silent launch path gains the hedge.
- Modify (whole file): `crates/agent-profile/tests/launch.rs` — Two tests break because `fake` is the only non-Proven adapter and the hedge now precedes its stderr on every plain launch.
- Modify (whole file): `crates/agent-profile/tests/windows_console.rs` — One test breaks the same way and needed a first_lines(pipe, n) helper: reading two lines synchronously would risk a hang.
- Modify: `crates/agent-profile/src/adapter/mod.rs` — The tenth call site, and the one that matters: SECRETIVE declares `capabilities: &[]`, so this is the path that proves a capability with no claim renders rather than panicking.

**Before:** `crates/agent-profile/src/output.rs` contains `pub fn report_lines(`; `crates/agent-profile/src/output.rs` does not contain `support_hedge`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/output.rs`** — The isolation block, the three label helpers and support_hedge, plus the re-baselined report tests and the position-independent field() helper they now use. Write the whole file, byte-exact.

```rust
//! Human output: the dry-run and `--verbose` report (spec §26, SP1 design §7.4, SP2 design §7.3), the
//! repository command reports (SP3 design §7.5) and redaction (spec §22). JSON output (spec §32) arrives in
//! SP5.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::adapter::{
    AdapterMetadata, Capability, CapabilityState, PathKind, PlannedLaunch, SupportLevel,
};
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

/// How a support level is spelled in the report and in the launch hedge (SP4 design §7.1).
pub fn support_label(support: SupportLevel) -> &'static str {
    match support {
        SupportLevel::Proven => "proven",
        SupportLevel::Experimental => "experimental",
    }
}

/// How a capability is spelled. One vocabulary covers the report and the hedge (SP4 design §7.2).
pub fn capability_label(capability: Capability) -> &'static str {
    match capability {
        Capability::ConfigIsolation => "config",
        Capability::CredentialIsolation => "credentials",
        Capability::StateIsolation => "state",
    }
}

/// How a capability state is spelled: lower case, with spaces (SP4 design §7.1).
pub fn state_label(state: CapabilityState) -> &'static str {
    match state {
        CapabilityState::Supported => "supported",
        CapabilityState::NotSupported => "not supported",
        CapabilityState::NotGuaranteed => "not guaranteed",
        CapabilityState::Conditional => "conditional",
        CapabilityState::Unknown => "unknown",
    }
}

/// The one-line stderr hedge for an adapter that is not `Proven`, or `None` when it is (SP4 design §7.2).
///
/// It lists every capability whose state is not `Supported`, in `Capability::ALL` order, using the
/// report's own vocabulary. `NotGuaranteed` counts: leaving it out would have hidden the state the design
/// predicts for an adapter whose documented mechanism is ignored by part of the agent.
pub fn support_hedge(metadata: &AdapterMetadata) -> Option<String> {
    if metadata.support == SupportLevel::Proven {
        return None;
    }
    let support = support_label(metadata.support);
    let weak: Vec<String> = Capability::ALL
        .into_iter()
        .filter_map(|capability| {
            let claim =
                metadata.capabilities.iter().find(|claim| claim.capability == capability)?;
            (claim.state != CapabilityState::Supported)
                .then(|| format!("{} {}", capability_label(capability), state_label(claim.state)))
        })
        .collect();
    let detail = if weak.is_empty() { String::new() } else { format!(": {}", weak.join(", ")) };
    Some(format!("{} is {support}{detail}. Run with --dry-run for detail.", metadata.id))
}

/// The report lines, without trailing newlines.
pub fn report_lines(
    planned: &PlannedLaunch,
    resolution: &Resolution,
    mode: ReportMode,
    metadata: &AdapterMetadata,
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
    lines.push(line("support", support_label(metadata.support).to_owned()));
    // Every capability in `Capability::ALL`, in that fixed order, each followed by its basis. The matrix is
    // unconditional: SP4 design §6.1 rests on a support level never being shown without it.
    for (index, capability) in Capability::ALL.into_iter().enumerate() {
        let claim = metadata.capabilities.iter().find(|claim| claim.capability == capability);
        let label = capability_label(capability);
        let value = match claim {
            Some(claim) => format!("{label}: {}", state_label(claim.state)),
            // Not every caller is a registry adapter: `metadata_invariants` requires one claim per
            // capability, but `report_lines` is reachable from a fixture that declares none. "not declared"
            // is visibly different from "unknown", and neither panics.
            None => format!("{label}: not declared"),
        };
        lines.push(if index == 0 { line("isolation", value) } else { continuation(value) });
        if let Some(claim) = claim {
            lines.push(continuation(format!("  {}", claim.basis)));
        }
    }
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

/// A path to paste into a command: in double quotes when it contains whitespace, which POSIX shells, PowerShell
/// and `cmd` all read as one argument.
fn shell_word(path: &Path) -> String {
    let shown = path.display().to_string();
    if shown.contains(char::is_whitespace) { format!("\"{shown}\"") } else { shown }
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
            shell_word(key)
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

    use crate::adapter::{AdapterEvidence, CapabilityClaim};

    /// A registry-shaped fixture: one claim per capability, as `metadata_invariants` requires of a real
    /// adapter. `Fake` is not usable here — it is `#[cfg(debug_assertions)]` and this module is `cfg(test)`.
    static TEST_METADATA: AdapterMetadata = AdapterMetadata {
        id: "fake",
        executable: "fake-agent",
        mechanism_summary: "environment variable FAKE_AGENT_HOME",
        support: SupportLevel::Proven,
        evidence: AdapterEvidence {
            mechanism_id: "fake-home-v1",
            verified_at: "2026-09-16",
            upstream_version: "0.0.0",
            source_url: "measured",
            notes: "test fixture",
        },
        capabilities: &[
            CapabilityClaim {
                capability: Capability::ConfigIsolation,
                state: CapabilityState::Supported,
                basis: "measured: the fixture writes only under the profile directory",
            },
            CapabilityClaim {
                capability: Capability::CredentialIsolation,
                state: CapabilityState::Unknown,
                basis: "unmeasured: the fixture has no credentials",
            },
            CapabilityClaim {
                capability: Capability::StateIsolation,
                state: CapabilityState::NotGuaranteed,
                basis: "measured: the fixture keeps no state",
            },
        ],
        env: &[],
        conflicts: &[],
    };

    /// The report line beginning `<label>:`. Assertions that used a hard-coded index all broke when the
    /// isolation block shifted every position; looking the field up by name keeps them from breaking again.
    fn field<'a>(lines: &'a [String], label: &str) -> &'a str {
        let prefix = format!("{label}:");
        lines
            .iter()
            .find(|line| line.starts_with(&prefix))
            .unwrap_or_else(|| panic!("no {label}: line in {lines:?}"))
    }

    /// The seven lines every report now carries between `mechanism:` and `environment:`.
    fn isolation_block() -> Vec<String> {
        vec![
            "support:      proven".to_owned(),
            "isolation:    config: supported".to_owned(),
            "                measured: the fixture writes only under the profile directory"
                .to_owned(),
            "              credentials: unknown".to_owned(),
            "                unmeasured: the fixture has no credentials".to_owned(),
            "              state: not guaranteed".to_owned(),
            "                measured: the fixture keeps no state".to_owned(),
        ]
    }

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
        let lines = report_lines(
            &planned(home_env(), vec![], false),
            &resolution(),
            ReportMode::DryRun,
            &TEST_METADATA,
        );
        let dir = PathBuf::from("/root/profiles/work/fake");
        let mut expected = vec![
            "agent:        fake".to_owned(),
            "profile:      work (explicit)".to_owned(),
            format!("executable:   {} (configured)", PathBuf::from("/bin/fake-agent").display()),
            "repository:   none".to_owned(),
            "mechanism:    environment variable FAKE_AGENT_HOME".to_owned(),
        ];
        expected.extend(isolation_block());
        expected.extend([
            "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake (would be created)".to_owned(),
            format!("creates:      {} (would be created)", dir.display()),
            "arguments:    [\"--foo\", \"a b\"]".to_owned(),
        ]);
        assert_eq!(lines, expected);
        let existing = report_lines(
            &planned(home_env(), vec![], true),
            &resolution(),
            ReportMode::DryRun,
            &TEST_METADATA,
        );
        assert_eq!(
            field(&existing, "environment"),
            "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake"
        );
        assert_eq!(field(&existing, "creates"), "creates:      none");
    }

    #[test]
    fn verbose_has_no_markers_and_no_creates_line() {
        let lines = report_lines(
            &planned(home_env(), vec![], false),
            &resolution(),
            ReportMode::Verbose,
            &TEST_METADATA,
        );
        assert_eq!(lines.len(), 14, "{lines:?}");
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
        let lines = report_lines(&planned, &resolution(), ReportMode::DryRun, &TEST_METADATA);
        assert_eq!(field(&lines, "environment"), "environment:  none");
        assert_eq!(
            field(&lines, "creates"),
            format!("creates:      {} (would be created)", planned.paths[0].path.display())
        );
        assert_eq!(lines[14], format!("              {} (would be created)", file.display()));
        assert_eq!(
            lines.iter().filter(|line| line.starts_with("note:")).collect::<Vec<_>>(),
            ["note:         first note", "note:         second note"]
        );
        let verbose = report_lines(&planned, &resolution(), ReportMode::Verbose, &TEST_METADATA);
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
        let text =
            report_lines(&planned, &resolution(), ReportMode::Verbose, &TEST_METADATA).join("\n");
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
            &TEST_METADATA,
        );
        let text = lines.join("\n");
        assert!(text.contains("PLAIN=visible"), "{text}");
        for secret in ["hidden-1", "hidden-2", "hidden-3"] {
            assert!(!text.contains(secret), "{text}");
        }
        assert_eq!(text.matches("<redacted>").count(), 3, "{text}");
        assert!(lines[13].starts_with("              DECLARED="), "{text}");
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

    #[test]
    fn the_unlink_hint_quotes_a_path_with_whitespace() {
        let spaced = host("/my repos", r"C:\my repos");
        let inner = spaced.join("inner");
        let text = format!("[repositories.{}]\nprofile = \"outer\"\n", key(&spaced));
        let config = config(&text);
        let nothing = UnlinkOutcome::NothingToRemove { shown: inner.clone() };
        assert_eq!(
            unlink_lines(&nothing, None, &config, std::slice::from_ref(&inner))[1],
            format!(
                "note:         {} has a mapping; remove it with agent-profile unlink --repo \"{}\"",
                spaced.display(),
                spaced.display()
            )
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

    /// `TEST_METADATA` with the support level and the state-isolation claim replaced.
    ///
    /// The hedge's own branches are unreachable through any shipped adapter: `fake` is the only
    /// non-`Proven` one and all three of its claims are `Unknown`, so nothing in the workspace ever sends
    /// a `NotGuaranteed` claim through `support_hedge`. That is the branch §10 asks for a fixture for.
    fn hedged(support: SupportLevel, state: CapabilityState) -> AdapterMetadata {
        const CLAIMS: [CapabilityClaim; 3] = [
            CapabilityClaim {
                capability: Capability::ConfigIsolation,
                state: CapabilityState::Supported,
                basis: "measured: the fixture writes only under the profile directory",
            },
            CapabilityClaim {
                capability: Capability::CredentialIsolation,
                state: CapabilityState::Unknown,
                basis: "unmeasured: the fixture has no credentials",
            },
            CapabilityClaim {
                capability: Capability::StateIsolation,
                state: CapabilityState::NotGuaranteed,
                basis: "measured: the fixture keeps no state",
            },
        ];
        let mut claims = CLAIMS;
        claims[2].state = state;
        // `capabilities` is `&'static [_]`, so the modified claims have to outlive this call.
        let leaked: &'static [CapabilityClaim] = Box::leak(Box::new(claims));
        AdapterMetadata { support, capabilities: leaked, ..TEST_METADATA }
    }

    #[test]
    fn a_proven_adapter_does_not_hedge() {
        assert_eq!(
            support_hedge(&hedged(SupportLevel::Proven, CapabilityState::NotGuaranteed)),
            None
        );
    }

    /// The regression the design names: an earlier draft listed only `Unknown` capabilities, which would
    /// have hedged about an adapter whose configuration demonstrably leaks without ever saying so.
    #[test]
    fn the_hedge_names_every_capability_that_is_not_supported() {
        let hedge =
            support_hedge(&hedged(SupportLevel::Experimental, CapabilityState::NotGuaranteed))
                .expect("a non-proven adapter hedges");
        assert_eq!(
            hedge,
            "fake is experimental: credentials unknown, state not guaranteed. \
             Run with --dry-run for detail.",
            "{hedge}"
        );
    }

    #[test]
    fn the_hedge_lists_only_the_capabilities_that_are_weak() {
        // `CredentialIsolation` stays `Unknown` in the fixture, so that is all this should name.
        assert_eq!(
            support_hedge(&hedged(SupportLevel::Experimental, CapabilityState::Supported))
                .as_deref(),
            Some("fake is experimental: credentials unknown. Run with --dry-run for detail.")
        );
    }

    /// `report_lines` is reachable from a fixture that declares no capabilities at all
    /// (`adapter/mod.rs`'s `SECRETIVE`). A lookup-and-unwrap would panic there; a silent skip would make
    /// "not claimed" indistinguishable from "not rendered".
    #[test]
    fn a_capability_with_no_claim_renders_as_not_declared() {
        let metadata = AdapterMetadata { capabilities: &[], ..TEST_METADATA };
        let launch = planned(home_env(), Vec::new(), false);
        let lines = report_lines(&launch, &resolution(), ReportMode::DryRun, &metadata);
        assert!(field(&lines, "isolation").contains("config: not declared"), "{lines:?}");
        assert!(
            lines.iter().all(|line| !line.contains("measured:") && !line.contains("unmeasured:")),
            "a capability with no claim has no basis line: {lines:?}"
        );
    }
}
```

- [ ] **Step 2: Write `crates/agent-profile/src/cli.rs`** — Both report call sites gain the metadata argument, and the silent launch path gains the hedge. Write the whole file, byte-exact.

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

/// The directory a `--repo` value is joined to. An absolute `--repo` needs no current directory, so a command
/// given one still works from a working directory that no longer exists (design §6.2, §10 "`--repo` is the
/// explicit override").
fn base_dir(repo: Option<&OsStr>) -> Result<PathBuf> {
    match repo {
        Some(repo) if std::path::Path::new(repo).is_absolute() => Ok(PathBuf::new()),
        _ => current_dir(),
    }
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
    let metadata = adapter.metadata();
    if dry_run {
        let lines = output::report_lines(&planned, &resolution, ReportMode::DryRun, metadata);
        write_out(&lines.iter().map(|line| format!("{line}\n")).collect::<String>())?;
        return Ok(0);
    }

    // Step 5. The verbose report is rendered after initialization, so it never says "(would be created)".
    adapter.initialize(&planned)?;
    if verbose {
        let mut stderr = io::stderr();
        for line in &output::report_lines(&planned, &resolution, ReportMode::Verbose, metadata) {
            let _ = writeln!(stderr, "agent-profile: {line}");
        }
        let _ = stderr.flush();
    } else if let Some(hedge) = output::support_hedge(metadata) {
        // SP4 design §7.2: the only disclosure on the silent path. Suppressed under `--verbose`, which just
        // printed the whole matrix, and never reached on a dry run, which returned above.
        let mut stderr = io::stderr();
        let _ = writeln!(stderr, "agent-profile: {hedge}");
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
    let cwd = base_dir(repo.as_deref())?;
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
    fn an_absolute_repo_is_joined_to_nothing_so_no_current_directory_is_needed() {
        let absolute = if cfg!(windows) { r"C:\src\acme" } else { "/src/acme" };
        assert_eq!(base_dir(Some(OsStr::new(absolute))).unwrap(), PathBuf::new());
        assert_eq!(PathBuf::new().join(absolute), PathBuf::from(absolute));
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(base_dir(Some(OsStr::new("relative"))).unwrap(), cwd);
        assert_eq!(base_dir(None).unwrap(), cwd);
    }

    /// Every `--repo` spelling must name the same target whether it is joined to the base `base_dir` chose or to
    /// the working directory; only then is skipping `current_dir()` invisible.
    #[test]
    fn every_repo_spelling_names_the_same_target_with_or_without_the_working_directory() {
        let cwd = std::env::current_dir().unwrap();
        let spellings: &[&str] = if cfg!(windows) {
            &[
                r"C:\src\acme",
                r"C:\",
                "C:/src/acme",
                r"\\server\share\x",
                r"\\?\C:\x",
                r"\\?\UNC\server\share\x",
                r"\\.\pipe\x",
                r"\foo",
                "C:rel",
                r"..\up",
                "relative",
            ]
        } else {
            &["/src/acme", "/", "../up", "relative", "./relative"]
        };
        for spelling in spellings {
            let base = base_dir(Some(OsStr::new(spelling))).unwrap();
            // `join` discards the base for an absolute value, so the equality below holds whatever `base_dir`
            // returned. This is the discriminating assertion: the working directory is skipped exactly for the
            // spellings that do not need it.
            assert_eq!(
                base.as_os_str().is_empty(),
                std::path::Path::new(spelling).is_absolute(),
                "{spelling:?} took the wrong branch of base_dir"
            );
            assert_eq!(
                base.join(spelling),
                cwd.join(spelling),
                "{spelling:?} resolves differently without the working directory"
            );
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

- [ ] **Step 3: Write `crates/agent-profile/tests/launch.rs`** — Two tests break because `fake` is the only non-Proven adapter and the hedge now precedes its stderr on every plain launch. Write the whole file, byte-exact.

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
fn secret_argument_values_reach_the_agent_but_never_the_report() {
    let root = Root::new();
    let args =
        ["fake", "work", "--verbose", "--", "--api-key", "sk-live-1", "OPENAI_API_KEY=sk-live-2"];
    let output = root.agent_profile(args).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        support::report(&output.stdout)["argv"],
        serde_json::json!(["--api-key", "sk-live-1", "OPENAI_API_KEY=sk-live-2"])
    );
    let text = stderr(&output);
    assert!(!text.contains("sk-live"), "{text}");
    assert!(text.contains(r#"["--api-key", "<redacted>", "OPENAI_API_KEY=<redacted>"]"#), "{text}");

    let dry = root
        .agent_profile(["fake", "work", "--dry-run", "--", "--api-key=sk-live-3"])
        .output()
        .unwrap();
    assert_eq!(dry.status.code(), Some(0), "{}", stderr(&dry));
    let text = stdout(&dry);
    assert!(!text.contains("sk-live"), "{text}");
    assert!(text.contains(r#"["--api-key=<redacted>"]"#), "{text}");
}

#[test]
fn verbose_launch_reports_to_stderr_after_creating_the_profile_directory() {
    let root = Root::new();
    let output = root.agent_profile(["fake", "work", "--verbose"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stderr(&output);
    let report: Vec<&str> =
        text.lines().filter(|line| line.starts_with("agent-profile: ")).collect();
    assert_eq!(report.len(), 14, "{text}");
    assert!(report[0].starts_with("agent-profile: agent:"), "{text}");
    assert!(text.contains("agent-profile: environment:  FAKE_AGENT_HOME="), "{text}");
    assert!(!text.contains("(would be created)"), "{text}");
    assert!(support::report(&output.stdout).get("pid").is_some());
}

#[test]
fn dry_run_of_an_existing_profile_omits_would_be_created() {
    let root = Root::new();
    let first = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(first.status.code(), Some(0), "{}", stderr(&first));
    assert!(root.profile_dir("work").is_dir());
    let output = root.agent_profile(["fake", "work", "--dry-run"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    let environment: Vec<&str> =
        text.lines().filter(|line| line.starts_with("environment:")).collect();
    assert_eq!(environment.len(), 1, "{text}");
    assert!(environment[0].contains("FAKE_AGENT_HOME="), "{text}");
    assert!(!text.contains("(would be created)"), "{text}");
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
    // `fake` is Experimental, so the launch hedge precedes the agent's own stderr (SP4 design §7.2).
    let text = stderr(&output);
    assert!(text.ends_with("agent stderr"), "{text}");
    assert!(text.starts_with("agent-profile: fake is experimental"), "{text}");
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
        (&["link", "work", "extra"], 2, "`link` takes one profile"),
        (&["fake", "create", "work"], 2, "`fake create` is not yet implemented"),
        (&["fake", "work", "--json"], 2, "`--json` is not yet implemented"),
        (
            &["zzz", "work"],
            2,
            "unknown agent `zzz` (known agents: `claude`, `codex`, `aider`, `fake`)",
        ),
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
fn bare_invocation_prints_top_level_help_to_stderr_only() {
    let root = Root::new();
    let output = root.agent_profile(Vec::<&str>::new()).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("Usage: agent-profile <agent> <profile>"),
        "{}",
        stderr(&output)
    );
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
    assert!(stderr(&output).contains("cannot launch without a shell"), "{}", stderr(&output));
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

- [ ] **Step 4: Write `crates/agent-profile/tests/windows_console.rs`** — One test breaks the same way and needed a first_lines(pipe, n) helper: reading two lines synchronously would risk a hang. Write the whole file, byte-exact.

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
use windows_sys::Win32::System::JobObjects::{
    JOB_OBJECT_LIMIT_BREAKAWAY_OK, JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK,
};
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
    drive_program(env!("CARGO_BIN_EXE_agent-profile"), &["fake", "work"], event, readiness, env)
}

/// Runs `program args` under `console-driver` in a new console and returns its result. `drive` runs the wrapper;
/// tests that compare against direct invocation run `fake-agent` itself through the same driver mode.
fn drive_program(
    program: &str,
    args: &[&str],
    event: &str,
    readiness: &str,
    env: &[(&str, &str)],
) -> serde_json::Value {
    let root = Root::new();
    let result = root.path().join("result.json");
    let mut command = Command::new(env!("CARGO_BIN_EXE_console-driver"));
    command.arg(&result).arg(event).arg(readiness).arg(program).args(args);
    for name in support::FIXTURE_VARS.iter().chain(support::WRAPPER_VARS.iter()) {
        command.env_remove(name);
    }
    command
        .env_remove("CONSOLE_DRIVER_JOB_LIMITS")
        .env_remove("CONSOLE_DRIVER_IGNORE_CTRL_C")
        .env("AGENT_PROFILE_HOME", root.path());
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
    first_lines(pipe, 1).pop().expect("one line")
}

/// The first `count` lines, read on a worker thread so a silent pipe times out instead of hanging.
fn first_lines(pipe: impl std::io::Read + Send + 'static, count: usize) -> Vec<String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(pipe);
        let mut lines = Vec::new();
        for _ in 0..count {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                break;
            }
            lines.push(line);
        }
        let _ = tx.send(lines);
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
fn breakaway_process_creation_matches_direct_invocation() {
    // CREATE_BREAKAWAY_FROM_JOB fails when the creator's innermost job forbids breakaway. Whatever the test
    // runner's own job allows (cargo test's job forbids it, nextest's per-test job allows it, both measured), the
    // wrapper must not change the outcome (V3 §24): compare against a direct run.
    let env = [("FAKE_AGENT_SPAWN_SLEEPER", "1000"), ("FAKE_AGENT_BREAKAWAY", "1")];
    let mut direct = support::fake_agent();
    let root = Root::new();
    let mut wrapped = root.agent_profile(["fake", "work"]);
    for (name, value) in env {
        direct.env(name, value);
        wrapped.env(name, value);
    }
    let direct = direct.output().unwrap();
    let wrapped = wrapped.output().unwrap();
    assert_eq!(
        wrapped.status.code(),
        direct.status.code(),
        "direct stderr: {}\nwrapped stderr: {}",
        String::from_utf8_lossy(&direct.stderr),
        String::from_utf8_lossy(&wrapped.stderr)
    );
}

#[test]
fn breakaway_follows_a_controlled_caller_job() {
    // The caller job's limits are set by the test, so the expected outcome does not depend on the test runner's
    // own job. The direct run proves the fixture really requests breakaway (it must fail without a breakaway flag).
    let cases =
        [(0, 125), (JOB_OBJECT_LIMIT_BREAKAWAY_OK, 0), (JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK, 0)];
    for (limits, expected) in cases {
        let limits = limits.to_string();
        let env = [
            ("FAKE_AGENT_SPAWN_SLEEPER", "1000"),
            ("FAKE_AGENT_BREAKAWAY", "1"),
            ("CONSOLE_DRIVER_JOB_LIMITS", limits.as_str()),
        ];
        let direct = drive_program(env!("CARGO_BIN_EXE_fake-agent"), &[], "none", "none", &env);
        assert_eq!(exit_code(&direct), expected, "direct, caller job limits {limits}: {direct}");
        let wrapped = drive("none", "none", &env);
        assert_eq!(exit_code(&wrapped), expected, "wrapped, caller job limits {limits}: {wrapped}");
    }
}

#[test]
fn an_inherited_ignore_ctrl_c_attribute_reaches_the_agent_unchanged() {
    // With the attribute inherited, a direct agent never sees Ctrl-C: its handler would exit 42, but it sleeps
    // to completion and exits 0. The wrapper must leave the attribute alone (design §7.6 step 2).
    let env = [
        ("CONSOLE_DRIVER_IGNORE_CTRL_C", "1"),
        ("FAKE_AGENT_CTRL_C_EXIT", "42"),
        ("FAKE_AGENT_SLEEP_MS", "3000"),
    ];
    let direct = drive_program(env!("CARGO_BIN_EXE_fake-agent"), &[], "ctrl-c", "report", &env);
    assert_eq!(exit_code(&direct), 0, "direct: {direct}");
    let wrapped = drive("ctrl-c", "report", &env);
    assert_eq!(exit_code(&wrapped), 0, "wrapped: {wrapped}");
}

#[test]
fn killing_the_wrapper_before_spawn_starts_no_agent() {
    let root = Root::new();
    let mut wrapper =
        spawn_wrapper(&root, &[("AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS", "10000")]);
    let child = wrapper.0.as_mut().unwrap();
    // `fake` is Experimental, so the launch hedge is the first stderr line and the pause marker the second
    // (SP4 design §7.2).
    let lines = first_lines(child.stderr.take().unwrap(), 2);
    assert!(lines[0].starts_with("agent-profile: fake is experimental"), "{lines:?}");
    assert_eq!(lines[1].trim_end(), "agent-profile: debug: paused before spawn");
    child.kill().unwrap();
    child.wait().unwrap();
    let mut stdout = String::new();
    std::io::Read::read_to_string(&mut child.stdout.take().unwrap(), &mut stdout).unwrap();
    assert_eq!(stdout, "", "an agent started after the wrapper was killed");
}
```

- [ ] **Step 5: Edit `crates/agent-profile/src/adapter/mod.rs`** — The tenth call site, and the one that matters: SECRETIVE declares `capabilities: &[]`, so this is the path that proves a capability with no claim renders rather than panicking. Replace exactly this text, which occurs once:

```rust
        let text =
            crate::output::report_lines(&planned, &resolution, crate::output::ReportMode::DryRun)
                .join("\n");
```

with:

```rust
        let text = crate::output::report_lines(
            &planned,
            &resolution,
            crate::output::ReportMode::DryRun,
            &SECRETIVE,
        )
        .join("\n");
```

- [ ] **Step 6: Run this task's own checks**

```bash
cargo nextest run --workspace --no-tests=pass
```

Expected: passes, including `the_hedge_names_every_capability_that_is_not_supported`, `a_capability_with_no_claim_renders_as_not_declared`, `stdin_and_stderr_are_inherited`.

- [ ] **Step 7: Run the gate** (see "Gate commands"). Expected on Windows: `232 tests run: 232 passed`. Linux and macOS run fewer tests, because the Windows-only tests are compiled out; every test must pass.

- [ ] **Step 8: Commit**

```bash
git add crates/agent-profile/src/output.rs crates/agent-profile/src/cli.rs crates/agent-profile/tests/launch.rs crates/agent-profile/tests/windows_console.rs crates/agent-profile/src/adapter/mod.rs
git commit -m "feat: report the capability matrix, and hedge on a non-proven launch

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 9: Prove this task's tests are not vacuous** (rule 6). This is the draft defect the design names: a hedge listing only Unknown capabilities stays silent about a NotGuaranteed one — the very state predicted for an adapter whose documented mechanism part of the agent ignores. No shipped adapter can reach that branch, because `fake` is the only non-Proven one and all three of its claims are Unknown, which is why the fixture exists.

In `crates/agent-profile/src/output.rs` replace exactly:

```rust
(claim.state != CapabilityState::Supported)
```

with:

```rust
(claim.state == CapabilityState::Unknown)
```

Run `cargo nextest run --no-fail-fast --workspace --no-tests=pass`. Expected: it FAILS, and a line reporting FAIL names `the_hedge_names_every_capability_that_is_not_supported`. Then restore with `git checkout -- crates/agent-profile/src/output.rs` and confirm `git status --short` prints nothing.


### Task 2: The evidence gates, as library predicates with negative fixtures

Turn the evidence rules into predicates the test suite runs over every adapter, with a negative fixture per rule so each one is proven to reject something.

**Files:**

- Create: `crates/agent-profile/src/adapter/gate.rs` — GateFailure plus gate_a_shape, gate_b, gate_c and gates_before_transcripts. They live in the library, not the test, because a gate expressed only as an assertion over `static` items cannot be proven non-vacuous: cargo mutants mutates functions, not statics.
- Modify (whole file): `crates/agent-profile/tests/adapter_contract.rs` — The gates run over the registry, plus a sound fixture and one negative fixture per rule.
- Modify (whole file): `crates/agent-profile/src/adapter/claude.rs` — Every basis gains its provenance prefix, which Gate B requires.
- Modify (whole file): `crates/agent-profile/src/adapter/codex.rs` — Same.
- Modify (whole file): `crates/agent-profile/src/adapter/aider.rs` — Same.
- Modify (whole file): `crates/agent-profile/src/adapter/fake.rs` — Same, and its three Unknown claims now carry `unmeasured:` — Gate B's biconditional requires the prefix exactly when the state is Unknown.
- Modify: `crates/agent-profile/src/adapter/mod.rs` — The gate module is public because the contract suite is an integration test and reaches it through the crate's public surface.

**Before:** `crates/agent-profile/src/adapter/gate.rs` does not exist; `crates/agent-profile/src/adapter/claude.rs` does not contain `cited:`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `crates/agent-profile/src/adapter/gate.rs`** — GateFailure plus gate_a_shape, gate_b, gate_c and gates_before_transcripts. They live in the library, not the test, because a gate expressed only as an assertion over `static` items cannot be proven non-vacuous: cargo mutants mutates functions, not statics. Write the whole file, byte-exact.

```rust
//! The evidence gates (SP4 design §5), as predicates over `AdapterMetadata`.
//!
//! These live in the library rather than in `tests/adapter_contract.rs` for one reason: a gate whose only
//! expression is an assertion over `static METADATA` items cannot be proven non-vacuous. `cargo mutants`
//! mutates functions, not statics, so a weaker-than-intended gate — Gate B's biconditional written as a
//! single implication, say — would pass purely because no shipped adapter exhibits the excluded
//! combination. As functions they take negative fixtures, and a mutant reaches them.

use super::metadata::{AdapterMetadata, Capability, CapabilityState, SupportLevel};

/// The provenance prefixes a `basis` may start with (SP4 design D5).
pub const MEASURED: &str = "measured: ";
pub const CITED: &str = "cited: ";
pub const UNMEASURED: &str = "unmeasured: ";

/// The encoding of a failed probe in `evidence.upstream_version` (SP4 design §9 outcome 2).
pub const UNKNOWN_VERSION: &str = "unknown";

/// Why a gate refused. One variant per rule, so a test names the rule it is pinning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateFailure {
    /// Gate A: `source_url` is neither a URL nor the literal `measured`.
    SourceUrlShape { id: &'static str, source_url: &'static str },
    /// Gate A: `source_url` is `measured` but `notes` is empty.
    MeasuredWithoutNotes { id: &'static str },
    /// Gate A: `upstream_version` is outside `[A-Za-z0-9._-]+`, so it cannot name one evidence file.
    VersionCharset { id: &'static str, version: &'static str },
    /// Gate B: a `basis` carries no provenance prefix.
    BasisPrefix { id: &'static str, capability: Capability, basis: &'static str },
    /// Gate B: a `basis` contains a newline, which would render as two report lines.
    BasisNewline { id: &'static str, capability: Capability },
    /// Gate B: `unmeasured:` without `Unknown`, or `Unknown` without `unmeasured:`.
    UnmeasuredMismatch { id: &'static str, capability: Capability, state: CapabilityState },
    /// Gate C: `Proven` with empty `notes`.
    ProvenWithoutNotes { id: &'static str },
    /// Gate C: `Proven` with more than one `Unknown` claim.
    ProvenWithTooManyUnknowns { id: &'static str, unknowns: usize },
    /// Gate C: a failed probe that did not degrade to `Experimental` with an unknown config claim.
    UnknownVersionNotExperimental { id: &'static str },
}

/// Gate A's shape half: what can be checked before any transcript exists (SP4 design §5.2).
///
/// The transcript clauses are deliberately absent. They assert files the probe run produces, and the probe
/// run happens after the phase that adds this function, so asserting them here would leave the branch red
/// for three adapters with no artefact able to make it green.
pub fn gate_a_shape(metadata: &AdapterMetadata) -> Result<(), GateFailure> {
    let evidence = metadata.evidence;
    let url =
        evidence.source_url.starts_with("https://") || evidence.source_url.starts_with("http://");
    if !url && evidence.source_url != "measured" {
        return Err(GateFailure::SourceUrlShape {
            id: metadata.id,
            source_url: evidence.source_url,
        });
    }
    if evidence.source_url == "measured" && evidence.notes.is_empty() {
        return Err(GateFailure::MeasuredWithoutNotes { id: metadata.id });
    }
    if evidence.upstream_version.is_empty()
        || !evidence
            .upstream_version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(GateFailure::VersionCharset {
            id: metadata.id,
            version: evidence.upstream_version,
        });
    }
    Ok(())
}

/// Gate B: every `basis` carries a provenance prefix, no newline, and `unmeasured:` exactly when `Unknown`.
///
/// The biconditional is what makes D7 mechanical. Without it an implementer reaches `Proven` by writing
/// `NotGuaranteed` with an `unmeasured:` basis, and every other gate still passes.
pub fn gate_b(metadata: &AdapterMetadata) -> Result<(), GateFailure> {
    for claim in metadata.capabilities {
        let unmeasured = claim.basis.starts_with(UNMEASURED);
        if !unmeasured && !claim.basis.starts_with(MEASURED) && !claim.basis.starts_with(CITED) {
            return Err(GateFailure::BasisPrefix {
                id: metadata.id,
                capability: claim.capability,
                basis: claim.basis,
            });
        }
        if claim.basis.contains('\n') {
            return Err(GateFailure::BasisNewline {
                id: metadata.id,
                capability: claim.capability,
            });
        }
        if unmeasured != (claim.state == CapabilityState::Unknown) {
            return Err(GateFailure::UnmeasuredMismatch {
                id: metadata.id,
                capability: claim.capability,
                state: claim.state,
            });
        }
    }
    Ok(())
}

/// Gate C: what `Proven` costs, and what a failed probe forces (SP4 design §5, D6).
pub fn gate_c(metadata: &AdapterMetadata) -> Result<(), GateFailure> {
    let unknowns = metadata
        .capabilities
        .iter()
        .filter(|claim| claim.state == CapabilityState::Unknown)
        .count();
    if metadata.support == SupportLevel::Proven {
        if metadata.evidence.notes.is_empty() {
            return Err(GateFailure::ProvenWithoutNotes { id: metadata.id });
        }
        // V3:132-133 permits `Proven` while ONE capability remains unresolved; a second means the
        // mechanism itself is not understood, which is what `Experimental` is for.
        if unknowns > 1 {
            return Err(GateFailure::ProvenWithTooManyUnknowns { id: metadata.id, unknowns });
        }
    }
    // A failed probe observed nothing, so it cannot support a claim about the mechanism. Without this the
    // token exemption in Gate A becomes a hole: `upstream_version: "unknown"` beside a `Supported` config
    // claim would reach `Proven` with no mechanism token observed anywhere.
    if metadata.evidence.upstream_version == UNKNOWN_VERSION {
        let config_unknown = metadata.capabilities.iter().any(|claim| {
            claim.capability == Capability::ConfigIsolation
                && claim.state == CapabilityState::Unknown
        });
        if metadata.support != SupportLevel::Experimental || !config_unknown {
            return Err(GateFailure::UnknownVersionNotExperimental { id: metadata.id });
        }
    }
    Ok(())
}

/// Every gate that applies before the probe run, for one adapter.
pub fn gates_before_transcripts(metadata: &AdapterMetadata) -> Result<(), GateFailure> {
    gate_a_shape(metadata)?;
    gate_b(metadata)?;
    gate_c(metadata)
}
```

- [ ] **Step 2: Write `crates/agent-profile/tests/adapter_contract.rs`** — The gates run over the registry, plus a sound fixture and one negative fixture per rule. Write the whole file, byte-exact.

```rust
//! The common adapter contract suite (spec §34 "LaunchPlan", SP2 design §8.1): every registered adapter is
//! checked by the same assertions. Adding an adapter without a row here fails `plan_contract`.

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use agent_profile::adapter::gate::GateFailure;
use agent_profile::adapter::{
    self, Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    PathKind, PlanContext, PlannedLaunch, ProfilePath, ProfilePresence, SupportLevel, gate,
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

    // Gates A-shape, B and C over every real adapter. `fake` is excluded from A and C by design: it has no
    // upstream product, so demanding evidence of one would force a fabricated transcript (SP4 design D4).
    for adapter in adapter::REAL_ADAPTERS {
        let metadata = adapter.metadata();
        assert_eq!(gate::gates_before_transcripts(metadata), Ok(()), "{}", metadata.id);
    }
    // Gate B applies to every adapter including the fixture, which has real claims and must not model bad
    // ones.
    for adapter in &registry {
        let metadata = adapter.metadata();
        assert_eq!(gate::gate_b(metadata), Ok(()), "{}", metadata.id);
    }

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

#[test]
fn case_twin_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        fs::create_dir_all(fixture.root.profiles_dir().join("work")).unwrap();
        let error = fixture.plan(adapter, "WORK", &[]).unwrap_err();
        assert!(
            matches!(error, Error::ProfileCaseConflict { ref existing, .. } if existing == "work"),
            "{id}: {error:?}"
        );
        let entries: Vec<OsString> = fs::read_dir(fixture.root.profiles_dir())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(entries, [OsString::from("work")], "{id}");
    }
}

#[test]
fn not_installed_wins_over_a_case_twin_contract() {
    for adapter in adapter::registry() {
        let dir = tempfile::tempdir().unwrap();
        let root = AppRoot::from_path(dir.path().join("root"));
        fs::create_dir_all(root.profiles_dir().join("work")).unwrap();
        let empty = dir.path().join("empty");
        fs::create_dir(&empty).unwrap();
        let config = Config::load(&root).unwrap();
        let twin = profile("WORK");
        let id = adapter.metadata().id;
        let error = adapter
            .plan(&PlanContext {
                profile: &twin,
                root: &root,
                config: &config,
                args: &[],
                path_var: Some(empty.as_os_str()),
            })
            .unwrap_err();
        assert!(matches!(error, Error::AgentNotInstalled { .. }), "{id}: {error:?}");
    }
}

#[test]
fn existed_follows_the_declared_kind_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        let planned = fixture.plan(adapter, "work", &[]).unwrap();
        adapter.initialize(&planned).unwrap();
        let replanned = fixture.plan(adapter, "work", &[]).unwrap();
        assert!(replanned.paths.iter().all(|entry| entry.existed), "{id}: {:?}", replanned.paths);
    }
    let fixture = fixture();
    let aider = adapter::lookup("aider").unwrap();
    let planned = fixture.plan(aider, "work", &[]).unwrap();
    fs::create_dir_all(&planned.paths[1].path).unwrap();
    let existed: Vec<bool> =
        fixture.plan(aider, "work", &[]).unwrap().paths.iter().map(|entry| entry.existed).collect();
    assert_eq!(existed, [true, false], "a directory at the Aider file is not the file");
}

#[test]
fn initialization_ignores_a_stale_existed_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        let first = fixture.plan(adapter, "work", &[]).unwrap();
        adapter.initialize(&first).unwrap();
        let stale = fixture.plan(adapter, "work", &[]).unwrap();
        fs::remove_dir_all(&stale.profile_dir).unwrap();
        adapter.initialize(&stale).unwrap();
        assert_eq!(
            adapter.presence(&fixture.root, &profile("work")),
            ProfilePresence::Materialized,
            "{id}"
        );
    }
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

// --- Gate negative fixtures (SP4 design §10) ---------------------------------------------------------
//
// The gates are library functions rather than inline assertions precisely so these can exist: a rule
// expressed only over `static METADATA` items is unreachable by `cargo mutants`, so a weaker-than-intended
// gate would pass because no shipped adapter exhibits the excluded combination.

/// A metadata value that passes every gate, for a test to break in exactly one way.
fn sound_metadata() -> AdapterMetadata {
    AdapterMetadata {
        id: "fixture",
        executable: "fixture",
        mechanism_summary: "environment variable FIXTURE_HOME",
        support: SupportLevel::Proven,
        evidence: AdapterEvidence {
            mechanism_id: "fixture-home-v1",
            verified_at: "2026-09-16",
            upstream_version: "1.0.0",
            source_url: "measured",
            notes: "measured in a sandbox",
        },
        capabilities: &[
            CapabilityClaim {
                capability: Capability::ConfigIsolation,
                state: CapabilityState::Supported,
                basis: "measured: config moved with the variable",
            },
            CapabilityClaim {
                capability: Capability::CredentialIsolation,
                state: CapabilityState::Unknown,
                basis: "unmeasured: requires an authenticated session",
            },
            CapabilityClaim {
                capability: Capability::StateIsolation,
                state: CapabilityState::NotSupported,
                basis: "measured: sessions stay in the default location",
            },
        ],
        env: &[],
        conflicts: &[],
    }
}

#[test]
fn the_sound_fixture_passes_every_gate() {
    assert_eq!(gate::gates_before_transcripts(&sound_metadata()), Ok(()));
}

#[test]
fn gate_b_rejects_an_unmeasured_prefix_on_a_non_unknown_state() {
    // The D5 attack verbatim: without the biconditional this reaches `Proven` with nothing measured.
    let mut metadata = sound_metadata();
    metadata.capabilities = &[CapabilityClaim {
        capability: Capability::ConfigIsolation,
        state: CapabilityState::NotGuaranteed,
        basis: "unmeasured: no vendor session available",
    }];
    assert_eq!(
        gate::gate_b(&metadata),
        Err(GateFailure::UnmeasuredMismatch {
            id: "fixture",
            capability: Capability::ConfigIsolation,
            state: CapabilityState::NotGuaranteed,
        })
    );
}

#[test]
fn gate_b_rejects_an_unknown_state_without_the_unmeasured_prefix() {
    let mut metadata = sound_metadata();
    metadata.capabilities = &[CapabilityClaim {
        capability: Capability::CredentialIsolation,
        state: CapabilityState::Unknown,
        basis: "measured: this claim was never measured, whatever it says",
    }];
    assert_eq!(
        gate::gate_b(&metadata),
        Err(GateFailure::UnmeasuredMismatch {
            id: "fixture",
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
        })
    );
}

#[test]
fn gate_b_rejects_a_basis_without_a_provenance_prefix() {
    let mut metadata = sound_metadata();
    metadata.capabilities = &[CapabilityClaim {
        capability: Capability::ConfigIsolation,
        state: CapabilityState::Supported,
        basis: "config moved with the variable",
    }];
    assert!(matches!(gate::gate_b(&metadata), Err(GateFailure::BasisPrefix { .. })));
}

#[test]
fn gate_b_rejects_a_basis_containing_a_newline() {
    // One `Vec` entry is one report line; an embedded newline would silently render as two.
    let mut metadata = sound_metadata();
    metadata.capabilities = &[CapabilityClaim {
        capability: Capability::ConfigIsolation,
        state: CapabilityState::Supported,
        basis: "measured: first line\nsecond line",
    }];
    assert_eq!(
        gate::gate_b(&metadata),
        Err(GateFailure::BasisNewline { id: "fixture", capability: Capability::ConfigIsolation })
    );
}

#[test]
fn gate_c_rejects_proven_with_two_unknown_claims() {
    let mut metadata = sound_metadata();
    metadata.capabilities = &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: the probe never ran",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: requires an authenticated session",
        },
    ];
    assert_eq!(
        gate::gate_c(&metadata),
        Err(GateFailure::ProvenWithTooManyUnknowns { id: "fixture", unknowns: 2 })
    );
}

#[test]
fn gate_c_rejects_proven_with_empty_notes() {
    let mut metadata = sound_metadata();
    metadata.evidence.notes = "";
    assert_eq!(gate::gate_c(&metadata), Err(GateFailure::ProvenWithoutNotes { id: "fixture" }));
}

#[test]
fn gate_c_rejects_an_unknown_version_that_did_not_degrade_to_experimental() {
    // Without this clause, Gate A's token exemption is a hole: a failed probe could still ship `Proven`
    // beside a `Supported` config claim, with no mechanism token observed anywhere.
    let mut metadata = sound_metadata();
    metadata.evidence.upstream_version = "unknown";
    assert_eq!(
        gate::gate_c(&metadata),
        Err(GateFailure::UnknownVersionNotExperimental { id: "fixture" })
    );

    metadata.support = SupportLevel::Experimental;
    assert_eq!(
        gate::gate_c(&metadata),
        Err(GateFailure::UnknownVersionNotExperimental { id: "fixture" }),
        "experimental alone is not enough; the config claim must be Unknown too"
    );

    metadata.capabilities = &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: the probe never ran",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: requires an authenticated session",
        },
    ];
    assert_eq!(gate::gate_c(&metadata), Ok(()));
}

#[test]
fn gate_a_rejects_a_version_outside_the_permitted_charset() {
    // Gate A builds `docs/evidence/<id>-<version>.md` from this field, so it must name one file.
    let mut metadata = sound_metadata();
    metadata.evidence.upstream_version = "1.0.0 (build 7)";
    assert_eq!(
        gate::gate_a_shape(&metadata),
        Err(GateFailure::VersionCharset { id: "fixture", version: "1.0.0 (build 7)" })
    );
}

#[test]
fn gate_a_rejects_a_source_url_that_is_neither_a_url_nor_measured() {
    let mut metadata = sound_metadata();
    metadata.evidence.source_url = "the vendor told me";
    assert!(matches!(gate::gate_a_shape(&metadata), Err(GateFailure::SourceUrlShape { .. })));
    metadata.evidence.source_url = "https://example.com/docs";
    assert_eq!(gate::gate_a_shape(&metadata), Ok(()));
}

#[test]
fn gate_a_rejects_measured_without_notes() {
    let mut metadata = sound_metadata();
    metadata.evidence.notes = "";
    assert_eq!(
        gate::gate_a_shape(&metadata),
        Err(GateFailure::MeasuredWithoutNotes { id: "fixture" })
    );
}
```

- [ ] **Step 3: Write `crates/agent-profile/src/adapter/claude.rs`** — Every basis gains its provenance prefix, which Gate B requires. Write the whole file, byte-exact.

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
            basis: "cited: user settings live in the config directory; project .claude/settings*.json \
                    and .mcp.json still layer on top",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Conditional,
            basis: "measured: credential environment variables override it; the per-directory \
                    .credentials.json and macOS Keychain entry are cited, not measured",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::NotGuaranteed,
            basis: "cited: history and project state moving with the directory is community-sourced only",
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

- [ ] **Step 4: Write `crates/agent-profile/src/adapter/codex.rs`** — Same. Write the whole file, byte-exact.

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
            basis: "cited: config.toml and <name>.config.toml live in CODEX_HOME; project-level \
                    configuration layering is not measured",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Conditional,
            basis: "measured: OPENAI_API_KEY, CODEX_API_KEY and CODEX_ACCESS_TOKEN bypass it; auth.json \
                    and the keyring key follow CODEX_HOME",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Conditional,
            basis: "measured: CODEX_SQLITE_HOME relocates the state database",
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

- [ ] **Step 5: Write `crates/agent-profile/src/adapter/aider.rs`** — Same. Write the whole file, byte-exact.

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
            basis: "measured: the user-level ~/.aider.conf.yml, repository and cwd config files, .env \
                    files and AIDER_* variables still apply",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::NotSupported,
            basis: "measured: API keys come from the environment, .env files and config files",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::NotSupported,
            basis: "measured: history files are written in the working directory",
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

- [ ] **Step 6: Write `crates/agent-profile/src/adapter/fake.rs`** — Same, and its three Unknown claims now carry `unmeasured:` — Gate B's biconditional requires the prefix exactly when the state is Unknown. Write the whole file, byte-exact.

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
            basis: "unmeasured: a fixture has nothing to isolate",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: a fixture has nothing to isolate",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: a fixture has nothing to isolate",
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

- [ ] **Step 7: Edit `crates/agent-profile/src/adapter/mod.rs`** — The gate module is public because the contract suite is an integration test and reaches it through the crate's public surface. Replace exactly this text, which occurs once:

```rust
pub mod metadata;
```

with:

```rust
pub mod gate;
pub mod metadata;
```

- [ ] **Step 8: Run this task's own checks**

```bash
cargo nextest run --workspace --no-tests=pass
```

Expected: passes, including `the_sound_fixture_passes_every_gate`, `gate_b_rejects_an_unknown_state_without_the_unmeasured_prefix`.

- [ ] **Step 9: Run the gate** (see "Gate commands"). Expected on Windows: `243 tests run: 243 passed`. Linux and macOS run fewer tests, because the Windows-only tests are compiled out; every test must pass.

- [ ] **Step 10: Commit**

```bash
git add crates/agent-profile/src/adapter/gate.rs crates/agent-profile/tests/adapter_contract.rs crates/agent-profile/src/adapter/claude.rs crates/agent-profile/src/adapter/codex.rs crates/agent-profile/src/adapter/aider.rs crates/agent-profile/src/adapter/fake.rs crates/agent-profile/src/adapter/mod.rs
git commit -m "feat: enforce the evidence gates as library predicates

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 11: Prove this task's tests are not vacuous** (rule 6). Weakening the biconditional to a single implication is the defect the fixtures exist for: it still rejects `unmeasured:` on a known state, but it lets an implementer reach Proven by writing NotGuaranteed with an unmeasured basis, and every other gate passes.

In `crates/agent-profile/src/adapter/gate.rs` replace exactly:

```rust
if unmeasured != (claim.state == CapabilityState::Unknown) {
```

with:

```rust
if unmeasured && (claim.state != CapabilityState::Unknown) {
```

Run `cargo nextest run --no-fail-fast --workspace --no-tests=pass`. Expected: it FAILS, and a line reporting FAIL names `gate_b_rejects_an_unknown_state_without_the_unmeasured_prefix`. Then restore with `git checkout -- crates/agent-profile/src/adapter/gate.rs` and confirm `git status --short` prints nothing.


### Task 3: Repair the probe harness, and rework the three shipped probes onto it

Make a failed probe fail, give every recorded command a timeout, and put the three shipped probes on the six-step order the design makes a contract.

**Files:**

- Create: `sandbox/probes/text.sh` — Control-sequence stripping and bounded excerpts. Separate from common.sh because sourcing it has no side effects: common.sh installs an EXIT trap and truncates a file on sourcing, so the transcript assembler cannot source it.
- Modify (whole file): `sandbox/probes/common.sh` — probe_record (timeout, accumulate, never abort), probe_fail, probe_finish, the four step helpers, and the four-kind mechanism vocabulary.
- Modify (whole file): `sandbox/probes/claude.sh` — Reworked to the six-step order; it called the deleted probe_agent_profile, which launched the agent through a CLI that rejects unknown agents.
- Modify (whole file): `sandbox/probes/codex.sh` — Same.
- Modify (whole file): `sandbox/probes/aider.sh` — Same, plus the candidate sweep that re-establishes SP2 design D5's `{}` finding.
- Create: `sandbox/tests/probe-harness.sh` — The harness's own tests, including the six-step order asserted over every probe script in the directory.
- Modify: `sandbox/run.sh` — Without this the argument cannot reach a probe at all: run.sh accepts exactly one. Unquoted on purpose: an empty version must produce no argument, not an empty one. The one finding shellcheck reports on the existing harness, and it is a false positive: the function is reached through a trap. Silenced where it happens, with the reason, rather than by lowering the severity the whole gate runs at.
- Modify: `.gitattributes` — Without this every transcript fails verification, and the only message is "not byte-identical": the file is written in a Linux container and committed from a workstation whose working tree is CRLF.
- Modify: `.github/workflows/ci.yml` — The same check CI runs, so a maintainer whose local gate is green is not told otherwise by a pull request.
- Modify: `justfile` — The suite needs no container and no agent, so it belongs in the ordinary gate; and the shell files are now load-bearing enough to lint, since they decide what an evidence transcript says.

**Before:** `sandbox/probes/common.sh` contains `probe_agent_profile`; `sandbox/probes/text.sh` does not exist; `sandbox/tests/probe-harness.sh` does not exist. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `sandbox/probes/text.sh`** — Control-sequence stripping and bounded excerpts. Separate from common.sh because sourcing it has no side effects: common.sh installs an EXIT trap and truncates a file on sourcing, so the transcript assembler cannot source it. Write the whole file, byte-exact.

```bash
# Text handling shared by the probe (sandbox/probes/common.sh) and the transcript assembler
# (sandbox/transcript.sh). Sourcing this file has no side effects, which is why it is separate from
# common.sh: common.sh installs an EXIT trap and truncates a file the moment it is sourced, so the
# assembler cannot source it.
#
# SP4 design §7.4: "Captured output is untrusted vendor text. Before it becomes a committed file it is
# stripped of terminal control sequences and bounded in size; an excerpt that must be truncated says so at
# the truncation point." Both halves live here, in one copy, because they have two callers.

# probe_strip_ansi: remove terminal control sequences and CR line endings from stdin.
#
# The version extraction needs this as much as the transcript does. `ESC[0m` contains the run `0m`, so a
# vendor that colours its `--version` banner would otherwise hand the registry a version of `0m` — and
# Gate A accepts it, because `0m` is inside the charset it checks. `\033` is not portable sed syntax, so
# the escape is built by printf and interpolated.
#
# Two passes, and the second is the one that makes this safe rather than tidy. The first removes CSI
# sequences, which is what produces spurious digit runs. The second deletes every remaining control
# character except tab and newline, so no escape byte reaches a committed file however it was spelled —
# an OSC title, a lone BEL, a NUL from a binary help text. Enumerating each sequence family instead would
# mean the file is safe only against the families someone thought of.
# The `s` command uses a comma delimiter because the CSI intermediate range `[ -/]` ends at a slash, and
# a slash inside a bracket expression is not reliably exempt from ending an `s/.../.../`.
probe_strip_ansi() {
    probe_esc=$(printf '\033')
    sed "s,${probe_esc}\\[[0-9;?]*[ -/]*[@-~],,g" | tr -d '\000-\010\013-\037\177'
}

# probe_excerpt <max-lines>: strip control sequences, bound the text, and say where it was cut.
#
# Both bounds are stated rather than silent. A transcript that stops mid-way with no marker reads as a
# complete record of a short output, and §5.3 commits it byte-for-byte, so an unmarked truncation is a
# durable false statement about what the agent printed.
probe_excerpt() {
    probe_strip_ansi | awk -v max="$1" '
        NR <= max {
            if (length($0) > 500) {
                print substr($0, 1, 500) "  [line truncated]"
            } else {
                print
            }
        }
        NR == max + 1 { print "  [truncated: more than " max " lines]" }
    '
}
```

- [ ] **Step 2: Write `sandbox/probes/common.sh`** — probe_record (timeout, accumulate, never abort), probe_fail, probe_finish, the four step helpers, and the four-kind mechanism vocabulary. Write the whole file, byte-exact.

```bash
# Shared steps for probe scripts. Sourced by sandbox/probes/<agent>.sh inside the sandbox container, from a
# copy of the checkout; results go to /out, which sandbox/run.sh copies back to target/sandbox/.
#
# A probe installs one agent, records its version and help text, and measures whether the agent's own
# isolation mechanism actually moves its files (SP4 design §7.3). It never launches the agent through
# `agent-profile`: the CLI refuses an agent word it does not know, and the nine SP4 agents are unknown to
# the binary at the commit that runs the probe.

set -eu

# Sourced by relative path, because the container runs a probe from the copied checkout's root
# (`sandbox/run.sh:135` cds to /home/probe/work) and so does the shell suite. A missing file here would
# otherwise surface as `probe_strip_ansi: not found` three steps later, naming a symptom instead of a cause.
if [ ! -f sandbox/probes/text.sh ]; then
    echo "probe: run from the repository root; sandbox/probes/text.sh is not here" >&2
    exit 2
fi
. sandbox/probes/text.sh

# Where the probe puts the profile it points the agent at. Fixed, so a probe script never invents one.
# `/out` in the container; overridable so the shell suite can run this file outside one.
PROBE_OUT=${PROBE_OUT:-/out}
PROBE_TARGET=${PROBE_TARGET:-/home/probe/probe-target}
# How long any one recorded command may run. A probe never waits for input; a hang is a recorded fact, not
# a job that burns its whole budget.
PROBE_TIMEOUT=${PROBE_TIMEOUT:-300}
# The version the maintainer asked for, or empty for "whatever the registry serves" (§7.4). `run.sh:138`
# passes it as the probe script's first argument, and a sourced file sees its caller's positional
# parameters, so no probe script has to thread it through.
PROBE_VERSION=${PROBE_VERSION:-${1:-}}
# The candidate file's name inside the profile directory. Some agents key on the extension — Continue
# reads a `config.yaml` and Amp a `settings.json` — so a probe that must name it can, and one whose agent
# does not care leaves it alone.
PROBE_CONFIG_NAME=${PROBE_CONFIG_NAME:-config}

: > "$PROBE_OUT/failures"
: > "$PROBE_OUT/steps"
mkdir -p "$PROBE_TARGET"

# probe_record <name> <command...>: run a command under a timeout, keep its output and exit code in /out.
#
# It returns 0 even when the command failed, and accumulates the failure instead. That is deliberate: these
# scripts run under `set -e`, so returning the command's status would abort the probe at its first failure
# and leave every later step unrecorded — and the later steps are the evidence. `probe_finish`, wired to an
# EXIT trap so a script cannot forget it, is what turns an accumulated failure into a non-zero exit.
probe_record() {
    name=$1
    shift
    # What was run, beside what it printed. The transcript has to say `install: npm install --global
    # @google/gemini-cli@0.60.0` rather than just the install's output, and only the probe knows the
    # command after PROBE_VERSION has been folded into it.
    printf '%s\n' "$*" > "$PROBE_OUT/$name.cmd"
    set +e
    timeout --kill-after=10s "$PROBE_TIMEOUT" "$@" > "$PROBE_OUT/$name.txt" 2>&1
    status=$?
    set -e
    echo "$status" > "$PROBE_OUT/$name.exit-code"
    # Appended in STEP ORDER, which a directory listing cannot reconstruct: sorted by name, `behaviour`
    # precedes `install`, and a transcript in that order reads as an agent launched before it existed.
    printf '%s %s\n' "$name" "$status" >> "$PROBE_OUT/steps"
    if [ "$status" -ne 0 ]; then
        echo "$name $status" >> "$PROBE_OUT/failures"
    fi
    return 0
}

# probe_fail <name> <status>: record a failed step that was not a single recorded command.
#
# It exists so that a step which fails for its own reasons — an executable that vanished after a
# successful install, an installer asked for a version it cannot honour — lands in all three of the same
# places a failed command does. A step that reported only to stderr would leave `exit-codes:` in the
# transcript claiming the probe ran clean.
#
# It deliberately does NOT share a helper with `probe_record`'s failure path. It writes the same three
# records, but it is the whole of its caller's bookkeeping, whereas `probe_record` has already written
# two of them by the time it knows the status. Routing one through the other appended the step to
# `steps` twice, which reads as the probe having run it twice.
probe_fail() {
    echo "$2" > "$PROBE_OUT/$1.exit-code"
    echo "$1 $2" >> "$PROBE_OUT/failures"
    printf '%s %s\n' "$1" "$2" >> "$PROBE_OUT/steps"
}

# probe_finish: write the summary and exit non-zero if any recorded command failed.
probe_finish() {
    status=$?
    if [ -s "$PROBE_OUT/failures" ]; then
        echo "probe: recorded failures:" >&2
        cat "$PROBE_OUT/failures" >&2
        # A script that already failed for its own reason KEEPS that status. `exit 2` means "you asked
        # for something this probe cannot do" and 1 means "the probe ran and a step failed" — a maintainer
        # re-runs after the first and investigates after the second. Flattening both to 1 would hide the
        # distinction behind a `failures` file that looks the same either way.
        [ "$status" -ne 0 ] || status=1
    fi
    exit "$status"
}
trap probe_finish EXIT

# --- Step 1: install ------------------------------------------------------------------------------
#
# Each installer is a separate function rather than one with a mode argument, because only some of them
# can honour a pinned version and the difference has to be visible at the call site. All three record
# under the name `install`, so the transcript's step names are the same six for every agent.

# probe_npm_install <package>: install one npm package globally, at the requested version if given.
probe_npm_install() {
    probe_pkg=$1
    shift
    # Extra flags come from the vendor's own documented command, not from us: Pi documents
    # `npm install -g --ignore-scripts @earendil-works/pi-coding-agent`, and dropping the flag would
    # measure an install the vendor does not describe.
    probe_record install npm install --global "$@" "$probe_pkg${PROBE_VERSION:+@$PROBE_VERSION}"
}

# probe_uv_install <package> <python-version>: install one Python tool, at the requested version if given.
# The interpreter is pinned by the caller; CONTRIBUTING.md:108-109's pinning guidance covers exactly this.
probe_uv_install() {
    probe_record install uv tool install --python "$2" "$1${PROBE_VERSION:+==$PROBE_VERSION}"
}

# probe_script_install <url>: run a vendor install script, as the vendor documents it.
#
# A version request is REFUSED rather than ignored. `run.sh` accepts one for every agent, but a vendor
# script takes whatever it takes; silently dropping the argument would produce a transcript whose
# `install:` line names a version the installer never saw, and §5.3 then commits that false statement
# byte-for-byte. Failing is recoverable — re-run without the argument; a wrong transcript is not.
probe_script_install() {
    if [ -n "$PROBE_VERSION" ]; then
        echo "probe: $1 takes no version argument (asked for $PROBE_VERSION)" >&2
        probe_fail install 2
        exit 2
    fi
    # The interpreter is the vendor's, because the script is the vendor's. Both script-installed agents
    # document `| bash`, and a bash script run under dash fails in ways that would be recorded as the
    # agent failing to install.
    probe_record install sh -c "curl -fsSL '$1' | ${2:-sh}"
}

# --- Step 2: version ------------------------------------------------------------------------------

# probe_version <executable>: record `--version` verbatim, and beside it the one token SP4b types into
# the registry.
#
# The extraction rule is §7.3 step 2's: the first `[A-Za-z0-9._-]` run in the output that contains a
# digit. It lives here rather than in a reviewer's head because Gate A resolves a path from the recorded
# version and SP4b resolves the same path from the registry; if the two derive the token differently, the
# gate fails on a file that exists. `aider --version` prints `aider 0.86.2` — a line, not a token, and the
# space alone violates Gate A's charset.
#
# The raw capture stays verbatim: it is the evidence. The extraction reads a stripped copy, because a
# coloured banner begins `ESC[0m`, whose `0m` is a digit-bearing run that Gate A would happily accept.
probe_version() {
    probe_record version "$1" --version
    probe_extracted=$(
        probe_strip_ansi < "$PROBE_OUT/version.txt" \
            | tr -cs 'A-Za-z0-9._-' '\n' \
            | grep -m1 '[0-9]' \
            || true
    )
    # No such run means the agent printed no version: a §9 outcome-2 probe. `unknown` is the encoding
    # Gate C keys on to force the adapter to Experimental, so the probe states it rather than leaving the
    # field empty for a human to fill in.
    printf '%s\n' "${probe_extracted:-unknown}" > "$PROBE_OUT/version.extracted"
}

# --- Step 3: help ---------------------------------------------------------------------------------

# probe_help <executable>: record `--help`, the artefact Gate A reads the mechanism token from.
probe_help() {
    probe_record help "$1" --help
}

# --- Step 4: strings ------------------------------------------------------------------------------

# probe_strings <executable> [documented-variable...]
#
# Two questions, one file: does each DOCUMENTED credential variable actually appear in what was installed,
# and what UNDOCUMENTED ones appear beside them? The second is the one that pays. Claude's adapter cites
# `CLAUDE_CODE_USE_BEDROCK` and three OAuth variables as "binary strings measured" (`claude.rs:27-29`) —
# no page documented them, and each one is a way a user defeats the isolation the adapter promises.
#
# There is no `strings(1)` in the image: `sandbox/Containerfile:17` installs no binutils. `grep -a` reads
# a binary as text and is already a dependency, so this uses that rather than growing the image.
probe_strings() {
    probe_exe=$1
    shift
    probe_out_file=$PROBE_OUT/strings.txt
    probe_resolved=$(command -v "$probe_exe" 2>/dev/null || true)
    if [ -z "$probe_resolved" ]; then
        printf '# %s is not on PATH; nothing to scan\n' "$probe_exe" > "$probe_out_file"
        probe_fail strings 127
        return 0
    fi
    probe_real=$(readlink -f "$probe_resolved")
    probe_dir=$(dirname "$probe_real")
    printf '# %s -> %s\n' "$probe_resolved" "$probe_real" > "$probe_out_file"

    # The resolved file and its siblings. An npm package's bundle sits beside its entry point and a
    # script-installed agent is one binary, so one directory covers both shapes. `-size -64M` keeps a
    # vendored toolchain from turning this step into the job's time limit.
    for probe_var in "$@"; do
        # Tested by what grep PRINTS, not by find's exit status. `-exec ... +` batches, and it reports
        # failure when ANY batch's grep found nothing — so a variable present in the first of two batches
        # would be recorded ABSENT, which is the answer that ends an investigation early.
        if find "$probe_dir" -maxdepth 1 -type f -size -64M \
            -exec grep -aFl -e "$probe_var" {} + 2>/dev/null | grep -q .; then
            printf 'documented %s: present\n' "$probe_var" >> "$probe_out_file"
        else
            printf 'documented %s: ABSENT\n' "$probe_var" >> "$probe_out_file"
        fi
    done

    printf '#\n# variable-shaped tokens that name a credential or a location, documented or not\n' \
        >> "$probe_out_file"
    find "$probe_dir" -maxdepth 1 -type f -size -64M \
        -exec grep -aohE '[A-Z][A-Z0-9_]{3,}' {} + 2>/dev/null \
        | grep -E '(KEY|TOKEN|SECRET|CREDENTIAL|PASSWORD|AUTH|HOME|CONFIG|DATA_DIR|PROFILE|SETTINGS)' \
        | LC_ALL=C sort -u \
        | probe_excerpt 200 >> "$probe_out_file"
}

# --- Steps 5 and 6: baseline and behaviour --------------------------------------------------------

# probe_snapshot <name> <dir...>: record a sorted listing of each directory, for a before/after comparison.
probe_snapshot() {
    name=$1
    shift
    : > "$PROBE_OUT/$name.txt"
    for dir in "$@"; do
        echo "# $dir" >> "$PROBE_OUT/$name.txt"
        if [ -d "$dir" ]; then
            find "$dir" -printf '%y %P\n' 2>/dev/null | LC_ALL=C sort >> "$PROBE_OUT/$name.txt"
        elif [ -e "$dir" ]; then
            # A default location is not always a directory: Aider's is the file `.aider.conf.yml`
            # (`aider.rs:15`). Reporting an existing file as `(absent)` would read as the agent having
            # written nothing to its default location, which is the finding the whole probe is for.
            find "$dir" -maxdepth 0 -printf '%y %f\n' 2>/dev/null >> "$PROBE_OUT/$name.txt"
        else
            echo "(absent)" >> "$PROBE_OUT/$name.txt"
        fi
    done
}

# probe_delta <baseline-file> <after-file> <out-file>: what the launch actually changed.
#
# This is a COMPUTED difference, not the after-state under a suggestive name. The distinction is the whole
# measurement: the probe creates the target directory itself, and for a file mechanism it writes the
# candidate file, so a post-launch listing contains the probe's own bytes. Reading that listing as "what
# the agent wrote" yields a false `ConfigIsolation: Supported` carrying a `measured:` basis — and every
# gate still passes, because the gates check a claim's shape, not whether the measurement behind it meant
# anything.
#
# `+` is a path that appeared, `-` one that went away. An empty file means the launch changed nothing,
# unambiguously, which is a finding rather than a gap.
probe_delta() {
    LC_ALL=C sort "$1" > "$PROBE_OUT/.delta-before"
    LC_ALL=C sort "$2" > "$PROBE_OUT/.delta-after"
    {
        comm -13 "$PROBE_OUT/.delta-before" "$PROBE_OUT/.delta-after" | sed 's/^/+ /'
        comm -23 "$PROBE_OUT/.delta-before" "$PROBE_OUT/.delta-after" | sed 's/^/- /'
    } > "$3"
    rm -f "$PROBE_OUT/.delta-before" "$PROBE_OUT/.delta-after"
}

# probe_label <mechanism>: the filename-safe form a mechanism's artefacts are named by.
probe_label() {
    printf '%s' "$1" | tr -c 'A-Za-z0-9_' '-'
}

# probe_prepare_target [candidate]: an empty profile directory, holding the candidate file if one is given.
#
# `@none` means "create no file", which is a content worth testing in its own right: Aider refuses a
# MISSING `.aider.conf.yml` exactly as it refuses an empty one (`aider.rs:17-18`), and an adapter that
# creates nothing would hit that.
probe_prepare_target() {
    rm -rf "$PROBE_TARGET"
    mkdir -p "$PROBE_TARGET"
    case "${1:-@none}" in
        @none) ;;
        *) printf '%s' "$1" > "$PROBE_TARGET/$PROBE_CONFIG_NAME" ;;
    esac
}

# probe_apply <name> <executable> <mechanism> <arg>...: run the agent once with the mechanism applied.
#
# The mechanism vocabulary is FOUR words, not the two §7.3 first named, because two cannot express what §8
# asks the probe to test. `OPENCODE_CONFIG` is a variable naming a FILE (§8.2) and Cline's `--data-dir` is
# a flag naming a DIRECTORY (§8.3). Under an `env:`/`flag:` vocabulary the probe would have pointed a file
# variable at a directory and a directory flag at a file, the agent would have refused, and the refusal
# would have been recorded as evidence that the mechanism does not isolate — a wrong answer that looks
# like a measurement.
#
#   env:<VAR>        the variable is set to the profile directory
#   envfile:<VAR>    the variable is set to a file inside it
#   flagdir:<FLAG>   the flag is passed the profile directory
#   flagfile:<FLAG>  the flag is passed a file inside it
probe_apply() {
    probe_name=$1
    probe_exe=$2
    probe_mech=$3
    shift 3
    case "$probe_mech" in
        env:*)
            probe_record "$probe_name" env "${probe_mech#env:}=$PROBE_TARGET" "$probe_exe" "$@" ;;
        envfile:*)
            probe_record "$probe_name" env "${probe_mech#envfile:}=$PROBE_TARGET/$PROBE_CONFIG_NAME" "$probe_exe" "$@" ;;
        flagdir:*)
            probe_record "$probe_name" "$probe_exe" "${probe_mech#flagdir:}" "$PROBE_TARGET" "$@" ;;
        flagfile:*)
            probe_record "$probe_name" "$probe_exe" "${probe_mech#flagfile:}" "$PROBE_TARGET/$PROBE_CONFIG_NAME" "$@" ;;
        *)
            echo "probe: unknown mechanism $probe_mech" >&2
            probe_fail "$probe_name" 2
            return 1
            ;;
    esac
}

# probe_behaviour <executable> <default-location> <mechanism> <candidate|@none> [launch-arg...]
#
# Steps 5 and 6 as a pair: baseline both locations, apply the agent's own isolation mechanism, launch, and
# record what changed at both. Repeat per mechanism under test, each repetition re-baselining first — a
# second launch measured against the first launch's baseline is not attributable, which is the whole point
# of step 5.
#
# The first argument is the EXECUTABLE, not the agent id: they differ for kiro (kiro-cli), cursor
# (cursor-agent) and continue (cn), and this file has no registry to look one up in.
#
# The launch arguments default to `--version` and each probe script overrides them where it can, because
# `--version` is a FLOOR, not a good measurement: an agent that exits before initialising writes nothing,
# and an empty delta then reads as "the mechanism does not move anything" when it means "nothing was asked
# to move". Which cheap command makes a given agent initialise is per-agent and itself unmeasured, so a
# probe that has no better answer records the empty delta as the fact it is rather than inventing one.
probe_behaviour() {
    probe_exe=$1
    probe_default=$2
    probe_mech=$3
    probe_cand=${4:-@none}
    # `shift` is a special builtin: shifting past $# is an error that terminates a non-interactive POSIX
    # shell outright, so the count is bounded rather than guarded with `|| true`.
    if [ $# -ge 4 ]; then shift 4; else shift $#; fi
    [ $# -gt 0 ] || set -- --version
    probe_lbl=$(probe_label "$probe_mech")

    # Baseline BOTH locations, after the install and after anything this probe itself created. The install
    # ran vendor code, and a file mechanism means the probe wrote the candidate; neither is the agent's
    # doing, and attributing them to the agent would read as isolation that did not happen.
    probe_prepare_target "$probe_cand"
    # <default-location> is a whitespace-separated LIST, deliberately unquoted below. OpenCode needs two:
    # `OPENCODE_CONFIG_DIR` governs the config directory while `auth.json` lives under the XDG data
    # directory (§8.2), and watching only the first would record "credentials did not move" as an absence
    # of evidence rather than the measured fact §6 says is enough to claim NotSupported honestly.
    # shellcheck disable=SC2086 # the caller supplies literal paths, and the split is the point
    probe_snapshot "baseline-$probe_lbl" "$PROBE_TARGET" $probe_default
    probe_apply "behaviour-$probe_lbl" "$probe_exe" "$probe_mech" "$@"
    # shellcheck disable=SC2086
    probe_snapshot "after-$probe_lbl" "$PROBE_TARGET" $probe_default
    probe_delta "$PROBE_OUT/baseline-$probe_lbl.txt" "$PROBE_OUT/after-$probe_lbl.txt" \
        "$PROBE_OUT/delta-$probe_lbl.txt"
}

# probe_candidates <executable> <mechanism> <candidate|@none>...
#
# §8.4's question, which is about ACCEPTANCE and not about isolation: what is the smallest file content the
# agent accepts that sets no option? `agent-profile` has to create that file before the agent reads it, so
# the content is part of the mechanism. Aider set both the precedent and the cost of guessing — missing,
# empty and comment-only `.aider.conf.yml` each exit 2, and `{}` was accepted only because it was measured
# (`aider.rs:17-18`, SP2 design D5).
#
# One exit code per candidate, plus an index naming what each one held, because the transcript has to say
# which content the exit code belongs to.
probe_candidates() {
    probe_exe=$1
    probe_mech=$2
    shift 2
    probe_n=0
    : > "$PROBE_OUT/candidates.txt"
    for probe_cand in "$@"; do
        probe_n=$((probe_n + 1))
        probe_prepare_target "$probe_cand"
        printf '%s: %s\n' "$probe_n" "$(printf '%s' "$probe_cand" | tr '\n' ' ')" \
            >> "$PROBE_OUT/candidates.txt"
        probe_apply "candidate-$probe_n" "$probe_exe" "$probe_mech" --version
    done
}
```

- [ ] **Step 3: Write `sandbox/probes/claude.sh`** — Reworked to the six-step order; it called the deleted probe_agent_profile, which launched the agent through a CLI that rejects unknown agents. Write the whole file, byte-exact.

```bash
# Claude Code probe: `sandbox/run.sh probe claude [version]`.
#
# A re-probe. SP2 measured 2.1.270 and never committed a transcript (7.4), so there is nothing to
# reproduce and this resolves latest rather than pinning -- the fold commit updates `claude.rs:23` to
# whatever it finds.
#
# Package @anthropic-ai/claude-code, executable `claude`. The documented bypass list is longer than any
# one page: `claude.rs:27-29` records ANTHROPIC_PROFILE and the BEDROCK/VERTEX switches as "binary strings
# measured", so step 4 is the step that established them and this re-probe has to reproduce it.
. sandbox/probes/common.sh

probe_npm_install @anthropic-ai/claude-code
probe_version claude
probe_help claude
probe_strings claude \
    ANTHROPIC_API_KEY ANTHROPIC_AUTH_TOKEN ANTHROPIC_PROFILE \
    CLAUDE_CODE_OAUTH_TOKEN CLAUDE_CODE_OAUTH_REFRESH_TOKEN \
    CLAUDE_CODE_USE_BEDROCK CLAUDE_CODE_USE_VERTEX CLAUDE_CODE_USE_FOUNDRY
probe_behaviour claude "$HOME/.claude" env:CLAUDE_CONFIG_DIR @none
```

- [ ] **Step 4: Write `sandbox/probes/codex.sh`** — Same. Write the whole file, byte-exact.

```bash
# Codex CLI probe: `sandbox/run.sh probe codex [version]`.
#
# A re-probe; see claude.sh for why it resolves latest rather than pinning `codex.rs:27`'s 0.153.4.
# Package @openai/codex, executable `codex`, mechanism CODEX_HOME (`codex.rs:13`).
. sandbox/probes/common.sh

probe_npm_install @openai/codex
probe_version codex
probe_help codex
probe_strings codex OPENAI_API_KEY OPENAI_BASE_URL CODEX_API_KEY
probe_behaviour codex "$HOME/.codex" env:CODEX_HOME @none
```

- [ ] **Step 5: Write `sandbox/probes/aider.sh`** — Same, plus the candidate sweep that re-establishes SP2 design D5's `{}` finding. Write the whole file, byte-exact.

```bash
# Aider probe: `sandbox/run.sh probe aider [version]`.
#
# A re-probe; see claude.sh for why it resolves latest rather than pinning `aider.rs:32`'s 0.86.2.
# Python 3.12 is pinned because newer interpreters can start long source builds of Aider's native
# dependencies -- that is the INTERPRETER pin CONTRIBUTING.md:108-109 describes, not a package pin.
#
# Aider's default location is a FILE, `$HOME/.aider.conf.yml` (`aider.rs:15`), not a directory.
#
# The candidate sweep re-establishes SP2 design D5's finding -- missing, empty and comment-only each exit
# 2 while `{}` is accepted (`aider.rs:17-18`). It was measured once and never recorded; 7.4 says a
# transcript is what makes a measurement evidence, so the one adapter whose file content is already known
# is also the one that proves the sweep reports what SP2 found.
. sandbox/probes/common.sh

PROBE_CONFIG_NAME=.aider.conf.yml

probe_uv_install aider-chat 3.12
probe_version aider
probe_help aider
probe_strings aider OPENAI_API_KEY ANTHROPIC_API_KEY AIDER_MODEL
probe_candidates aider flagfile:--config @none "" "# nothing" "{}"
probe_behaviour aider "$HOME/.aider.conf.yml" flagfile:--config "{}"
```

- [ ] **Step 6: Write `sandbox/tests/probe-harness.sh`** — The harness's own tests, including the six-step order asserted over every probe script in the directory. Write the whole file, byte-exact.

```bash
#!/bin/sh
# Tests for sandbox/probes/common.sh and sandbox/probes/text.sh. Run by `just check` and by CI; no
# container and no agent needed.
#
# The defect these exist for: `probe_record` used to end with `set -e`, and a shell function returns the
# status of its last command, so it always returned 0. Every probe script ends in a recorded command, so a
# probe whose install failed and whose every later step returned 127 still exited 0 and the job went green.
# A test of `probe_record` alone would not have caught it — the status that mattered was the SCRIPT's.

# shellcheck disable=SC2016 # a probe body is single-quoted ON PURPOSE: it must expand in the generated
# script, not in this shell. Expanding it here would substitute the test runner's empty $PROBE_OUT and
# every check would then assert against a path in the filesystem root.

set -eu
root=$(cd "$(dirname "$0")/../.." && pwd)
failures=0

check() {
    if [ "$2" = "$3" ]; then
        echo "ok   - $1"
    else
        echo "FAIL - $1: expected '$3', got '$2'"
        failures=$((failures + 1))
    fi
}

# A probe script, written to a temporary directory and run exactly as the container runs one: from the
# repository root, because that is where `sandbox/run.sh:135` leaves a probe and it is how common.sh
# resolves its own sibling files.
run_probe() {
    out=$(mktemp -d)
    script=$(mktemp)
    cat > "$script" <<SCRIPT
cd $root
PROBE_OUT=$out
PROBE_TARGET=$out/target
PROBE_TIMEOUT=5
PATH=$out/bin:\$PATH
. sandbox/probes/common.sh
$1
SCRIPT
    mkdir -p "$out/bin"
    set +e
    sh "$script" > "$out/stdout" 2>&1
    probe_status=$?
    set -e
    PROBE_OUT_DIR=$out
}

# fake_agent <name> <body>: an executable on the probe's PATH, so a step can be tested without an agent.
# Call it after run_probe has made the directory, and before the run that uses it — see `staged` below.
staged=

# stage <name> <body>: queue an executable to be created inside the next run_probe, at $PROBE_OUT/bin.
stage() {
    staged="$staged
mkdir -p \"\$PROBE_OUT/bin\"
cat > \"\$PROBE_OUT/bin/$1\" <<'BODY'
#!/bin/sh
$2
BODY
chmod +x \"\$PROBE_OUT/bin/$1\""
}

run_staged() {
    run_probe "$staged
$1"
    staged=
}

# --- probe_record ---------------------------------------------------------------------------------

run_probe 'probe_record ok true'
check "a probe whose steps all pass exits zero" "$probe_status" "0"

run_probe 'probe_record boom sh -c "exit 3"'
check "a probe whose only step failed exits non-zero" "$probe_status" "1"
check "the failing step's exit code is recorded" "$(cat "$PROBE_OUT_DIR/boom.exit-code")" "3"

# The regression that matters: a failure in the MIDDLE must neither abort the probe nor be forgotten by it.
run_probe 'probe_record first sh -c "exit 3"
probe_record second true
probe_record third true'
check "a middle failure still exits non-zero" "$probe_status" "1"
check "steps after a failure are still recorded" "$(cat "$PROBE_OUT_DIR/third.exit-code")" "0"

# `steps` is what the transcript's exit-codes block is built from, in the order the probe ran them.
run_probe 'probe_record one true
probe_record two sh -c "exit 3"
probe_record three true'
check "every step is logged once, in order" \
    "$(tr "\n" "," < "$PROBE_OUT_DIR/steps")" "one 0,two 3,three 0,"
check "the command is recorded beside its output" \
    "$(cat "$PROBE_OUT_DIR/two.cmd")" "sh -c exit 3"

# A step that fails for its OWN reason must land in the log exactly once too. Routing probe_record's
# failure path through probe_fail appended it twice, which reads as the probe having run it twice.
run_probe 'probe_record ok true
probe_strings nosuchagent SOME_KEY'
check "a non-command failure is logged once" \
    "$(tr "\n" "," < "$PROBE_OUT_DIR/steps")" "ok 0,strings 127,"

run_probe 'probe_record slow sleep 30'
check "a hanging step is killed and recorded" "$(cat "$PROBE_OUT_DIR/slow.exit-code")" "124"
check "a hanging step fails the probe" "$probe_status" "1"

# --- probe_behaviour ------------------------------------------------------------------------------

# probe_behaviour must baseline the target as well as the default location: agent-profile and the probe
# itself create files there, and counting them as the agent's would read as isolation that did not happen.
run_probe 'mkdir -p "$PROBE_TARGET" && echo ours > "$PROBE_TARGET/pre-existing"
probe_behaviour true /nonexistent-default env:SOME_HOME @none'
check "the behaviour step records a baseline" \
    "$([ -f "$PROBE_OUT_DIR/baseline-env-SOME_HOME.txt" ] && echo yes || echo no)" "yes"
check "the baseline covers the default location too" \
    "$(grep -c '^# ' "$PROBE_OUT_DIR/baseline-env-SOME_HOME.txt")" "2"

# THE assertion that separates "the agent's config moved" a from "the probe created it". The probe makes
# the target directory itself and, for a file mechanism, writes the candidate file — so a post-launch
# LISTING always contains the probe's own bytes. Reading that as the agent's work yields a false
# ConfigIsolation: Supported carrying a `measured:` basis, and every gate still passes, because the gates
# check a claim's shape and not whether the measurement meant anything.
stage quiet 'exit 0'
run_staged 'probe_behaviour quiet /nonexistent-default flagfile:--config "{}"'
check "a target populated before the launch is not in the delta" \
    "$(cat "$PROBE_OUT_DIR/delta-flagfile---config.txt")" ""
check "the candidate the PROBE wrote is in the baseline" \
    "$(grep -c 'config$' "$PROBE_OUT_DIR/baseline-flagfile---config.txt")" "1"

# The positive control, without which the check above passes for a delta that is always empty.
stage noisy 'mkdir -p "$2"/sub 2>/dev/null || true; : > "$(dirname "$2")/written-by-the-agent"'
run_staged 'probe_behaviour noisy /nonexistent-default flagfile:--config "{}"'
check "a file the agent wrote IS in the delta" \
    "$(grep -c '^+ f written-by-the-agent$' "$PROBE_OUT_DIR/delta-flagfile---config.txt")" "1"

# Each of the four mechanism kinds must reach the agent as the SHAPE it expects. Two kinds cannot express
# OpenCode's OPENCODE_CONFIG (a variable naming a file) or Cline's --data-dir (a flag naming a directory),
# and pointing either at the wrong shape makes the agent's refusal look like a failure to isolate.
stage recorder 'echo "argv: $*"; echo "SOME_VAR=${SOME_VAR-unset}"'
run_staged 'probe_behaviour recorder /nonexistent-default env:SOME_VAR @none'
check "env: sets the variable to the profile directory" \
    "$(sed -n 2p "$PROBE_OUT_DIR/behaviour-env-SOME_VAR.txt")" "SOME_VAR=$PROBE_OUT_DIR/target"

stage recorder 'echo "argv: $*"; echo "SOME_VAR=${SOME_VAR-unset}"'
run_staged 'probe_behaviour recorder /nonexistent-default envfile:SOME_VAR "{}"'
check "envfile: sets the variable to a file inside it" \
    "$(sed -n 2p "$PROBE_OUT_DIR/behaviour-envfile-SOME_VAR.txt")" "SOME_VAR=$PROBE_OUT_DIR/target/config"
check "envfile: writes the candidate to that file" \
    "$(cat "$PROBE_OUT_DIR/target/config")" "{}"

stage recorder 'echo "argv: $*"'
run_staged 'probe_behaviour recorder /nonexistent-default flagdir:--data-dir @none'
check "flagdir: passes the profile directory" \
    "$(cat "$PROBE_OUT_DIR/behaviour-flagdir---data-dir.txt")" \
    "argv: --data-dir $PROBE_OUT_DIR/target --version"

stage recorder 'echo "argv: $*"'
run_staged 'probe_behaviour recorder /nonexistent-default flagfile:--config "{}"'
check "flagfile: passes a file inside it" \
    "$(cat "$PROBE_OUT_DIR/behaviour-flagfile---config.txt")" \
    "argv: --config $PROBE_OUT_DIR/target/config --version"

# `--version` is a floor: an agent that exits before initialising writes nothing, and an empty delta then
# reads as "the mechanism moves nothing". A probe that knows a better command must be able to say so.
stage recorder 'echo "argv: $*"'
run_staged 'probe_behaviour recorder /nonexistent-default env:SOME_VAR @none --print hello'
check "a probe can choose the command that makes its agent initialise" \
    "$(cat "$PROBE_OUT_DIR/behaviour-env-SOME_VAR.txt")" "argv: --print hello"

run_probe 'probe_behaviour true /nonexistent-default bogus:THING @none'
check "an unknown mechanism fails the probe" "$probe_status" "1"

# --- probe_candidates -----------------------------------------------------------------------------

# §8.4: the smallest content the agent accepts is part of the mechanism, and Aider proved guessing costs —
# missing, empty and comment-only each exit 2 while `{}` is accepted (aider.rs:17-18).
stage picky 'test -s "$2" || exit 2; grep -q "{}" "$2" || exit 2'
run_staged 'probe_candidates picky flagfile:--config @none "" "# comment" "{}"'
check "a missing candidate file is recorded as refused" \
    "$(cat "$PROBE_OUT_DIR/candidate-1.exit-code")" "2"
check "an empty candidate is recorded as refused" \
    "$(cat "$PROBE_OUT_DIR/candidate-2.exit-code")" "2"
check "a comment-only candidate is recorded as refused" \
    "$(cat "$PROBE_OUT_DIR/candidate-3.exit-code")" "2"
check "the accepted candidate is recorded as accepted" \
    "$(cat "$PROBE_OUT_DIR/candidate-4.exit-code")" "0"
check "the index says what each candidate held" \
    "$(sed -n 4p "$PROBE_OUT_DIR/candidates.txt")" "4: {}"

# --- probe_strip_ansi and probe_excerpt -----------------------------------------------------------

run_probe 'printf "\033[1;31mred\033[0m plain\n" | probe_strip_ansi > "$PROBE_OUT/stripped"'
check "colour sequences are removed" "$(cat "$PROBE_OUT_DIR/stripped")" "red plain"

run_probe 'printf "a\007b\r\n" | probe_strip_ansi | od -c | head -n1 > "$PROBE_OUT/stripped"'
check "stray control bytes are removed" \
    "$(awk '{print $2 $3 $4}' "$PROBE_OUT_DIR/stripped")" "ab\n"

run_probe 'printf "1\n2\n3\n4\n" | probe_excerpt 2 > "$PROBE_OUT/excerpt"'
check "an excerpt is bounded" "$(head -n2 "$PROBE_OUT_DIR/excerpt" | tr "\n" " ")" "1 2 "
check "a truncated excerpt says so at the cut" \
    "$(sed -n 3p "$PROBE_OUT_DIR/excerpt")" "  [truncated: more than 2 lines]"

run_probe 'printf "1\n2\n" | probe_excerpt 5 > "$PROBE_OUT/excerpt"'
check "an excerpt that fits carries no marker" "$(grep -c truncated "$PROBE_OUT_DIR/excerpt")" "0"

# --- probe_version --------------------------------------------------------------------------------

stage fakeagent 'echo "fakeagent 1.2.3-beta (build 77)"'
run_staged 'probe_version fakeagent'
check "the version is the first digit-bearing run" \
    "$(cat "$PROBE_OUT_DIR/version.extracted")" "1.2.3-beta"
check "the raw version output is kept verbatim" \
    "$(cat "$PROBE_OUT_DIR/version.txt")" "fakeagent 1.2.3-beta (build 77)"

# Gate A accepts `0m` — it is inside the charset it checks — so an un-stripped colour reset would be
# recorded as the version, name an evidence file, and pass every gate. This is the check that pins it.
stage fakeagent 'printf "\033[0m\033[1mfakeagent\033[0m 4.5.6\n"'
run_staged 'probe_version fakeagent'
check "a coloured banner does not become the version" \
    "$(cat "$PROBE_OUT_DIR/version.extracted")" "4.5.6"

stage fakeagent 'echo "no version here"'
run_staged 'probe_version fakeagent'
check "an agent that prints no version records unknown" \
    "$(cat "$PROBE_OUT_DIR/version.extracted")" "unknown"

stage fakeagent 'echo "boom" >&2; exit 4'
run_staged 'probe_version fakeagent'
check "a failing --version fails the probe" "$probe_status" "1"

# --- probe_strings --------------------------------------------------------------------------------

stage fakeagent 'echo "reads FAKE_API_KEY and FAKE_HOME"'
run_staged 'probe_strings fakeagent FAKE_API_KEY MISSING_TOKEN'
check "a documented variable that is present is recorded" \
    "$(grep -c '^documented FAKE_API_KEY: present$' "$PROBE_OUT_DIR/strings.txt")" "1"
check "a documented variable that is absent is recorded" \
    "$(grep -c '^documented MISSING_TOKEN: ABSENT$' "$PROBE_OUT_DIR/strings.txt")" "1"
# The undocumented sweep is the half that pays: FAKE_HOME was never passed in.
check "an undocumented variable is still found" \
    "$(grep -c '^FAKE_HOME$' "$PROBE_OUT_DIR/strings.txt")" "1"

run_probe 'probe_strings nosuchagent SOME_KEY'
check "scanning a missing executable fails the probe" "$probe_status" "1"
check "scanning a missing executable records an exit code" \
    "$(cat "$PROBE_OUT_DIR/strings.exit-code")" "127"

# --- the version passthrough ----------------------------------------------------------------------

# A vendor script takes whatever version it takes. Dropping the request silently would put a version the
# installer never saw into a transcript that §5.3 then commits byte-for-byte.
run_probe 'PROBE_VERSION=9.9.9
probe_script_install https://example.invalid/install.sh'
check "an installer that cannot pin refuses a version" "$probe_status" "2"
check "the refusal is recorded as a failed install" \
    "$(cat "$PROBE_OUT_DIR/install.exit-code")" "2"

# --- the step order, over every probe script -------------------------------------------------------

# §7.3: "Every probe script then follows this order exactly. The order is part of the contract, because
# probe_behaviour's delta is only attributable if nothing has run the agent before it." A contract that
# only exists in prose is one a twelfth probe script can quietly break — and it would not fail, it would
# produce a plausible, wrong measurement: a baseline taken after a launch attributes the launch's own
# files to the install.
for script in "$root"/sandbox/probes/*.sh; do
    name=$(basename "$script" .sh)
    case "$name" in
        common | text) continue ;;
    esac

    # The step each call appears at, so the ORDER can be asserted rather than mere presence.
    order=$(grep -n '^probe_\(npm_install\|uv_install\|script_install\|version\|help\|strings\|behaviour\|candidates\)' \
        "$script" | sed 's/:.*probe_/ /' | sed 's/_install//' | awk '{print $2}' | tr '\n' ' ')

    case "$order" in
        # install, version, help, strings, then one or more behaviour/candidates steps.
        "npm version help strings "* | "uv version help strings "* | "script version help strings "*) ;;
        *)
            echo "FAIL - $name.sh does not follow the six-step order: '$order'"
            failures=$((failures + 1))
            continue
            ;;
    esac
    case "$order" in
        *behaviour*) ;;
        *)
            echo "FAIL - $name.sh never measures behaviour"
            failures=$((failures + 1))
            continue
            ;;
    esac
    echo "ok   - $name.sh follows the six-step order"
done

if [ "$failures" -ne 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all probe-harness checks passed"
```

- [ ] **Step 7: Edit `sandbox/run.sh`** — Without this the argument cannot reach a probe at all: run.sh accepts exactly one. Replace exactly this text, which occurs once:

```bash
        [ $# -eq 1 ] || usage
```

with:

```bash
        # An optional second argument pins the agent version, so a measurement can be repeated.
        [ $# -eq 1 ] || [ $# -eq 2 ] || usage
        version=${2:-}
```

- [ ] **Step 8: Edit `sandbox/run.sh`** — Unquoted on purpose: an empty version must produce no argument, not an empty one. Replace exactly this text, which occurs once:

```bash
    probe) script="$copy && sh sandbox/probes/$agent.sh" ;;
```

with:

```bash
    probe) script="$copy && sh sandbox/probes/$agent.sh $version" ;;
```

- [ ] **Step 9: Edit `.gitattributes`** — Without this every transcript fails verification, and the only message is "not byte-identical": the file is written in a Linux container and committed from a workstation whose working tree is CRLF. Replace exactly this text, which occurs once:

```
sandbox/Containerfile text eol=lf
```

with:

```
sandbox/Containerfile text eol=lf
# Evidence transcripts are compared byte for byte against the CI artifact that produced
# them, so they must never be translated on checkout or commit (SP4 design §5.3).
docs/evidence/** -text
```

- [ ] **Step 10: Edit `sandbox/run.sh`** — The one finding shellcheck reports on the existing harness, and it is a false positive: the function is reached through a trap. Silenced where it happens, with the reason, rather than by lowering the severity the whole gate runs at. Replace exactly this text, which occurs once:

```bash
cleanup() {
```

with:

```bash
# shellcheck disable=SC2329 # invoked by the EXIT/INT/TERM/HUP traps below, which shellcheck does
# not follow. Removing it as "unused" would leave every run's container and image behind.
cleanup() {
```

- [ ] **Step 11: Edit `.github/workflows/ci.yml`** — The same check CI runs, so a maintainer whose local gate is green is not told otherwise by a pull request. Replace exactly this text, which occurs once:

```yaml
  clippy:
    name: Clippy
```

with:

```yaml
  shellcheck:
    name: Shellcheck
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      # Pre-installed on ubuntu-latest, so there is nothing to pin and nothing to download.
      - run: find sandbox -name '*.sh' -exec shellcheck -s sh {} +

  clippy:
    name: Clippy
```

- [ ] **Step 12: Edit `justfile`** — The suite needs no container and no agent, so it belongs in the ordinary gate; and the shell files are now load-bearing enough to lint, since they decide what an evidence transcript says. Replace exactly this text, which occurs once:

```
# The local gate: fmt + clippy + typos + test
check: fmt-check clippy typos test
```

with:

```
# Every shell file in the repository. SP4a took this from three files to seventeen, and the harness is
# now load-bearing: it decides what an evidence transcript says. `-s sh` because these run under the
# container's /bin/sh, not bash, and `find` rather than a glob because an unmatched glob passes through
# literally and would hand shellcheck a filename that does not exist.
shellcheck:
    find sandbox -name '*.sh' -exec shellcheck -s sh {} +

# The sandbox harness's own tests: probe steps, transcript assembly, matrix resolution.
# No container and no agent needed, so these run in the ordinary gate rather than in the Sandbox workflow.
probe-tests:
    sh sandbox/tests/probe-harness.sh

# The local gate: fmt + clippy + typos + shellcheck + test + the probe harness
check: fmt-check clippy typos shellcheck test probe-tests
```

- [ ] **Step 13: Run this task's own checks**

```bash
sh sandbox/tests/probe-harness.sh
```

Expected: passes, including `a probe whose only step failed exits non-zero`, `a coloured banner does not become the version`, `a target populated before the launch is not in the delta`.

- [ ] **Step 14: Run the gate** (see "Gate commands"). Expected on Windows: `243 tests run: 243 passed`. Linux and macOS run fewer tests, because the Windows-only tests are compiled out; every test must pass.

- [ ] **Step 15: Commit**

```bash
git add sandbox/probes/text.sh sandbox/probes/common.sh sandbox/probes/claude.sh sandbox/probes/codex.sh sandbox/probes/aider.sh sandbox/tests/probe-harness.sh sandbox/run.sh .gitattributes .github/workflows/ci.yml justfile
git commit -m "fix: make a failed probe exit non-zero, and fix the step order

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 16: Prove this task's tests are not vacuous** (rule 6). The design calls this the highest error-cost class in the document, and it is the one defect here no gate can catch. A "delta" that is really the after-state lists the files the PROBE created — it makes the target directory itself, and for a file mechanism writes the candidate — which reads as the agent having moved its configuration. The result is a false `ConfigIsolation: Supported` carrying a `measured:` basis, and every gate passes.

In `sandbox/probes/common.sh` replace exactly:

```bash
        comm -13 "$PROBE_OUT/.delta-before" "$PROBE_OUT/.delta-after" | sed 's/^/+ /'
```

with:

```bash
        sed 's/^/+ /' "$PROBE_OUT/.delta-after"
```

Run `sh sandbox/tests/probe-harness.sh`. Expected: it FAILS, and a line reporting FAIL names `a target populated before the launch is not in the delta`. Then restore with `git checkout -- sandbox/probes/common.sh` and confirm `git status --short` prints nothing.


### Task 4: The nine new probe scripts

Add a probe script for each of the nine agents, from install commands verified against the package registry.

**Files:**

- Create: `sandbox/probes/gemini.sh` — Settles §8.1: GEMINI_CLI_HOME is documented as naming a directory CONTAINING .gemini, so the delta should show .gemini/ created inside the target.
- Create: `sandbox/probes/copilot.sh` — Three token variables with a documented precedence, so all three defeat isolation and D11 makes the adapter declare each.
- Create: `sandbox/probes/opencode.sh` — Two default locations and two mechanisms: §8.2's whole point is that auth.json lives where OPENCODE_CONFIG_DIR does not reach.
- Create: `sandbox/probes/cline.sh` — Three mechanism repetitions, one per question §8.3 names.
- Create: `sandbox/probes/pi.sh` — The least well cited of the twelve; step 6's delta is what settles its default.
- Create: `sandbox/probes/kiro.sh` — Vendor script, executable kiro-cli, and §8.5's expectation that KIRO_HOME is ignored by some subsystems — which shows as a delta at BOTH locations.
- Create: `sandbox/probes/cursor.sh` — Vendor script, executable cursor-agent.
- Create: `sandbox/probes/continue.sh` — Executable cn. The --config flag name is NOT established; step 3 is its oracle.
- Create: `sandbox/probes/amp.sh` — @ampcode/cli, not the @sourcegraph/amp rename stub.

**Before:** `sandbox/probes/gemini.sh` does not exist; `sandbox/probes/common.sh` contains `probe_behaviour()`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `sandbox/probes/gemini.sh`** — Settles §8.1: GEMINI_CLI_HOME is documented as naming a directory CONTAINING .gemini, so the delta should show .gemini/ created inside the target. Write the whole file, byte-exact.

```bash
# Gemini CLI probe: `sandbox/run.sh probe gemini [version]`.
#
# Package @google/gemini-cli (registry-verified 2026-09-16; `bin` is `gemini`), default `$HOME/.gemini`
# (https://geminicli.com/docs/reference/configuration/), mechanism GEMINI_CLI_HOME
# (https://geminicli.com/docs/cli/enterprise/).
#
# 8.1 is the question this probe settles: GEMINI_CLI_HOME is documented as naming a directory CONTAINING
# `.gemini`, not the configuration directory itself. If that holds, the delta shows `.gemini/` created
# INSIDE the target -- which is why the snapshot recurses, and why the adapter must not declare the nested
# path as one of its own.
. sandbox/probes/common.sh

probe_npm_install @google/gemini-cli
probe_version gemini
probe_help gemini
probe_strings gemini GEMINI_API_KEY GOOGLE_API_KEY GOOGLE_APPLICATION_CREDENTIALS GEMINI_CLI_HOME
probe_behaviour gemini "$HOME/.gemini" env:GEMINI_CLI_HOME @none
```

- [ ] **Step 2: Write `sandbox/probes/copilot.sh`** — Three token variables with a documented precedence, so all three defeat isolation and D11 makes the adapter declare each. Write the whole file, byte-exact.

```bash
# GitHub Copilot CLI probe: `sandbox/run.sh probe copilot [version]`.
#
# Package @github/copilot (registry-verified 2026-09-16; `bin` is `copilot`), default `$HOME/.copilot`
# and mechanism COPILOT_HOME, both from
# https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference
#
# The three token variables have a documented precedence -- COPILOT_GITHUB_TOKEN, then GH_TOKEN, then
# GITHUB_TOKEN -- so all three defeat isolation and D11 makes the adapter declare each.
. sandbox/probes/common.sh

probe_npm_install @github/copilot
probe_version copilot
probe_help copilot
probe_strings copilot COPILOT_GITHUB_TOKEN GH_TOKEN GITHUB_TOKEN COPILOT_HOME
probe_behaviour copilot "$HOME/.copilot" env:COPILOT_HOME @none
```

- [ ] **Step 3: Write `sandbox/probes/opencode.sh`** — Two default locations and two mechanisms: §8.2's whole point is that auth.json lives where OPENCODE_CONFIG_DIR does not reach. Write the whole file, byte-exact.

```bash
# OpenCode probe: `sandbox/run.sh probe opencode [version]`.
#
# Package opencode-ai (registry-verified 2026-09-16; `bin` is `opencode`). The vendor documents a curl
# script first and npm second (https://opencode.ai/docs); npm is used here because it is the one that can
# honour a pinned version, which 7.4 needs to repeat a measurement.
#
# TWO default locations, and that is 8.2's whole point: config lives at `$HOME/.config/opencode` while
# `auth.json` lives at `$HOME/.local/share/opencode` (https://opencode.ai/docs/providers/), which
# OPENCODE_CONFIG_DIR does not move. Watching only the first would record "credentials did not appear in
# the target" -- an absence of evidence -- where watching both records WHERE they went, which 6 says is
# enough to claim NotSupported honestly rather than Unknown.
#
# Two mechanisms, because 7.3's Q2 asks which of them the adapter should use: OPENCODE_CONFIG_DIR names
# a directory and OPENCODE_CONFIG names a single file (https://opencode.ai/docs/cli/).
. sandbox/probes/common.sh

default="$HOME/.config/opencode $HOME/.local/share/opencode"

probe_npm_install opencode-ai
probe_version opencode
probe_help opencode
probe_strings opencode ANTHROPIC_API_KEY OPENAI_API_KEY OPENCODE_CONFIG_DIR OPENCODE_CONFIG XDG_DATA_HOME
probe_behaviour opencode "$default" env:OPENCODE_CONFIG_DIR @none
PROBE_CONFIG_NAME=opencode.json
probe_behaviour opencode "$default" envfile:OPENCODE_CONFIG "{}"
```

- [ ] **Step 4: Write `sandbox/probes/cline.sh`** — Three mechanism repetitions, one per question §8.3 names. Write the whole file, byte-exact.

```bash
# Cline CLI probe: `sandbox/run.sh probe cline [version]`.
#
# Package cline (registry-verified 2026-09-16: repository github.com/cline/cline, `bin` is `cline` -- the
# name is generic enough to be worth checking rather than assuming). Default `$HOME/.cline`
# (https://docs.cline.bot/cli/cli-reference).
#
# 8.3 says the mechanism is undecided BY DESIGN and names the three questions this probe answers, in
# order: does CLINE_DATA_DIR isolate state; does it enable sandbox mode; does --config work alongside it.
# Hence three mechanism repetitions rather than one. `--config` and `--data-dir` both name DIRECTORIES
# here, not files, which is why they are `flagdir:` -- the docs describe --config as a "Configuration
# directory" and --data-dir as "isolated local state at this directory path".
#
# Two first-party sources disagree about --config's default by one path segment (docs.cline.bot says
# ~/.cline/data/settings, the repository README implies ~/.cline/data). The probe does not have to resolve
# that: it watches `$HOME/.cline`, which contains both.
. sandbox/probes/common.sh

probe_npm_install cline
probe_version cline
probe_help cline
probe_strings cline ANTHROPIC_API_KEY CLINE_API_KEY OPENAI_API_KEY OPENROUTER_API_KEY CLINE_DATA_DIR
probe_behaviour cline "$HOME/.cline" env:CLINE_DATA_DIR @none
probe_behaviour cline "$HOME/.cline" flagdir:--data-dir @none
probe_behaviour cline "$HOME/.cline" flagdir:--config @none
```

- [ ] **Step 5: Write `sandbox/probes/pi.sh`** — The least well cited of the twelve; step 6's delta is what settles its default. Write the whole file, byte-exact.

```bash
# Pi probe: `sandbox/run.sh probe pi [version]`.
#
# Package @earendil-works/pi-coding-agent (registry-verified 2026-09-16: repository
# github.com/earendil-works/pi, `bin` is `pi`). This is the pi.dev coding agent, not one of the several
# unrelated tools named `pi`; the repository field is what distinguishes them.
#
# `--ignore-scripts` is part of the vendor's own documented npm command (https://pi.dev/), so it is passed
# rather than dropped -- an install run differently from the way the vendor documents it measures
# something the vendor does not ship.
#
# Default `$HOME/.pi/agent` and mechanism PI_CODING_AGENT_DIR are the LEAST well cited of the twelve: the
# path comes from the providers documentation's resolution order rather than a sentence stating the
# default. Step 6's delta is what settles it, and if the delta is empty the adapter is an outcome-2.
. sandbox/probes/common.sh

probe_npm_install @earendil-works/pi-coding-agent --ignore-scripts
probe_version pi
probe_help pi
probe_strings pi ANTHROPIC_API_KEY OPENAI_API_KEY PI_CODING_AGENT_DIR
probe_behaviour pi "$HOME/.pi" env:PI_CODING_AGENT_DIR @none
```

- [ ] **Step 6: Write `sandbox/probes/kiro.sh`** — Vendor script, executable kiro-cli, and §8.5's expectation that KIRO_HOME is ignored by some subsystems — which shows as a delta at BOTH locations. Write the whole file, byte-exact.

```bash
# Kiro CLI probe: `sandbox/run.sh probe kiro`.
#
# Kiro CLI is the rebrand of Amazon Q Developer CLI and the executable is `kiro-cli`, not `kiro` -- the id
# names the file, the executable names the thing launched (7.3).
#
# Installed by vendor script (https://cli.kiro.dev/install, documented at https://kiro.dev/cli/). There is
# no npm package, so this probe takes NO version argument and refuses one rather than recording a version
# the installer never saw. The `.deb` 8's table also lists is documented for the Kiro IDE, not for
# kiro-cli, so it is not used here.
#
# 8.5 is what this probe tests: KIRO_HOME is documented, but an open upstream report describes
# subsystems ignoring it and using ~/.kiro regardless, failing silently. That shape shows up as a delta at
# BOTH locations -- files under the target AND new files under $HOME/.kiro -- which is exactly what
# `NotGuaranteed` exists to describe. A probe that watched only the target would have called it isolated.
. sandbox/probes/common.sh

probe_script_install https://cli.kiro.dev/install bash
probe_version kiro-cli
probe_help kiro-cli
probe_strings kiro-cli KIRO_API_KEY KIRO_HOME AWS_ACCESS_KEY_ID AWS_PROFILE
probe_behaviour kiro-cli "$HOME/.kiro" env:KIRO_HOME @none
```

- [ ] **Step 7: Write `sandbox/probes/cursor.sh`** — Vendor script, executable cursor-agent. Write the whole file, byte-exact.

```bash
# Cursor Agent CLI probe: `sandbox/run.sh probe cursor`.
#
# The executable is `cursor-agent`, not `cursor`. Installed by vendor script
# (https://cursor.com/install, documented at https://cursor.com/docs/cli/installation); there is no npm
# package, so this probe refuses a version argument.
#
# The installer puts its binary in `$HOME/.local/bin`, which `sandbox/Containerfile:23` already has on
# PATH -- so a "command not found" at step 2 means the install failed, not that the probe looked in the
# wrong place.
#
# Default `$HOME/.cursor` and mechanism CURSOR_CONFIG_DIR, both from
# https://cursor.com/docs/cli/reference/configuration. The docs also describe an XDG fallback
# (`$XDG_CONFIG_HOME/cursor`), so the container delta is the record of which of the two it actually used.
. sandbox/probes/common.sh

probe_script_install https://cursor.com/install bash
probe_version cursor-agent
probe_help cursor-agent
probe_strings cursor-agent CURSOR_API_KEY CURSOR_CONFIG_DIR XDG_CONFIG_HOME
probe_behaviour cursor-agent "$HOME/.cursor" env:CURSOR_CONFIG_DIR @none
```

- [ ] **Step 8: Write `sandbox/probes/continue.sh`** — Executable cn. The --config flag name is NOT established; step 3 is its oracle. Write the whole file, byte-exact.

```bash
# Continue CLI probe: `sandbox/run.sh probe continue [version]`.
#
# Package @continuedev/cli (registry-verified 2026-09-16; `bin` is `cn`, which is why the executable and
# the id differ). The vendor documents a shell script first and npm second; npm is used because it can
# honour a pinned version. Default `$HOME/.continue`, holding `config.yaml`.
#
# THE FLAG NAME IS NOT ESTABLISHED. 8's table claims `--config <file>`, and a documentation pass on
# 2026-09-16 could not find a first-party page showing that flag for `cn`. Step 3's `--help` capture is
# the oracle: if the flag is spelled differently, the behaviour step records a non-zero exit and the
# transcript shows the real spelling one file away. That is a 9 outcome, not a probe failure -- and it is
# why SP4b plans the adapter only after the transcripts exist.
#
# The candidate sweep answers 8.4 for this agent: `agent-profile` must create the file before `cn` reads
# it, so the smallest content that sets no option is part of the mechanism.
. sandbox/probes/common.sh

PROBE_CONFIG_NAME=config.yaml

probe_npm_install @continuedev/cli
probe_version cn
probe_help cn
probe_strings cn CONTINUE_API_KEY ANTHROPIC_API_KEY OPENAI_API_KEY
probe_candidates cn flagfile:--config @none "" "# nothing" "{}"
probe_behaviour cn "$HOME/.continue" flagfile:--config "{}"
```

- [ ] **Step 9: Write `sandbox/probes/amp.sh`** — @ampcode/cli, not the @sourcegraph/amp rename stub. Write the whole file, byte-exact.

```bash
# Amp probe: `sandbox/run.sh probe amp [version]`.
#
# Package @ampcode/cli, NOT @sourcegraph/amp. Both resolve and carry the same version, which looks like a
# choice; it is not. @sourcegraph/amp is a rename stub whose only dependency is @ampcode/cli and whose own
# registry description reads "Renamed to @ampcode/cli" (registry-verified 2026-09-16). Installing the stub
# would work and would record the wrong package as the thing measured.
#
# Default `$HOME/.config/amp/settings.json` and mechanism `--settings-file <path>`, both from
# https://ampcode.com/docs/cli/settings
#
# The same page documents that workspace settings override user settings and managed settings override
# both. That is a layering the flag cannot defeat, so it belongs in the adapter's basis the way Aider's
# layering note does (`aider.rs:21`) -- the probe records `--help` and the container delta, and SP4b
# writes the claim.
. sandbox/probes/common.sh

PROBE_CONFIG_NAME=settings.json

probe_npm_install @ampcode/cli
probe_version amp
probe_help amp
probe_strings amp AMP_API_KEY AMP_URL AMP_SETTINGS_FILE
probe_candidates amp flagfile:--settings-file @none "" "{}"
probe_behaviour amp "$HOME/.config/amp" flagfile:--settings-file "{}"
```

- [ ] **Step 10: Run this task's own checks**

```bash
sh sandbox/tests/probe-harness.sh
```

Expected: passes, including `gemini.sh follows the six-step order`, `amp.sh follows the six-step order`.

- [ ] **Step 11: Run the gate** (see "Gate commands"). Expected on Windows: `243 tests run: 243 passed`. Linux and macOS run fewer tests, because the Windows-only tests are compiled out; every test must pass.

- [ ] **Step 12: Commit**

```bash
git add sandbox/probes/gemini.sh sandbox/probes/copilot.sh sandbox/probes/opencode.sh sandbox/probes/cline.sh sandbox/probes/pi.sh sandbox/probes/kiro.sh sandbox/probes/cursor.sh sandbox/probes/continue.sh sandbox/probes/amp.sh
git commit -m "feat: add probe scripts for the nine new agents

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 13: Prove this task's tests are not vacuous** (rule 6). Breaking the step order does not fail — it produces a plausible, WRONG measurement, because a baseline taken after a launch attributes the launch's own files to the install.

In `sandbox/probes/gemini.sh` replace exactly:

```bash
probe_version gemini
probe_help gemini
```

with:

```bash
probe_help gemini
probe_version gemini
```

Run `sh sandbox/tests/probe-harness.sh`. Expected: it FAILS, and a line reporting FAIL names `gemini.sh does not follow the six-step order`. Then restore with `git checkout -- sandbox/probes/gemini.sh` and confirm `git status --short` prints nothing.


### Task 5: The transcript assembler

Turn one probe run's output into the finished evidence transcript, written whole by the machine so a human never edits one.

**Files:**

- Create: `sandbox/transcript.sh` — Turns one probe run into the finished docs/evidence/<id>-<version>.md. A workflow step rather than a probe step because run.sh passes the container no --env, so GITHUB_RUN_ID is not visible where the measurements are taken.
- Create: `sandbox/tests/transcript.sh` — Its tests, including the refusals.
- Modify: `justfile` — Each suite joins the gate with the code it covers.

**Before:** `sandbox/transcript.sh` does not exist; `sandbox/probes/text.sh` contains `probe_excerpt()`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `sandbox/transcript.sh`** — Turns one probe run into the finished docs/evidence/<id>-<version>.md. A workflow step rather than a probe step because run.sh passes the container no --env, so GITHUB_RUN_ID is not visible where the measurements are taken. Write the whole file, byte-exact.

```bash
#!/bin/sh
# Assembles one evidence transcript from a finished probe run.
#
# Usage: sandbox/transcript.sh <agent-id> <results-dir> [output-dir]
#          <results-dir>  target/sandbox/probe-<id>-<stamp>/, what sandbox/run.sh leaves behind
#          [output-dir]   docs/evidence by default
#
# It runs as a WORKFLOW STEP, not inside the container, and that is forced rather than chosen: the
# container gets no `--env` (`sandbox/run.sh:150`), so GITHUB_RUN_ID is not visible to a probe and the
# custody header cannot be written where the measurements are taken.
#
# It writes `<id>-<version>.md` whole, and the maintainer commits those bytes unaltered. §5.3's
# verification job then compares the committed file against the artifact byte for byte, which is what
# makes the custody header load-bearing instead of decorative — every field above the separator is bound
# by the same comparison as the measurements below it.

set -eu

usage() {
    sed -n '3,6p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

[ $# -ge 2 ] && [ $# -le 3 ] || usage
id=$1
results=$2
outdir=${3:-docs/evidence}

case "$id" in
    '' | *[!a-z0-9-]*) echo "transcript: agent ids are [a-z0-9-]" >&2; exit 2 ;;
esac
[ -d "$results" ] || { echo "transcript: no such results directory: $results" >&2; exit 2; }

root=$(cd "$(dirname "$0")/.." && pwd)
. "$root/sandbox/probes/text.sh"

# The version names the file, so §5.3 can resolve one transcript from the registry's `upstream_version`.
# A probe that recorded none wrote `unknown`, which Gate C keys on; a missing file means the probe did not
# reach step 2 at all, and `unknown` is the honest encoding of that too.
version=unknown
if [ -s "$results/version.extracted" ]; then
    version=$(head -n1 "$results/version.extracted")
fi
case "$version" in
    '' | *[!A-Za-z0-9._-]*)
        echo "transcript: recorded version '$version' is outside Gate A's charset" >&2
        exit 2
        ;;
esac

mkdir -p "$outdir"
out="$outdir/$id-$version.md"

# section <label> <file> <max-lines>: one labelled block, stripped and bounded, or an explicit absence.
#
# A missing artefact is PRINTED rather than skipped. A transcript with no `help:` line reads as an agent
# with no help text; one saying `(not recorded)` reads as a probe that did not get that far, and those are
# different findings.
section() {
    printf '%s:\n' "$1"
    if [ -s "$2" ]; then
        probe_excerpt "$3" < "$2" | sed 's/^/  /'
    else
        echo "  (not recorded)"
    fi
}

{
    # --- custody, bound by the byte comparison exactly as the measurements are -------------------
    echo "custody: ci"
    echo "run-id: ${GITHUB_RUN_ID:-unknown}"
    echo "run-url: ${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY:-unknown}/actions/runs/${GITHUB_RUN_ID:-unknown}"
    echo "harness-commit: ${GITHUB_SHA:-unknown}"
    echo "---"

    # --- what was run, and what it returned -----------------------------------------------------
    printf 'install:\n'
    if [ -s "$results/install.cmd" ]; then
        sed 's/^/  /' "$results/install.cmd"
    else
        echo "  (not recorded)"
    fi

    printf 'exit-codes:\n'
    if [ -s "$results/steps" ]; then
        # In step order, which is the order the probe ran them. A directory listing would sort
        # `behaviour` before `install` and read as though the agent was launched before it existed.
        sed 's/^/  /' "$results/steps"
    else
        echo "  (not recorded)"
    fi

    section version "$results/version.txt" 20
    echo "version-extracted: $version"
    section help "$results/help.txt" 200
    section strings "$results/strings.txt" 200

    # --- one baseline/delta pair per mechanism under test ---------------------------------------
    for f in "$results"/baseline-*.txt; do
        [ -e "$f" ] || break
        label=$(basename "$f" .txt)
        section "$label" "$f" 200
        section "delta-${label#baseline-}" "$results/delta-${label#baseline-}.txt" 200
    done

    # §8.4's acceptance sweep, present only for the configuration-file agents.
    if [ -s "$results/candidates.txt" ]; then
        section candidates "$results/candidates.txt" 40
    fi

    # --- the unscoped observation ---------------------------------------------------------------
    # The only record of a write to a location nobody predicted, which is the only way §9 outcome 4 —
    # the agent exposes a DIFFERENT mechanism from the claimed one — is ever detected. The `delta:`
    # sections above see only the locations chosen in advance.
    #
    # /home/probe/work is dropped because it is OUR copy of the checkout (`sandbox/run.sh:135`), some
    # thousands of paths the probe put there itself. Leaving it in would bound the summary away to
    # nothing and bury the handful of lines that matter.
    printf 'container-delta:\n'
    if [ -s "$results/diff.txt" ]; then
        grep -v ' /home/probe/work' "$results/diff.txt" \
            | probe_excerpt 300 \
            | sed 's/^/  /'
    else
        echo "  (not recorded)"
    fi
} > "$out"

echo "$out"
```

- [ ] **Step 2: Write `sandbox/tests/transcript.sh`** — Its tests, including the refusals. Write the whole file, byte-exact.

```bash
#!/bin/sh
# Tests for sandbox/transcript.sh. Run by `just check` and by CI; no container and no agent needed.
#
# The assembler writes the file §5.3 later compares byte for byte against the CI artifact, so a defect
# here is not a formatting problem: it is a durable false statement about what an agent did, committed to
# the repository and carrying a chain of custody that says it was measured.

set -eu
root=$(cd "$(dirname "$0")/../.." && pwd)
failures=0

check() {
    if [ "$2" = "$3" ]; then
        echo "ok   - $1"
    else
        echo "FAIL - $1: expected '$3', got '$2'"
        failures=$((failures + 1))
    fi
}

# A finished probe's results directory, as sandbox/run.sh leaves it.
fixture() {
    results=$(mktemp -d)
    outdir=$(mktemp -d)
    printf 'npm install --global @example/agent@1.2.3\n' > "$results/install.cmd"
    printf 'install 0\nversion 0\nhelp 0\nstrings 0\nbehaviour-env-EXAMPLE_HOME 0\n' > "$results/steps"
    printf 'agent 1.2.3\n' > "$results/version.txt"
    printf '1.2.3\n' > "$results/version.extracted"
    printf 'Usage: agent [--config <dir>]\n' > "$results/help.txt"
    printf 'documented EXAMPLE_API_KEY: present\n' > "$results/strings.txt"
    printf '# /target\n# /home/probe/.example\n' > "$results/baseline-env-EXAMPLE_HOME.txt"
    printf '# /target\nd .example\n' > "$results/delta-env-EXAMPLE_HOME.txt"
    printf 'A /home/probe/.example\nA /home/probe/work/src\nC /home/probe/.npm\n' > "$results/diff.txt"
}

run_assembler() {
    set +e
    written=$(cd "$root" && env GITHUB_RUN_ID=4242 GITHUB_SERVER_URL=https://github.com \
        GITHUB_REPOSITORY=ckir/aiprofiles GITHUB_SHA=0123456789abcdef0123456789abcdef01234567 \
        sh sandbox/transcript.sh "$1" "$results" "$outdir" 2>"$outdir/err")
    assembler_status=$?
    set -e
}

field() {
    sed -n "s/^$1: //p" "$written" | head -n1
}

# --- the custody header ---------------------------------------------------------------------------

fixture
run_assembler example
check "the assembler succeeds on a complete run" "$assembler_status" "0"
check "the file is named by id and recorded version" "$(basename "$written")" "example-1.2.3.md"
check "custody says ci" "$(field custody)" "ci"
check "the run id is carried" "$(field run-id)" "4242"
check "the run url is built from the run id" \
    "$(field run-url)" "https://github.com/ckir/aiprofiles/actions/runs/4242"
check "the harness commit is carried" \
    "$(field harness-commit)" "0123456789abcdef0123456789abcdef01234567"
check "the separator follows the header" "$(sed -n 5p "$written")" "---"

# --- the measurements -----------------------------------------------------------------------------

check "the install command is the resolved one" \
    "$(sed -n '/^install:/{n;p;}' "$written")" "  npm install --global @example/agent@1.2.3"
check "exit codes are in step order" \
    "$(sed -n '/^exit-codes:/{n;p;}' "$written")" "  install 0"
check "the extracted version is stated beside the raw output" \
    "$(field version-extracted)" "1.2.3"
check "each mechanism's baseline is emitted" \
    "$(grep -c '^baseline-env-EXAMPLE_HOME:$' "$written")" "1"
check "each mechanism's delta follows its baseline" \
    "$(grep -c '^delta-env-EXAMPLE_HOME:$' "$written")" "1"

# The copied checkout is thousands of paths the probe put there itself. Leaving it in would bound the
# summary away to nothing and bury the handful of lines that show a write nobody predicted.
check "the container delta keeps what the agent wrote" \
    "$(grep -c 'A /home/probe/.example$' "$written")" "1"
check "the container delta drops our own copy of the checkout" \
    "$(grep -c '/home/probe/work' "$written")" "0"

# §8.4's sweep is present only for the two configuration-file agents.
check "no candidates section when there was no sweep" "$(grep -c '^candidates:$' "$written")" "0"
fixture
printf '1: @none\n2: {}\n' > "$results/candidates.txt"
run_assembler example
check "a candidates section when there was one" "$(grep -c '^candidates:$' "$written")" "1"

# --- absence is a finding, not a gap ---------------------------------------------------------------

# A transcript with no `help:` line reads as an agent with no help text. One saying `(not recorded)` reads
# as a probe that did not get that far, and those are different findings.
fixture
rm "$results/help.txt"
run_assembler example
check "a missing artefact is stated, not skipped" \
    "$(sed -n '/^help:/{n;p;}' "$written")" "  (not recorded)"

# --- refusals -------------------------------------------------------------------------------------

# Gate A resolves `docs/evidence/<id>-<version>.md` from the registry's upstream_version. A version
# carrying a space or a slash would name a file the gate can never resolve, or escape the directory.
fixture
printf 'agent 1.2.3\n' > "$results/version.extracted"
run_assembler example
check "a version outside Gate A's charset is refused" "$assembler_status" "2"

fixture
printf '../escape\n' > "$results/version.extracted"
run_assembler example
check "a version that would escape the directory is refused" "$assembler_status" "2"

fixture
run_assembler 'Bad Id'
check "an id outside the id charset is refused" "$assembler_status" "2"

# A probe that never reached step 2 recorded nothing. `unknown` is the encoding Gate C keys on to force
# the adapter to Experimental, so the transcript states it rather than failing to exist.
fixture
rm "$results/version.extracted"
run_assembler example
check "a probe that recorded no version yields an unknown transcript" \
    "$(basename "$written")" "example-unknown.md"

if [ "$failures" -ne 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all transcript checks passed"
```

- [ ] **Step 3: Edit `justfile`** — Each suite joins the gate with the code it covers. Replace exactly this text, which occurs once:

```
    sh sandbox/tests/probe-harness.sh
```

with:

```
    sh sandbox/tests/probe-harness.sh
    sh sandbox/tests/transcript.sh
```

- [ ] **Step 4: Run this task's own checks**

```bash
sh sandbox/tests/transcript.sh
```

Expected: passes, including `the file is named by id and recorded version`, `a missing artefact is stated, not skipped`.

- [ ] **Step 5: Run the gate** (see "Gate commands"). Expected on Windows: `243 tests run: 243 passed`. Linux and macOS run fewer tests, because the Windows-only tests are compiled out; every test must pass.

- [ ] **Step 6: Commit**

```bash
git add sandbox/transcript.sh sandbox/tests/transcript.sh justfile
git commit -m "feat: assemble an evidence transcript from a probe run

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 7: Prove this task's tests are not vacuous** (rule 6). Keeping our own copy of the checkout bounds the summary away to nothing and buries the few lines that show a write to a location nobody predicted — the only way outcome 4 is detected.

In `sandbox/transcript.sh` replace exactly:

```bash
grep -v ' /home/probe/work' "$results/diff.txt" \
```

with:

```bash
cat "$results/diff.txt" \
```

Run `sh sandbox/tests/transcript.sh`. Expected: it FAILS, and a line reporting FAIL names `the container delta drops our own copy of the checkout`. Then restore with `git checkout -- sandbox/transcript.sh` and confirm `git status --short` prints nothing.


### Task 6: Matrix resolution, and the sandbox workflow as three jobs

Let one dispatch probe several agents as a bounded matrix, without losing the job that keeps the harness runnable.

**Files:**

- Create: `sandbox/resolve-agents.sh` — Dedupe, cap, charset and the is-this-a-probe-script check. A script, not a `run:` block, because every rule here fails QUIETLY when it is wrong and YAML cannot be tested.
- Create: `sandbox/tests/resolve-agents.sh` — Its tests.
- Modify (whole file): `.github/workflows/sandbox.yml` — test, prepare and probe. Naming only two is how the draft silently deleted the job that keeps the harness runnable.
- Modify: `justfile` — Each suite joins the gate with the code it covers.

**Before:** `sandbox/resolve-agents.sh` does not exist; `.github/workflows/sandbox.yml` contains `sandbox-results`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `sandbox/resolve-agents.sh`** — Dedupe, cap, charset and the is-this-a-probe-script check. A script, not a `run:` block, because every rule here fails QUIETLY when it is wrong and YAML cannot be tested. Write the whole file, byte-exact.

```bash
#!/bin/sh
# Resolves the Sandbox workflow's `agents` input into the JSON array its probe matrix runs over.
#
# Usage: sandbox/resolve-agents.sh <agents> [version]
#          <agents>   whitespace- or comma-separated probe names, or the word `all`
#          [version]  a version to pin; valid only when exactly one agent is named
#
# Prints the JSON array on stdout. Diagnostics go to stderr and a refusal exits non-zero.
#
# This is a script rather than a `run:` block because every rule below is a rule that can be got wrong
# quietly: a naive split-and-loop iterates zero times on empty input and reports success having probed
# nothing, and a duplicated name doubles the runner cost while colliding on the artifact name the matrix
# keeps unique — so the second upload silently overwrites the first. Shell embedded in YAML cannot be
# tested; sandbox/tests/resolve-agents.sh tests this.

set -eu

# The probe run needs all twelve at once, so the cap is not the registry's current size — that would make
# the thirteenth adapter a workflow edit.
CAP=24

usage() {
    sed -n '3,6p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

[ $# -ge 1 ] && [ $# -le 2 ] || usage
spec=$1
version=${2:-}

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

if [ "$spec" = all ]; then
    # `all` means every PROBE script, and a probe script is one that sources the harness. Listing
    # sandbox/probes/*.sh would offer `common` and `text` as agents; excluding those two by name would be
    # a list that rots the next time a library file is added. This is the actual property.
    raw=$(grep -l '^\. sandbox/probes/common\.sh$' sandbox/probes/*.sh \
        | sed 's|.*/||; s|\.sh$||' | LC_ALL=C sort)
else
    raw=$(printf '%s' "$spec" | tr ',' ' ' | tr -s ' \t' '\n')
fi

# Deduplicated preserving the order given, so the log reads back the way it was typed.
list=$(printf '%s\n' "$raw" | grep -v '^$' | awk '!seen[$0]++' || true)
count=$(printf '%s\n' "$list" | grep -c . || true)

if [ "$count" -eq 0 ]; then
    echo "resolve-agents: no agents named" >&2
    exit 1
fi
if [ "$count" -gt "$CAP" ]; then
    echo "resolve-agents: $count agents requested; the cap is $CAP" >&2
    exit 1
fi

for agent in $list; do
    case "$agent" in
        *[!a-z0-9-]*)
            echo "resolve-agents: agent name '$agent' is not [a-z0-9-]" >&2
            exit 1
            ;;
    esac
    # The SAME property `all` selects on, applied to a name given by hand. Checking only that the file
    # exists would accept `common` and `text`, which are the harness rather than agents — so the two
    # directions would disagree about what an agent is, and `all` would be the stricter of them.
    if ! grep -q '^\. sandbox/probes/common\.sh$' "sandbox/probes/$agent.sh" 2>/dev/null; then
        echo "resolve-agents: sandbox/probes/$agent.sh is not a probe script" >&2
        exit 1
    fi
done

# A pinned version belongs to ONE package. Applying it across a matrix would ask npm for @github/copilot
# at Gemini's version number, and the refusal would be recorded as a failed install rather than as the
# mistake it is.
if [ -n "$version" ] && [ "$count" -ne 1 ]; then
    echo "resolve-agents: a version pins one agent; $count were named" >&2
    exit 1
fi

# The same charset Gate A requires of `upstream_version`, applied at the other end of the pipe. A version
# reaches a probe's install command and then names an evidence file, so anything outside this set could
# not have produced a resolvable transcript anyway.
#
# It also closes a hole that has nothing to do with versions. The caller writes both this value and the
# agent list to `$GITHUB_OUTPUT`, which is a `key=value`-per-line file — so a version containing a NEWLINE
# writes a second `agents=` line that overrides the validated one, and the list reaching the matrix would
# never have passed a single check above. MEASURED: `version` of `1.0\nagents=["evil"]` yields exactly
# that. Validating here rather than at the call site keeps every rule about these two inputs in the one
# file that is tested.
case "$version" in
    '') ;;
    *[!A-Za-z0-9._-]*)
        echo "resolve-agents: version '$version' is not [A-Za-z0-9._-]" >&2
        exit 1
        ;;
esac

# No escaping, and none needed: every name has been checked against [a-z0-9-], which contains no character
# JSON gives a meaning to. That check is what makes this safe, so it must stay above this line.
printf '%s\n' "$list" | awk 'BEGIN { printf "[" } { printf "%s\"%s\"", (NR > 1 ? "," : ""), $0 } END { print "]" }'
```

- [ ] **Step 2: Write `sandbox/tests/resolve-agents.sh`** — Its tests. Write the whole file, byte-exact.

```bash
#!/bin/sh
# Tests for sandbox/resolve-agents.sh, the Sandbox workflow's matrix resolution.
#
# Every rule it enforces is one that fails QUIETLY when it is missing: an empty input makes a
# split-and-loop iterate zero times and report success having probed nothing, and a duplicated name
# doubles the runner cost while colliding on the artifact name the matrix keeps unique, so the second
# upload overwrites the first. None of that turns a job red on its own.

set -eu
root=$(cd "$(dirname "$0")/../.." && pwd)
failures=0

check() {
    if [ "$2" = "$3" ]; then
        echo "ok   - $1"
    else
        echo "FAIL - $1: expected '$3', got '$2'"
        failures=$((failures + 1))
    fi
}

resolve() {
    set +e
    out=$(sh "$root/sandbox/resolve-agents.sh" "$@" 2>/dev/null)
    status=$?
    set -e
}

resolve claude
check "one agent resolves to a one-element array" "$out" '["claude"]'
check "one agent succeeds" "$status" "0"

resolve "claude codex aider"
check "whitespace separates names" "$out" '["claude","codex","aider"]'

resolve "claude,codex"
check "commas separate names" "$out" '["claude","codex"]'

resolve "claude, codex,  aider"
check "commas and spaces mix" "$out" '["claude","codex","aider"]'

resolve "codex claude codex"
check "a repeated name appears once, in the order given" "$out" '["codex","claude"]'

resolve all
check "all resolves to every probe script" "$out" \
    '["aider","amp","claude","cline","codex","continue","copilot","cursor","gemini","kiro","opencode","pi"]'

# The library files live in the same directory. Naming them by hand and excluding them by hand would be a
# list that rots; `all` is defined by the property of sourcing the harness instead.
check "all excludes the harness itself" "$(printf '%s' "$out" | grep -c common)" "0"
check "all excludes the text helpers" "$(printf '%s' "$out" | grep -c text)" "0"

resolve ""
check "an empty input is refused" "$status" "1"

resolve "   "
check "whitespace alone is refused" "$status" "1"

resolve "nosuchagent"
check "an unknown agent is refused" "$status" "1"

resolve "common"
check "a library file is not an agent" "$status" "1"

resolve "Claude"
check "a name outside the charset is refused" "$status" "1"

resolve "../../etc/passwd"
check "a path is refused" "$status" "1"

resolve "a b c d e f g h i j k l m n o p q r s t u v w x y"
check "more than the cap is refused" "$status" "1"

resolve claude 2.1.270
check "a version with one agent is accepted" "$status" "0"

# A pinned version belongs to one package. Applied across a matrix it would ask npm for one agent at
# another's version number, and the refusal would be recorded as a failed install.
resolve "claude codex" 2.1.270
check "a version with several agents is refused" "$status" "1"

# The agent list is validated one name at a time, and then BOTH values are written to $GITHUB_OUTPUT,
# which is a key=value-per-line file. A version containing a newline writes a second `agents=` line that
# overrides the validated one, so the list reaching the matrix would have passed no check at all.
resolve claude 'x
agents=["evil"]'
check "a version carrying a newline is refused" "$status" "1"

resolve claude 'v1.0 --flag'
check "a version outside Gate A's charset is refused" "$status" "1"

resolve claude 1.2.3-beta.1
check "a version using every permitted character is accepted" "$status" "0"

if [ "$failures" -ne 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all resolve-agents checks passed"
```

- [ ] **Step 3: Write `.github/workflows/sandbox.yml`** — test, prepare and probe. Naming only two is how the draft silently deleted the job that keeps the harness runnable. Write the whole file, byte-exact.

```yaml
name: Sandbox

# Runs sandbox/run.sh on a fresh GitHub-hosted Linux runner: probes by hand (Actions → Sandbox → Run workflow).
# The runner VM is discarded after the job, so real coding agents installed by a probe never touch a maintainer's
# machine, and pull-request CI never runs them.
#
# No secrets are used, and none may be added: a probe runs third-party installers, which is precisely the job
# that must never hold a credential. `permissions: contents: read` is the whole grant.
#
# A pull request that changes the harness runs it in `test` mode only (the crate's suite, no agent), so the
# harness itself stays working. That job is not optional scaffolding — it is the only CI that keeps the probes
# runnable, and splitting the workflow into a matrix is exactly the change that could quietly drop it.

on:
  pull_request:
    paths:
      - sandbox/**
      - .github/workflows/sandbox.yml
  workflow_dispatch:
    inputs:
      mode:
        description: What to run
        type: choice
        options: [test, probe]
        default: probe
      agents:
        description: Agents to probe — names from sandbox/probes/, whitespace- or comma-separated, or `all`
        type: string
        default: claude
      version:
        description: Version to pin (one agent only; vendor install scripts refuse it)
        type: string
        default: ''

permissions:
  contents: read

# One wave at a time. A re-dispatch must not put a second set of installer jobs alongside a set already in
# flight. NOT cancel-in-progress: a probe killed half way leaves partial measurements, and the assembler
# would write a transcript stating them as though the run had finished.
concurrency:
  group: sandbox-${{ github.ref }}
  cancel-in-progress: false

jobs:
  test:
    name: Harness test
    if: github.event_name == 'pull_request' || inputs.mode == 'test'
    runs-on: ubuntu-latest
    timeout-minutes: 60
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false

      - name: Run the crate suite in the sandbox
        env:
          AGENT_PROFILE_SANDBOX_ENGINE: podman
        run: sandbox/run.sh test

      - name: Upload results
        if: always()
        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1
        with:
          name: sandbox-test-results
          path: target/sandbox/
          retention-days: 14
          if-no-files-found: warn

  prepare:
    name: Resolve the agent list
    # The event gate is not optional. On `pull_request` there are no `workflow_dispatch` inputs, so
    # `inputs.agents` is empty and an ungated job would fail every pull request that touches sandbox/** —
    # reddening the very CI that exists to keep the harness working.
    if: github.event_name == 'workflow_dispatch' && inputs.mode == 'probe'
    runs-on: ubuntu-latest
    timeout-minutes: 5
    outputs:
      agents: ${{ steps.resolve.outputs.agents }}
      version: ${{ steps.resolve.outputs.version }}
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false

      - name: Resolve, validate and cap
        id: resolve
        env:
          AGENTS: ${{ inputs.agents }}
          VERSION: ${{ inputs.version }}
        # The rules live in sandbox/resolve-agents.sh, not here. Every one of them — dedupe, the cap, the
        # charset, the "is this actually a probe script" check — fails quietly when it is wrong, and shell
        # embedded in YAML cannot be tested. sandbox/tests/resolve-agents.sh tests it.
        run: |
          set -eu
          agents=$(sandbox/resolve-agents.sh "$AGENTS" "$VERSION")
          echo "resolved: $agents"
          {
            printf 'agents=%s\n' "$agents"
            printf 'version=%s\n' "$VERSION"
          } >> "$GITHUB_OUTPUT"

  probe:
    name: Probe ${{ matrix.agent }}
    needs: prepare
    runs-on: ubuntu-latest
    timeout-minutes: 60
    strategy:
      # One agent's failure or hang must not consume another's budget: an agent that cannot be installed
      # is itself a finding (§9 outcome 2), and cancelling its eleven siblings would discard eleven
      # measurements to record one.
      fail-fast: false
      matrix:
        agent: ${{ fromJSON(needs.prepare.outputs.agents) }}
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false

      - name: Probe
        env:
          AGENT_PROFILE_SANDBOX_ENGINE: podman
          AGENT: ${{ matrix.agent }}
          VERSION: ${{ needs.prepare.outputs.version }}
        # A failed probe is a RESULT, not a broken job: §9's outcomes 2 and 3 are both recorded by a
        # non-zero exit, and the transcript that states them is written by the next step. The job's own
        # status is set at the end, after the evidence has been assembled and uploaded.
        continue-on-error: true
        run: sandbox/run.sh probe "$AGENT" $VERSION

      - name: Assemble the transcript
        if: always()
        env:
          AGENT: ${{ matrix.agent }}
        run: |
          set -eu
          results=$(ls -d target/sandbox/probe-"$AGENT"-* 2>/dev/null | tail -n1 || true)
          if [ -z "$results" ]; then
            echo "::error::the probe left no results directory; there is nothing to assemble"
            exit 1
          fi
          mkdir -p transcript
          sandbox/transcript.sh "$AGENT" "$results" transcript

      # The transcript is what §5.3 compares byte for byte against the committed file, so it is uploaded
      # alone. The bounded container-delta summary is INSIDE it, not beside it: that summary is the only
      # record of a write to a location nobody predicted, and routing it into the expiring logs artifact
      # would leave the broadest evidence the least protected.
      - name: Upload the transcript
        if: always()
        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1
        with:
          name: sandbox-transcript-${{ matrix.agent }}
          path: transcript/
          # Long enough that committing a transcript inside its artifact's lifetime is routine rather than
          # a race. §5.3's verification depends on that window still being open.
          retention-days: 90
          if-no-files-found: error

      # Megabytes of toolchain-download noise, kept out of the file a human reviews.
      - name: Upload the logs
        if: always()
        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1
        with:
          name: sandbox-logs-${{ matrix.agent }}
          path: |
            target/sandbox/probe-${{ matrix.agent }}-*/build.log
            target/sandbox/probe-${{ matrix.agent }}-*/output.log
            target/sandbox/probe-${{ matrix.agent }}-*/diff.txt
          retention-days: 14
          if-no-files-found: warn

      # The probe's status is applied HERE rather than by the probe step, so that a failed probe still
      # uploads its evidence — and still goes red. Both halves matter. D13's whole point is that a probe
      # which failed must exit non-zero, because otherwise the run's green light carries no information;
      # but a red job that discarded its transcript would lose the very measurement that explains why.
      - name: Apply the probe's own status
        if: always()
        env:
          AGENT: ${{ matrix.agent }}
        run: |
          set -eu
          results=$(ls -d target/sandbox/probe-"$AGENT"-* 2>/dev/null | tail -n1 || true)
          if [ -z "$results" ] || [ ! -f "$results/exit-code" ]; then
            echo "::error::$AGENT left no exit code; the run cannot be called a measurement"
            exit 1
          fi
          code=$(cat "$results/exit-code")
          echo "$AGENT exit code: $code"
          if [ "$code" -ne 0 ]; then
            echo "::error::$AGENT probed with exit code $code; its transcript records which steps failed"
          fi
          exit "$code"
```

- [ ] **Step 4: Edit `justfile`** — Each suite joins the gate with the code it covers. Replace exactly this text, which occurs once:

```
    sh sandbox/tests/transcript.sh
```

with:

```
    sh sandbox/tests/transcript.sh
    sh sandbox/tests/resolve-agents.sh
```

- [ ] **Step 5: Run this task's own checks**

```bash
sh sandbox/tests/resolve-agents.sh
```

Expected: passes, including `a repeated name appears once, in the order given`, `a version carrying a newline is refused`.

- [ ] **Step 6: Run the gate** (see "Gate commands"). Expected on Windows: `243 tests run: 243 passed`. Linux and macOS run fewer tests, because the Windows-only tests are compiled out; every test must pass.

- [ ] **Step 7: Commit**

```bash
git add sandbox/resolve-agents.sh sandbox/tests/resolve-agents.sh .github/workflows/sandbox.yml justfile
git commit -m "feat: probe several agents in one bounded matrix dispatch

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 8: Prove this task's tests are not vacuous** (rule 6). A duplicate doubles the runner cost while colliding on the artifact name the matrix keeps unique, so the second upload silently overwrites the first.

In `sandbox/resolve-agents.sh` replace exactly:

```bash
list=$(printf '%s\n' "$raw" | grep -v '^$' | awk '!seen[$0]++' || true)
```

with:

```bash
list=$(printf '%s\n' "$raw" | grep -v '^$' || true)
```

Run `sh sandbox/tests/resolve-agents.sh`. Expected: it FAILS, and a line reporting FAIL names `a repeated name appears once, in the order given`. Then restore with `git checkout -- sandbox/resolve-agents.sh` and confirm `git status --short` prints nothing.


### Task 7: Bind each committed transcript to the run that produced it

Prove a committed transcript is the bytes a specific CI job produced, from a harness commit the pull request contains.

**Files:**

- Create: `sandbox/verify-transcripts.sh` — §5.3, and the highest-stakes file in SP4a.
- Create: `sandbox/tests/verify-transcripts.sh` — One check per forgery the spec names, with a faked API client and a real git repository for the ancestry check.
- Create: `crates/agent-profile/examples/evidence-manifest.rs` — The id and version come from the REGISTRY, never the filename: upstream_version permits `-`, so cursor-1.0.0-beta.1.md splits two ways.
- Modify (whole file): `.github/workflows/ci.yml` — The Evidence job. pull_request never pull_request_target; actions: read and nothing else; separate from the probe job, which keeps contents: read.
- Modify: `justfile` — Each suite joins the gate with the code it covers.

**Before:** `sandbox/verify-transcripts.sh` does not exist; `.github/workflows/ci.yml` does not contain `evidence:`. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `sandbox/verify-transcripts.sh`** — §5.3, and the highest-stakes file in SP4a. Write the whole file, byte-exact.

```bash
#!/bin/sh
# Binds every evidence transcript changed by a pull request to the CI run that produced it (SP4 §5.3).
#
# Usage: sandbox/verify-transcripts.sh <manifest> <head-sha> [changed-file...]
#          <manifest>       one `<id> <upstream_version>` line per real adapter, from the REGISTRY —
#                           `cargo run --example evidence-manifest`
#          <head-sha>       the pull request's head commit
#          [changed-file]   paths the pull request changes; only docs/evidence/** are considered
#
# Every gate in §5 checks a file's SHAPE. None checks that it came from anywhere, and a convincing
# transcript — real run URL copied from the public Actions tab, plausible --help excerpt, a column of zero
# exit codes — can be written by hand in minutes and passes all of them. This is the job that makes a
# transcript evidence rather than prose.
#
# `GH` is injectable so the rules below can be tested without the network; sandbox/tests/verify-transcripts.sh
# substitutes a fake. The rules are the point, and shell embedded in YAML cannot be tested.

set -eu

GH=${GH:-gh}
REPO=${GITHUB_REPOSITORY:-}

usage() {
    sed -n '3,7p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

[ $# -ge 2 ] || usage
manifest=$1
head_sha=$2
shift 2

[ -f "$manifest" ] || { echo "verify: no manifest at $manifest" >&2; exit 2; }
[ -n "$REPO" ] || { echo "verify: GITHUB_REPOSITORY is not set" >&2; exit 2; }

failures=0
fail() {
    echo "verify: FAIL $*" >&2
    failures=$((failures + 1))
}

# The changed evidence files, so a transcript nobody touched is never re-verified. That is deliberate and
# §5.3 says why: artifacts expire, and a check that cannot fail is not a check. An existing verified
# transcript stays verified; introducing or modifying one after its artifact has expired is refused.
changed=$(mktemp)
for path in "$@"; do
    case "$path" in
        docs/evidence/*.md) printf '%s\n' "$path" >> "$changed" ;;
    esac
done

if [ ! -s "$changed" ]; then
    echo "verify: no evidence transcript changed"
    rm -f "$changed"
    exit 0
fi

# field <label> <file>: the value of a `label: value` line, from the header only.
field() {
    sed -n "s/^$1: //p" "$2" | head -n1
}

verified=$(mktemp)

while read -r id version; do
    [ -n "$id" ] || continue
    file="docs/evidence/$id-$version.md"
    grep -qxF "$file" "$changed" || continue
    printf '%s\n' "$file" >> "$verified"

    # A deletion is legitimate: §7.4's retention rule replaces a superseded transcript and deletes an
    # `unknown` one once the agent installs. There is nothing to compare.
    if [ ! -f "$file" ]; then
        echo "verify: $file was removed"
        continue
    fi

    custody=$(field custody "$file")
    if [ "$custody" = off-ci ]; then
        # §7.4.1: an authenticated measurement cannot run in CI, so it carries a weaker custody block and
        # is reviewed by hand. Gate A is what stops this becoming a one-word opt-out — it requires
        # `custody: ci` for any transcript backing a ConfigIsolation or StateIsolation claim.
        echo "verify: $file is off-ci; reviewed by hand"
        continue
    fi
    if [ "$custody" != ci ]; then
        fail "$file: custody is '$custody', expected ci or off-ci"
        continue
    fi

    run_id=$(field run-id "$file")
    case "$run_id" in
        '' | *[!0-9]*)
            fail "$file: run-id '$run_id' is not a run id"
            continue
            ;;
    esac
    harness=$(field harness-commit "$file")
    case "$harness" in
        *[!0-9a-f]* | '')
            fail "$file: harness-commit '$harness' is not a commit"
            continue
            ;;
    esac
    [ "${#harness}" -eq 40 ] || { fail "$file: harness-commit is not 40 hex"; continue; }

    # Without this, a person with push access dispatches Sandbox on a throwaway branch carrying an edited
    # probe script that prints a fabricated --help. The run is genuinely green, the artifact is genuine,
    # the bytes match exactly, and the forging branch never appears in the pull request.
    #
    # Ancestry of the PR HEAD, not of the base branch. The probe is dispatched on the feature branch, so
    # its head SHA is a commit on that branch, and a commit on an open pull request's head is never an
    # ancestor of the base — the base-branch form is unsatisfiable by construction. The PR-head form says
    # what is actually meant: the harness that produced this evidence is in the history you are reviewing.
    if ! git merge-base --is-ancestor "$harness" "$head_sha" 2>/dev/null; then
        fail "$file: harness-commit $harness is not an ancestor of the pull request head"
        continue
    fi

    # §5.3 requires the run id to reach the API without passing through `${{ }}` interpolation, because it
    # is read out of a pull-request-supplied file and expression interpolation would splice it into the
    # shell the runner generates before this script ever ran. That rule binds the WORKFLOW, and the
    # workflow satisfies it by interpolating nothing: this script opens the file itself. Here the id is an
    # ordinary shell variable, already constrained to [0-9]+ above, passed as one argument.
    if ! run=$($GH api "repos/$REPO/actions/runs/$run_id" 2>/dev/null); then
        fail "$file: run $run_id could not be read from $REPO"
        continue
    fi

    got_repo=$(printf '%s' "$run" | jq -r '.repository.full_name // ""')
    got_path=$(printf '%s' "$run" | jq -r '.path // ""')
    got_sha=$(printf '%s' "$run" | jq -r '.head_sha // ""')

    [ "$got_repo" = "$REPO" ] || fail "$file: run $run_id belongs to $got_repo, not $REPO"
    [ "$got_path" = ".github/workflows/sandbox.yml" ] \
        || fail "$file: run $run_id is $got_path, not the Sandbox workflow"
    [ "$got_sha" = "$harness" ] \
        || fail "$file: run $run_id ran at $got_sha, but the transcript records $harness"

    # The MATRIX JOB's conclusion, not the run's. Twelve matrix jobs share one run id, and §9 outcome 2 is
    # a DESIGNED outcome in which a probe legitimately fails — so a run-level check would let one
    # uninstallable agent invalidate eleven good transcripts.
    if ! jobs=$($GH api "repos/$REPO/actions/runs/$run_id/jobs" --paginate 2>/dev/null); then
        fail "$file: the jobs of run $run_id could not be read"
        continue
    fi
    conclusion=$(printf '%s' "$jobs" | jq -r --arg name "Probe $id" \
        '[.jobs[]? | select(.name == $name)] | first | .conclusion // ""')

    # An outcome-2 transcript is verified against a job that CONCLUDED FAILURE, which is the whole point
    # of outcome 2: D13 makes a failed probe exit non-zero, so requiring success here would reject the one
    # transcript §9 requires to exist and Gate A requires to be present.
    if [ "$version" = unknown ]; then
        expected=failure
    else
        expected=success
    fi
    if [ "$conclusion" != "$expected" ]; then
        fail "$file: job 'Probe $id' concluded '$conclusion', expected '$expected'"
        continue
    fi

    # Expiry FAILS CLOSED. Degrading to "the run exists and was green" would check nothing that is a
    # function of the agent, the version, or the bytes — so the original run of <id>-2.1.0.md would
    # satisfy it for a forged <id>-3.0.0.md, restoring both forgeries this whole section exists to stop.
    dir=$(mktemp -d)
    if ! $GH run download "$run_id" --name "sandbox-transcript-$id" --dir "$dir" >/dev/null 2>&1; then
        fail "$file: artifact sandbox-transcript-$id is unavailable (expired?); a transcript cannot be introduced or modified after its artifact has gone"
        rm -rf "$dir"
        continue
    fi

    downloaded=$(find "$dir" -type f -name '*.md' | head -n1)
    if [ -z "$downloaded" ]; then
        fail "$file: artifact sandbox-transcript-$id holds no transcript"
        rm -rf "$dir"
        continue
    fi
    if cmp -s "$file" "$downloaded"; then
        echo "verify: ok $file"
    else
        # The machine assembles, the human reviews, the job compares. A mismatch means somebody edited a
        # transcript; an excerpt that needs changing means the PROBE needs changing and the probe re-runs.
        fail "$file: not byte-identical to the artifact from run $run_id"
    fi
    rm -rf "$dir"
done < "$manifest"

# A changed evidence file that no registry row resolves to. Without this, adding a transcript for an agent
# the registry does not carry is simply never checked by anything here.
while read -r path; do
    grep -qxF "$path" "$verified" && continue
    case "$path" in
        docs/evidence/README.md) continue ;;
    esac
    if [ -f "$path" ] && [ "$(field custody "$path")" = off-ci ]; then
        echo "verify: $path is off-ci; reviewed by hand"
        continue
    fi
    fail "$path: no adapter in the registry resolves to this transcript"
done < "$changed"

rm -f "$changed" "$verified"

if [ "$failures" -ne 0 ]; then
    echo "verify: $failures transcript check(s) failed" >&2
    exit 1
fi
echo "verify: all changed transcripts are bound to their runs"
```

- [ ] **Step 2: Write `sandbox/tests/verify-transcripts.sh`** — One check per forgery the spec names, with a faked API client and a real git repository for the ancestry check. Write the whole file, byte-exact.

```bash
#!/bin/sh
# Tests for sandbox/verify-transcripts.sh, the control that makes a committed transcript evidence.
#
# Every check below corresponds to a forgery §5.3 names. They matter more than most tests in this
# repository because the thing being defended is not a behaviour but a CLAIM: a hand-written transcript
# with a real run URL, a plausible --help excerpt and a column of zero exit codes passes every SHAPE gate
# in §5, and reads in a diff as a textbook evidence refresh.

set -eu
root=$(cd "$(dirname "$0")/../.." && pwd)
failures=0

check() {
    if [ "$2" = "$3" ]; then
        echo "ok   - $1"
    else
        echo "FAIL - $1: expected '$3', got '$2'"
        failures=$((failures + 1))
    fi
}

# A repository with a transcript, a manifest, and a faked API whose answers the test controls.
fixture() {
    work=$(mktemp -d)
    gh_dir="$work/gh"
    mkdir -p "$work/repo/docs/evidence" "$gh_dir/artifact"

    cat > "$work/gh/gh" <<'FAKE'
#!/bin/sh
# Stands in for the GitHub CLI. It answers from files the test wrote, so every rule can be exercised
# without the network and an expired artifact is just a missing file.
set -eu
case "$1 $2" in
    "api repos"*) ;;
esac
case "$*" in
    *"/jobs"*) cat "$FAKE_GH_DIR/jobs.json" ;;
    "api repos/"*"/actions/runs/"*) cat "$FAKE_GH_DIR/run.json" ;;
    "run download"*)
        [ -f "$FAKE_GH_DIR/artifact.md" ] || exit 1
        dir=$(printf '%s\n' "$@" | sed -n '/^--dir$/{n;p;}')
        cp "$FAKE_GH_DIR/artifact.md" "$dir/transcript.md"
        ;;
    *) exit 1 ;;
esac
FAKE
    chmod +x "$work/gh/gh"

    (
        cd "$work/repo"
        git init -q .
        git config user.email t@example.invalid
        git config user.name test
        # The fixture only needs commits to hang an ancestry check on. Without this, a maintainer running
        # the suite on Windows gets a CRLF warning per file per fixture, which buries the check output.
        git config core.autocrlf false
        echo harness > harness.txt
        git add -A
        git commit -qm harness
        harness_sha=$(git rev-parse HEAD)
        echo more > more.txt
        git add -A
        git commit -qm head
        git rev-parse HEAD > "$work/head"
        printf '%s' "$harness_sha" > "$work/harness"
    )
    harness_sha=$(cat "$work/harness")
    head_sha=$(cat "$work/head")

    printf 'example 1.2.3\n' > "$work/manifest"

    cat > "$work/repo/docs/evidence/example-1.2.3.md" <<TRANSCRIPT
custody: ci
run-id: 4242
run-url: https://github.com/ckir/aiprofiles/actions/runs/4242
harness-commit: $harness_sha
---
install:
  npm install --global @example/agent@1.2.3
TRANSCRIPT
    cp "$work/repo/docs/evidence/example-1.2.3.md" "$gh_dir/artifact.md"

    cat > "$gh_dir/run.json" <<RUN
{"repository":{"full_name":"ckir/aiprofiles"},"path":".github/workflows/sandbox.yml","head_sha":"$harness_sha"}
RUN
    printf '{"jobs":[{"name":"Probe example","conclusion":"success"}]}\n' > "$gh_dir/jobs.json"
}

verify() {
    set +e
    out=$(cd "$work/repo" && env \
        GITHUB_REPOSITORY=ckir/aiprofiles \
        FAKE_GH_DIR="$gh_dir" \
        GH="$work/gh/gh" \
        sh "$root/sandbox/verify-transcripts.sh" "$work/manifest" "$head_sha" \
        "${1:-docs/evidence/example-1.2.3.md}" 2>&1)
    status=$?
    set -e
}

# --- the happy path -------------------------------------------------------------------------------

fixture
verify
check "a transcript matching its artifact verifies" "$status" "0"
check "and says so" "$(printf '%s' "$out" | grep -c 'ok docs/evidence/example-1.2.3.md')" "1"

# --- the forgeries §5.3 names ---------------------------------------------------------------------

# The plainest one: somebody edited the committed file. The machine assembles, the human reviews, the job
# compares; an excerpt that needs changing means the PROBE needs changing.
fixture
printf 'edited by hand\n' >> "$work/repo/docs/evidence/example-1.2.3.md"
verify
check "an edited transcript is refused" "$status" "1"
check "and the reason is the bytes" "$(printf '%s' "$out" | grep -c 'not byte-identical')" "1"

# Renaming <id>-2.1.0.md to <id>-3.0.0.md and editing one version string reads as an evidence refresh.
# The manifest is what stops it: the version comes from the REGISTRY, so the forged name resolves to
# nothing the registry claims.
fixture
mv "$work/repo/docs/evidence/example-1.2.3.md" "$work/repo/docs/evidence/example-3.0.0.md"
verify docs/evidence/example-3.0.0.md
check "a transcript no registry row resolves to is refused" "$status" "1"
check "and the reason names the registry" \
    "$(printf '%s' "$out" | grep -c 'no adapter in the registry')" "1"

# The throwaway-branch forgery: a genuine green run, a genuine artifact, matching bytes -- produced by an
# edited probe script on a branch that never appears in the pull request.
fixture
orphan=$(cd "$work/repo" && git commit-tree -m orphan "$(git rev-parse 'HEAD^{tree}')" < /dev/null)
sed -i "s/^harness-commit: .*/harness-commit: $orphan/" "$work/repo/docs/evidence/example-1.2.3.md"
sed -i "s/\"head_sha\":\"[0-9a-f]*\"/\"head_sha\":\"$orphan\"/" "$gh_dir/run.json"
cp "$work/repo/docs/evidence/example-1.2.3.md" "$gh_dir/artifact.md"
verify
check "a harness commit outside the pull request's history is refused" "$status" "1"
check "and the reason is ancestry" "$(printf '%s' "$out" | grep -c 'not an ancestor')" "1"

# --- the API assertions ---------------------------------------------------------------------------

fixture
sed -i 's|"full_name":"ckir/aiprofiles"|"full_name":"someone/else"|' "$gh_dir/run.json"
verify
check "a run from another repository is refused" "$status" "1"

fixture
sed -i 's|.github/workflows/sandbox.yml|.github/workflows/ci.yml|' "$gh_dir/run.json"
verify
check "a run of a different workflow is refused" "$status" "1"

fixture
sed -i 's/"head_sha":"[0-9a-f]*"/"head_sha":"0000000000000000000000000000000000000000"/' "$gh_dir/run.json"
verify
check "a run whose head differs from the recorded harness commit is refused" "$status" "1"

fixture
sed -i 's/^run-id: 4242/run-id: 42x/' "$work/repo/docs/evidence/example-1.2.3.md"
verify
check "a run id that is not a run id is refused" "$status" "1"

# Twelve matrix jobs share one run id, so the RUN's conclusion says nothing about this agent.
fixture
printf '{"jobs":[{"name":"Probe other","conclusion":"success"},{"name":"Probe example","conclusion":"failure"}]}\n' \
    > "$gh_dir/jobs.json"
verify
check "a matrix job that failed is refused for a successful probe" "$status" "1"
check "and the reason names the job" "$(printf '%s' "$out" | grep -c "job 'Probe example'")" "1"

# §9 outcome 2 is a DESIGNED outcome: D13 makes a failed probe exit non-zero, so requiring success would
# reject the one transcript §9 requires to exist and Gate A requires to be present.
fixture
printf 'example unknown\n' > "$work/manifest"
mv "$work/repo/docs/evidence/example-1.2.3.md" "$work/repo/docs/evidence/example-unknown.md"
cp "$work/repo/docs/evidence/example-unknown.md" "$gh_dir/artifact.md"
printf '{"jobs":[{"name":"Probe example","conclusion":"failure"}]}\n' > "$gh_dir/jobs.json"
verify docs/evidence/example-unknown.md
check "an outcome-2 transcript verifies against a job that failed" "$status" "0"

fixture
printf 'example unknown\n' > "$work/manifest"
mv "$work/repo/docs/evidence/example-1.2.3.md" "$work/repo/docs/evidence/example-unknown.md"
cp "$work/repo/docs/evidence/example-unknown.md" "$gh_dir/artifact.md"
verify docs/evidence/example-unknown.md
check "an outcome-2 transcript from a job that SUCCEEDED is refused" "$status" "1"

# --- expiry fails closed --------------------------------------------------------------------------

# Degrading to "the run exists and was green" would check nothing that is a function of the agent, the
# version or the bytes, so the original run of <id>-1.2.3.md would satisfy it for a forged <id>-3.0.0.md.
fixture
rm "$gh_dir/artifact.md"
verify
check "an expired artifact refuses a changed transcript" "$status" "1"
check "and the reason says so" "$(printf '%s' "$out" | grep -c 'expired')" "1"

# --- the exemptions -------------------------------------------------------------------------------

fixture
sed -i 's/^custody: ci/custody: off-ci/' "$work/repo/docs/evidence/example-1.2.3.md"
verify
check "an off-ci transcript is exempt and reviewed by hand" "$status" "0"

fixture
verify docs/superpowers/specs/whatever.md
check "a pull request touching no transcript verifies trivially" "$status" "0"

fixture
rm "$work/repo/docs/evidence/example-1.2.3.md"
verify
check "a removed transcript is allowed by the retention rule" "$status" "0"

if [ "$failures" -ne 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all verify-transcripts checks passed"
```

- [ ] **Step 3: Write `crates/agent-profile/examples/evidence-manifest.rs`** — The id and version come from the REGISTRY, never the filename: upstream_version permits `-`, so cursor-1.0.0-beta.1.md splits two ways. Write the whole file, byte-exact.

```rust
//! Prints one `<id> <upstream_version>` line per real adapter, in registry order.
//!
//! This exists for §5.3 step 1, which requires the verification job to take the agent id and the version
//! **from the registry, never by parsing the filename**. The filename-to-id map is not invertible:
//! `upstream_version` permits `-` (§5), so `cursor-1.0.0-beta.1.md` splits two ways, and the wrong split
//! silently verifies the wrong transcript.
//!
//! It is an example rather than a binary because it is a build-time tool, not a shipped surface: examples
//! are covered by `cargo clippy --all-targets` but never land in a release archive.
//!
//! `REAL_ADAPTERS` rather than `registry()`: the `fake` adapter exists only under debug assertions
//! (`adapter/mod.rs:119-121`) and has no evidence to verify.

use agent_profile::adapter::REAL_ADAPTERS;

fn main() {
    for adapter in REAL_ADAPTERS {
        let metadata = adapter.metadata();
        println!("{} {}", metadata.id, metadata.evidence.upstream_version);
    }
}
```

- [ ] **Step 4: Write `.github/workflows/ci.yml`** — The Evidence job. pull_request never pull_request_target; actions: read and nothing else; separate from the probe job, which keeps contents: read. Write the whole file, byte-exact.

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  # Run from the Actions tab or `gh workflow run ci.yml --ref <branch>`, on any branch.
  workflow_dispatch:

# Least privilege, workflow-wide. Every job here runs code the pull request supplies -- `cargo nextest
# run --workspace` executes its tests, `cargo doc` its build scripts -- so the token those jobs hold is
# bounded here rather than left to the repository's default Actions permission, which is a setting outside
# this file and can be widened without anyone reading this workflow. `sandbox.yml` already pins it; this
# workflow did not. A job that needs more grants it for itself.
permissions:
  contents: read

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
      - uses: crate-ci/typos@512fc24f32f44ab01972217aaaf3dc86ec234d53 # v1.50.2

  shellcheck:
    name: Shellcheck
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      # Pre-installed on ubuntu-latest, so there is nothing to pin and nothing to download.
      - run: find sandbox -name '*.sh' -exec shellcheck -s sh {} +

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

  evidence:
    # Binds every evidence transcript this pull request changes to the CI run that produced it (§5.3).
    #
    # Every gate in the contract suite checks a transcript's SHAPE. None checks that it came from
    # anywhere, and a convincing one — real run URL from the public Actions tab, plausible --help excerpt,
    # a column of zero exit codes — can be written by hand in minutes and passes all of them.
    #
    # It lives here rather than in sandbox.yml because that workflow filters on `paths: sandbox/**`
    # (.github/workflows/sandbox.yml:13-15) and would never fire on an evidence-only pull request; ci.yml
    # has no paths filter.
    #
    # `pull_request`, NEVER `pull_request_target`. The reflex when a fork's token cannot reach the API is
    # to switch, which would run a base-repository token against pull-request-supplied bytes. Fork
    # coverage is a separate problem and is not solved that way.
    name: Evidence
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-latest
    # `actions: read` and nothing else, and this is a SEPARATE job from the probe — which keeps
    # `contents: read`, because the probe executes third-party installers and must stay the least
    # privileged thing in the repository.
    permissions:
      contents: read
      actions: read
    steps:
      - uses: actions/checkout@v7
        with:
          # The ancestry check needs the harness commit and the base branch, neither of which a
          # depth-1 checkout of the merge commit has.
          fetch-depth: 0
          persist-credentials: false

      - name: Find the transcripts this pull request changes
        id: changed
        env:
          BASE: ${{ github.event.pull_request.base.sha }}
          HEAD: ${{ github.event.pull_request.head.sha }}
        run: |
          set -eu
          files=$(git diff --name-only "$BASE" "$HEAD" -- docs/evidence/ | tr '\n' ' ')
          echo "files=$files" >> "$GITHUB_OUTPUT"
          if [ -z "$files" ]; then
            echo "no evidence transcript changed"
          else
            echo "changed: $files"
          fi

      - uses: dtolnay/rust-toolchain@stable
        if: steps.changed.outputs.files != ''

      - uses: Swatinem/rust-cache@v2
        if: steps.changed.outputs.files != ''

      # Building the manifest compiles pull-request-supplied Rust: the example itself, and every build
      # script of every dependency its Cargo.toml names. That is unavoidable -- the id and version must
      # come from the REGISTRY as the pull request states it, never from the filename, because
      # upstream_version permits `-` and `cursor-1.0.0-beta.1.md` splits two ways. What IS avoidable is
      # holding an API token while doing it, so this step declares none.
      - name: Build the registry manifest
        if: steps.changed.outputs.files != ''
        run: cargo run --quiet --example evidence-manifest > "$RUNNER_TEMP/manifest"

      - name: Verify each against its run
        if: steps.changed.outputs.files != ''
        env:
          GH_TOKEN: ${{ github.token }}
          HEAD: ${{ github.event.pull_request.head.sha }}
          FILES: ${{ steps.changed.outputs.files }}
        # Nothing from a transcript is interpolated into this shell. The script opens the files itself,
        # which is what keeps a pull-request-supplied run id out of the runner's generated script.
        run: |
          set -eu
          # shellcheck disable=SC2086 # $FILES is a space-separated list of paths, and the split is wanted
          sandbox/verify-transcripts.sh "$RUNNER_TEMP/manifest" "$HEAD" $FILES
```

- [ ] **Step 5: Edit `justfile`** — Each suite joins the gate with the code it covers. Replace exactly this text, which occurs once:

```
    sh sandbox/tests/resolve-agents.sh
```

with:

```
    sh sandbox/tests/resolve-agents.sh
    sh sandbox/tests/verify-transcripts.sh
```

- [ ] **Step 6: Run this task's own checks**

```bash
sh sandbox/tests/verify-transcripts.sh
```

Expected: passes, including `an edited transcript is refused`, `a harness commit outside the pull request's history is refused`.

- [ ] **Step 7: Run the gate** (see "Gate commands"). Expected on Windows: `243 tests run: 243 passed`. Linux and macOS run fewer tests, because the Windows-only tests are compiled out; every test must pass.

- [ ] **Step 8: Commit**

```bash
git add sandbox/verify-transcripts.sh sandbox/tests/verify-transcripts.sh crates/agent-profile/examples/evidence-manifest.rs .github/workflows/ci.yml justfile
git commit -m "feat: bind each committed transcript to the run that produced it

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 9: Prove this task's tests are not vacuous** (rule 6). Without the commit binding, someone with push access dispatches the workflow on a throwaway branch carrying an edited probe script: the run is genuinely green, the artifact genuine, the bytes identical, and the forging branch never appears in the pull request.

In `sandbox/verify-transcripts.sh` replace exactly:

```bash
if ! git merge-base --is-ancestor "$harness" "$head_sha" 2>/dev/null; then
```

with:

```bash
if false; then
```

Run `sh sandbox/tests/verify-transcripts.sh`. Expected: it FAILS, and a line reporting FAIL names `a harness commit outside the pull request's history is refused`. Then restore with `git checkout -- sandbox/verify-transcripts.sh` and confirm `git status --short` prints nothing.


### Task 8: Documentation, and the ruling that ProfilePresence::Known is reserved

Say what a transcript is for, how to add an adapter, and why ProfilePresence::Known stays unconstructed.

**Files:**

- Create: `docs/evidence/README.md` — What a reviewer reads before deciding to trust a transcript, including what one cannot tell them.
- Modify (whole file): `crates/agent-profile/src/adapter/metadata.rs` — The Known variant says at its declaration why it is unconstructed.
- Modify (whole file): `README.md` — §6.1's "read the whole row" sentence and §12's compounding Windows limit.
- Modify (whole file): `CONTRIBUTING.md` — Adding an adapter, and three places SP4a made stale.
- Modify (whole file): `ROADMAP.md` — The Known question is closed rather than left to be re-litigated.
- Modify (whole file): `TODO.md` — §6.1's three binding consequences, which the spec said were recorded and were not.

**Before:** `crates/agent-profile/src/adapter/metadata.rs` does not contain `Reserved, not dead`; `docs/evidence/README.md` does not exist. Every "Edit" anchor below occurs exactly once.

- [ ] **Step 0: State check** (rule 1).

- [ ] **Step 1: Write `docs/evidence/README.md`** — What a reviewer reads before deciding to trust a transcript, including what one cannot tell them. Write the whole file, byte-exact.

````markdown
# Evidence transcripts

Every capability an adapter claims is backed by a file in this directory. A transcript is the record of
one probe run: what was installed, what version it reported, what its help text said, and — the part that
matters — what moved on disk when the agent's own isolation mechanism was applied.

The claims live in the adapter's `AdapterEvidence`; the measurement lives here. A `basis` beginning
`measured:` means a file in this directory shows it.

## Why these files exist at all

An adapter could simply assert that `FOO_HOME` isolates configuration. Nothing in the code would
contradict it, the tests would pass, and a user would have no way to tell a measured claim from a
plausible one. Isolation is the product's whole promise, so a claim about it has to be checkable by
someone who does not trust us.

That is also why the format is deliberately dull. A transcript is not a report written about a
measurement; it is the measurement's own output, bounded and stripped of terminal control sequences, with
a header saying where it came from.

## Naming

```
<id>-<version>.md              the CI transcript for that adapter at that upstream version
<id>-<version>-credentials.md  an off-CI transcript for an authenticated measurement
```

`<version>` is the adapter's `evidence.upstream_version`, and the two must agree: the contract suite
resolves this path from the registry, so a mismatch fails the build rather than going unnoticed.

A probe that could not install or run the agent records `unknown` as the version. That is not a gap — it
is a finding, and the gates force the adapter to `Experimental` because of it.

## The two custody shapes

The first line says which, and it is the only thing that distinguishes them at a glance.

**`custody: ci`** — produced by the Sandbox workflow. It carries the run id, the run URL and the commit
of the harness that produced it. Everything an adapter claims about configuration or state isolation
rests on one of these.

**`custody: off-ci`** — produced in a disposable sandbox on a maintainer's machine, carrying the sandbox
and its version, the host platform, the date, and who performed it. It exists for one reason: proving
that stored credentials separate requires logging in to each vendor, and an authenticated measurement
cannot run in CI. It is weaker evidence, it says so on its first line, and it can never be the only
backing for a configuration or state claim.

## How a CI transcript is verified

The `Evidence` job in `ci.yml` runs on every pull request that changes a file here. For each one it:

1. takes the agent id and version **from the registry**, never from the filename — `upstream_version`
   permits `-`, so `cursor-1.0.0-beta.1.md` splits two ways and the wrong split would verify the wrong
   file;
2. reads `run-id:` and `harness-commit:` out of the committed file;
3. checks the run belongs to this repository, is the Sandbox workflow, ran at that harness commit, and
   that the commit is an ancestor of the pull request's head;
4. checks the matrix job for *this agent* concluded as expected — `success`, or `failure` for an
   `unknown` transcript, because a probe that legitimately could not install is exactly what outcome 2
   records;
5. downloads that run's `sandbox-transcript-<id>` artifact and requires the committed file to be
   **byte-identical** to it.

Step 3 is not ceremony. Without it, someone with push access could dispatch the workflow on a throwaway
branch carrying an edited probe script that prints whatever they liked: the run would be genuinely green,
the artifact genuine, the bytes identical — and the branch that forged it would never appear in the pull
request.

Step 5 is why **no one edits a transcript**. If an excerpt is wrong, the probe script is wrong; fix the
script and run it again.

## Producing one

1. Actions → Sandbox → Run workflow, mode `probe`, and name the agents (or `all`).
2. Download the `sandbox-transcript-<id>` artifact from that run.
3. Read it. This is the review step, and it is the only one a machine cannot do: the job proves the bytes
   came from that run, not that the probe captured the right thing.
4. Commit it unchanged, and update the adapter's `upstream_version` to match.

Do all of this while the artifact still exists. Verification **fails closed** on an expired artifact, so a
transcript cannot be introduced or modified once its artifact has gone — the run has to be repeated.
Artifacts are retained for 90 days for exactly this reason.

## Refreshing one

A re-measurement **replaces** the file it supersedes; it does not sit beside it. One `custody: ci`
transcript per adapter is live at a time — the one the current `upstream_version` resolves to — and the
gates fail if an orphan is left behind. Git history holds the old one, which is where superseded evidence
belongs.

The exception is an `off-ci` credentials transcript, which sits alongside the CI one rather than
replacing it: it answers a different question, and the CI transcript is still the backing for every other
claim.

An `unknown` transcript from a failed probe is **deleted** once the agent is probed successfully. Leaving
it would be a durable false statement that the agent cannot be installed.

## What a transcript cannot tell you

It proves where files landed. It does not prove nothing leaked: a delta showing the configuration
directory moved cannot show that no credential was read from a shared location. State isolation is
measured only for agents that write history before they need the network, and credential isolation is not
measured in CI at all.

Evidence is also a snapshot — one version, one day. A mechanism can change upstream without this
repository noticing, and nothing here detects that drift.
````

- [ ] **Step 2: Write `crates/agent-profile/src/adapter/metadata.rs`** — The Known variant says at its declaration why it is unconstructed. Write the whole file, byte-exact.

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
    /// Nothing has been created for this profile yet.
    Absent,
    /// Every path the adapter declares exists on disk.
    Materialized,
    /// The profile can be identified from the agent's own profile mechanism, without anything of ours
    /// existing on disk.
    ///
    /// **Reserved, not dead.** No adapter constructs this today and the SP3 review flagged it as unused
    /// code, which it is — but it is unclaimed rather than obsolete. Spec §8 defines it normatively for
    /// an agent that manages named profiles itself, where a profile is real because the agent says so
    /// and `agent-profile` may have created no directory at all. Every adapter shipped so far is
    /// directory-based, so the distinction has not yet had a case to express.
    ///
    /// Deleting it was considered in SP4 and rejected: the variant costs one line, while removing it
    /// would make the first native-profile adapter a change to a public enum — and in the meantime
    /// `presence()` would have to report such a profile as `Absent`, which is a false statement about a
    /// profile the agent itself lists.
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

- [ ] **Step 3: Write `README.md`** — §6.1's "read the whole row" sentence and §12's compounding Windows limit. Write the whole file, byte-exact.

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
backed by a transcript in [`docs/evidence/`](docs/evidence/) recording the measurement it rests on; a state
weaker than `Supported` means exactly what its reason says.

**Read the whole row, not the support column.** `Proven` is a statement about the mechanism, not about the
account: an adapter can honestly be `Proven` while its credential isolation is `Unknown`, because proving
that stored credentials separate requires logging in to each vendor and that cannot be measured in CI. The
capability states are never hidden behind a flag for this reason.

| Agent | Mechanism | Support | Config isolation | Credential isolation | State isolation |
|---|---|---|---|---|---|
| Claude Code 2.1.270 (`claude`) | `CLAUDE_CONFIG_DIR` | Proven | Supported (project `.claude/` settings still layer on top) | Conditional (`ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `CLAUDE_CODE_OAUTH_TOKEN` and similar variables bypass it) | NotGuaranteed (history and project state moving with the directory is community-sourced only) |
| Codex CLI 0.153.4 (`codex`) | `CODEX_HOME` | Proven | Supported (project-level configuration layering is not measured) | Conditional (`OPENAI_API_KEY`, `CODEX_API_KEY`, `CODEX_ACCESS_TOKEN` bypass it) | Conditional (`CODEX_SQLITE_HOME`) |
| Aider 0.86.2 (`aider`) | `--config <profile>/.aider.conf.yml` | Proven | NotGuaranteed (home, repository and working-directory `.aider.conf.yml`, `.env` and `AIDER_*` still apply) | NotSupported | NotSupported |

A new Codex profile starts logged out. Arguments that select the same mechanism (Aider's `-c`, `--config`
and its abbreviations) are refused before launch. On Windows an agent must be a native executable (`.exe`, or a configured `.com`): an npm or pnpm
`.cmd` shim is refused, and the error names the `[agents.<id>] executable` setting to use instead.

**On Windows, two limits compound for the npm-installed agents.** The executable is a shim, and a shim is
refused with a hint rather than parsed, so `executable` must be set in the configuration. Meanwhile the
probes run in a Linux container, so the evidence behind every claim above is Linux evidence. Neither fact
is hidden by the other: an agent can be both refused by default on Windows *and* unmeasured there.

## Planned adapters — not yet implemented or evidence-verified

The mechanisms below are the specification's starting point, not a claim about isolation. Each is cited
from first-party documentation and none is yet measured; where a probe contradicts a row, the probe wins.

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

- [ ] **Step 4: Write `CONTRIBUTING.md`** — Adding an adapter, and three places SP4a made stale. Write the whole file, byte-exact.

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
cargo binstall -y cargo-nextest just lefthook cargo-deny typos-cli bacon cargo-mutants
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
launched through `crates/agent-profile/tests/support`. Never launch a real coding agent from the test suite;
real agents run only in the disposable sandbox described under "Measuring agent behaviour".

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

**Linux (and anywhere Podman or Docker runs): the container harness.** `sandbox/run.sh` builds a disposable
image (`sandbox/Containerfile`: Rust, Node/npm, Python/uv, no agent), runs one workload in it, and removes the
container and the image afterwards, also after Ctrl-C or a closed terminal. With local rootless Podman (Linux)
each run also uses its own temporary image store under `$TMPDIR` or `/var/tmp`, deleted at the end, so no image,
layer or cache survives; only a `kill -9` of the script can leave a store behind (remove it with
`podman unshare rm -rf /var/tmp/agent-profile-sandbox.*`). With Docker, and with remote Podman such as Podman
Desktop on macOS, the pulled base image and the build cache (toolchains, never agents) stay in the engine's store;
remove them with `docker builder prune` and `docker image rm` on the pinned `debian@sha256:…` base from
`sandbox/Containerfile` (or the `podman` equivalents).
Your checkout is mounted read-only and copied inside.

```bash
just probe claude     # sandbox/run.sh probe claude: install the agent inside, record version, help and a dry run
just sandbox-test     # sandbox/run.sh test: cargo nextest run --workspace in a clean container
just sandbox-shell    # sandbox/run.sh --net shell: interactive shell with network, to write a new probe
```

Results land in `target/sandbox/<mode>[-<agent>]-<timestamp>/`: `build.log`, `exit-code`, `output.log` (not for
`shell`), `diff.txt` (files the run added, changed or deleted in the container) and the probe's own files. The
image is built for the host's architecture (x86_64 or aarch64).

A probe is a short script in `sandbox/probes/<agent>.sh` built on `sandbox/probes/common.sh`, and it runs six
steps **in this order**, because the order is what makes the result attributable:

```sh
. sandbox/probes/common.sh

probe_npm_install @example/agent          # 1. the only step that runs vendor code before a snapshot exists
probe_version example                     # 2. --version, verbatim, plus the token the registry will carry
probe_help example                        # 3. --help, where the mechanism token is read from
probe_strings example EXAMPLE_API_KEY     # 4. which credential variables are really in the binary
probe_behaviour example "$HOME/.example" env:EXAMPLE_HOME @none   # 5 and 6: baseline, then apply and diff
```

Step 5 is inside `probe_behaviour` and is not optional: it baselines **both** the profile target and the agent's
default location, after the install and after anything the probe itself created. Skip it and the install's own
files get attributed to the agent, which reads as isolation that did not happen. Repeat `probe_behaviour` once per
mechanism under test; each repetition re-baselines, because a second launch measured against the first launch's
baseline is not attributable.

A probe never supplies a credential, never waits for input, and never launches the agent through `agent-profile` —
the CLI refuses an agent word it does not know, so a probe written that way silently measures nothing.

`sandbox/tests/` holds the harness's own tests. They need no container and no agent, and `just check` runs them.

**No container engine? Use CI.** **Actions → Sandbox → Run workflow** runs the same script on a fresh GitHub
runner, whose virtual machine is discarded afterwards. Name one agent, several, or `all`; each gets its own job.
Two artifacts come back per agent: `sandbox-transcript-<id>`, holding the finished evidence transcript, and
`sandbox-logs-<id>`, holding the raw logs and container diff. A `test`-mode run uploads `sandbox-test-results`
instead. Probes run only when triggered by hand; a pull request that changes the harness runs it in `test` mode
only (the crate's test suite, no agent), so pull-request CI never installs or runs a real agent. **It uses no
secrets, and none may be added** — a probe executes third-party installers, which is precisely the job that must
never hold a credential.

**macOS:** the harness runs under Docker Desktop, OrbStack, Colima or Podman Desktop for behaviour that does not
depend on macOS, with the base image and build cache kept in that engine's store as described above; use Tart
disposable macOS virtual machines when the behaviour does depend on macOS (for example the Keychain).
`sandbox-exec` is deprecated, so it is not a substitute.

Record each measurement in the adapter's `AdapterEvidence` (`verified_at`, `upstream_version`, `source_url`,
`notes`), in the design document's decision table, and — for anything a probe measured — as a committed transcript
under `docs/evidence/`. See [docs/evidence/README.md](docs/evidence/README.md) for what a transcript must contain
and how CI binds it to the run that produced it.

## Adding an adapter

An adapter is a small file in `crates/agent-profile/src/adapter/`, but most of the work is the evidence behind it.
In order:

1. **Write the probe first.** `sandbox/probes/<id>.sh`, as above. Run it and read what comes back; the mechanism
   the documentation describes and the one the agent implements are not always the same, and it is cheaper to find
   that out now than after the adapter is written around the wrong one.
2. **Commit the transcript**, byte for byte, from the CI artifact. Never hand-edited.
3. **Write the adapter**, with one `CapabilityClaim` per capability. Every `basis` starts with `measured:`,
   `cited:` or `unmeasured:`, and `unmeasured:` is required exactly when the state is `Unknown` — the contract
   suite enforces the biconditional, so an `Unknown` with a confident-sounding basis will not build.
4. **Register it** in `REAL_ADAPTERS` and add its contract and end-to-end rows.
5. **Add its README row**, with the capability states beside the support level. A support level on its own
   overclaims: `proven` describes the mechanism, not the account, and the two are only safe to print together.

The gates in `src/adapter/gate.rs` are the mechanical part of this and they will tell you what is missing. What
they cannot tell you is whether a claim is honest, which is what steps 1 and 2 are for.

Claim the weakest state the evidence supports. `NotSupported` with a reason is a better adapter than `Supported`
with a hope, because a user can work around a stated limit and cannot work around a wrong promise.

## Commit messages

We follow [Conventional Commits](https://www.conventionalcommits.org/). Pull requests are squash-merged and
the repository's squash commit title is set to the PR title, so the **PR title** becomes the commit on `main`; a
required check (`Conventional PR title`) enforces its form, because release-plz chooses the next version and
writes the changelog from these commits:

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

Releases are automated with [release-plz](https://release-plz.dev) (`release-plz.toml`,
`.github/workflows/release-plz.yml`); nobody runs a release command locally.

1. Every push to `main` with a `feat`, `fix`, `perf` or `refactor` commit that changes files under
   `crates/agent-profile/` opens or updates a release PR titled `chore: release vX.Y.Z`, which bumps the
   workspace version and updates `crates/agent-profile/CHANGELOG.md`. Commits that touch only files outside
   the crate, including `Cargo.lock` and the root `Cargo.toml`, never open one; to ship such a change (for
   example a security bump of a dependency), follow it with a `fix:` PR that changes a file under
   `crates/agent-profile/`.
2. Review that PR like any other. Edits pushed to it survive only until the next push to `main`: release-plz
   then closes the PR and opens a fresh one without them. Merging it is the release.
3. The merge tags `vX.Y.Z`, creates a GitHub release, and builds and uploads the binaries for Linux (x86_64,
   aarch64), macOS (x86_64, aarch64) and Windows (x86_64).

Until V3 v0.1 is complete (SP5), versions stay `0.0.x` and every release is marked as a pre-release.

Recovery, all from the GitHub web UI:
- A binary is missing from a release: **Actions → Release binaries → Run workflow** with the release tag.
- The tag exists but the release does not (release creation failed after tagging; re-running the Release-plz
  run is a silent no-op then): create the pre-release for that tag under **Releases → Draft a new release**,
  publish it, then run **Release binaries** with the tag.

The release PR is opened with the `RELEASE_PLZ_TOKEN` repository secret: a fine-grained personal access token
for this repository with Contents and Pull requests read/write, so CI runs on the release PR. It expires; when
release PRs stop appearing, renew it.

## Licence

By contributing, you agree that your contributions will be licensed under the
[PolyForm Noncommercial License 1.0.0](LICENSE).
````

- [ ] **Step 5: Write `ROADMAP.md`** — The Known question is closed rather than left to be re-litigated. Write the whole file, byte-exact.

```markdown
# Agent Profile Roadmap

v0.1 scope is fixed by the V3 specification. The build order follows spec §35, split into sub-projects.
Each sub-project has its own design spec and implementation plan under `docs/superpowers/`, and ends
green on Linux, macOS and Windows.

| SP | Deliverable | V3 §35 phases | State |
|---|---|---|---|
| SP0 | Scaffold — workspace, tooling, licence, community docs, CI, `fake-agent` fixture | — | done (#1) |
| SP1 | Core + explicit launch — profile-name validation, application root, TOML configuration with locked atomic writes, structured errors, `LaunchPlan`, Unix `exec` / Windows child launcher, passthrough, environment overrides, dry run, exit codes | 1, 3 | done (#8) |
| SP2 | Architecture gate — adapter model, capability and evidence metadata, common contract suite, Claude Code, Codex CLI, Aider | 4A | done (#11) |
| SP3 | Repository resolution — discovery, canonical identity, mapping storage, precedence, `resolve`, `current`, `status`, `link`, `unlink` | 2 + part of 5 | done (#19) |
| SP4 | Remaining adapters — Gemini CLI, GitHub Copilot CLI, OpenCode, Cline CLI, Pi, Kiro CLI, Cursor Agent CLI, Continue CLI, Amp | 4B | **next** |
| SP5 | Lifecycle and quality — `create`, `list`, `delete`, `repositories`, `doctor`, JSON output, shell completions, docs | rest of 5, 6 | not started |

`link`/`unlink` land with resolution (SP3), earlier than §35's Phase 5, so resolution never ships
without a way to create the mappings it resolves. The remaining adapters (SP4) build on the SP2
adapter model and contract suite.

`ProfilePresence::Known` was settled in SP4: it is **reserved, not dead**. Spec §8 defines a profile as
*known* when it can be identified from the adapter's documented profile mechanism rather than a
materialized directory. None of the twelve adapters is native-profile, so nothing constructs it — but
removing it would make the first such adapter a change to a public enum, and until then `presence()`
would have to report a profile the agent itself lists as `Absent`. The variant is documented at its
declaration and stays unconstructed.

v0.1 is done when every item of spec §37 (Final Definition of Done) is checked.
```

- [ ] **Step 6: Write `TODO.md`** — §6.1's three binding consequences, which the spec said were recorded and were not. Write the whole file, byte-exact.

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
- [ ] A stdout write that fails after `link` or `unlink` already changed `config.toml` exits 1 (`Io`), so the change
      is not visible in the output or the exit code; `<agent> resolve` with no profile loses its exit 4 the same way.
      Decide whether a write failure after a committed change deserves its own exit code.
- [ ] A hand-written `[repositories.'\\?\C:\…']` key is accepted but never matches the canonical `C:\…` root
      (`Path` treats the verbatim prefix as a different component), and `link` then adds a second entry for the same
      directory. Consider refusing verbatim keys, or comparing keys through `repo::strip_verbatim`.
- [ ] Discovery does not stop at a filesystem boundary the way `git` does, so a directory on a mount inside a
      checkout resolves to the enclosing repository (design §10). Decide with `doctor` (SP5) whether to warn.
- [ ] `unlink --repo <p>` can remove a different repository's live mapping when `<p>`'s meaning changed since `link`
      stored the key (design §10). Consider refusing when a later candidate key also matches an entry.
- [ ] A top-level `unlink` that removes the repository profile while agent mappings remain prints only `unlinked …`;
      the "agent mappings remain" note appears on the next run. Consider showing it on the run that creates the state.

## SP4 known limits

The support level and the capability states are one statement, not two. V3:139-140 forbids presenting an
agent as a guaranteed isolated account without evidence, and SP4's answer is that `proven` never appears
without its capability row beside it. Three consequences bind everything after SP4:

- [ ] The report's capability matrix may not become conditional, opt-in or hidden behind a flag. It is what
      makes an honest `proven` honest; making it optional reintroduces exactly the presentation V3 forbids.
- [ ] SP5's JSON output must carry the capability states **in the same object** as the support level, for
      the same reason. A consumer that can read one field without the other is the flag case again.
- [ ] Any future surface that names a support level must name the capabilities too. The draft README broke
      this rule while §6.1 was being written, which is why it is recorded here rather than assumed.

Also open after SP4:

- [ ] Credential isolation is unproven for every adapter that has not had an authenticated measurement.
      Each one is a metadata edit plus an `off-ci` transcript, not new code.
- [ ] The launch hedge cannot be suppressed. Deferred to SP5 deliberately: its frequency is unmeasured
      until the probe run, and the strict configuration schema makes a new key a compatibility break.
- [ ] The hedge can be erased by the agent it warns about — it is written to stderr immediately before
      `exec`, and a full-screen TUI may clear it. A first-launch acknowledgement belongs with `create`.
- [ ] A transcript cannot be introduced or modified after its artifact expires, so a probe run left
      uncommitted past the retention window must be re-run. The window is a workflow setting that can be
      shortened without noticing what depends on it.
- [ ] **The `Evidence` job only gates a merge if someone makes it a required status check.** That is a
      branch-protection setting, not a file in this repository, so nothing here can assert it. Until it is
      set, the byte-binding in the design's §5.3 is advice: a pull request can go green with the job red.
- [ ] **`custody: off-ci` currently exempts a transcript from every mechanical check.** The clause that
      confines it — Gate A requiring `custody: ci` for any transcript backing a config or state claim —
      lands with the transcript clauses in the fold commit, not in SP4a. Between SP4a merging and that
      commit, a green `Evidence` check on a pull request that adds an `off-ci` transcript conveys nothing,
      and the only backstop is a human reading it.
- [ ] **The `Evidence` job runs pull-request-supplied code in a job that later holds an API token.** The
      manifest build is a separate step that declares no token, and the workflow pins `contents: read`, so
      the exposure is bounded — but `sandbox/verify-transcripts.sh` is itself supplied by the pull request
      and does hold one. This is true of every `pull_request` job here (`cargo nextest run --workspace`
      runs the pull request's tests), and on a public repository with a fork's read-only token the
      marginal grant is `actions: read`. Recorded because it is a real property, not because it is
      currently exploitable.

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

- [ ] **Step 7: Run this task's own checks**

```bash
typos
```

Expected: prints nothing.

- [ ] **Step 8: Run the gate** (see "Gate commands"). Expected on Windows: `243 tests run: 243 passed`. Linux and macOS run fewer tests, because the Windows-only tests are compiled out; every test must pass.

- [ ] **Step 9: Commit**

```bash
git add docs/evidence/README.md crates/agent-profile/src/adapter/metadata.rs README.md CONTRIBUTING.md ROADMAP.md TODO.md
git commit -m "docs: explain evidence transcripts and resolve the Known question

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

This task adds no executable behaviour, so it carries no mutant. Inventing one would mean inventing a test for prose.


## After the last task

- [ ] Push `sp4-adapters` and open a PR only with the owner's confirmation. CI (Windows, macOS, Linux) must be green.
- [ ] Ask the owner about two repository settings this plan deliberately does not change: whether the new `Evidence` job becomes a required status check (the design says "it is a required status check, or it is advice"), and whether `shellcheck` joins the gate now that the repository holds sixteen shell files rather than three.
- [ ] Run AGY-CAPSTONE (on subagents, owner-directed) over the committed range, then AGY-TEST-AUDIT, before declaring SP4a complete.
- [ ] **Then the probe run, which is not part of this plan.** Dispatch Sandbox with `agents: all`, review each transcript by hand, commit them unchanged, and update the three shipped adapters' `upstream_version` to what the probe resolved. Only after that does SP4b — the nine adapters — become writable as a plan rather than a spec.
