# SP1 — Core and explicit launch: design

**Status:** approved by the owner on 2026-09-14 after six panel rounds; implemented by
`docs/superpowers/plans/2026-09-14-sp1-core-launch.md`.
**Branch:** `sp1-core-launch` (from `main` at `204efb7`).
**Oracle:** `agent-profile-implementation-spec-v3.md` (called "V3" below). Where this document and V3
disagree, V3 wins; report the conflict instead of resolving it silently.
**Previous sub-project:** SP0, `docs/superpowers/specs/2026-09-13-sp0-scaffold-design.md`.

## 1. Goal

SP1 delivers V3 §35 phases 1 (foundation) and 3 (launcher). It ends with the explicit launch path
`agent-profile <agent> <profile> [options] [-- args]` proven end-to-end against the real binary on Linux,
macOS and Windows CI, without pre-empting the SP2 adapter model and without shipping any agent to users.

## 2. Scope

### 2.1 In scope

| Area | V3 |
|---|---|
| Profile-name validation | §5.3, §6 |
| Application root, injectable for tests | §7 |
| `config.toml` strict reads and a library-level locked atomic writer | §17, §18, §18.1 |
| Structured errors mapped to exit codes | §33 |
| `LaunchPlan`, `LaunchOutcome` | §4, §25 |
| Executable discovery: explicit override and `PATH` | §20 |
| Environment overrides | §22 |
| Lazy profile-directory creation | §9, §9.1 |
| Unix `exec` launcher; Windows child launcher with control-event handling | §23, §24 |
| Passthrough and wrapper-level dry run | §5.1, §5.2, §26 |
| A `fake` agent, compiled only under `cfg(debug_assertions)` | §34 ("Passthrough") |

### 2.2 Out of scope

| Area | Lands in |
|---|---|
| Adapter trait, capability and evidence model, contract suite, real adapters | SP2 |
| Adapter-owned argument-conflict rules (§21) | SP2 |
| How to launch npm-style `.cmd`/`.bat` shims without shell mediation | SP2 (open decision, §10 step 3) |
| Repository discovery, mappings, global default, `resolve`, `current`, `status`, `link`, `unlink` | SP3 |
| `create`, `delete`, `list`, `repositories`, `doctor`, JSON output, completions | SP5 |

## 3. Decisions and their evidence

Each fork was consulted with the agy peer first (AGY-FIRST); the owner decided.

| # | Decision | agy | Evidence |
|---|---|---|---|
| D1 | SP1 launches a `fake` agent compiled only under `cfg(debug_assertions)`. Integration tests spawn the real `agent-profile` binary; release builds (`release.yml`, `cargo install`) contain no agent. | ALIGNED | `[profile.ci]` in `Cargo.toml` inherits `dev`, so CI test builds keep debug assertions. |
| D2 | Application root is `~/.agent-profile` (`%USERPROFILE%\.agent-profile` on Windows), overridden by `AGENT_PROFILE_HOME`. No directories crate. | recommended platform dirs; owner chose the V3 §7 example | Same `<TOOL>_HOME` convention as `CARGO_HOME`, `RUSTUP_HOME`, `CODEX_HOME`. |
| D3 | Windows: cargo's control handler pattern plus a self-assigned `KILL_ON_JOB_CLOSE` job whose flag is cleared before a normal exit. | NEGOTIATE, then ALIGNED after measurement | M1, M2 in §9. Cargo's Windows `exec_replace` (`crates/cargo-util/src/process_builder.rs`) installs a handler returning TRUE. Microsoft: the NULL-handler "ignore" attribute is inherited by child processes. |
| D4 | Config writes: `std::fs::File::lock` on a separate `config.toml.lock`, `tempfile` in the same directory, sync, rename; any failure fails the write. | ALIGNED | `File::lock` is stable since Rust 1.89 (MSRV 1.98 compiles it, measured). `std::fs::rename` on Windows uses `FileRenameInfoEx` POSIX semantics where supported. |
| D5 | Writes use `toml_edit`, preserving comments, key order and formatting. | ALIGNED | V3 §17. |
| D6 | CLI parsing: Clap derive for top-level reserved words, `external_subcommand` capturing `<agent> …`, a hand-written splitter for the rest. | ALIGNED (consult breached review-only; owner waived the re-run; driver re-verified) | Measured: `external_subcommand` yields `["claude","work","--dry-run","--","--dry-run"]` verbatim. |
| D7 | Unknown keys inside a known configuration table are a configuration error. | recommended lenient reads; owner chose strict | A misspelled key fails loudly instead of surfacing as "agent not installed". |
| D8 | Executable discovery refuses `.bat` and `.cmd`. | ALIGNED | `library/std/src/sys/process/windows.rs` runs batch files through `cmd.exe` (V3 §23.2 forbids shell mediation). |
| D9 | A profile whose name differs from an existing profile directory only in letter case is refused (§7.3). | not consulted (peer on quota failure; found by the round-4 subagent panel); owner chose refusal over case-folding, lowercase-only names, or documenting the limit | Case-insensitive default filesystems on Windows and macOS would otherwise merge two profiles into one directory. |

## 4. Command surface

### 4.1 Grammar

```text
agent-profile <agent> <profile> [WRAPPER OPTIONS] [-- <agent args...>]
agent-profile <agent> [WRAPPER OPTIONS] [-- <agent args...>]
agent-profile <reserved-top-level-word> ...
agent-profile <agent> <reserved-agent-word> ...
agent-profile --help | --version
```

Wrapper options: `--dry-run`, `--verbose`, `--json`, `-h`/`--help`, `-V`/`--version` (after the agent word,
`--version` prints the version and exits 0, exactly like the top-level form). Options placed BEFORE the agent
word are parsed by Clap at the top level, which knows only `--help` and `--version`; anything else there is
Clap's own usage error (exit 2).

### 4.2 Parsing rules

1. Clap parses the top level with `disable_help_subcommand` (so `help` is not a hidden command word) and
   `allow_external_subcommands`. The reserved words of V3 §5.3 (`agents profiles status list create delete
   current resolve doctor link unlink repositories completions`) are Clap subcommands. Each one accepts
   and ignores any further arguments.
2. Any other first word is captured, with everything after it, by `#[command(external_subcommand)]` as
   `Vec<OsString>`. Its first element is the agent word. A non-UTF-8 agent word is `UnknownAgent`, rendered
   lossily in the message.
3. The splitter cuts the remaining elements at the FIRST `--`. Everything after that `--` is opaque
   agent input: kept as `OsString`, never inspected, reordered or re-encoded. Later `--` tokens are opaque.
   The elements before the cut are the *pre-cut elements*. A pre-cut element that begins with `-` is an
   *option token*; every other pre-cut element, including the empty string, is a *bare word*.
4. The splitter then applies these checks IN THIS ORDER; the first that matches decides the outcome:
   1. Any option token is `-h` or `--help`: launch-form usage text, exit 0.
   2. Any option token is `-V` or `--version`: the version, exit 0.
   3. The agent word is not a known agent of this build: `UnknownAgent` (exit 2). Matching is exact and
      case-sensitive (`Fake` is not `fake`, consistent with the lowercase `AgentId` syntax); a non-UTF-8
      agent word never matches and is rendered lossily. Only to fill `unknown_configured`, this path makes
      one best-effort attempt at §6.1 root resolution and §6.2 configuration load before building the error;
      any failure of that attempt leaves the list empty and never changes the error or its exit code.
   4. The first bare word equals a reserved word (ASCII case-insensitive): `NotYetImplemented` for
      "`<agent> <word>`" (exit 2). Every other pre-cut element, option tokens included, is ignored, so
      `fake create work` and `fake create --bogus` both give `NotYetImplemented`.
   5. Any option token is not one of `--dry-run`, `--verbose`, `--json`: `Usage` (exit 2).
   6. There are two or more bare words: `Usage` "agent arguments must follow `--`" (exit 2).
   7. `--json` is present: `NotYetImplemented` for "`--json`" (exit 2).
   8. The single bare word is not valid UTF-8: `Usage` (exit 2).
   9. Otherwise the invocation is a launch: no bare word means no explicit profile (§5.1 step 4 then gives
      `NoProfile`, exit 4); one bare word is the explicit profile, validated by V3 §6 in §5.1 step 2 (the
      empty string fails with `InvalidProfileName`, exit 4).

### 4.3 Behaviour in SP1

| Invocation | Result | Exit |
|---|---|---|
| No arguments at all | Clap prints the top-level help to stderr | 2 |
| Top-level reserved word | "`<word>` is not yet implemented" | 2 |
| `<agent> <reserved word> ...` | "`<agent> <word>` is not yet implemented" | 2 |
| Unknown agent | Usage error listing the known agents; a build with none says "no agents are available in this build". If the configuration loads and has `agents.<id>` tables for ids this build does not know, the message lists them too; a configuration that fails to load does not change this error. | 2 |
| `--json` anywhere before `--` | "`--json` is not yet implemented" | 2 |
| Unknown option before `--`, or a second bare word | Usage error | 2 |
| `<agent> --help`, `<agent> --version` | Launch-form usage text; version | 0 |
| `<agent>` with no profile | The resolver stub returns `ResolutionSource::None`; error "no profile selected for `<agent>`" (V3 §11 no-profile condition) | 4 |
| `<agent> <invalid profile>` | `InvalidProfileName`, stating the reason | 4 |
| `<agent> <profile>` whose name differs only in case from an existing `profiles/` entry | `ProfileCaseConflict` naming the existing entry (§7.3), also under `--dry-run` | 4 |
| `<agent> <profile> --dry-run` | Dry-run report (§7.4) | 0 |
| `<agent> <profile>` | Launch (§7) | agent's status |

When several rows apply, the order of checks in §4.2 rule 4 decides, then the order of steps in §5.1
(profile validation, application root, configuration, resolution, then §5.1 step 5's sub-steps).

## 5. Module layout

The SP0 library already has one empty module per V3 §4 layer. SP1 fills them and adds `error` and `exe`.

| Module | Responsibility | Depends on |
|---|---|---|
| `name` | `ProfileName` and `AgentId` newtypes. `ProfileName::parse(&str, Platform)` implements V3 §6; `Platform::{Unix, Windows}` selects the Windows-only rules, and runtime code passes `Platform::host()`. `AgentId` syntax: `[a-z][a-z0-9-]*`. | — |
| `error` | `Error` (thiserror) and `Error::exit_code()` (§6.3). | — |
| `config` | `AppRoot` (§6.1), `Config` strict read (§6.2), `config::update` writer (§6.4). | `error` |
| `exe` | Executable discovery (§7.2). | `config`, `error` |
| `resolve` | V3 §12 `Resolution` and `ResolutionSource` types verbatim; SP1 stub `resolve(agent, explicit)` returns `Explicit` or `None`. SP3 replaces the body, not the types. | `name` |
| `adapter` | SP1-internal `plan(&AgentId, &Resolution, &AppRoot, &Config, args: Vec<OsString>, path_var: Option<&OsStr>) -> Result<PlannedLaunch>` plus `ensure_profile_dir(&PlannedLaunch)` and `known_agents()`; one arm, `fake`, under `cfg(debug_assertions)`; owns the case-only-twin check (§7.3). No trait: SP2 designs it. | `exe`, `config`, `launch`, `resolve`, `name`, `error` |
| `launch` | `LaunchPlan` (V3 §4 struct, unchanged), `LaunchOutcome`, `launch(&LaunchPlan, verbose: bool)`, `launch/unix.rs`, `launch/windows.rs`. | `error` |
| `output` | Dry-run and `--verbose` rendering, environment-value redaction. | `adapter` (`PlannedLaunch`), `resolve`, `launch` |
| `cli` | Clap types, splitter, dispatch; `pub fn run(args: impl IntoIterator<Item = OsString>) -> i32`. | all |

**Public test surface.** Integration tests in `tests/` see only the public API, so these items are `pub`:
every module listed above (`pub mod`); `LaunchPlan` with all four fields public (V3 §4) and
`LaunchPlan::command`; `config::AppRoot::from_path(PathBuf) -> AppRoot` (tests build roots for temp
directories without touching the process environment, since `std::env::set_var` is `unsafe` in edition
2024); `config::AppRoot::resolve() -> Result<AppRoot>` (§6.1); `config::Config::load(&AppRoot) ->
Result<Config>` with `Config::agent_executable(&self, id: &str) -> Option<&Path>`; and
`config::update(&AppRoot, edit) -> Result<()>`. Crate-private, for unit tests inside `config`:
`update_with(root: &AppRoot, edit, before_persist: impl FnOnce() -> Result<()>, replace: impl
FnOnce(tempfile::NamedTempFile, &Path) -> std::io::Result<()>) -> Result<()>`, where `update` calls it with a
no-op `before_persist` and `|tmp, dest| tmp.persist(dest).map(drop).map_err(|e| e.error)` as `replace`
(`NamedTempFile::persist` consumes the temp file; on failure the returned `PersistError` is dropped, which
deletes the temp file, so a failing injected `replace` exercises the same cleanup path).

`PlannedLaunch` carries the `LaunchPlan` plus what dry run reports but the launcher does not need: the
validated profile, the profile directory, whether it exists, the executable's origin (`Configured` or `Path`) and a one-line
mechanism description.

`main.rs` becomes `std::process::exit(agent_profile::cli::run(std::env::args_os()))`; this is the ONLY
exit path. The Windows launcher returns `LaunchOutcome::Exited(code)` up through `run`, which returns
`code`; on Unix a successful `exec` never returns. Errors print as
`agent-profile: error: <message>` on stderr.

### 5.1 Launch data flow

1. Parse (§4).
2. If a profile word was given, validate it (V3 §6) before anything touches the filesystem.
3. Resolve the application root; load the configuration.
4. `resolve::resolve`. (From SP3 a resolver may also produce a profile from a mapping; a name read from
   configuration is validated when it is read, so every `ProfileName` reaching step 5 is valid by type.)
5. `adapter::plan`, in this order: (a) discover the executable (§7.2; exit 3 or 6 on failure); (b) the
   case-only-twin check (§7.3; exit 4); (c) compute `<root>/profiles/<profile>/fake`; (d) build
   `LaunchPlan { executable, args: opaque, env: [("FAKE_AGENT_HOME", dir)], cwd: None }`.
6. With `--dry-run`: render the report and exit 0. Nothing is created. Executable discovery still runs, so
   a dry run of an agent that is not installed fails with exit 3, exactly like a launch.
7. Otherwise: lazily create the profile directory (§7.3); with `--verbose`, render the report to stderr;
   launch (§7).

## 6. Application root, configuration and errors

### 6.1 Application root

- `AGENT_PROFILE_HOME` set: it must be non-empty and an absolute path; that path is the root.
- Unset: `std::env::home_dir()` joined with `.agent-profile`.
- Empty or relative `AGENT_PROFILE_HOME`, or no home directory: `AppRoot` error (exit 4) advising to set
  `AGENT_PROFILE_HOME` to an absolute path.
- Resolving the root never creates it.

### 6.2 Configuration file and schema

Path: `<root>/config.toml`. SP1 schema:

```toml
[agents.fake]
executable = "/absolute/path/to/fake-agent"
```

- Top level: only the `agents` table is allowed.
- `agents`: keys must be valid `AgentId`s. Any valid id is accepted, including agents this build does not
  know, so a configuration written for later versions' agents does not fail.
- Each `agents.<id>` table: only `executable` is allowed; it is a string holding an
  absolute path.
- A missing file is an empty configuration. A missing `agents` table or agent table means "no override".
- Everything else is `ConfigInvalid` (exit 4): unreadable file, non-UTF-8 content, TOML syntax error, unknown
  key, wrong type, invalid agent id, relative `executable`. The message names the file, the key when there is
  one, and says: "agent-profile never rewrites an invalid configuration; fix or move the file."
- Reads never take the lock and open only `config.toml`.
- Implementation note (from the verified prototype): validation walks the parsed `toml::Table` rather than
  deserializing with serde `deny_unknown_fields`, because the walk can name the exact offending key for every
  error class; `serde` is therefore not a dependency.

### 6.3 Errors and exit codes

| Variant | Meaning | Exit |
|---|---|---|
| `Usage { message }` | Grammar violation (§4.2) | 2 |
| `NotYetImplemented { command }` | Reserved command or `--json` | 2 |
| `UnknownAgent { agent, known, unknown_configured }` | Agent word not in this build; `unknown_configured` lists `agents.<id>` tables for ids this build does not know (empty when the configuration did not load) | 2 |
| `AgentNotInstalled { agent, reason, unknown_configured }` | `reason` is `NotOnPath` or `ExplicitMissing(path)`; `unknown_configured` as above | 3 |
| `InvalidProfileName { name, reason }` | V3 §6 | 4 |
| `NoProfile { agent }` | Resolution returned `None` | 4 |
| `AppRoot { message }` | §6.1 | 4 |
| `ConfigInvalid { path, key, detail }` | §6.2 | 4 |
| `ConfigWrite { path, source }` | Lock, temp-file, sync or replace failure | 4 |
| `ProfileDir { path, source }` | Cannot create, or exists but is not a directory | 4 |
| `ProfileCaseConflict { requested, existing }` | An entry in `<root>/profiles/` whose name differs from the requested profile only in ASCII letter case already exists (§7.3) | 4 |
| `UnsupportedExecutable { path }` | `.bat` or `.cmd` | 6 |
| `Launch { executable, source }` | `exec` or spawn failure; Windows job or handler setup failure | 6 |
| `Io { context, source }` | Anything else, e.g. writing the dry-run report fails | 1 |

Once the agent has started, its exit status is the wrapper's exit status (V3 §33).

### 6.4 Locked atomic writer

`config::update(root: &AppRoot, edit: impl FnOnce(&mut toml_edit::DocumentMut) -> Result<()>) -> Result<()>`,
library-level only in SP1 (no CLI command writes configuration yet).

1. Create the root directory if missing.
2. Open or create `<root>/config.toml.lock` and acquire the exclusive lock with `File::try_lock()`, retrying
   every 50 ms for at most 10 s. Timeout: `ConfigWrite` for the lock path, whose message reads "could not write
   `<root>/config.toml.lock`: another agent-profile process holds the configuration lock; retry, or check for a
   stuck process". Any other error: `ConfigWrite`. Nothing is
   written in either case. The lock file is never deleted or replaced.
3. Delete files in the root named `.config.toml.*.tmp`. Under the lock no writer is active, so they are
   leftovers of crashed writers. The sweep is best-effort: a deletion error is ignored (and mentioned only
   under `--verbose` once a CLI command writes), so an undeletable leftover never blocks a write.
4. Read `config.toml` (missing = empty document), parse it with `toml_edit` and validate it against §6.2.
   Invalid: `ConfigInvalid`, file untouched.
5. Apply `edit`; validate the result against §6.2. Invalid: `ConfigInvalid`, file untouched.
6. Create a `tempfile::Builder` temp file in the root with prefix `.config.toml.` and suffix `.tmp`; write
   the document; `sync_all`.
7. `persist` onto `config.toml`. A `persist` error: `ConfigWrite`; the temp file is removed; the previous
   `config.toml` is untouched. There is no non-atomic fallback (V3 §18.1).
7a. After a successful `persist`, on Unix open the root directory and `sync_all` it. The new configuration
   is already the active file at this point, so a failure here must NOT claim the old file survived: it is
   `ConfigWrite` with the message "configuration replaced, but the directory could not be synced; the change
   may not survive a power loss".
8. Release the lock (drop).

**§18.1 assumption, documented:** SP1 treats `std::fs::rename` as the platform's atomic replace on all three
OSes. No Microsoft document states that `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` is atomic; on Windows 10
1607+ with `FileRenameInfoEx` support, std uses POSIX rename semantics.

## 7. Launch

### 7.1 Shared

`LaunchPlan::command(&self) -> std::process::Command` is the single conversion both launchers use:
`executable`; `args` as given; `.env(k, v)` for each override
(the child starts from the inherited environment, overrides win: V3 §22); `.current_dir` only when `cwd` is
`Some`; stdin, stdout and stderr inherited. The wrapper never modifies its own environment. `--verbose`
output is flushed before launching.

```rust
pub enum LaunchOutcome {
    /// Unix: `exec` never returns on success, so this is never constructed there. Kept for V3 §25 parity.
    ReplacedProcess,
    /// Windows: the child's full 32-bit exit code.
    Exited(i32),
}
```

### 7.2 Executable discovery (V3 §20)

1. Explicit override `agents.<id>.executable`: must exist and be a file, else
   `AgentNotInstalled { ExplicitMissing(path) }` (exit 3).
2. Otherwise search `PATH` in order (hand-rolled: `std::env::var_os("PATH")` split with
   `std::env::split_paths`; empty entries skipped; no new crate) for the adapter's executable name (`fake-agent` for `fake`). Unix: a
   regular file (after following symlinks) with at least one of the owner, group or other execute bits set. Windows: `<name>.exe`. Not found: `AgentNotInstalled { NotOnPath }`.
   When `config.toml` has `agents.<id>` tables for ids this build does not know, every `AgentNotInstalled`
   message lists them ("config.toml also configures unknown agents: `fakr`"), so a misspelled table is
   visible at the moment it matters.
3. A resolved path whose extension is `.bat` or `.cmd` (ASCII case-insensitive, any platform) is
   `UnsupportedExecutable` (exit 6). No shell is ever involved.

### 7.3 Lazy profile directory (V3 §9, §9.1)

**Case-only twins are refused (owner decision D9).** V3 §6 allows upper- and lower-case letters, and the
storage path is `<root>/profiles/<profile>/` exactly as typed. On case-insensitive filesystems (the Windows
and macOS defaults) `work` and `WORK` would silently share one directory, and with it one agent's
credentials; on Linux they would be two profiles. So `adapter::plan` (§5.1 step 5 (b), before dry run and before
lazy creation) lists `<root>/profiles/` when it exists and fails with `ProfileCaseConflict`
(exit 4, naming the existing entry) if any entry's name - directory, file or symlink alike, since any of
them blocks a same-named directory on a case-insensitive filesystem - equals the requested profile under
ASCII case-insensitive comparison but not byte-for-byte. Entries whose names are not valid UTF-8 are skipped. A missing
`profiles/` directory means no conflict. The check is read-only, so a dry run reports the same error.

`std::fs::create_dir_all(dir)`, then `std::fs::metadata(dir)?.is_dir()` (which follows symlinks, so a
symlink to a directory counts) must be true, else `ProfileDir`. An already-exists race is success once the path is verified to be a directory.
Nothing else is written: no credentials, no copied state, no configuration.

### 7.4 Dry run (V3 §26)

Printed to stdout; exit 0; no directory created, no configuration read beyond the normal load, no process
started.

```text
agent:        fake
profile:      work (explicit)
executable:   /abs/path/fake-agent (configured)
repository:   none
mechanism:    environment variable FAKE_AGENT_HOME
environment:  FAKE_AGENT_HOME=/home/me/.agent-profile/profiles/work/fake (would be created)
arguments:    ["--foo", "bar"]
```

- `profile` shows the resolution source in parentheses; `executable` shows `configured` or `PATH`.
- `(would be created)` appears only when the directory does not exist.
- Arguments are rendered with Rust `{:?}` string quoting; a non-UTF-8 argument is rendered lossily followed
  by ` (non-UTF-8)`.
- Redaction (V3 §22 "Secret-bearing environment values must not be printed"): `PlannedLaunch` carries a
  `sensitive_env: Vec<OsString>` list declared by the adapter; a listed variable's value renders as
  `<redacted>`. As a backstop, any override whose name contains `TOKEN`, `SECRET`, `KEY`, `PASSWORD`,
  `CREDENTIAL` or `AUTH` (ASCII case-insensitive) is also redacted. SP1's `fake` declares none; the SP2
  adapter contract makes the declaration mandatory for every override.
- `--verbose` on a real launch renders the same lines to stderr, each prefixed `agent-profile: `, built after
  §5.1 step 7's lazy creation so the directory is never reported as "(would be created)". Every
  `--verbose` and diagnostic write uses `writeln!` on `std::io::stderr()` and ignores the result (never
  `eprintln!`, which panics on a broken pipe and would turn the exit status into 101).

### 7.5 Unix (V3 §23.1)

`std::os::unix::process::CommandExt::exec`. It returns only on failure, mapped to `Launch` (exit 6). A
profile directory created by step 7 of §5.1 before a failed `exec` stays; lazy initialization is idempotent
and creates nothing but the empty directory. On
success the agent replaces the wrapper: same PID, exit status and stdio; no wrapper code runs afterwards.

### 7.6 Windows (V3 §23.2, §24)

In this order:

1. **Job.** `CreateJobObjectW`; `SetInformationJobObject` with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`;
   `AssignProcessToJobObject(GetCurrentProcess())`. Any failure: `Launch` (exit 6) before a child exists.
2. **Handler.** `SetConsoleCtrlHandler(Some(handler), TRUE)`. The child shares the console and receives
   every event itself. The handler:
   - `CTRL_C_EVENT`, `CTRL_BREAK_EVENT`: return TRUE at once (the wrapper survives and keeps waiting).
   - `CTRL_CLOSE_EVENT`, `CTRL_LOGOFF_EVENT`, `CTRL_SHUTDOWN_EVENT`: if the child handle has been published
     (step 3), wait on it with `WaitForSingleObject(child, INFINITE)`, then clear the job's limit flags
     itself (the same call as step 5, on the published job handle), then block forever
     (`std::thread::park` in a loop) and never return; if nothing has been published, return FALSE.
     Windows terminates the wrapper as soon as a close-type handler returns, so a handler that returned
     could kill the wrapper before the main thread's `process::exit(code)` and replace the agent's exit
     status; by never returning, the handler lets the main thread (which wakes from the same child exit)
     finish steps 5-6 and exit with the agent's code, while Windows' own close timeout (Microsoft
     HandlerRoutine "Timeouts") remains the backstop if the main thread is stuck. Clearing the limit in the
     handler as well covers that backstop case; two threads clearing the same limit is harmless. Without the
     wait, the wrapper would exit at once and its job would kill the agent mid-cleanup; without the clear,
     the job would kill processes the agent left running. (In practice only `CTRL_CLOSE_EVENT` reaches an
     interactive console application; the logoff and shutdown branches are defensive.) The wrapper never calls
   `SetConsoleCtrlHandler(NULL, …)`: an inherited "ignore Ctrl-C" attribute is left as direct invocation
   would leave it.
3. **Spawn** directly with `LaunchPlan::command().spawn()`. The child inherits job membership. Failure:
   `Launch` (exit 6). On success, publish two handles for the handler in a process-global
   `OnceLock<Published>` where `Published { job: usize, child: usize }` holds raw handle values as `usize`
   (raw handles are not `Send`/`Sync`): `job` is the step-1 job handle, which the launcher keeps open until
   process exit; `child` is a `DuplicateHandle` copy of the child's process handle with `SYNCHRONIZE` access,
   owned by the global and never closed, so it stays valid after `Child` is dropped.
4. **Wait.**
5. **Release the job.** `SetInformationJobObject` with no limit flags, so processes the agent left running
   survive the wrapper's exit. A failure is reported only under `--verbose`; it does not change the exit code.
6. **Return** `LaunchOutcome::Exited(code)` with the child's full 32-bit exit code; `main` passes it to
   `std::process::exit` (§5), which preserves all 32 bits (M2). The job handle is closed by that exit.

Consequences:
- Wrapper terminated while waiting: the job handle closes, the agent is killed; no orphan. (Tested.)
- Ctrl-C or Ctrl-Break while the agent runs: the agent handles it exactly as under direct invocation; the
  wrapper survives and returns the agent's exit code. (Tested with an agent that handles the event.)
- Ctrl-Break before step 2, or Ctrl-C before step 2 unless the "ignore Ctrl-C" attribute was inherited: the
  wrapper dies by default processing; no child exists. (Not separately tested; "terminated before child
  creation" is tested by killing the wrapper.)
- An event from step 2 until the child has attached to the console (inside `CreateProcessW`): swallowed;
  the agent still starts. This is the one documented difference from direct invocation (V3 §24), and it is
  tested with the debug pause hook.
- Console window closed: the agent gets its normal close-event cleanup time. (Not tested: CI cannot close a
  console window; see §11.)
- Normal exit: background processes survive (M1 "clear"). (Tested.)

## 8. Testing

### 8.1 `fake-agent` fixture extensions (additive)

The SP0 fixture contract (SP0 design §3.3) is unchanged, and the nine `fake_agent_*` smoke tests (including the helper test, which extends to the five new variables)
stay as they are. The three stub-CLI smoke tests change with the CLI: `version_exits_zero` and
`help_exits_zero_and_says_scaffold` keep their assertions except that `--help` no longer says "SP0
scaffold" (the new help text is asserted instead), and `unimplemented_invocation_is_usage_error` is
replaced by the §4.3 table tests, because `claude work` and `zzz-unknown` become `UnknownAgent` in SP1.

| Addition | Behaviour |
|---|---|
| `"pid"` report key | `std::process::id()` |
| `FAKE_AGENT_STDIN=1` | Read all of stdin before reporting; `"stdin"` holds it (non-UTF-8 = fixture error 125) |
| `FAKE_AGENT_STDERR=<text>` | Write `<text>` to stderr after reporting |
| `FAKE_AGENT_SLEEP_MS=<u64>` | After printing and flushing the report, sleep that long, then exit with the requested code |
| `FAKE_AGENT_SPAWN_SLEEPER=<u64>` | Before reporting, spawn a copy of itself with every fixture control variable (the seven `FAKE_AGENT_*` variables this table and SP0 define) removed from its environment and then `FAKE_AGENT_SLEEP_MS=<u64>` set (so the copy never spawns another sleeper), with its stdin, stdout and stderr all null, and report its PID as `"sleeper_pid"`. On Windows a spawned process also inherits every inheritable handle, including the pipes the fixture's own stdout and stderr are attached to (measured in the prototype), so a test must never wait for the wrapper's pipes to close while a sleeper runs |
| `FAKE_AGENT_CTRL_C_EXIT=<u8>` | Windows only (a fixture error elsewhere): BEFORE printing the report, install a console handler that, on `CTRL_C_EVENT` or `CTRL_BREAK_EVENT`, sleeps 300 ms and then exits with `<u8>` - an agent that outlives a wrapper that failed to survive the event |

Any unparsable value of these variables is a fixture error (exit 125, empty stdout). The SP0 helper
`support::fake_agent()` additionally removes all five new variables.

### 8.2 Unit tests (in-module)

- `name`: a table covering every V3 §6 rule for both `Platform` values (the reserved-word reason is reachable
  only here in SP1, because §4.2 check 4 routes a reserved bare word before profile validation): valid names; empty; leading
  `.`/`_`/`-`; each forbidden character class; `.` and `..`; separators; control characters and NUL; every
  §5.3 reserved word in mixed case; `CON`, `PRN`, `AUX`, `NUL`, `COM1`-`COM9`, `LPT1`-`LPT9` with and without
  extensions; trailing dot and space. Windows-only rules must be rejected for `Windows` and accepted for
  `Unix` where the general rules allow it.
- `cli` splitter: first-`--` cut, later `--` opaque, option placement, second bare word, non-UTF-8 opaque
  argument, non-UTF-8 bare word, reserved word routing, `--json`.
- `config` schema: every §6.2 error class and the accepted forms.
- `output`: redaction table and argument rendering.

### 8.3 Configuration tests (V3 §34 "Configuration")

In `tests/config.rs` unless a bullet says "unit test inside `config`".

- Valid TOML: the read succeeds and `agents.fake.executable` has the configured value.
- Invalid TOML; unknown key; relative `executable`: each read reports `ConfigInvalid`.
- Corrupt file refused by `update`, byte-identical afterwards.
- Comments and key order preserved across `update`.
- Concurrent writers: N threads each add a distinct agent table; all N present afterwards. Threads are a
  valid proxy for processes here because each `update` opens its own lock-file handle: `flock` locks
  belong to the open file description and `LockFileEx` locks to the handle, so two handles in one
  process contend exactly as two processes do.
- Stale-writer race: writer B's edit is applied to writer A's result, never to a pre-A snapshot.
- Reader during writes (unit test inside `config`, because the hook is crate-private and `tests/` files see
  only the public API), with a deterministic overlap: `update_with` also takes a hook that runs
  after the temp file is written and synced and before `persist`. The test's hook blocks on a barrier until
  a reader thread has read `config.toml` at least once, and the reader must see the complete PREVIOUS
  content; after `update` returns, a read sees the complete NEW content. A read never observes the temp
  file's content or a partial file.
- Failed replacement, two tests with one assertion (`ConfigWrite`, previous content byte-identical, no
  `.config.toml.*.tmp` left):
  - All OSes (unit test in `config`): `update` is implemented over a crate-private
    `update_with` (signature in §5) whose `replace` step is injectable; the test injects a replace step
    that returns an error, so step 7 fails deterministically after steps 1-6 succeeded.
  - Windows (integration): the test holds `config.toml` open with `OpenOptionsExt::share_mode` granting
    only `FILE_SHARE_READ` (no `FILE_SHARE_DELETE`), so step 4's read succeeds and the real rename in step 7
    fails with a sharing violation.
- Temp cleanup: a pre-planted `.config.toml.stale.tmp` is gone after a successful `update`, and was never
  read as configuration.

### 8.4 `tests/launch.rs` and `tests/windows_console.rs` (V3 §34 "Passthrough", "Environment", "Process behavior", §24)

`tests/launch.rs` spawns `CARGO_BIN_EXE_agent-profile` with a temporary `AGENT_PROFILE_HOME` whose
`config.toml` points `agents.fake.executable` at `CARGO_BIN_EXE_fake-agent`.

- Passthrough: `fake work -- --foo bar` and `fake work -- --dry-run` arrive verbatim; a non-UTF-8 opaque
  argument arrives intact.
- Environment: inherited `FAKE_AGENT_HOME=/wrong` is replaced by the profile directory; the test process's
  environment is unchanged afterwards.
- Exit status: `FAKE_AGENT_EXIT=7` gives 7.
- Dry run: exit 0, no fake-agent report on stdout, profile directory not created, report contains every
  §7.4 field.
- Lazy init: the profile directory exists after a launch; a regular file at that path gives `ProfileDir`
  (exit 4) and no launch.
- Concurrent lazy init: 8 simultaneous first launches of one profile all exit 0.
- `--verbose`: a real launch prints the seven report lines on stderr, each prefixed `agent-profile: `, without
  "(would be created)", and the agent still runs.
- Dry run of a not-installed agent: with no override and an empty `PATH`, `fake work --dry-run` exits 3 with
  nothing on stdout.
- Case-only twin: with `<root>/profiles/work/` present, `fake WORK` and `fake WORK --dry-run` both exit 4
  with `ProfileCaseConflict` naming `work`, and the byte-exact entry names listed from `<root>/profiles/` are
  exactly `["work"]` afterwards (a `WORK` existence check would be meaningless on case-insensitive
  filesystems); `fake work` still launches. A unit test inside `src/bin/fake-agent.rs` asserts that the
  sleeper's command removes every fixture control variable except its own `FAKE_AGENT_SLEEP_MS`.
- Stdio: stdin bytes reach `"stdin"`; `FAKE_AGENT_STDERR` text appears on the wrapper's stderr.
- Errors: each §4.3 row with a non-zero exit; explicit executable missing (3); empty `PATH` and no override
  (3); corrupt configuration (4, file untouched); a `.cmd` override (6); an override pointing at a non-
  executable or garbage file (6).
- Unix: the reported `"pid"` equals the wrapper child's PID (proves `exec`).

`tests/launch_plan.rs` pins `LaunchPlan::command` independently of any adapter, because SP1's only adapter
uses `cwd: None`: it builds a `LaunchPlan` for `CARGO_BIN_EXE_fake-agent` with `cwd: Some(<temp dir>)`, one
environment override replacing an inherited value, and args including `--` and a space, spawns
`plan.command()`, and asserts the reported `cwd`, `env` and `argv`. A second case with `cwd: None` asserts
the child inherits the test's working directory.

`tests/windows_console.rs` (`cfg(windows)`) uses a helper binary `src/bin/console-driver.rs` (a stub that
exits 125 on non-Windows) with the command line
`console-driver <result-file> <none|ctrl-c|ctrl-break> <none|report|pause-marker> <program> [args...]`. The driver is started with `CREATE_NEW_CONSOLE`, calls
`SetConsoleCtrlHandler(NULL, FALSE)` and installs a swallowing handler for itself, runs the wrapper against a
sleeping fake agent, sends the event with `GenerateConsoleCtrlEvent(event, 0)`, and writes a result file.
The driver never uses `CREATE_NEW_PROCESS_GROUP` for the wrapper (that flag sets the ignore-Ctrl-C
attribute). It captures the wrapper's stdout and stderr, draining each pipe on its own thread from the moment
of spawning (so a full pipe can never block the wrapper), and sends an event only after a readiness signal:
for agent-running tests, after the complete fake-agent report line has been read (the fixture installs its
handler before printing it); for the swallowed-window test, after the pause marker line has been read.

- Ctrl-C, handled agent: fake agent with `FAKE_AGENT_CTRL_C_EXIT=42` and a long sleep; after `CTRL_C_EVENT`
  the wrapper exits 42. A wrapper without a working handler would die first (default processing exits
  `0xC000013A`) and its job would kill the agent, so this is the test that proves §7.6 step 2.
- Ctrl-Break, handled agent: the same with `CTRL_BREAK_EVENT` and `FAKE_AGENT_CTRL_C_EXIT=43`, exit 43.
- Ctrl-C, default agent: without `FAKE_AGENT_CTRL_C_EXIT`, the wrapper exits `0xC000013A` well before the
  sleep ends (the agent's own default code, propagated).
- Swallowed window: `AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS=2000`, fake agent with `FAKE_AGENT_EXIT=7`
  and no sleep; the driver sends `CTRL_C_EVENT` after reading the pause marker. The fake-agent report appears
  and the wrapper exits exactly 7 (an event that reached a running agent would have produced `0xC000013A`
  instead).
- Normal completion and non-zero exit through the job path.
- No orphan: kill the wrapper mid-sleep; the fake agent has exited within a bounded wait, checked by waiting
  on its process handle (`WaitForSingleObject`) or `GetExitCodeProcess`, never by "the PID can be opened".
- Background survival: the test reads the report line, waits for the wrapper process (never for its pipes,
  see §8.1), and asserts `"sleeper_pid"` is still alive; the drop guard then kills it.
- Every PID a test learns (fake agent, sleeper, wrapper) is held by a drop guard that kills it if still alive,
  so a panicking assertion cannot leak processes onto the runner.
- `GenerateConsoleCtrlEvent(event, 0)` targets every process attached to the CALLER's console. Only
  `console-driver`, running in its own `CREATE_NEW_CONSOLE` console, may call it; the test process itself
  must never share that console, or the event would reach the test runner.
- Terminated before child creation: the debug-only hook `AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS`
  (`cfg(all(windows, debug_assertions))`, read only by the Windows launcher; immediately before step 3 it
  writes the line `agent-profile: debug: paused before spawn` to stderr, flushes, then sleeps; unparsable
  values are ignored) lets the test kill the wrapper in that window;
  no fake-agent report ever appears.
- Child creation failure: covered by the garbage-executable test in `tests/launch.rs`.

## 9. Measurements behind D3

Windows 11 Pro 10.0.26200, non-interactive agent shell, 2026-09-14; probe sources were kept under
`.clavity/scratch/sp1-platform-forks/probe/`.

- **M1 (job).** A process assigned itself to a new `KILL_ON_JOB_CLOSE` job and spawned `ping -n 30`. Exiting
  with the flag set killed the child; clearing the flag first (`SetInformationJobObject` returned success)
  left the child alive. The self-assignment was very likely nested inside an existing job.
- **M2 (headless Ctrl-C).** A driver started with `CREATE_NEW_CONSOLE` ran a cargo-pattern wrapper around
  `ping -n 30` and called `GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0)`. Without resetting the inherited ignore
  attribute, ping ran 30 s (exit 0). With `SetConsoleCtrlHandler(NULL, FALSE)` in the driver, the wrapper
  exited with `-1073741510` (`0xC000013A`) after 2.1 s.

M2 was first measured locally; the prototype then passed the same tests on GitHub's Windows runner (§10 step 1).

## 10. Delivery

1. **CI proof — done before planning.** A throwaway prototype of this whole design ran on draft PR #7
   (closed unmerged), run 34853308762: the Windows runner passed 97/97 tests including all eight
   `tests/windows_console.rs` tests (headless Ctrl-C and Ctrl-Break delivery proven on GitHub's runner);
   macOS and Linux passed 91/91 including the Unix `exec` PID test. The implementation plan is written from
   that verified prototype. If a later change makes a console test fail only on CI, STOP and ask the owner.
2. New dependencies (workspace-pinned, checked by `cargo deny`): `toml` (reads), `toml_edit` (writes),
   `thiserror`, `tempfile` (normal dependency), `windows-sys` (Windows target only; features
   `Win32_Foundation`, `Win32_Security`, `Win32_Storage_FileSystem`, `Win32_System_Console`,
   `Win32_System_JobObjects`, `Win32_System_Threading`).
3. `TODO.md`: remove the four SP1 open decisions; add the SP2 open decision "launching `.cmd`/`.bat`
   shims without shell mediation"; extend the `cargo install` debt item to name `console-driver`.
4. `README.md`/`ROADMAP.md`: SP1 row state updated when merged; README gains the `AGENT_PROFILE_HOME` note.
5. Gates unchanged: `just check`, CI matrix, capstone, test audit.

## 11. Known limits

- The Windows event window from handler installation until the child attaches to the console (§7.6).
- The close/logoff/shutdown handler path (§7.6 step 2) is argued from Microsoft's HandlerRoutine
  documentation and conhost's dispatch order (newest process first, `microsoft/terminal`
  `src/server/ProcessList.cpp`), not tested: CI cannot close a console window. Reported, unverified: in
  sessions handed off to Windows Terminal, closing a tab may not deliver close events at all
  (`microsoft/terminal` PR #20650).
- `std::fs::rename` is assumed atomic on Windows (§6.4).
- Release builds of SP1 know no agents; every launch in a release build is `UnknownAgent`.
- The case-only-twin check (§7.3) is not atomic with directory creation: two simultaneous first launches of
  `work` and `WORK` on a case-sensitive filesystem can both pass it. Not defended; it needs two conflicting
  names launched in the same instant.
- `File::lock` may be advisory; every writer is `agent-profile` itself. A user deleting `config.toml.lock`
  during a write can break mutual exclusion; not defended.
- Dry-run and `--verbose` render paths from the invoking user's own environment (`AGENT_PROFILE_HOME`,
  `PATH`) and configuration without escaping terminal control characters; that input is the user's own,
  not repository-controlled (V3 §1, §36), and opaque arguments are already rendered with `{:?}` quoting.

## 12. Stand-downs

Findings from the AGY-AFTER panel that were not folded, one line each. Rounds 1-2 ran on the agy peer; rounds
3-6 ran on independent subagent reviewers at the owner's direction after the peer hit quota failures. Round 6
was the owner-approved final round at the six-round cap.

- DISCARDED-BELOW-FLOOR: "§7.6 step 2's rationale assumes simultaneous close delivery" - conhost dispatches
  newest process first; the design is safe under either order, so only the rationale's wording is affected.
- DISCARDED-BELOW-FLOOR: "the sleeper smoke test sentence sits in the case-twin bullet" - placement only.

- DISCARDED-BELOW-FLOOR: "a lone `-` is classified as an option token" - §4.2 rule 3 defines every element
  beginning with `-` as an option token and check 5 rejects it deterministically; no implementer divergence.
- DISCARDED-BELOW-FLOOR: "`--dry-run=yes` is rejected" - §4.2 check 5 compares option tokens by exact string;
  no implementer divergence.
- DISCARDED-BELOW-FLOOR: "the default-agent Ctrl-C test alone cannot distinguish a working wrapper" - the
  handled-agent Ctrl-C and Ctrl-Break tests (§8.4) are the discriminating tests; the default-agent test only
  pins exit-code propagation.

- DISCARDED-BELOW-FLOOR: "no dedicated shell-metacharacter passthrough test" - shell mediation is excluded
  structurally: §7.1 builds the child only through `LaunchPlan::command` (`std::process::Command`, no shell),
  and §7.2 step 3 refuses the only extensions std routes through `cmd.exe`.
- DISCARDED-BELOW-FLOOR: "the terminated-before-creation test passes trivially if the kill lands before the
  pause" - that outcome is still termination before child creation (V3 §24), so it cannot turn a broken
  launcher green; the swallowed-window test (§8.4) exercises the pause itself.

- DISCARDED-BELOW-FLOOR: terminal escape sequences in dry-run output via `AGENT_PROFILE_HOME` - unreachable
  as an attack because that variable is set by the invoking user (§6.1), outside V3 §36's
  repository-controlled trust boundary.
- REJECTED: "Windows readers without `FILE_SHARE_DELETE` block the rename" - std's default share mode is
  `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE` (`library/std/src/sys/fs/windows.rs:210`).
- REJECTED: "self-assigning to a job fails inside an existing job" - Microsoft `AssignProcessToJobObject`:
  since Windows 8 the target job "must be empty or ... in the hierarchy of nested jobs ... and it cannot have
  UI limits set"; the SP1 job is new, empty and has no UI limits.
- REJECTED: "`std::env::home_dir` is deprecated" - compiled with `#![deny(deprecated)]` on Rust 1.98
  without a diagnostic (driver measurement, 2026-09-14).
- REJECTED: "directory-sync failure after a successful replace should not be exit 4" - V3 §33 has no
  I/O-warning class; `4 profile/configuration error` is the closest code for a configuration write whose
  durability is unconfirmed, and the message states that the replace happened.
- REJECTED: "the concurrent lazy-init test proves nothing" - V3 §9.1 requires exactly idempotent
  `create_dir_all` with a directory check for a single resource; the test targets that contract.
