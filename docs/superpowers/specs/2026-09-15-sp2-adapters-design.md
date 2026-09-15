# SP2 — Adapter architecture gate: design

**Status:** sections approved by the owner on 2026-09-15; written spec under panel review.
**Branch:** `sp2-adapters` (from `main` at `10166af`).
**Oracle:** `agent-profile-implementation-spec-v3.md` (called "V3" below). Where this document and V3
disagree, V3 wins; report the conflict instead of resolving it silently.
**Previous sub-project:** SP1, `docs/superpowers/specs/2026-09-14-sp1-core-launch-design.md` (merged at
`316980b`).

## 1. Goal

SP2 delivers V3 §35 phase 4A, the architecture gate: an adapter model with capability and evidence metadata,
a common contract suite, and the first three real adapters (Claude Code, Codex CLI, Aider). It ends with
`agent-profile claude|codex|aider <profile> [options] [-- args]` launching the real agent with the selected
profile, proven by library contracts and one end-to-end test per adapter on Linux, macOS and Windows CI, with
no real agent ever running on CI.

## 2. Scope

### 2.1 In scope

| Area | V3 |
|---|---|
| `trait Adapter` with a static registry, pure planning and separate initialization | §4, §38 |
| Capability states and evidence metadata per adapter | §3, §28 |
| Claude Code adapter (`CLAUDE_CONFIG_DIR`) | §2 |
| Codex CLI adapter (`CODEX_HOME`) | §2 |
| Aider adapter (`--config <file>`) | §2 |
| Shallow argument-conflict checks | §21 |
| Lazy creation of directories and files | §8, §9 |
| Executable discovery: Windows shim detection, relative `PATH` entries ignored | §20, §23.2, §36 |
| Common contract suite and per-adapter end-to-end tests | §34 |
| Sandbox prerequisite for measuring agent behaviour (CONTRIBUTING, recommended tools) | §2 "reverified" |

### 2.2 Out of scope

- The other nine V3 §2 adapters (a later SP).
- Repository discovery, `link`, `unlink`, `current`, `status` (SP3).
- `doctor`, and printing capabilities or evidence in any command (metadata is stored and tested only).
- Live smoke tests against real installed agents.
- Parsing `.cmd`/`.bat` shims or guessing vendored package layouts.

## 3. Decisions and their evidence

Evidence marks: **measured** (run on this machine on 2026-09-15), **cited** (URL), **reasoned**.

| # | Decision | Owner choice | Evidence |
|---|---|---|---|
| D1 | Codex isolates a profile with `CODEX_HOME=<root>/profiles/<p>/codex`, not native `--profile` | F1-A | measured: Codex 0.153.4 `--help` "-p, --profile <CONFIG_PROFILE_V2> Layer $CODEX_HOME/<name>.config.toml on top of the base user config", so a native profile is config-only and shares `auth.json`; cited: Codex docs, `CODEX_HOME` holds config, auth and history. V3 §34's `--profile` conflict example applies only to the rejected native-profile mechanism. |
| D2 | Shims are refused with detection and a config fix hint; no parsing | F2 refuse + detect | measured: pnpm installs `codex.cmd`, which runs `node …/codex.js`, which spawns a vendored `codex.exe`; reasoned: parsing trusts attacker-writable PATH files and varies per package manager; V3 §23.2 forbids shell mediation. |
| D3 | `trait Adapter` in a static registry; pure `plan()`, separate `initialize()`, declarative `metadata()` | F3-A | reasoned: V3 §38 thin wrapper; dry run and tests need a plan without side effects. |
| D4 | Library contract table for every adapter plus one end-to-end test per adapter using a renamed `fake-agent` | F4 | reasoned: CI has no real agents; the end-to-end test proves the plan reaches the child. |
| D5 | Aider stays in SP2; `initialize()` creates an empty `.aider.conf.yml` (never overwrites) | owner | measured: a missing `--config` file is an argparse error, exit 2. |
| D6 | Aider `--config` layers over default config files, so config isolation is `NotGuaranteed` | — | measured on aider-chat 0.86.2: with `.aider.conf.yml` in the working directory and `--config p.yml`, the cwd file's `user-input-color` stayed effective and `p.yml`'s `code-theme` won; Aider's docs saying `--config` loads only that file are wrong. |
| D7 | Aider conflicts: `-c`, `--config`, `--confi`, `--conf`, `--con` | — | measured: `-c` and `--conf` select the config file; `--co` is ambiguous (`--code-theme`, `--commit`, `--copy-paste`, …); source `aider/args.py` defines `-c/--config` with `is_config_file=True`; `configargparse` gives config-file options no `AIDER_*` variable, so there is no `AIDER_CONFIG`. |
| D8 | Credential bypass variables recorded per adapter | — | measured strings in `codex.exe` 0.153.4: `OPENAI_API_KEY`, `CODEX_API_KEY`, `CODEX_ACCESS_TOKEN`, `CODEX_SQLITE_HOME`; in `claude.exe` 2.1.270: `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `CLAUDE_CODE_OAUTH_TOKEN`, `CLAUDE_CODE_OAUTH_REFRESH_TOKEN`, `ANTHROPIC_PROFILE`, `CLAUDE_CODE_USE_BEDROCK`, `CLAUDE_CODE_USE_VERTEX`, `CLAUDE_CODE_USE_FOUNDRY`; cited: https://code.claude.com/docs/en/authentication (`CLAUDE_CONFIG_DIR` relocates `.credentials.json` and keys the macOS Keychain entry per directory). |
| D9 | Agent behaviour is measured only inside a disposable sandbox | owner | measured: Sandboxie-Plus 1.18.2 `Start.exe /box:<b> /wait` propagates the exit code, redirects profile writes into `C:\Sandbox\<user>\<b>`, and `ClosedFilePath=%USERPROFILE%\<dir>` denies reads ("Access is denied"). |

## 4. Architecture

### 4.1 Module layout

```text
crates/agent-profile/src/adapter/
  mod.rs       trait Adapter, REGISTRY, lookup, PlanContext, PlannedLaunch, Created, shared helpers,
               conflict scan, case-twin check
  metadata.rs  AdapterMetadata, AdapterEvidence, Capability, CapabilityState, ConflictOption
  claude.rs    Claude Code
  codex.rs     Codex CLI
  aider.rs     Aider
  fake.rs      debug-only test agent (moved out of mod.rs)
```

### 4.2 Types

```rust
pub trait Adapter: Sync {
    fn metadata(&self) -> &'static AdapterMetadata;
    /// Reads the filesystem at most; never writes.
    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch>;
    /// Creates what `planned.creates` names; idempotent; never overwrites a file.
    fn initialize(&self, planned: &PlannedLaunch) -> Result<()>;
}

pub struct PlanContext<'a> {
    pub profile: &'a ProfileName,
    pub root: &'a AppRoot,
    pub config: &'a Config,
    pub args: &'a [OsString],
    pub path_var: Option<&'a OsStr>,
}

pub struct AdapterMetadata {
    pub id: &'static str,
    pub executable: &'static str,           // base name, without `.exe`
    pub mechanism: &'static str,            // human text for the report
    pub evidence: AdapterEvidence,
    pub capabilities: &'static [(Capability, CapabilityState)],
    pub sensitive_env: &'static [&'static str],
    pub conflicts: &'static [ConflictOption],
}

pub struct AdapterEvidence {                // V3 §28, extended with version and source
    pub mechanism_id: &'static str,
    pub verified_at: &'static str,          // ISO date
    pub upstream_version: &'static str,
    pub source_url: &'static str,           // URL, or "measured" with the method in notes
    pub notes: &'static str,
}

pub enum Capability { ConfigIsolation, CredentialIsolation, StateIsolation }
pub enum CapabilityState { Supported, NotSupported, NotGuaranteed, Conditional, Unknown } // V3 §3

pub struct ConflictOption {
    pub long: &'static [&'static str],      // every accepted spelling, each starting with `--`
    pub short: Option<char>,
}

pub enum Created { Dir(PathBuf), File(PathBuf) }
```

`PlannedLaunch` keeps its SP1 fields (`plan`, `profile`, `profile_dir`, `executable_origin`, `mechanism`,
`sensitive_env`), replaces `profile_dir_exists` with `creates: Vec<Created>` (the paths missing at plan
time, in creation order), and adds `notes: Vec<String>` (non-sensitive facts shown in the report).

`REGISTRY: &[&dyn Adapter]` holds `Claude`, `Codex`, `Aider`, and `Fake` only under `debug_assertions`.
`known_agents()` is derived from it. Release builds contain the three real adapters.

### 4.3 Launch data flow

Replaces SP1 design §5.1 step 5 onwards:

1. Look up the agent in the registry; unknown agents fail as in SP1 (`UnknownAgent`, exit 2).
2. Scan the opaque arguments against `metadata().conflicts` (§6). Pure; runs before any filesystem access, so
   a conflict is reported even when the agent is not installed.
3. Refuse a case-only twin profile name (SP1 design §7.3, unchanged).
4. `adapter.plan(ctx)`.
5. With `--dry-run`: print the report (§7.3) and exit 0.
6. `adapter.initialize(&planned)`, then the SP1 launch (`--verbose` report built after initialization).

### 4.4 Shared helpers

- `env_dir_plan(ctx, meta, var, subdir)`: discovers the executable, sets `var=<root>/profiles/<p>/<subdir>`,
  passes `args` verbatim, `cwd: None`, `creates` holds the directory when absent. Used by Claude, Codex, Fake.
- `config_file_arg_plan(ctx, meta, flag, subdir, file_name)`: discovers the executable, prepends
  `flag <root>/profiles/<p>/<subdir>/<file_name>` to `args`, no environment overrides, `creates` holds the
  directory and then the file when absent. Used by Aider.
- `ensure_created(&[Created])`: the shared `initialize()` body. For `Dir`: `create_dir_all`, then verify it
  is a directory. For `File`: `OpenOptions::new().write(true).create_new(true)`; `AlreadyExists` is success
  only when the existing path is a file. Every failure is `Error::ProfileDir { path, source }` (exit 4).
  SP1's `ensure_profile_dir` becomes this helper. `initialize()` ensures every path the adapter owns, not
  only those listed as missing, so a path created or removed between plan and initialize is still handled.

## 5. Adapters

All profile data lives under `<root>/profiles/<p>/<agent>/`. Wrapper-owned variables override inherited
ones (V3 §22). No SP2 adapter declares `sensitive_env`: every override value is a directory path.

### 5.1 Claude Code (`claude`)

- **Mechanism:** `CLAUDE_CONFIG_DIR=<root>/profiles/<p>/claude` via `env_dir_plan`.
- **Conflicts:** none. `--settings`, `--mcp-config` and `--strict-mcp-config` layer over the directory.
- **Notes:** none.
- **Evidence:** `claude-config-dir-v1`, 2026-09-15, 2.1.270, https://code.claude.com/docs/en/authentication.

| Capability | State | Basis |
|---|---|---|
| ConfigIsolation | Supported | user settings live in the config directory |
| CredentialIsolation | Conditional | per-directory `.credentials.json` and macOS Keychain entry (cited); D8 variables override it (measured) |
| StateIsolation | NotGuaranteed | history and project state moving with the directory is community-sourced only |

### 5.2 Codex CLI (`codex`)

- **Mechanism:** `CODEX_HOME=<root>/profiles/<p>/codex` via `env_dir_plan`. Codex treats a missing
  `CODEX_HOME` as fatal, so the directory is always ensured before launch.
- **Conflicts:** none. A user's `-p/--profile <name>` layers `<name>.config.toml` inside the selected home.
- **Notes:** when the home directory is absent at plan time: `new profile starts logged out; run codex login
  with this profile`.
- **Evidence:** `codex-home-v1`, 2026-09-15, 0.153.4, Codex CLI documentation for `CODEX_HOME` plus `--help`.

| Capability | State | Basis |
|---|---|---|
| ConfigIsolation | Supported | `config.toml` and `<name>.config.toml` live in `CODEX_HOME` |
| CredentialIsolation | Conditional | `auth.json` and the keyring key follow `CODEX_HOME`; `OPENAI_API_KEY`, `CODEX_API_KEY`, `CODEX_ACCESS_TOKEN` bypass it (measured) |
| StateIsolation | Conditional | `CODEX_SQLITE_HOME` relocates the state database (measured) |

### 5.3 Aider (`aider`)

- **Mechanism:** `--config <root>/profiles/<p>/aider/.aider.conf.yml` prepended via `config_file_arg_plan`.
- **Initialization:** the directory, then an empty `.aider.conf.yml`; an existing file is never modified.
- **Conflicts:** `ConflictOption { long: &["--config", "--confi", "--conf", "--con"], short: Some('c') }`.
- **Notes:** always: `--config is layered over .aider.conf.yml in the working directory, git root and home`.
- **Evidence:** `aider-config-file-v1`, 2026-09-15, 0.86.2, `measured` (D6, D7).

| Capability | State | Basis |
|---|---|---|
| ConfigIsolation | NotGuaranteed | default config files, `.env` files and `AIDER_*` variables still apply (D6) |
| CredentialIsolation | NotSupported | API keys come from the environment, `.env` files and config files |
| StateIsolation | NotSupported | history files are written in the working directory |

### 5.4 Fake (`fake`, debug builds only)

- **Mechanism:** `FAKE_AGENT_HOME=<root>/profiles/<p>/fake` via `env_dir_plan`, executable `fake-agent`.
- **Conflicts:** `ConflictOption { long: &["--fake-profile"], short: None }`, so the conflict path has an
  end-to-end test without a real agent.
- **Capabilities:** all `Unknown`. **Evidence:** `fake-home-v1`, `measured`, test fixture.

## 6. Argument conflicts (V3 §21)

An opaque argument matches a `ConflictOption` when, for any `long` spelling `L`, it equals `L` or starts with
`L=`; or, when `short` is `Some(c)`, it equals `-c` or starts with `-c` followed by at least one character.

- The scan stops at the first `--` among the opaque arguments; later arguments are positional for the agent.
- Arguments that are not valid UTF-8 are skipped.
- The first match fails with `Error::ArgumentConflict { agent, option, mechanism }`, exit 2:
  `` `--conf` conflicts with the profile agent-profile selects for aider (--config <file>); remove it, or edit
  the profile's config file <root>/profiles/work/aider/.aider.conf.yml ``. The path in the message is
  computed from the root and profile without touching the filesystem.

**Known limits (TODO.md):** values are not parsed, so a value equal to a conflicting spelling (for example
`--message --config`) is refused; clustered short options (`-vc f`) are not detected.

## 7. Executable discovery and reporting

### 7.1 Relative `PATH` entries (all platforms)

`exe::discover` searches only absolute `PATH` entries. Empty entries were already skipped; relative entries
such as `.`, `bin` or `..\tools` are now skipped too, because they resolve against the working directory,
which is repository-local discovery (V3 §20, §36).

### 7.2 Windows shims

For each absolute `PATH` directory in order, check `<name>.exe`, `<name>.cmd`, `<name>.bat`. The first
directory containing any of them decides:

- `<name>.exe` exists there: found (origin `PATH`), even if a shim sits beside it.
- only `<name>.cmd` or `<name>.bat`: `Error::UnsupportedExecutable { path, agent }`, exit 6.

A later directory's `.exe` is never chosen over an earlier shim, because the user's shell would run the shim.
`UnsupportedExecutable` gains the agent id and a fix hint, also for a configured `.cmd`/`.bat`:
`` `codex` resolves to a batch shim (C:\…\codex.cmd); agent-profile launches without a shell and cannot run
it. Set the native executable: [agents.codex] executable = "<absolute path to codex.exe>" in
<root>\config.toml ``.

Unix discovery is unchanged apart from §7.1: npm shims there are executable scripts that `exec` runs.

### 7.3 Report (dry run and `--verbose`)

SP1 lines stay (`agent`, `profile`, `executable`, `repository`, `mechanism`, `environment`, `arguments`).
Changes:

- `environment` values equal to a `Created::Dir` path carry `(would be created)` in dry run (SP1 behaviour,
  now driven by `creates`).
- New `creates:` line(s), one path per line in creation order, each with `(would be created)` in dry run;
  `none` when nothing is missing. Omitted from `--verbose` (initialization has already run).
- New `note:` line(s), one per adapter note, in order.
- `arguments` shows the injected `--config <path>` before the user's arguments.

## 8. Testing (V3 §34)

### 8.1 Library contract suite: `tests/adapter_contract.rs`

One table row per registered adapter (including `fake` in debug builds); every assertion runs on every row.

- **Plan contract:** profile `work`, a configured executable, opaque args `["x", "a b"]` produce the exact
  `LaunchPlan` (executable, args, env, `cwd: None`), `creates`, `notes` and `mechanism` from §5.
- **Conflict contract:** each row lists refused and accepted argument vectors.
  Aider refused: `["--config","f"]`, `["--config=f"]`, `["--conf","f"]`, `["--con=f"]`, `["-c","f"]`, `["-cf"]`.
  Aider accepted: `["--co"]`, `["--code-theme","x"]`, `["--","--config","f"]`.
  Codex accepted: `["-p","personal"]`, `["--profile","personal"]`. Claude accepted: `["--settings","s"]`.
  Fake refused: `["--fake-profile","x"]`.
- **Metadata invariants:** ids unique, each parses as `AgentId` and equals the lookup key; each `Capability`
  appears exactly once per adapter; `verified_at` is `YYYY-MM-DD`; `mechanism_id`, `upstream_version`,
  `source_url` non-empty; every `long` spelling starts with `--`; release registry is exactly
  `claude, codex, aider`.
- **Initialization contract:** idempotent; an existing Aider config file keeps its bytes; a file where a
  directory belongs, and a directory where the Aider file belongs, each give `ProfileDir`.

### 8.2 End-to-end: `tests/adapters_e2e.rs`

For each of `claude`, `codex`, `aider`: copy the built `fake-agent` into a temporary directory as the agent's
executable name (`.exe` on Windows), set `PATH` to only that absolute directory, run
`agent-profile <agent> work -- …` and assert from the fixture's echo:

- argv as planned (Aider's `--config <path>` first);
- the override variable equals the profile path even when the parent set it to `/wrong` (V3 §22);
- the working directory is inherited;
- the fixture's exit code propagates;
- the planned directory, and for Aider the config file, exist afterwards.

Plus:

- Aider `-- --conf f`: exit 2, the fixture never ran (no echo output, no profile directory).
- Windows only: a `PATH` directory holding only `codex.cmd` gives exit 6 and the config hint.
- All platforms: a relative `PATH` entry holding the executable is ignored (`AgentNotInstalled`, exit 3).

### 8.3 Unit tests

- `exe`: relative entries skipped; Windows `.cmd`-only directory refused with agent id; earlier shim beats a
  later `.exe`; `.exe` beside `.cmd` found.
- conflict scan: every match form in §6, the `--` stop, non-UTF-8 skip.
- `output`: `creates` and `note` lines in dry run and verbose.
- `error`: `ArgumentConflict` maps to 2; exit-code table test extended.

Every new test must fail under a logic mutant of the behaviour it guards (PINNING-ASSERTION-STRENGTH).

## 9. Documentation

- **CONTRIBUTING.md**, new section "Measuring agent behaviour": evidence rows are refreshed only from
  measurements taken inside a disposable sandbox, never by installing agents on the host.
  - Windows: Sandboxie-Plus. Create a box, add `ClosedFilePath=%USERPROFILE%\.claude` and
    `ClosedFilePath=%USERPROFILE%\.codex` (and any other agent home) so the box behaves like a clean
    machine, run `Start.exe /box:<b> /wait cmd /c "… > C:\m\out.txt"`, read results from
    `C:\Sandbox\<user>\<b>\drive\C\m\out.txt`, then `Start.exe /box:<b> /terminate` and
    `Start.exe /box:<b> delete_sandbox_silent`.
  - Linux: rootless Podman or Docker `run --rm` for installs; Bubblewrap or Firejail with a private home
    for probing host binaries.
  - macOS: Tart disposable macOS VMs when behaviour is macOS-specific (Keychain); OrbStack, Colima or
    Docker `--rm` for Linux-only checks. Not `sandbox-exec` (deprecated, deny-only).
- **`.claude/recommended-tools.json`**: Sandboxie-Plus, `file_exists`
  `C:/Program Files/Sandboxie-Plus/Start.exe`, install `winget install Sandboxie.Plus`.
- **README.md**: supported agents with mechanism and capability tables from §5.
- **TODO.md**: close the SP2 `.cmd` shim decision (D2); add §6 known limits.

## 10. Known limits

- Conflict scan false positive and clustered short options (§6).
- Credential isolation is `Conditional` for Claude and Codex: inherited credential variables (D8) are passed
  through unchanged; SP2 neither strips nor warns about them.
- Aider isolation is configuration layering only; the report says so on every launch.
- A new Codex profile starts logged out; the dry run and verbose report say so.
- Evidence is static; drift detection belongs to `doctor` (later SP).
