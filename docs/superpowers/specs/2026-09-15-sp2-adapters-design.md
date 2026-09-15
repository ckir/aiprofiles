# SP2 — Adapter architecture gate: design

**Status:** sections approved by the owner on 2026-09-15; written spec under panel review (rounds 1–3 folded).
**Branch:** `sp2-adapters` (from `main` at `10166af`).
**Oracle:** `agent-profile-implementation-spec-v3.md` (called "V3" below). Where this document and V3
disagree, V3 wins; report the conflict instead of resolving it silently.
**Previous sub-project:** SP1, `docs/superpowers/specs/2026-09-14-sp1-core-launch-design.md` (merged at
`316980b`).

## 1. Goal

SP2 delivers V3 §35 phase 4A, the architecture gate: an adapter model with capability, support-level,
presence and evidence metadata, a common contract suite, and the first three real adapters (Claude Code,
Codex CLI, Aider). It ends with `agent-profile claude|codex|aider <profile> [options] [-- args]` launching the
real agent with the selected profile, proven by library contracts and one end-to-end test per adapter on
Linux, macOS and Windows CI, with no real agent ever running on CI.

## 2. Scope

### 2.1 In scope

| Area | V3 |
|---|---|
| `trait Adapter` with a registry, pure planning, presence and separate initialization | §4, §8, §38 |
| Capability states, support level and evidence metadata per adapter | §3, §28, §37 |
| Claude Code adapter (`CLAUDE_CONFIG_DIR`) | §2 |
| Codex CLI adapter (`CODEX_HOME`) | §2 |
| Aider adapter (`--config <file>`) | §2 |
| Mandatory per-override sensitivity declaration (SP1 design §7.4 forward promise) | §22 |
| Shallow argument-conflict checks | §21 |
| Lazy, atomic creation of directories and files | §8, §9, §9.1 |
| Executable discovery: Windows shim detection, relative `PATH` entries ignored | §20, §23.2, §36 |
| Common contract suite and per-adapter end-to-end tests | §34 |
| Sandbox prerequisite for measuring agent behaviour (CONTRIBUTING, recommended tools) | §2 "reverified" |

### 2.2 Out of scope

- The other nine V3 §2 adapters (a later SP).
- Repository discovery, `link`, `unlink`, `current`, `status`, `profiles` (SP3 and later); SP2 defines
  `presence()` but no command prints it.
- `doctor`, and printing capabilities, support level or evidence in any command (metadata is stored and
  tested only).
- Live smoke tests against real installed agents.
- Parsing shims or guessing vendored package layouts.

## 3. Decisions and their evidence

Evidence marks: **measured** (run on this machine on 2026-09-15; agents the owner had not installed were run
only inside a disposable Sandboxie-Plus box), **cited** (URL), **reasoned**.

| # | Decision | Owner choice | Evidence |
|---|---|---|---|
| D1 | Codex isolates a profile with `CODEX_HOME=<root>/profiles/<p>/codex`, not native `--profile` | F1-A | measured: Codex 0.153.4 `--help` "-p, --profile <CONFIG_PROFILE_V2> Layer $CODEX_HOME/<name>.config.toml on top of the base user config", so a native profile is config-only and shares `auth.json`; cited: Codex docs, `CODEX_HOME` holds config, auth and history. V3 §34's `--profile` conflict example applies only to the rejected native-profile mechanism. |
| D2 | Shims are refused with detection and a fix hint; no parsing | F2 refuse + detect | measured: pnpm installs `codex.cmd`, which runs `node …/codex.js`, which spawns a vendored `codex.exe`; reasoned: parsing trusts attacker-writable PATH files and varies per package manager; V3 §23.2 forbids shell mediation. |
| D3 | `trait Adapter` in a registry; pure `plan()` and `presence()`, separate `initialize()`, declarative `metadata()` | F3-A | reasoned: V3 §38 thin wrapper; dry run and tests need a plan without side effects; V3 §8 requires a presence strategy per adapter. |
| D4 | Library contract table for every adapter plus one end-to-end test per adapter using a renamed `fake-agent` | F4 | reasoned: CI has no real agents; the end-to-end test proves the plan reaches the child. |
| D5 | Aider stays in SP2; `initialize()` atomically creates `.aider.conf.yml` containing `{}` and a newline, never overwriting | owner (empty file), corrected by measurement | measured in a sandbox on aider-chat 0.86.2 with `--config <file> --version`: missing file "Unable to open config file", exit 2; empty file and comment-only file "The config file doesn't appear to contain 'key: value' pairs … returned type 'NoneType' instead of 'dict'", exit 2; `{}` accepted ("aider 0.86.2"). An empty mapping sets no option, so no configuration is invented (V3 §9). |
| D6 | Aider `--config` layers over the default config files, including the user-level `~/.aider.conf.yml`, so config isolation is `NotGuaranteed` | — | measured: with `.aider.conf.yml` in the working directory and `--config p.yml`, the cwd file's `user-input-color` stayed effective and `p.yml`'s `code-theme` won; Aider's docs saying `--config` loads only that file are wrong; source `aider/main.py` adds cwd, git root and home files to `default_config_files`. |
| D7 | Aider conflicts: `-c`, `--config`, `--confi`, `--conf`, `--con` | — | measured: `-c`, `--con`, `--conf`, `--confi` each select the config file (a missing file errors, a present one is loaded); `--co` is ambiguous (`--code-theme`, `--commit`, `--copy-paste`, …); source `aider/args.py` defines `-c/--config` with `is_config_file=True`; `configargparse` gives config-file options no `AIDER_*` variable, so there is no `AIDER_CONFIG`. |
| D8 | Credential bypass variables recorded per adapter | — | measured strings in `codex.exe` 0.153.4: `OPENAI_API_KEY`, `CODEX_API_KEY`, `CODEX_ACCESS_TOKEN`, `CODEX_SQLITE_HOME`; in `claude.exe` 2.1.270: `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `CLAUDE_CODE_OAUTH_TOKEN`, `CLAUDE_CODE_OAUTH_REFRESH_TOKEN`, `ANTHROPIC_PROFILE`, `CLAUDE_CODE_USE_BEDROCK`, `CLAUDE_CODE_USE_VERTEX`, `CLAUDE_CODE_USE_FOUNDRY`; cited: https://code.claude.com/docs/en/authentication (`CLAUDE_CONFIG_DIR` relocates `.credentials.json` and keys the macOS Keychain entry per directory). |
| D9 | Agent behaviour is measured only inside a disposable sandbox | owner | measured: Sandboxie-Plus 1.18.2 `Start.exe /box:<b> /wait` propagates the exit code, redirects profile writes into `C:\Sandbox\<user>\<b>`, and `ClosedFilePath=%USERPROFILE%\<dir>` denies reads ("Access is denied"). |
| D10 | Error precedence keeps SP1's discovery-before-case-twin order; the conflict scan runs after SP1 steps 2–4 and before discovery | — | measured: SP1 `adapter/mod.rs` `fake::plan` calls `exe::discover` then `check_case_twins`; SP1 design §5.1 step 5. |

## 4. Architecture

### 4.1 Module layout

```text
crates/agent-profile/src/adapter/
  mod.rs       trait Adapter, REAL_ADAPTERS, registry(), lookup, PlanContext, PlannedLaunch, ProfilePath,
               PathKind, shared helpers (plan helpers, ensure_paths, write_new_file), conflict scan,
               case-twin check
crates/agent-profile/src/output.rs
               ReportMode added to report_lines
  metadata.rs  AdapterMetadata, AdapterEvidence, SupportLevel, Capability, CapabilityState, CapabilityClaim,
               EnvOverride, ConflictOption, ProfilePresence
  claude.rs    Claude Code
  codex.rs     Codex CLI
  aider.rs     Aider
  fake.rs      debug-only test agent (moved out of mod.rs)
```

### 4.2 Types

```rust
pub trait Adapter: Sync {
    fn metadata(&self) -> &'static AdapterMetadata;
    /// Pure: no filesystem access. Every path the adapter owns for `profile`, in creation order.
    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)>;
    /// Reads the filesystem at most; never writes. Discovers the executable, then refuses case-only twins.
    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch>;
    /// Reads the filesystem at most; never writes; never consults `PATH` or the executable (V3 §8).
    fn presence(&self, root: &AppRoot, profile: &ProfileName) -> ProfilePresence;
    /// Ensures every path in `planned.paths`, in order; idempotent; never overwrites a file.
    fn initialize(&self, planned: &PlannedLaunch) -> Result<()>;
}

pub struct PlanContext<'a> {                   // the agent id is always `metadata().id`
    pub profile: &'a ProfileName,
    pub root: &'a AppRoot,
    pub config: &'a Config,
    pub args: &'a [OsString],
    pub path_var: Option<&'a OsStr>,
}

pub struct AdapterMetadata {
    pub id: &'static str,
    pub executable: &'static str,              // base name, without `.exe`
    pub mechanism_summary: &'static str,       // path-free, e.g. "argument --config <file>"; used in messages
    pub support: SupportLevel,
    pub evidence: AdapterEvidence,
    pub capabilities: &'static [CapabilityClaim],
    pub env: &'static [EnvOverride],           // every variable plan() may set, with its sensitivity
    pub conflicts: &'static [ConflictOption],
}

pub enum SupportLevel { Proven, Experimental } // V3 §3, §37: separate from capabilities

pub struct AdapterEvidence {                   // V3 §28, extended with version and source
    pub mechanism_id: &'static str,
    pub verified_at: &'static str,             // ISO date
    pub upstream_version: &'static str,
    pub source_url: &'static str,              // URL, or "measured" with the method in notes
    pub notes: &'static str,
}

pub enum Capability { ConfigIsolation, CredentialIsolation, StateIsolation }
pub enum CapabilityState { Supported, NotSupported, NotGuaranteed, Conditional, Unknown } // V3 §3
pub struct CapabilityClaim { pub capability: Capability, pub state: CapabilityState, pub basis: &'static str }

pub struct EnvOverride { pub name: &'static str, pub sensitive: bool }

pub struct ConflictOption {
    pub long: &'static [&'static str],         // every accepted spelling, each starting with `--`
    pub short: Option<char>,
}

pub enum ProfilePresence { Absent, Materialized, Known } // V3 §8

pub enum PathKind { Dir, File { contents: &'static [u8] } }
pub struct ProfilePath { pub path: PathBuf, pub kind: PathKind, pub existed: bool }
```

**Capability definitions** (one rule, applied identically to every adapter):

- `ConfigIsolation`: the profile replaces the user-level configuration the agent reads from its default user
  location. Repository or working-directory configuration layered on top is a caveat recorded in `basis`, not
  a failure. `NotGuaranteed` when user-level configuration outside the profile still applies.
- `CredentialIsolation`: stored credentials (files or OS keychain entries) are separated per profile.
  `Conditional` when documented environment variables bypass the separation.
- `StateIsolation`: history, sessions and caches are separated per profile. `Conditional` when a documented
  variable relocates part of it; `NotGuaranteed` when only community-sourced.

**Support level:** `Proven` requires an evidence entry, every capability claimed, and passing the contract
suite (§8.1). All three real adapters are `Proven`; `fake` is `Experimental`.

`PlannedLaunch` keeps its SP1 fields `plan`, `profile`, `profile_dir` (always the first `Dir` from `paths()`,
i.e. `<root>/profiles/<p>/<agent>`), `executable_origin`, `mechanism` (the per-launch report text, as in SP1;
exact strings per adapter in §5), `sensitive_env` (now filled from
`metadata().env` entries with `sensitive: true`), replaces `profile_dir_exists` with
`paths: Vec<ProfilePath>` (built from `paths()`; `existed` is true exactly when `fs::metadata` reports the
declared kind, so a directory where a file belongs is not "existing"), and adds `notes: Vec<String>`
(non-sensitive facts shown in the report).

`REAL_ADAPTERS: &[&dyn Adapter]` is exactly `[&Claude, &Codex, &Aider]` in every build. `registry()` returns
`REAL_ADAPTERS` plus `&Fake` when `debug_assertions` is on. `known_agents()` is derived from `registry()`.

**Presence** (V3 §8): `Absent` when the SP1 case-only-twin check would refuse the profile (so presence never
disagrees with launch); otherwise `Materialized` when every path from `paths()` has the declared kind per
`fs::metadata`, else `Absent`. It is independent of whether the agent is installed. No SP2 adapter reports
`Known` (none has a native profile registry).

### 4.3 Launch data flow and error precedence

SP1 steps 1–4 (argument split, profile validation, application root, configuration load, resolution) are
unchanged. From SP1 design §5.1 step 5 onwards:

1. Get the adapter from `registry()`. SP1 parsing (step 1) already rejected unknown agents with `UnknownAgent`
   (exit 2), using `known_agents()`.
2. Scan the opaque arguments against `metadata().conflicts` (§6). Pure: it runs before executable discovery,
   so a conflict is reported even when the agent is not installed.
3. `adapter.plan(ctx)`: executable discovery (§7; exit 3 or 6), then the SP1 case-only-twin check (SP1 design
   §7.3; exit 4), then the plan.
4. With `--dry-run`: print the report (§7.3) and exit 0.
5. `adapter.initialize(&planned)` (exit 4 on failure), then the SP1 launch (exit 6 on spawn failure); the
   `--verbose` report is rendered in verbose mode after initialization.

Precedence when several errors apply: usage and unknown agent (2) → SP1 steps 2–4 errors (4) → argument
conflict (2) → executable not installed (3) or unsupported executable (6) → case-only twin (4) →
initialization (4) → launch (6).

### 4.4 Shared helpers

Both plan helpers return a `PlannedLaunch` with empty `notes`; the adapter adds its notes afterwards.

- `env_dir_plan(adapter, ctx, var, mechanism)`: discovers `metadata().executable`, checks case twins, takes
  the directory from `adapter.paths()` (`vec![(<root>/profiles/<p>/<agent>, PathKind::Dir)]`), sets `var` to
  it, passes `args` verbatim, `cwd: None`. Used by Claude, Codex, Fake.
- `config_file_arg_plan(adapter, ctx, flag, mechanism)`: discovers, checks case twins, takes
  `vec![(<dir>, PathKind::Dir), (<file>, PathKind::File { contents })]` from `adapter.paths()`, prepends
  `flag <file>` to `args`, no environment overrides. Used by Aider.
- `ensure_paths(&[ProfilePath])`: the shared `initialize()` body, ignoring `existed` (a path created or
  removed between plan and initialize is still handled).
  - `Dir`: `create_dir_all`, then `fs::metadata` (follows symlinks) must report a directory.
  - `File`: if `fs::metadata` reports a file, done. Otherwise `write_new_file(path, contents)`, a small
    lock-free writer in `adapter/mod.rs`: a `tempfile::Builder` temp file with prefix `.<file name>.` and
    suffix `.tmp` in the same directory, `write_all`, `sync_all`, `persist_noclobber`, then (Unix) `sync_all`
    on the directory. Any persist error is success when `fs::metadata` now reports a file (a concurrent
    launch won); otherwise it is `ProfileDir`. There is no sweep: without a lock a sweep could delete another
    launch's live temp file, and a leftover temp from a killed launch is inert because Aider reads only
    `.aider.conf.yml`. A concurrent launch never sees a partial file, and a power loss cannot leave a
    zero-length file under the final name (V3 §9.1). SP1's `config.rs` is unchanged.
  - Every failure, including a dangling symlink or a directory where a file belongs, is
    `Error::ProfileDir { path, source }` (exit 4). SP1's `ensure_profile_dir` becomes the `Dir` case.

## 5. Adapters

All profile data lives under `<root>/profiles/<p>/<agent>/`. Wrapper-owned variables override inherited
ones (V3 §22). Every SP2 override is a directory path, so every `EnvOverride` is `sensitive: false`. The
contract suite (§8.1) enforces that every variable a plan sets is declared, and a test-only adapter proves the
`sensitive: true` path (§8.3).

### 5.1 Claude Code (`claude`)

- **Mechanism:** `CLAUDE_CONFIG_DIR=<root>/profiles/<p>/claude` via `env_dir_plan`. Report `mechanism`:
  `environment variable CLAUDE_CONFIG_DIR`; `mechanism_summary`: `environment variable CLAUDE_CONFIG_DIR`.
- **Conflicts:** none. `--settings`, `--mcp-config` and `--strict-mcp-config` layer over the directory.
- **Notes:** none.
- **Evidence:** `claude-config-dir-v1`, 2026-09-15, 2.1.270, https://code.claude.com/docs/en/authentication.
- **Support:** `Proven`.

| Capability | State | Basis |
|---|---|---|
| ConfigIsolation | Supported | user settings live in the config directory; project `.claude/settings*.json` and `.mcp.json` still layer on top |
| CredentialIsolation | Conditional | per-directory `.credentials.json` and macOS Keychain entry (cited); D8 variables override it (measured) |
| StateIsolation | NotGuaranteed | history and project state moving with the directory is community-sourced only |

### 5.2 Codex CLI (`codex`)

- **Mechanism:** `CODEX_HOME=<root>/profiles/<p>/codex` via `env_dir_plan`. Codex treats a missing
  `CODEX_HOME` as fatal, so the directory is always ensured before launch. Report `mechanism` and
  `mechanism_summary`: `environment variable CODEX_HOME`.
- **Conflicts:** none. A user's `-p/--profile <name>` layers `<name>.config.toml` inside the selected home.
- **Notes:** when the home directory did not exist at plan time: `new profile starts logged out; run codex
  login with this profile`.
- **Evidence:** `codex-home-v1`, 2026-09-15, 0.153.4, `measured` (notes: `codex --help` profile text and
  binary strings, D1 and D8; `CODEX_HOME` semantics per the Codex CLI configuration documentation).
- **Support:** `Proven`.

| Capability | State | Basis |
|---|---|---|
| ConfigIsolation | Supported | `config.toml` and `<name>.config.toml` live in `CODEX_HOME`; project-level configuration layering is not measured |
| CredentialIsolation | Conditional | `auth.json` and the keyring key follow `CODEX_HOME`; `OPENAI_API_KEY`, `CODEX_API_KEY`, `CODEX_ACCESS_TOKEN` bypass it (measured) |
| StateIsolation | Conditional | `CODEX_SQLITE_HOME` relocates the state database (measured) |

### 5.3 Aider (`aider`)

- **Mechanism:** `--config <root>/profiles/<p>/aider/.aider.conf.yml` prepended via `config_file_arg_plan`.
  Report `mechanism`: `argument --config <that path>`; `mechanism_summary`: `argument --config <file>`.
- **Initialization:** the directory, then `.aider.conf.yml` containing `{}\n` (D5); an existing file is never
  modified.
- **Conflicts:** `ConflictOption { long: &["--config", "--confi", "--conf", "--con"], short: Some('c') }`.
- **Notes:** always: `--config is layered over .aider.conf.yml in the working directory, git root and home`.
- **Evidence:** `aider-config-file-v1`, 2026-09-15, 0.86.2, `measured` (D5, D6, D7).
- **Support:** `Proven`.

| Capability | State | Basis |
|---|---|---|
| ConfigIsolation | NotGuaranteed | the user-level `~/.aider.conf.yml`, repository and cwd config files, `.env` files and `AIDER_*` variables still apply (D6) |
| CredentialIsolation | NotSupported | API keys come from the environment, `.env` files and config files |
| StateIsolation | NotSupported | history files are written in the working directory |

### 5.4 Fake (`fake`, debug builds only)

- **Mechanism:** `FAKE_AGENT_HOME=<root>/profiles/<p>/fake` via `env_dir_plan`, executable `fake-agent`
  (unchanged from SP1, so SP1's exact-env and report-line-count tests stay valid). Report `mechanism` and
  `mechanism_summary`: `environment variable FAKE_AGENT_HOME`.
- **Conflicts:** `ConflictOption { long: &["--fake-profile"], short: None }`, so the conflict path has an
  end-to-end test without a real agent.
- **Capabilities:** all `Unknown`, basis `test fixture`. **Evidence:** `fake-home-v1`, 2026-09-15, upstream
  version `0.0.0` (the in-repo fixture), `measured`, notes `test fixture`. **Support:** `Experimental`.

## 6. Argument conflicts (V3 §21)

Matching works on `OsStr::as_encoded_bytes()`, so non-UTF-8 arguments are scanned too; every spelling is
ASCII, which makes a byte-prefix comparison exact on all platforms. An opaque argument matches a
`ConflictOption` when, for any `long` spelling `L`, it equals `L` or starts with `L=`; or, when `short` is
`Some(c)`, it equals `-c` or starts with `-c` followed by at least one byte.

- The scan stops at the first `--` among the opaque arguments; later arguments are positional for the agent.
- The first match fails with `Error::ArgumentConflict { agent: String, option: String, mechanism: &'static str }`
  (`option` rendered lossily, `mechanism` from `metadata().mechanism_summary`), exit 2:
  `` `--conf` conflicts with how agent-profile selects the aider profile (argument --config <file>); remove it
  from the agent arguments ``.

**Known limits (TODO.md):** values are not parsed, so a value equal to a conflicting spelling (for example
`--message --config`) is refused; clustered short options (`-vc f`) are not detected.

## 7. Executable discovery and reporting

`exe::discover(agent, name, explicit, path_var, config_file)` gains `config_file: &Path`
(`ctx.root.config_path()`), used only in hint text.

### 7.1 Relative `PATH` entries (all platforms)

Only entries for which `Path::is_absolute` is true are searched. Empty entries were already skipped; relative
entries such as `.`, `bin`, `..\tools`, and on Windows rooted-without-drive `\tools` and drive-relative
`C:tools`, are skipped too, because they resolve against the working directory or current drive
(repository-local discovery, V3 §20, §36). `NotInstalledReason::NotOnPath` becomes
`NotOnPath { ignored_relative: usize }`; when non-zero the message adds
`(N relative PATH entries were ignored)`.

### 7.2 Windows shims

For each absolute `PATH` directory in order, check `<name>.exe`, `<name>.com`, `<name>.cmd`, `<name>.bat`,
`<name>.ps1`. The first directory containing any of them decides:

- `<name>.exe` exists there: found (origin `PATH`), even if another form sits beside it.
- otherwise: `Error::UnsupportedExecutable { agent, path, config_file }` for the first existing form in the
  order above, exit 6.

A later directory's `.exe` is never chosen over an earlier non-`.exe` form, because the user's shell would run
that form. A configured `.cmd`, `.bat` or `.ps1` gets the same error on every platform, like SP1 D8's
extension check (now with the agent and hint); a configured `.com` is launched as a native program. When one directory holds several non-`.exe`
forms, the error names the first in the order above. Message:
`` `codex` resolves to C:\…\codex.cmd, which agent-profile cannot launch without a shell. Install the agent's
native executable (for example the vendor's standalone installer) or set [agents.codex] executable =
"<absolute path to a native .exe>" in <config_file> ``.

Unix `PATH` discovery is unchanged apart from §7.1: npm shims there are executable scripts that `exec` runs.

### 7.3 Report (dry run and `--verbose`)

`output::report_lines(planned, resolution, mode)` gains `mode: ReportMode { DryRun, Verbose }`. SP1 lines stay
(`agent`, `profile`, `executable`, `repository`, `mechanism`, `environment`, `arguments`). Changes:

- `DryRun` only: `environment` values equal to a `Dir` path with `existed: false` carry `(would be created)`;
  new `creates:` line(s), one per `ProfilePath` with `existed: false` in order, each ending
  `(would be created)`, or `none`.
- `Verbose`: no `(would be created)` markers and no `creates:` line (initialization has already run).
- Both modes: new `note:` line(s), one per adapter note, in order; `arguments` shows the injected
  `--config <path>` before the user's arguments.

## 8. Testing (V3 §34)

### 8.1 Library contract suite: `tests/adapter_contract.rs`

One table row per `registry()` adapter (tests build with debug assertions, so `fake` is included). Each
bullet runs on every row unless it names specific adapters.

- **Plan contract:** profile `work`, a configured executable, opaque args `["x", "a b"]` produce the exact
  `LaunchPlan` (executable, args, env, `cwd: None`), `paths`, `notes` and `mechanism` from §5.
- **Declared environment:** every variable in the plan's `env` appears in `metadata().env`; `sensitive_env`
  equals the declared sensitive names.
- **Conflict contract:** each row lists refused and accepted argument vectors.
  Aider refused: `["--config","f"]`, `["--config=f"]`, `["--confi","f"]`, `["--conf","f"]`, `["--con=f"]`,
  `["-c","f"]`, `["-cf"]`, and on Unix a `--config=` argument with non-UTF-8 bytes after `=`.
  Aider accepted: `["--co"]`, `["--code-theme","x"]`, `["--","--config","f"]`.
  Codex accepted: `["-p","personal"]`, `["--profile","personal"]`. Claude accepted: `["--settings","s"]`.
  Fake refused: `["--fake-profile","x"]`.
- **Metadata invariants:** ids unique, each parses as `AgentId` and equals the lookup key; each `Capability`
  appears exactly once with a non-empty `basis`; `verified_at` is `YYYY-MM-DD`; `mechanism_id`,
  `upstream_version`, `source_url` non-empty; every `long` spelling starts with `--`; `REAL_ADAPTERS` ids are
  exactly `claude, codex, aider`, each `Proven`.
- **Paths contract:** `paths()` equals the `path` and `kind` of `plan().paths`, in order.
- **Presence contract:** `Absent` before `initialize()`; `Materialized` after, with no executable configured
  or present anywhere (the signature takes no `PATH`); `Absent` for the case-only twin `WORK` of a materialized `work`
  on every platform (the twin check reads directory entries, so this is filesystem-independent). Aider only:
  `Absent` again when the file is removed while the directory stays (the only multi-path row, so it alone
  pins that presence requires every path).
- **Initialization contract:** idempotent; an existing Aider config file keeps its bytes; a new Aider file
  holds exactly `{}\n`; a file where a directory belongs, a directory where the Aider file belongs, and (Unix)
  a dangling symlink at the Aider file each give `ProfileDir`.

### 8.2 End-to-end: `tests/adapters_e2e.rs`

For each of `claude`, `codex`, `aider`: copy the built `fake-agent` into a temporary directory as the agent's
executable name (`.exe` on Windows), set `PATH` to only that absolute directory, run
`agent-profile <agent> work -- …` and assert from the fixture's echo:

- the exact argv (Aider: exactly `--config`, the profile file path, then the user's arguments);
- `claude` and `codex`: the override variable equals the profile path even when the parent set it to
  `/wrong` (V3 §22);
- the working directory is inherited;
- the fixture's exit code propagates;
- the planned directory, and for Aider the config file with `{}\n`, exist afterwards.

Plus:

- Aider concurrent first launches: 8 simultaneous `agent-profile aider work` children of a new profile all
  exit 0 and the file holds `{}\n` (mirrors SP1's `concurrent_first_launches_of_one_profile_all_succeed`).
- Aider `-- --conf f` with the fixture on `PATH`: exit 2, the fixture never ran, no profile directory.
- Aider `-- --conf f` with an empty `PATH` and no configured executable: exit 2, not 3 (§4.3 step 2).
- Windows only: a `PATH` directory holding only `codex.cmd` gives exit 6 and the hint naming the config file.
- All platforms: a relative `PATH` entry holding the executable is ignored (exit 3, message counts it).

### 8.3 Unit tests

- `exe`: relative entries skipped including Windows `\tools` and `C:tools`; `ignored_relative` counted;
  Windows `.cmd`-only and `.ps1`-only directories refused with agent and config file; a directory holding
  both `.cmd` and `.ps1` names the `.cmd`; earlier shim beats a later `.exe`; `.exe` beside `.cmd` found;
  configured `.ps1` refused.
- conflict scan: every match form in §6, the `--` stop, non-UTF-8 bytes.
- `output`: `creates`, `note` and markers in `DryRun`; none of the markers in `Verbose`.
- `error`: `ArgumentConflict` maps to 2; exit-code table test extended.
- `ensure_paths`: concurrent calls on a new Aider file never observe a partial file (writer threads plus a
  reader asserting the content is `{}\n` whenever the file exists); a leftover `.aider.conf.yml.*.tmp` file in
  the directory never becomes the config file and does not stop creation.
- Sensitive declarations: a test-only adapter in `adapter/mod.rs` tests declares
  `EnvOverride { name: "PROFILE_SESSION_HANDLE", sensitive: true }` (no SP1 backstop substring) and sets it;
  its `PlannedLaunch.sensitive_env` is exactly that name and `report_lines` renders `<redacted>` for it.
  Removing the declaration-to-`sensitive_env` mapping must fail this test.

Every new test must fail under a logic mutant of the behaviour it guards (PINNING-ASSERTION-STRENGTH).

## 9. Documentation

- **CONTRIBUTING.md**, new section "Measuring agent behaviour": evidence rows are refreshed only from
  measurements taken inside a disposable sandbox, never by installing agents on the host.
  - Windows: Sandboxie-Plus. Create a box, add `ClosedFilePath=%USERPROFILE%\.claude` and
    `ClosedFilePath=%USERPROFILE%\.codex` (and any other agent home) so the box behaves like a clean
    machine, run `Start.exe /box:<b> /wait cmd /c "<script> > C:\m\out.txt"`, read results from
    `C:\Sandbox\<user>\<b>\drive\C\m\out.txt`, then `Start.exe /box:<b> /terminate` and
    `Start.exe /box:<b> delete_sandbox_silent`.
  - Linux: rootless Podman or Docker `run --rm` for installs; Bubblewrap or Firejail with a private home
    for probing host binaries.
  - macOS: Tart disposable macOS VMs when behaviour is macOS-specific (Keychain); OrbStack, Colima or
    Docker `--rm` for Linux-only checks. Not `sandbox-exec` (deprecated, deny-only).
- **`.claude/recommended-tools.json`**: Sandboxie-Plus, `file_exists`
  `C:/Program Files/Sandboxie-Plus/Start.exe`, install `winget install Sandboxie.Plus`; its `why` says it is
  the Windows sandbox and points Linux and macOS contributors to CONTRIBUTING (the schema has no platform
  guard, so non-Windows sessions will list it as missing).
- **README.md**: supported agents with mechanism, support level and capability tables from §5.
- **TODO.md**: close the SP2 `.cmd` shim decision (D2); add §6 and §10 known limits.

## 10. Known limits

- Conflict scan false positive and clustered short options (§6).
- Credential isolation is `Conditional` for Claude and Codex: inherited credential variables (D8) are passed
  through unchanged; SP2 neither strips nor warns about them.
- Aider isolation is configuration layering only; the report says so on every launch.
- A new Codex profile starts logged out; the note keys on the home directory being absent, so a present but
  empty home gives no note.
- Setting `[agents.codex] executable` to the vendored `codex.exe` bypasses the npm launcher, which may add
  bundled tools such as `rg` to `PATH`; not measured. An agent with no native executable cannot be launched
  on Windows until its vendor ships one.
- A `.com` beside a `.exe` in the same directory is ignored although `cmd.exe` would prefer it.
- Creating the Aider file needs a no-replace rename or hard links. On a filesystem with neither (for example
  some FUSE or virtual-machine shared folders) Aider profiles fail with `ProfileDir` (exit 4); Claude and Codex
  are unaffected. Behaviour on macOS smbfs, msdos and exfat is not measured.
- Evidence is static; drift detection belongs to `doctor` (later SP).
