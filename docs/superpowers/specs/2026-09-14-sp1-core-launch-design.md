# SP1 — Core and explicit launch: design

**Status:** approved in brainstorming on 2026-09-14; awaiting the written-spec review.
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

## 4. Command surface

### 4.1 Grammar

```text
agent-profile <agent> <profile> [WRAPPER OPTIONS] [-- <agent args...>]
agent-profile <agent> [WRAPPER OPTIONS] [-- <agent args...>]
agent-profile <reserved-top-level-word> ...
agent-profile <agent> <reserved-agent-word> ...
agent-profile --help | --version
```

Wrapper options: `--dry-run`, `--verbose`, `--json`, `-h`/`--help`.

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
4. Before the cut:
   - an element that begins with `-` must be a wrapper option; anything else is a usage error;
   - wrapper options may appear anywhere after the agent word;
   - at most one bare word may remain, and it must be valid UTF-8 (a non-UTF-8 word is a usage error);
   - a second bare word is a usage error: "agent arguments must follow `--`".
5. If the bare word equals a reserved word (ASCII case-insensitive), the invocation is an agent-scoped
   command. Otherwise it is a profile name and goes through §6 validation of V3.

### 4.3 Behaviour in SP1

| Invocation | Result | Exit |
|---|---|---|
| Top-level reserved word | "`<word>` is not yet implemented" | 2 |
| `<agent> <reserved word> ...` | "`<agent> <word>` is not yet implemented" | 2 |
| Unknown agent | Usage error listing the known agents; a build with none says "no agents are available in this build" | 2 |
| `--json` anywhere before `--` | "`--json` is not yet implemented" | 2 |
| Unknown option before `--`, or a second bare word | Usage error | 2 |
| `<agent> --help` | Launch-form usage text | 0 |
| `<agent>` with no profile | The resolver stub returns `ResolutionSource::None`; error "no profile selected for `<agent>`" (V3 §11 no-profile condition) | 4 |
| `<agent> <invalid profile>` | `InvalidProfileName`, stating the reason (reserved words included) | 4 |
| `<agent> <profile> --dry-run` | Dry-run report (§7.4) | 0 |
| `<agent> <profile>` | Launch (§7) | agent's status |

Precedence when several apply: `-h`/`--help` anywhere before `--` first (launch-form usage, exit 0, even for
an unknown agent), then usage errors from rule 4 of §4.2, then `--json`, then unknown agent, then reserved
word, then profile validation.

## 5. Module layout

The SP0 library already has one empty module per V3 §4 layer. SP1 fills them and adds `error` and `exe`.

| Module | Responsibility | Depends on |
|---|---|---|
| `name` | `ProfileName` and `AgentId` newtypes. `ProfileName::parse(&str, Platform)` implements V3 §6; `Platform::{Unix, Windows}` selects the Windows-only rules, and runtime code passes `Platform::host()`. `AgentId` syntax: `[a-z][a-z0-9-]*`. | — |
| `error` | `Error` (thiserror) and `Error::exit_code()` (§6.3). | — |
| `config` | `AppRoot` (§6.1), `Config` strict read (§6.2), `config::update` writer (§6.4). | `error` |
| `exe` | Executable discovery (§7.2). | `config`, `error` |
| `resolve` | V3 §12 `Resolution` and `ResolutionSource` types verbatim; SP1 stub `resolve(agent, explicit)` returns `Explicit` or `None`. SP3 replaces the body, not the types. | `name` |
| `adapter` | SP1-internal `plan(&AgentId, &Resolution, &AppRoot, &Config) -> Result<PlannedLaunch>`; one arm, `fake`, under `cfg(debug_assertions)`. No trait: SP2 designs it. | `exe`, `config`, `launch` |
| `launch` | `LaunchPlan` (V3 §4 struct, unchanged), `LaunchOutcome`, `launch/unix.rs`, `launch/windows.rs`. | `error` |
| `output` | Dry-run and `--verbose` rendering, environment-value redaction. | `launch` |
| `cli` | Clap types, splitter, dispatch; `pub fn run(args: impl IntoIterator<Item = OsString>) -> i32`. | all |

`PlannedLaunch` carries the `LaunchPlan` plus what dry run reports but the launcher does not need: the
profile directory, whether it exists, the executable's origin (`Configured` or `Path`) and a one-line
mechanism description.

`main.rs` becomes `std::process::exit(agent_profile::cli::run(std::env::args_os()))`; this is the ONLY
exit path. The Windows launcher returns `LaunchOutcome::Exited(code)` up through `run`, which returns
`code`; on Unix a successful `exec` never returns. Errors print as
`agent-profile: error: <message>` on stderr.

### 5.1 Launch data flow

1. Parse (§4).
2. Validate the profile name.
3. Resolve the application root; load the configuration.
4. `resolve::resolve`.
5. `adapter::plan`: discover the executable; compute `<root>/profiles/<profile>/fake`; build
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
- Each `agents.<id>` table: only `executable` is allowed (`deny_unknown_fields`); it is a string holding an
  absolute path.
- A missing file is an empty configuration. A missing `agents` table or agent table means "no override".
- Everything else is `ConfigInvalid` (exit 4): unreadable file, non-UTF-8 content, TOML syntax error, unknown
  key, wrong type, invalid agent id, relative `executable`. The message names the file, the key when there is
  one, and says: "agent-profile never rewrites an invalid configuration; fix or move the file."
- Reads never take the lock and open only `config.toml`.

### 6.3 Errors and exit codes

| Variant | Meaning | Exit |
|---|---|---|
| `Usage { message }` | Grammar violation (§4.2) | 2 |
| `NotYetImplemented { command }` | Reserved command or `--json` | 2 |
| `UnknownAgent { agent, known }` | Agent word not in this build | 2 |
| `AgentNotInstalled { agent, reason }` | `reason` is `NotOnPath` or `ExplicitMissing(path)` | 3 |
| `InvalidProfileName { name, reason }` | V3 §6 | 4 |
| `NoProfile { agent }` | Resolution returned `None` | 4 |
| `AppRoot { message }` | §6.1 | 4 |
| `ConfigInvalid { path, key, detail }` | §6.2 | 4 |
| `ConfigWrite { path, source }` | Lock, temp-file, sync or replace failure | 4 |
| `ProfileDir { path, source }` | Cannot create, or exists but is not a directory | 4 |
| `UnsupportedExecutable { path }` | `.bat` or `.cmd` | 6 |
| `Launch { executable, source }` | `exec` or spawn failure; Windows job or handler setup failure | 6 |
| `Io { context, source }` | Anything else, e.g. writing the dry-run report fails | 1 |

Once the agent has started, its exit status is the wrapper's exit status (V3 §33).

### 6.4 Locked atomic writer

`config::update(root: &AppRoot, edit: impl FnOnce(&mut toml_edit::DocumentMut) -> Result<()>) -> Result<()>`,
library-level only in SP1 (no CLI command writes configuration yet).

1. Create the root directory if missing.
2. Open or create `<root>/config.toml.lock` and call `File::lock()`. Any error: `ConfigWrite`, nothing written.
   The lock file is never deleted or replaced.
3. Delete files in the root named `.config.toml.*.tmp`. Under the lock no writer is active, so they are
   leftovers of crashed writers.
4. Read `config.toml` (missing = empty document), parse it with `toml_edit` and validate it against §6.2.
   Invalid: `ConfigInvalid`, file untouched.
5. Apply `edit`; validate the result against §6.2. Invalid: `ConfigInvalid`, file untouched.
6. Create a `tempfile::Builder` temp file in the root with prefix `.config.toml.` and suffix `.tmp`; write
   the document; `sync_all`.
7. `persist` onto `config.toml`. On Unix, open the root directory and `sync_all` it. Any error:
   `ConfigWrite`; the temp file is removed; the previous `config.toml` is untouched. There is no
   non-atomic fallback (V3 §18.1).
8. Release the lock (drop).

**§18.1 assumption, documented:** SP1 treats `std::fs::rename` as the platform's atomic replace on all three
OSes. No Microsoft document states that `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` is atomic; on Windows 10
1607+ with `FileRenameInfoEx` support, std uses POSIX rename semantics.

## 7. Launch

### 7.1 Shared

`LaunchPlan` becomes a `std::process::Command`: `executable`; `args` as given; `.env(k, v)` for each override
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
2. Otherwise search `PATH` in order for the adapter's executable name (`fake-agent` for `fake`). Unix: a
   regular file (after following symlinks) with at least one of the owner, group or other execute bits set. Windows: `<name>.exe`. Not found: `AgentNotInstalled { NotOnPath }`.
3. A resolved path whose extension is `.bat` or `.cmd` (ASCII case-insensitive, any platform) is
   `UnsupportedExecutable` (exit 6). No shell is ever involved.

### 7.3 Lazy profile directory (V3 §9, §9.1)

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
- Redaction: when an override variable's name contains `TOKEN`, `SECRET`, `KEY`, `PASSWORD` or `CREDENTIAL`
  (ASCII case-insensitive), its value renders as `<redacted>`.
- `--verbose` on a real launch renders the same lines to stderr, each prefixed `agent-profile: `.

### 7.5 Unix (V3 §23.1)

`std::os::unix::process::CommandExt::exec`. It returns only on failure, mapped to `Launch` (exit 6). A
profile directory created by step 7 of §5.1 before a failed `exec` stays; lazy initialization is idempotent
and creates nothing but the empty directory. On
success the agent replaces the wrapper: same PID, exit status and stdio; no wrapper code runs afterwards.

### 7.6 Windows (V3 §23.2, §24)

In this order:

1. **Job.** `CreateJobObjectW`; `SetInformationJobObject` with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`;
   `AssignProcessToJobObject(GetCurrentProcess())`. Any failure: `Launch` (exit 6) before a child exists.
2. **Handler.** `SetConsoleCtrlHandler(Some(handler), TRUE)`; the handler returns TRUE for `CTRL_C_EVENT`
   and `CTRL_BREAK_EVENT`, FALSE for every other event (close, logoff, shutdown keep default processing).
   The child shares the console and receives the events itself. The wrapper never calls
   `SetConsoleCtrlHandler(NULL, …)`: an inherited "ignore Ctrl-C" attribute is left as direct invocation
   would leave it.
3. **Spawn** directly with `Command::spawn`. The child inherits job membership. Failure: `Launch` (exit 6).
4. **Wait.**
5. **Release the job.** `SetInformationJobObject` with no limit flags, so processes the agent left running
   survive the wrapper's exit. A failure is reported only under `--verbose`; it does not change the exit code.
6. **Return** `LaunchOutcome::Exited(code)` with the child's full 32-bit exit code; `main` passes it to
   `std::process::exit` (§5), which preserves all 32 bits (M2). The job handle is closed by that exit.

Consequences, each tested (§8.4):
- Wrapper terminated while waiting: the job handle closes, the agent is killed; no orphan.
- Ctrl-C or Ctrl-Break before step 2: the wrapper dies by default processing; no child exists.
- Ctrl-C or Ctrl-Break between steps 2 and 3: swallowed; the agent still starts. This window is documented,
  not engineered away.
- Normal exit: background processes survive (M1 "clear").

## 8. Testing

### 8.1 `fake-agent` fixture extensions (additive)

The SP0 fixture contract (SP0 design §3.3) is unchanged, and the nine `fake_agent_*` smoke tests (including the helper test, which extends to the four new variables)
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
| `FAKE_AGENT_SPAWN_SLEEPER=<u64>` | Before reporting, spawn a copy of itself with only `FAKE_AGENT_SLEEP_MS=<u64>` set (no report is expected from it; its stdout is null) and report its PID as `"sleeper_pid"` |

Any unparsable value of these variables is a fixture error (exit 125, empty stdout). The SP0 helper
`support::fake_agent()` additionally removes all four new variables.

### 8.2 Unit tests (in-module)

- `name`: a table covering every V3 §6 rule for both `Platform` values: valid names; empty; leading
  `.`/`_`/`-`; each forbidden character class; `.` and `..`; separators; control characters and NUL; every
  §5.3 reserved word in mixed case; `CON`, `PRN`, `AUX`, `NUL`, `COM1`-`COM9`, `LPT1`-`LPT9` with and without
  extensions; trailing dot and space. Windows-only rules must be rejected for `Windows` and accepted for
  `Unix` where the general rules allow it.
- `cli` splitter: first-`--` cut, later `--` opaque, option placement, second bare word, non-UTF-8 opaque
  argument, non-UTF-8 bare word, reserved word routing, `--json`.
- `config` schema: every §6.2 error class and the accepted forms.
- `output`: redaction table and argument rendering.

### 8.3 `tests/config.rs` (V3 §34 "Configuration")

- Valid TOML; invalid TOML; unknown key; relative `executable`: each read reports `ConfigInvalid`.
- Corrupt file refused by `update`, byte-identical afterwards.
- Comments and key order preserved across `update`.
- Concurrent writers: N threads each add a distinct agent table; all N present afterwards. Threads are a
  valid proxy for processes here because each `update` opens its own lock-file handle: `flock` locks
  belong to the open file description and `LockFileEx` locks to the handle, so two handles in one
  process contend exactly as two processes do.
- Stale-writer race: writer B's edit is applied to writer A's result, never to a pre-A snapshot.
- Reader during writes: a loop of reads while writers run never sees a parse failure.
- Failed replacement, two tests with one assertion (`ConfigWrite`, previous content byte-identical, no
  `.config.toml.*.tmp` left):
  - All OSes (unit test in `config`): `update` is implemented over a crate-private
    `update_with(root, edit, replace)` whose `replace` step is injectable; the test injects a replace step
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
- Stdio: stdin bytes reach `"stdin"`; `FAKE_AGENT_STDERR` text appears on the wrapper's stderr.
- Errors: each §4.3 row with a non-zero exit; explicit executable missing (3); empty `PATH` and no override
  (3); corrupt configuration (4, file untouched); a `.cmd` override (6); an override pointing at a non-
  executable or garbage file (6).
- Unix: the reported `"pid"` equals the wrapper child's PID (proves `exec`).

`tests/windows_console.rs` (`cfg(windows)`) uses a helper binary `src/bin/console-driver.rs` (a stub that
exits 125 on non-Windows). The driver is started with `CREATE_NEW_CONSOLE`, calls
`SetConsoleCtrlHandler(NULL, FALSE)` and installs a swallowing handler for itself, runs the wrapper against a
sleeping fake agent, sends the event with `GenerateConsoleCtrlEvent(event, 0)`, and writes a result file.

- Ctrl-C: wrapper exit code `0xC000013A` well before the sleep ends.
- Ctrl-Break: the same with `CTRL_BREAK_EVENT` (expected code recorded by the plan's first task).
- Normal completion and non-zero exit through the job path.
- No orphan: kill the wrapper mid-sleep; the fake agent's PID is gone within a bounded wait.
- Background survival: after a normal exit, `"sleeper_pid"` is still alive; the test then kills it.
- Terminated before child creation: the debug-only hook `AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS`
  (`cfg(all(windows, debug_assertions))`, read only by the Windows launcher, sleeps immediately before
  step 3; unparsable values are ignored) lets the test kill the wrapper in that window;
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

M2 was measured locally only. GitHub's Windows runner is unproven; see §10 step 1.

## 10. Delivery

1. **First plan task — CI proof.** Land the `console-driver` Ctrl-C test against a minimal wrapper on the
   Windows runner. If GitHub's runner cannot deliver the event, STOP and ask the owner before any launcher
   work; the §24 test contract would need a documented alternative.
2. New dependencies (workspace-pinned, checked by `cargo deny`): `serde` (derive), `toml` (reads),
   `toml_edit` (writes), `thiserror`, `tempfile` (normal dependency), `windows-sys` (Windows target only;
   features `Win32_Foundation`, `Win32_Security`, `Win32_System_Console`, `Win32_System_JobObjects`,
   `Win32_System_Threading`).
3. `TODO.md`: remove the four SP1 open decisions; add the SP2 open decision "launching `.cmd`/`.bat`
   shims without shell mediation"; extend the `cargo install` debt item to name `console-driver`.
4. `README.md`/`ROADMAP.md`: SP1 row state updated when merged; README gains the `AGENT_PROFILE_HOME` note.
5. Gates unchanged: `just check`, CI matrix, capstone, test audit.

## 11. Stand-downs and known limits

- The Windows Ctrl-C window between handler installation and spawn (§7.6).
- `std::fs::rename` is assumed atomic on Windows (§6.4).
- `File::lock` may be advisory; every writer is `agent-profile` itself. A user deleting `config.toml.lock`
  during a write can break mutual exclusion; not defended.
- Dry-run redaction is name-based; SP1's only override (`FAKE_AGENT_HOME`) carries no secret.
- Release builds of SP1 know no agents; every launch in a release build is `UnknownAgent`.
