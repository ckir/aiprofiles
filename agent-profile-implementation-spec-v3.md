# agent-profile — Detailed Implementation Specification V3

**Status:** Authoritative V3 / v0.1 implementation specification  
**Language:** Rust  
**Supersedes:** `agent-profile-implementation-spec-v2.md`  
**Purpose:** Final normative clarification pass before implementation

---

## 0. V3 Status and Scope

V3 is a **surgical clarification of V2**, not an architectural redesign.

The architecture, security boundary, adapter model, repository-aware resolution model,
`LaunchPlan`, cross-platform process model, and evidence-driven support model remain
unchanged.

V3 exists to remove the remaining implementation-level ambiguities identified by the
V2 audit:

1. CLI command/profile-name collisions;
2. exact `current` semantics;
3. profile deletion versus repository mappings;
4. profile existence for native-profile adapters;
5. idempotent `create`;
6. concurrent lazy initialization;
7. corrupt configuration behavior;
8. atomic configuration replacement failure behavior;
9. Windows process-control acceptance criteria;
10. nested repositories and Git worktree identity;
11. obsolete global-agent-default language;
12. implementation-phase ordering;
13. `repositories prune` scope;
14. launcher return semantics;
15. wrapper-level option placement;
16. adapter mechanism metadata semantics.

### Normative precedence

This document supersedes conflicting or ambiguous earlier V2 language.

The sole v0.1 profile-resolution precedence is:

```text
explicit CLI profile
    >
agent-specific repository mapping
    >
repository-wide mapping
    >
global/default profile
    >
none
```

There is **no global per-agent default** in v0.1.

---

# 1. Product Boundary

`agent-profile` is a local, privacy-first Rust CLI for selecting and launching
profiles for multiple coding agents.

The core rule remains:

> `agent-profile` owns profile selection. The coding agent owns authentication and
> agent-specific configuration.

A profile represents an **agent environment/account context**, but its actual
isolation semantics are adapter-specific.

The product must never claim stronger isolation than the underlying agent officially
supports.

`agent-profile` must not:

- copy OAuth credentials;
- extract tokens;
- create credentials;
- maintain a credential store;
- silently migrate agent credentials;
- make hidden network requests;
- send telemetry;
- trust repository-controlled profile selection automatically.

---

# 2. Supported Agents

V0.1 implements the same 12 evidence-backed adapters defined by V2:

| Agent | Primary mechanism | Semantic tier |
|---|---|---|
| Claude Code | `CLAUDE_CONFIG_DIR` | Full profile/environment isolation, with credential caveats |
| Codex CLI | native `--profile` / `CODEX_HOME` | Native profile |
| Gemini CLI | `GEMINI_CLI_HOME` | Full home/state isolation |
| GitHub Copilot CLI | `COPILOT_HOME` | Full home/config isolation |
| OpenCode | `OPENCODE_CONFIG_DIR` / `OPENCODE_CONFIG` | Config/home isolation |
| Cline CLI | `--config`, `--data-dir` | Config/state selection |
| Pi | `PI_CODING_AGENT_DIR` | Agent home/state isolation |
| Kiro CLI | `KIRO_HOME` | Independent home/profile |
| Cursor Agent CLI | `CURSOR_CONFIG_DIR` | Configuration selection |
| Continue CLI | `--config` | Configuration selection |
| Aider | `--config` | Configuration selection |
| Amp | `--settings-file` | Settings selection |

Every adapter requires an evidence entry and capability declaration.

Evidence must be reverified against the supported upstream version before release.

Do not implement undocumented mechanisms merely because another agent uses a similar
mechanism.

---

# 3. Capability Semantics

Overall adapter support and individual capabilities are separate concepts.

Use capability states such as:

```rust
enum CapabilityState {
    Supported,
    NotSupported,
    NotGuaranteed,
    Conditional,
    Unknown,
}
```

An adapter may be `Proven` while one capability remains `Conditional`,
`NotGuaranteed`, or `Unknown`.

For example, Claude Code configuration-directory isolation must not automatically be
described as universal credential isolation.

Tier-B agents such as Aider, Amp, Continue, and Cursor must not be presented as
guaranteed isolated accounts unless evidence proves that claim.

---

# 4. Core Architecture

The architecture remains:

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

Adapters generate a `LaunchPlan`; adapters do not spawn processes.

```rust
pub struct LaunchPlan {
    pub executable: PathBuf,
    pub args: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
    pub cwd: Option<PathBuf>,
}
```

`LaunchPlan.env` is an **override set**, not a complete environment.

The launcher starts from the inherited environment and applies the adapter's
overrides.

---

# 5. CLI Grammar — Final

## 5.1 Launch syntax

Explicit profile:

```text
agent-profile <agent> <profile> [WRAPPER OPTIONS] [-- <agent args...>]
```

Repository-resolved launch:

```text
agent-profile <agent> [WRAPPER OPTIONS] [-- <agent args...>]
```

Everything after the `--` delimiter is opaque agent input.

The wrapper must not parse or reorder opaque agent arguments except for documented,
adapter-owned profile-conflict detection.

## 5.2 Wrapper options

Wrapper options such as:

```text
--dry-run
--json
--verbose
```

are recognized **only before** the `--` delimiter.

For example:

```bash
agent-profile claude work --dry-run
```

means wrapper dry-run.

```bash
agent-profile claude work -- --dry-run
```

passes `--dry-run` to Claude.

## 5.3 Reserved command words

The following words are reserved CLI command names in v0.1:

```text
agents
profiles
status
list
create
delete
current
resolve
doctor
link
unlink
repositories
completions
```

A profile name must not equal a reserved command word, case-insensitively.

This restriction exists to make the CLI grammar deterministic.

A user who attempts to create such a profile must receive an explicit
`InvalidProfileName` error explaining that the name is reserved.

---

# 6. Profile Name Validation

A profile name must:

- be non-empty;
- begin with an ASCII letter or digit;
- contain only ASCII letters, digits, `.`, `_`, and `-`;
- contain no path separators;
- not equal `.` or `..`;
- contain no control characters;
- contain no NUL;
- not be a reserved CLI command word.

On Windows additionally reject:

```text
CON
PRN
AUX
NUL
COM1 ... COM9
LPT1 ... LPT9
```

case-insensitively, including forms with extensions such as:

```text
CON.txt
COM1.profile
```

Also reject Windows profile names ending in a dot or space.

Validation occurs before:

- configuration writes;
- directory creation;
- deletion;
- linking;
- LaunchPlan generation.

Never rely on the filesystem alone to reject unsafe logical identifiers.

---

# 7. Profile Identity and Storage

The generic profile identity is:

```text
(profile name, agent)
```

The generic storage root is:

```text
<application-root>/profiles/<profile>/<agent>/
```

The application root is platform appropriate and injectable for tests.

Example:

```text
~/.agent-profile/
├── config.toml
└── profiles/
    ├── work/
    │   ├── claude/
    │   ├── codex/
    │   └── gemini/
    └── personal/
        ├── claude/
        └── codex/
```

Do not require every agent directory to exist.

Profile directories are lazily initialized.

---

# 8. Profile Existence — Final Definition

“Profile exists” is an adapter-level concept.

A profile is considered **known** when `agent-profile` can deterministically identify
the profile from its own configuration/metadata or from the adapter's documented
profile mechanism.

A physical directory is not universally required.

Adapters must define:

```text
profile_presence_strategy
```

conceptually:

```rust
enum ProfilePresence {
    Absent,
    Materialized,
    Known,
}
```

The exact internal type may differ.

`profiles` must not imply that a profile is authenticated merely because it is
materialized or known.

For native-profile adapters, existence may be represented by the native profile
mechanism rather than a generic directory.

---

# 9. Lazy Initialization — Final Rule

A valid profile may be launched without a preceding `create` command.

For directory-based adapters:

```bash
agent-profile claude work
```

may create:

```text
<application-root>/profiles/work/claude/
```

automatically.

Lazy initialization must never:

- authenticate;
- copy credentials;
- copy another profile's state;
- modify repository mappings;
- silently overwrite unrelated existing configuration;
- invent unsupported agent configuration.

## 9.1 Concurrency

Two simultaneous launches of the same `(profile, agent)` must converge on the same
valid profile state.

Directory creation must be idempotent.

Equivalent behavior to:

```rust
create_dir_all(...)
```

is required.

If the directory already exists, that is success **only after verifying that the
existing path is actually a directory**.

An ordinary already-exists race must not be treated as fatal.

If an adapter requires multiple initialization resources, initialization must not
report success while exposing an invalid partial state.

V0.1 does not require a separate profile registry file.

---

# 10. `create` — Final Semantics

```bash
agent-profile <agent> create <profile>
```

is an explicit, convenience lifecycle operation.

It must:

- validate the profile;
- validate the adapter;
- create the required profile structure;
- be safe to repeat.

If the profile already exists, `create` succeeds idempotently and reports that the
profile already exists.

`create` must **never**:

- delete existing profile state;
- reset configuration;
- copy another profile;
- authenticate;
- copy credentials;
- silently migrate state.

Authentication remains the underlying agent's responsibility.

---

# 11. `current` — Final Semantics

```bash
agent-profile <agent> current
```

means:

> Show the profile that the normal v0.1 resolution algorithm would select for this
> agent in the current execution context.

It is equivalent to asking the resolver for the current context and displaying only
the selected profile.

It:

- does not launch the agent;
- does not inspect running processes;
- does not remember the last-used profile;
- does not inspect shell history;
- does not create mutable “current profile” state.

If no profile resolves, it reports the same no-profile condition as `resolve`.

Example:

```text
$ agent-profile claude current
work
```

`current` must use the exact same resolver implementation as `resolve` and `status`.

---

# 12. Resolution

The only v0.1 precedence is:

```text
1. explicit CLI profile
2. agent-specific repository mapping
3. repository-wide mapping
4. global/default profile
5. none
```

There is no per-agent global default.

The canonical resolution object is:

```rust
pub struct Resolution {
    pub agent: AgentId,
    pub profile: Option<ProfileName>,
    pub source: ResolutionSource,
    pub repository: Option<PathBuf>,
}
```

with:

```rust
pub enum ResolutionSource {
    Explicit,
    RepositoryAgent,
    RepositoryDefault,
    GlobalDefault,
    None,
}
```

All commands must use the same resolver.

---

# 13. Repository Discovery

When no `--repo` is provided, discovery starts at the current working directory.

For Git:

- identify the repository/worktree associated with the current directory;
- canonicalize the discovered repository root;
- do not assume `.git` is always a physical directory.

If the current directory is inside a Git submodule, the submodule is the current
repository.

Do not silently apply a parent repository's mapping to an independent nested
repository.

If no repository is found, normal global/default fallback applies.

---

# 14. Repository Mapping Identity

Repository mappings are keyed by the canonical absolute repository/worktree root.

Use:

```rust
std::fs::canonicalize(...)
```

or an equivalent platform-aware implementation.

Do not store merely the user's symlink spelling.

Example:

```text
/Users/me/projects/acme-link
    ↓ symlink
/Volumes/work/acme
```

stores:

```text
/Volumes/work/acme
```

as the repository identity.

## 14.1 Nested mappings

A mapping is applicable only when its repository context legitimately applies to the
current repository.

Among applicable mappings, the longest canonical repository path match wins.

## 14.2 Worktrees

For v0.1, repository identity is based on the canonical filesystem/worktree root,
not an abstract Git object-database identity.

Therefore separate worktrees may have separate repository mappings.

The implementation must use Git's repository/worktree metadata to discover the actual
working-tree root.

---

# 15. Repository Linking

The canonical binding commands are:

```bash
agent-profile link <profile>
agent-profile unlink
agent-profile <agent> link <profile>
agent-profile <agent> unlink
```

Meanings:

```text
link <profile>
    current repository → repository-wide profile

<agent> link <profile>
    current repository → agent-specific profile
```

Explicit profile invocation always overrides these mappings.

Linking requires successful canonicalization of the repository root.

No repository-controlled configuration file is trusted automatically in v0.1.

---

# 16. Profile Deletion — Final Rule

```bash
agent-profile <agent> delete <profile>
```

is destructive.

It requires confirmation unless `--yes` is supplied.

Before deletion:

1. validate profile and agent identifiers;
2. resolve the canonical profile path;
3. verify that the path is inside the application profile root;
4. check repository mappings;
5. require confirmation where applicable;
6. perform safe deletion.

## 16.1 Referenced profiles

A profile that is referenced by any repository mapping must **not** be deleted in
v0.1.

Deletion must fail and explain which cleanup is required:

```text
Cannot delete profile 'work':
it is referenced by one or more repository mappings.

Unlink the profile first, then retry deletion.
```

The command must not automatically remove mappings.

This avoids an implicit multi-object destructive operation.

## 16.2 Running processes

V0.1 does not enumerate all running processes.

If the OS refuses deletion because files are in use:

- report a recoverable deletion failure;
- do not report full deletion;
- return non-zero.

On Windows, locked-file errors are deletion failures, not permission to bypass locks.

No deletion implementation may escape the profile root.

---

# 17. Configuration Corruption

The main configuration is:

```text
<application-root>/config.toml
```

For all commands:

> A configuration parse failure is an error. Never silently replace invalid
> configuration with defaults.

For read-only operations:

- report the configuration error;
- do not silently fall back to empty configuration.

For read-modify-write operations:

- refuse the modification;
- leave the corrupt file untouched;
- identify the configuration file;
- provide recovery guidance.

The application must never “repair” unknown corruption by overwriting the file.

---

# 18. Configuration Concurrency

Configuration reads do not require a read lock.

Configuration writes use:

```text
exclusive lock
    ↓
read latest config
    ↓
modify
    ↓
write temporary file in same directory/filesystem
    ↓
flush/sync as appropriate
    ↓
atomic replacement
    ↓
release lock
```

Readers may race with writers and observe either the previous complete configuration
or the new complete configuration.

They must never intentionally consume a partially written file.

## 18.1 Replacement failure

If atomic replacement cannot be guaranteed by the platform/filesystem:

- fail safely;
- preserve the previous valid configuration;
- do not claim the write succeeded;
- do not silently fall back to a non-atomic overwrite.

Temporary files must not become active configuration.

Interrupted/failed temporary files may be cleaned later, but must never be interpreted
as the active configuration.

---

# 19. Windows Path Semantics

V0.1 distinguishes:

1. **identity path** — canonical path used for repository identity;
2. **operational path** — compatible representation passed to an agent/process.

Canonical Windows paths may contain extended forms such as:

```text
\\?\C:\Users\me\src\project
```

An adapter must not blindly pass an identity path to an agent.

If an agent requires a compatible operational representation, the adapter may derive
one without changing repository identity.

Path normalization must not collapse distinct paths incorrectly.

Adapter-specific path compatibility belongs in adapter metadata and `doctor`.

---

# 20. Executable Discovery

Search order:

1. explicit configured executable;
2. supported repository-local executable mechanism;
3. current `PATH`;
4. explicitly documented platform-standard locations;
5. not installed.

At minimum v0.1 supports:

- `PATH`;
- explicit executable override.

Repository-local discovery must be explicitly declared by an adapter and must not
blindly scan arbitrary directories.

`doctor` distinguishes:

```text
Not found in PATH
Explicit executable configured but missing
Repository-local executable detected
```

---

# 21. Adapter Argument Conflicts

The wrapper must not implement a complete parser for any coding agent.

Adapters may declare a shallow set of known profile-conflicting options.

Example:

```text
--profile
--profile=<value>
```

If a user invokes:

```bash
agent-profile codex work -- --profile personal
```

and the adapter has proven that `--profile` controls the same profile mechanism,
the wrapper must fail before launch.

If the wrapper cannot prove a conflict, it must not guess.

Unknown arguments remain opaque.

---

# 22. Environment Overrides

For environment-based adapters:

```text
inherited environment
        +
adapter overrides
        ↓
child environment
```

Wrapper-owned profile variables win over inherited values.

Example:

```text
inherited:
CLAUDE_CONFIG_DIR=/wrong

selected:
CLAUDE_CONFIG_DIR=<work-profile>
```

The child receives the selected value.

The parent shell is never modified.

`--verbose` and `--dry-run` may show non-sensitive override decisions.

Secret-bearing environment values must not be printed.

---

# 23. Launch Semantics

## 23.1 Unix

Where supported, successful execution uses process replacement (`exec`).

The wrapper process does not remain alive after successful `exec`.

Therefore the observable result of:

```bash
agent-profile claude work
```

must have the same exit status and direct process behavior as the launched agent,
apart from the profile-specific environment/arguments.

If `exec` fails, report a launch error and exit non-zero.

No post-process logging may require keeping the wrapper alive on Unix.

## 23.2 Windows

Windows uses normal direct child-process creation.

The launcher must:

1. create the agent directly;
2. inherit stdin/stdout/stderr;
3. wait for the child;
4. return the child's exit code as the wrapper's exit code;
5. deliberately handle console control events.

No shell mediation is permitted.

---

# 24. Windows Control-Event Acceptance Criteria

The implementation mechanism is intentionally left to the Windows-specific launcher,
but the **observable behavior is normative**.

Tests must cover:

- normal child completion;
- child non-zero exit;
- child creation failure;
- Ctrl-C;
- Ctrl-Break where applicable;
- wrapper termination before child creation;
- no unintended orphaned child after interruption.

For interactive Ctrl-C:

> The active agent should experience interruption in substantially the same practical
> manner as a direct invocation of that agent, subject to Windows console semantics.

If exact signal/control-event propagation differs from direct invocation because of
Windows process-group rules, the difference must be documented and tested.

Do not use:

```text
cmd.exe /C
powershell.exe -Command
```

as a generic signal-forwarding mechanism.

The agent must still be launched directly.

---

# 25. Launcher API

The conceptual V2 API:

```rust
trait ProcessLauncher {
    fn launch(&self, plan: LaunchPlan) -> Result<ExitStatus, LaunchError>;
}
```

is retained as an implementation option, but its Unix semantics are clarified:

> On Unix, successful `exec` does not return. An `ExitStatus` is therefore returned
> only by implementations that actually wait for a child process.

Implementations may instead use:

```rust
enum LaunchOutcome {
    ReplacedProcess,
    Exited(ExitStatus),
}
```

or an equivalent abstraction.

The specification does not require a particular Rust type as long as observable
behavior is correct.

---

# 26. Dry Run

Wrapper-level dry run is:

```bash
agent-profile claude work --dry-run
```

It is recognized only before `--`.

Dry run must never launch the agent.

It should display:

- agent;
- selected profile;
- executable;
- repository;
- adapter mechanism;
- environment overrides;
- final arguments.

Sensitive values must be redacted.

After `--`, `--dry-run` is agent input.

---

# 27. `repositories` Command

V0.1 provides:

```bash
agent-profile repositories
```

as a read-only mapping report.

It reports active/missing/orphaned mapping paths where useful.

V0.1 does **not** implement:

```bash
agent-profile repositories prune
```

Pruning is future functionality.

There is no background garbage collection.

Orphaned mappings are reported but never silently removed.

---

# 28. Adapter Mechanism Versioning

Each adapter must maintain evidence metadata conceptually equivalent to:

```rust
pub struct AdapterEvidence {
    pub mechanism_id: &'static str,
    pub verified_at: &'static str,
    pub notes: &'static str,
}
```

Mechanism IDs are adapter metadata, not part of the generic profile model.

They exist for diagnostics and release verification.

If an adapter changes materially:

- `doctor` may report that the mechanism changed;
- existing profiles must not be silently reinterpreted;
- credentials/state must not be automatically migrated.

V0.1 does **not** require a migration engine.

Unless an adapter explicitly declares an old mechanism unusable, a mechanism mismatch
is diagnostic rather than an automatic migration trigger.

---

# 29. Doctor

`doctor` checks:

- executable availability;
- executable path;
- platform;
- adapter support;
- profile validity;
- profile path;
- writability;
- expected environment overrides;
- expected arguments;
- safe version detection;
- compatibility;
- credential-isolation caveats;
- repository mapping;
- effective resolution;
- known configuration conflicts;
- mechanism-version diagnostics;
- relevant path permissions.

`doctor` must never print secrets.

It must distinguish:

```text
supported
not guaranteed
conditional
unknown
```

rather than inventing guarantees.

---

# 30. Detection

Detection is lightweight.

Preferred behavior:

- locate executable;
- optionally run a safe non-interactive version command;
- never launch an interactive coding agent for detection.

A failed version probe is not equivalent to “not installed.”

---

# 31. Repository-Profile Deletion Invariant

The following state transition is forbidden:

```text
mapping → profile
delete profile
mapping remains silently dangling
```

V0.1 therefore requires:

```text
referenced profile
    ↓
delete
    ↓
FAIL
```

User must explicitly unlink first.

This rule applies to both repository-wide and agent-specific mappings.

---

# 32. Output

Human-readable output is the default.

Machine-readable JSON is required for:

```text
agents
profiles
status
resolve
doctor
```

JSON schemas must remain stable within a release series.

Never include:

- tokens;
- authorization headers;
- secret environment values;
- credential contents.

---

# 33. Exit Codes

Recommended v0.1 exit codes:

```text
0 success
1 general error
2 CLI usage error
3 agent not installed
4 profile/configuration error
5 doctor detected failure
6 launch failed
```

Launched-agent exit status takes precedence over wrapper success/error semantics when
the agent has successfully started.

On Unix this naturally follows from process replacement.

On Windows the wrapper explicitly exits with the child exit code.

---

# 34. Testing — Mandatory Contracts

The common contract suite must test:

## Profile

- valid names;
- invalid names;
- reserved command names;
- Windows reserved device names;
- trailing dots/spaces;
- path traversal;
- lazy initialization;
- concurrent initialization;
- idempotent create;
- delete;
- deletion of referenced profiles;
- deletion while files are in use.

## Resolution

```text
explicit
> repository agent
> repository default
> global default
> none
```

Test:

- `resolve`;
- `current`;
- `status`;
- symlinked repositories;
- nested repositories;
- submodules;
- worktrees;
- longest applicable mapping;
- no repository;
- canonicalization failure.

## Configuration

- valid TOML;
- invalid TOML;
- corrupt configuration;
- concurrent writers;
- stale writer races;
- atomic replacement;
- failed replacement;
- temporary-file cleanup;
- preservation of previous valid configuration.

## LaunchPlan

For every adapter:

```text
executable
args
environment overrides
cwd
capabilities
conflict rules
```

## Passthrough

Test:

```bash
agent-profile fake work -- --foo bar
```

and verify exact opaque argument preservation.

Test known conflicts such as:

```bash
agent-profile codex work -- --profile personal
```

where proven by the Codex adapter.

## Environment

Verify:

```text
inherited environment
+
wrapper override
=
child environment
```

and verify that the parent environment is unchanged.

## Process behavior

macOS/Linux:

- Unix process replacement;
- exit status;
- stdin/stdout/stderr inheritance.

Windows:

- child exit status;
- Ctrl-C;
- Ctrl-Break where applicable;
- child creation failure;
- no unintended orphan;
- no shell mediation.

---

# 35. Implementation Order — Final

The previous “implement all 12 immediately” wording is replaced by a staged adapter
gate.

## Phase 1 — Foundation

- Cargo project;
- Clap CLI;
- application root;
- TOML configuration;
- profile-name validation;
- filesystem safety;
- structured errors.

## Phase 2 — Resolution

- repository discovery;
- canonical repository identity;
- mapping storage;
- precedence;
- `resolve`;
- `current`;
- `status`.

## Phase 3 — Launcher

- `LaunchPlan`;
- direct process launch;
- Unix replacement;
- Windows child process;
- passthrough;
- environment overrides;
- dry run;
- exit-code handling.

## Phase 4A — Architecture proof

Implement:

1. Claude Code;
2. Codex CLI;
3. Aider.

Each must pass the common contract suite.

This phase is the architecture gate.

## Phase 4B — Remaining adapters

After Phase 4A passes:

4. Gemini CLI;
5. GitHub Copilot CLI;
6. OpenCode;
7. Cline CLI;
8. Pi;
9. Kiro CLI;
10. Cursor Agent CLI;
11. Continue CLI;
12. Amp.

Each adapter requires:

- current evidence;
- capability declaration;
- adapter contract tests;
- conflict/environment rules where applicable;
- doctor support.

## Phase 5 — Lifecycle

- create;
- list;
- delete;
- link/unlink;
- repository report.

## Phase 6 — Quality

- doctor;
- JSON;
- shell completion;
- cross-platform CI;
- evidence registry;
- README;
- roadmap.

---

# 36. Security Gate

Before v0.1 release, verify that `agent-profile` itself never:

- copies OAuth credentials;
- extracts tokens;
- creates credentials;
- stores credentials;
- silently migrates credentials;
- prints secrets;
- trusts repository-controlled profile configuration;
- performs hidden network requests;
- sends telemetry.

Underlying agents remain free to authenticate and access their own services.

That activity is outside `agent-profile`'s security boundary.

---

# 37. Final Definition of Done

V0.1 is implementation-complete only when:

### CLI

- [ ] grammar is deterministic;
- [ ] reserved command/profile collisions are rejected;
- [ ] passthrough is exact;
- [ ] wrapper options are separated from agent options;
- [ ] `current` uses the canonical resolver;
- [ ] exit behavior is correct;
- [ ] dry-run works.

### Profiles

- [ ] profile names are portable and safe;
- [ ] Windows reserved names are rejected;
- [ ] lazy creation is idempotent;
- [ ] concurrent initialization converges safely;
- [ ] `create` is non-destructive and idempotent;
- [ ] referenced profiles cannot be deleted;
- [ ] deletion cannot escape the profile root;
- [ ] failed deletion is never reported as success.

### Repository

- [ ] canonical roots are stored;
- [ ] symlink paths resolve correctly;
- [ ] nested repositories are handled;
- [ ] submodules are independent repositories;
- [ ] worktrees are handled correctly;
- [ ] applicable mapping selection is deterministic;
- [ ] orphan mappings are detectable;
- [ ] no automatic pruning occurs.

### Configuration

- [ ] invalid configuration is never silently replaced;
- [ ] writes use exclusive synchronization;
- [ ] read-modify-write reloads current state under lock;
- [ ] temporary files are on the same filesystem as the target;
- [ ] replacement is atomic where guaranteed;
- [ ] failed replacement preserves the previous valid file;
- [ ] readers never intentionally observe partial TOML.

### Adapters

- [ ] all 12 adapters have evidence;
- [ ] mechanisms are documented;
- [ ] capabilities are separate from support level;
- [ ] credential claims are scoped;
- [ ] conflict rules are adapter-owned;
- [ ] environment overrides are adapter-owned;
- [ ] detection is non-interactive;
- [ ] doctor exposes important caveats;
- [ ] mechanism changes are diagnosable.

### Process

- [ ] Unix uses process replacement where supported;
- [ ] Windows launches directly;
- [ ] stdin/stdout/stderr are inherited;
- [ ] child exit status is preserved;
- [ ] Windows control-event behavior is tested;
- [ ] no shell mediation is used;
- [ ] no unintended child orphaning occurs.

### Security

- [ ] no credential copying;
- [ ] no token extraction;
- [ ] no secret logging;
- [ ] no credential manager;
- [ ] no telemetry;
- [ ] no hidden network activity;
- [ ] no automatic repository-controlled profile selection.

### CI

CI must cover:

```text
macOS
Linux
Windows
```

with fake-agent contract tests and the complete cross-platform suite.

---

# 38. Final Implementation Principle

The project must optimize for:

```text
predictability
+
correctness
+
transparent behavior
+
minimal privilege
+
evidence-backed support
```

not feature count.

`agent-profile` should remain a thin wrapper.

It must never become:

- a second authentication system;
- a shell manager;
- a configuration-normalization framework;
- a hidden process supervisor.

---

# 39. Final V3 Decision

**V3 is the final normative implementation specification for v0.1.**

The architecture is frozen.

Implementation should begin with:

```text
Claude Code
Codex CLI
Aider
```

and the common contract tests.

After those adapters pass the architecture gate, implement the remaining nine adapters.

No further specification redesign is required for v0.1 unless implementation uncovers a
previously unknowable upstream-agent behavior or a concrete platform limitation.

Such a discovery must be handled as an evidence-backed change to the affected adapter,
not by silently changing generic profile semantics.

**Final rule:**

> `agent-profile` owns profile selection.  
> The coding agent owns authentication and agent-specific configuration.  
> The evidence determines what `agent-profile` is allowed to claim.
