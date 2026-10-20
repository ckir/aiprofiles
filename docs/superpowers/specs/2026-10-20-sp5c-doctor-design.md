# SP5c — `doctor`

Status: APPROVED by the owner on 2026-10-20 (panel GREEN at round 4). Owner decided F1-F3 on 2026-10-20
(§11); agy-first ALIGNED after one negotiation round; builder check folded (see "Consults").
V3 (`agent-profile-implementation-spec-v3.md`) wins over this document; §1 "Past V3" records every place this
document goes past it. Third slice of SP5 as the owner split it on 2026-10-19 (`.clavity/seams/sp5-order.md`): SP5a
lifecycle (merged `9039ee1`) -> SP5b `--json` + `repositories` (merged `e099b01`) -> **SP5c `doctor`** -> SP5d
completions, docs and CI-pinning debt.

## 1. Intent

A user whose launch does something surprising has no single place to ask "what does this tool see, and is any
of it wrong?". The facts exist, spread over `status`, `agents`, `profiles`, `repositories` and `--dry-run`, and
each of those stops at its first error. V3 §29 asks for `doctor`: one read-only report that runs every check,
never stops early, separates facts from problems, and ends with one exit code a script can act on (V3 §33:
`5 doctor detected failure`). V3 §32 requires it in JSON as well.

Goals
- `agent-profile doctor`: a host audit. Application root, configuration, the profiles directory, every stored
  mapping, the current directory's resolution, and all twelve agents.
- `agent-profile <agent> doctor [<profile>]`: the same host checks plus a deep look at one agent and one
  profile: the executable, the profile's paths on disk, what a launch would set, and the capability caveats.
- Every V3 §29 item maps to a named check (§5.4 is the traceability table).
- `--json` on both forms, under the SP5b envelope and conventions (SP5b design §3.1), as a new schema
  `agent-profile/doctor/1`.
- Close the tracked VerbatimDisk test gap (ROADMAP.md, last tracked-debt entry) with the `cfg(windows)` unit
  test it names.

Non-goals
- Running the agent binary for any reason, including `--version` (owner ruling F2, §11). V3 §30 makes the
  version command optional; this slice does not take the option.
- Repairing anything. Doctor writes nothing: no file, no directory, no lock, no `config.toml` rewrite.
- Detecting credential-override environment variables (for example `ANTHROPIC_API_KEY` for Claude). They are
  recorded only as prose in `AdapterEvidence.notes`; structuring them is an adapter-metadata change for a later
  slice. Doctor shows the capability claim and its basis, which already names the caveat.
- Validating launch arguments. Doctor sees no agent arguments; it lists the options a launch refuses (§5.3).
- Changing the permissions `create` or lazy initialization give a profile directory (owner question Q1, §11).

Past V3 (where this document decides what V3 leaves open)
- V3 §30 permits an optional version command; this slice never runs one (owner ruling F2).
- V3 §33 lists code 5 without saying which codes `doctor` may also use; this document restricts doctor to
  0, 2 and 5, plus 1 when stdout cannot be written (owner ruling F3).
- V3 §29 "writability" is read as "permission bits say read-only", on Unix only (§5.1, call C5).

Anti-goals — a result that "works" but is a failure
- A permanently red `doctor` on a healthy machine. An agent that is not installed is not a failure of the
  host audit; a profile directory that does not exist yet is lazy initialization working (V3 §9).
- Doctor GREEN while the same launch would be refused for a reason doctor can see without running the agent
  (shell shim, case twin, invalid configuration, a file where a directory must go).
- Any secret, credential content or sensitive environment value in either output (V3 §29, §32, §36).
- A support level printed without its full capability matrix (owner ruling, SP4 design §6.1).
- A JSON document whose keys come and go by platform or by what is installed (SP5b design §1).

## 2. Ground truth in the repository (read 2026-10-20 at `42a3a86`, branch `sp5c-doctor`)

- `doctor` is a clap stub: `Command::Doctor(Rest)` (`cli.rs:73`), named by `reserved_name()` (`cli.rs:88-94`)
  and turned into `NotYetImplemented`; test `doctor_stays_unimplemented` (`cli.rs:1485`) pins that and must be
  replaced. Every implemented word lives in `CommandWord` (enum at `cli.rs:99`) and is dispatched before clap;
  `doctor` follows, as `repositories` did in SP5b.
- `run_command(agent, command, profile, repo, yes, …)` (`cli.rs:724`) is the dispatcher. `--repo <path>` is
  accepted by `current`, `resolve`, `status`, `link`, `unlink` (`cli.rs:37-41`).
- Exit codes: `Error::exit_code` (`error.rs:83-100`). `AgentNotInstalled` 3; `InvalidProfileName`, `NoProfile`,
  `AppRoot`, `ConfigInvalid`, `ProfileDir`, `ProfileCaseConflict`, `Repository` 4; `UnsupportedExecutable`,
  `Launch` 6; `Io` 1; usage, `UnknownAgent`, `ArgumentConflict`, `NotYetImplemented` 2. Nothing maps to 5.
- Executable discovery: `exe::discover(agent, name, explicit, path_var, config_file)` (`exe.rs:33`) returns
  `Found { path, origin }` or `AgentNotInstalled` with `NotInstalledReason::{ExplicitMissing(path),
  NotOnPath { ignored_relative }}`, or `UnsupportedExecutable` for a `.bat`/`.cmd`/`.ps1` (`exe.rs:51-54`).
  Relative `PATH` entries are counted and never searched (`exe.rs:76-93`). `cli.rs:921-935` folds the result
  into `output::ExecutableStatus::{Found, NotOnPath, ConfiguredMissing, Unsupported}` (`output.rs:385-394`).
- Adapter metadata: `AdapterMetadata { id, executable, mechanism, support, evidence, capabilities, env,
  conflicts }` (`adapter/metadata.rs:221-234`); `AdapterEvidence { mechanism_id, verified_at, upstream_version,
  source_url, notes }` (`:18-26`); `CapabilityState` has five states (`:47-53`); `EnvOverride { name, sensitive }`
  (`:64-67`); `ConflictOption::spelling()` (`:85`); `Mechanism::sentence_for(target)` gives the concrete launch
  sentence. `Adapter::paths(root, profile) -> Vec<(PathBuf, PathKind)>` (`adapter/mod.rs:61`).
- `adapter::ensure_paths` creates profile directories with `fs::create_dir_all` and no explicit mode
  (`adapter/mod.rs:289-305`), so their mode is the process umask's default.
- Configuration: `AppRoot::resolve()` (`config.rs:38`), `config_path()`, `profiles_dir()` (`:71-75`),
  `Config::load(root)` (`:130`) returning `ConfigInvalid { path, key, detail }` on a bad file. A `[agents.<id>]`
  table is accepted for any syntactically valid id (`config.rs:324`, `AgentId::parse`), registered or not.
- Profiles on disk: `lifecycle::list_profiles(root, adapters) -> Listing { rows, ignored }`
  (`lifecycle.rs:210`, `list_profiles` at `:226`), read-only, dot-entries skipped. `lifecycle::presence(root, agent, profile) ->
  Presence::{Materialized, Known, Absent, Conflict { existing }}` (`lifecycle.rs:323`, `presence` at `:356`). Case twins are
  checked one profile at a time by `config::check_case_twins` (`config.rs:621-636`, ASCII case only, any entry
  type); nothing scans the whole directory for twin pairs.
- Mappings: `repositories::classify(key) -> (State, Option<String>)` (`repositories.rs:62-93`), states
  `Active, Moved, NotARepository, Missing, Unreadable, Foreign` (`:17-24`); `repositories::report` never fails.
- Resolution: `repo::discover(start) -> Result<Discovery>` (`repo.rs:106`), `resolve::resolve(…)` (`resolve.rs:43`).
- JSON: `json::render` (`json.rs:24`) and the token helpers `support_token`, `capability_token`, `state_token`
  (`json.rs:151-176`); the SP5b envelope is `schema` then `tool_version` first.
- Redaction: `output::SENSITIVE_NAME_PARTS` and its predicate (`output.rs:19`, `:497`).

## 3. Grammar

```text
agent-profile doctor [--repo <path>] [--json]
agent-profile <agent> doctor [<profile>] [--repo <path>] [--json]
```

- `doctor` moves from the clap enum to `CommandWord`. Bare `doctor` accepts no positional word. The agent form
  accepts at most one positional word, the profile.
  A positional word that is not valid UTF-8 stays a usage error (exit 2), because it cannot be printed in a record.
- `--repo <path>` replaces the current directory for discovery and resolution, as in `status`.
- `--help` prints a usage block in the style of the other `CommandWord`s. The top-level command list
  (`cli.rs:36-49`) gains both lines.
- Usage errors (unknown option, a second positional word, unknown agent) are exit 2 before any check runs,
  exactly as for other commands. A positional word that is not a valid profile name is NOT a usage error: it is
  the `resolution` check's FAIL (§5.3), so the rest of the report still prints. (My call, §11 C3.)

## 4. The check model

A run produces an ordered list of check records. Each record has:

| field | meaning |
|---|---|
| `id` | the check's identifier, from the closed set in §5 |
| `agent` | the agent id for per-agent checks, otherwise `null` |
| `subject` | what was checked: a path, a mapping key, a profile name, or `null` |
| `status` | one of `pass`, `info`, `warn`, `fail`, `skip` (closed) |
| `message` | one human sentence; the text form prints exactly this, so it always names its subject (a path, key, profile or pair of names) when `subject` is not `null` |
| `data` | an object of structured facts specific to the `id` (§6), or `null` |

Status meanings:
- `pass`: a check that can fail did not.
- `info`: a fact with no pass/fail sense (the platform, the evidence version, a lazily absent directory).
- `warn`: probably not what the user wants, but a launch is not refused because of it.
- `fail`: a command of this tool (a launch, `status`, `list`, `profiles`, ...) would be refused or would fail
  because of it, and doctor can see it without running the agent. A `fail` must always name what to change.
  Anything that only might make the agent itself misbehave is `warn`.
- `skip`: the check could not run because a check it depends on failed; `message` names that check. A
  skipped check emits exactly ONE `skip` record with `subject: null` (one per agent for a per-agent check),
  however many records it would otherwise have produced (`mapping` emits one per stored mapping when it
  runs, and one `skip` when it cannot).

Dependencies (the left column's condition, a `fail` unless the row names another condition, turns every check
on the right into `skip`):

| failed check | skipped checks |
|---|---|
| `app_root` | `app_root_writable`, `config` and everything that depends on it, `profiles_dir`, `case_twins`, `profile`, `profile_path`, `launch_environment` |
| `profiles_dir`, when the path it names is the profiles directory itself | `case_twins`, `profile`, `profile_path` |
| `config` | `config_agent`, `default_profile`, `mapping`, `mapping_agent`, `executable`, `resolution` and everything that depends on it |
| `repository` | `resolution` when no explicit `<profile>` was given |
| `resolution` with no valid profile: its `fail` (invalid name), its `warn` (nothing resolves) or its own `skip` | `profile`, `profile_path`, `launch_environment` |

`executable` depends on `config` because the configured override decides where to look; reporting a
PATH-only result would contradict what a launch does. These checks need neither the root nor the
configuration and always run: `platform`, `repository`, `repository_local_executable`, `support`,
`conflicting_option`, `capability`.

Exit rule (owner ruling F3): doctor runs every check in scope and always prints the full report. It exits
`5` if any record is `fail`, otherwise `0`. Usage errors exit `2` before any check runs. The one other code is
`1`, when writing stdout itself fails (`Error::Io`); nothing can be reported then. Doctor never exits 3, 4 or 6:
those classes become `fail` records.

Order: host checks (§5.1), then repository checks (§5.2), then per-agent checks (§5.3) in
`adapter::known_agents()` order. Within one `id`, records follow the order stated for that check. The order is
part of the contract, so tests and consumers can rely on it.

## 5. The checks

### 5.1 Host checks (both forms)

| id | subject | condition -> status |
|---|---|---|
| `platform` | `null` | always `info`: `std::env::consts::OS` and `ARCH`, and `tool_version` |
| `app_root` | the root path | `AppRoot::resolve` fails -> `fail` (message carries the error), and every later check that needs the root is `skip`; the root is a dangling symbolic link -> `fail`; the root does not exist -> `info` ("created on first use"); exists and is a directory -> `pass`; exists and is not a directory -> `fail` |
| `app_root_writable` | the root path | Unix only: root missing -> checks the nearest existing ancestor instead; that path's permissions are read-only (`Permissions::readonly()`, no write bit at all) -> `fail` (lazy initialization and `create` cannot create anything); otherwise `pass`. The message says only permission bits were read, never ACLs or ownership. Windows: always `info` ("not checked on this platform": Microsoft's file-attribute reference says `FILE_ATTRIBUTE_READONLY` "is not honored on directories", and `Permissions::readonly` reads only that attribute). Never creates a file. |
| `config` | `config.toml` path | file absent -> `info` ("no configuration; defaults apply"); `Config::load` ok -> `pass`; any `Config::load` error -> `fail` with its key and detail (an unreadable file is already `ConfigInvalid` with "cannot read the file", `config.rs:130-138`) |
| `config_agent` | the agent id | one record per `[agents.<id>]` table whose id is not registered -> `warn` ("not a known agent; this table is ignored"). No record when every configured id is registered. |
| `profiles_dir` | the path that failed | a dangling symbolic link -> `fail` (as an unreadable profiles directory); absent -> `info`; `list_profiles` fails -> `fail` with the path its error names (the profiles directory or one profile directory below it; `profiles` and `list` fail the same way), `data.profiles` and `data.ignored` `null`; otherwise `pass` with `data.profiles` (row count) and `data.ignored` (the `Listing.ignored` count) |
| `case_twins` | the first name of the pair (both are in `data.names`) | `profiles_dir` absent -> no record. Otherwise one `fail` per pair of entries of the profiles directory whose UTF-8 names differ only in ASCII case ("profiles `work` and `Work` differ only in case; every command on either exits 4; rename or remove one"). Pairs are listed in byte order of the first name. No record when there are none. The same comparison as `check_case_twins` (`eq_ignore_ascii_case`, every entry type, dot-entries included because `check_case_twins` includes them). |
| `default_profile` | the profile or `null` | always `info`: names the global default profile, or says there is none. A twin of it is reported by `case_twins`, not here. |

### 5.2 Repository checks (both forms)

| id | subject | condition -> status |
|---|---|---|
| `repository` | the start directory | `repo::discover` errors -> `fail` (message carries the reason); `NotInRepository` -> `info`; `Repository(root)` -> `info` naming the root |
| `mapping` | the stored key | one record per stored mapping in stored-key order, from `repositories::report`: `active` -> `pass`; `moved`, `missing`, `not_a_repository`, `unreadable` -> `warn` (each names `repositories` and editing `config.toml`); `foreign` -> `info`. `data` carries the SP5b `repositories` element for that key. |
| `mapping_agent` | `<key>` + agent | one `warn` per agent key in a mapping's agents table that is not a registered agent |

### 5.3 Per-agent checks

Bare `doctor` runs the first three rows for every registered agent. `<agent> doctor` runs every row for that
agent only.

| id | form | subject | condition -> status |
|---|---|---|---|
| `executable` | both | the path or `null` | `Found` -> `pass` (path and origin); `NotOnPath` -> `info` in bare doctor, `fail` in `<agent> doctor`; `ConfiguredMissing` -> `fail` in both ("the configured executable does not exist or is not a file"); `Unsupported` -> `fail` in both (the launch exits 6 for a shell script or shim) |
| `repository_local_executable` | both | the path | a relative `PATH` entry, resolved against the process's current directory (never `--repo`: a launch resolves relative entries against the real cwd, so that is the only place V3 §20's third state can arise), holds the executable as `exe.rs`'s private `search_dir` decides it for an absolute entry (Unix: the bare name with an execute bit; Windows: the first of `WINDOWS_FORMS`, so `claude.exe` counts and an extensionless `claude` does not) -> `warn` ("found but never used: v0.1 searches only the configured override and absolute PATH entries"). Read-only existence check; never run. No record otherwise. This is V3 §20's third state. |
| `support` | both | `null` | always `info`. `data` carries the SP5b `agents` element's `mechanism`, `support`, `evidence` and the full `capabilities` matrix, plus `"installed_version": null` and `"version_probe": "not_run"`. The message names the evidence's `upstream_version` and `verified_at` and says the installed version is not probed. |
| `resolution` | agent form | the profile | an explicit `<profile>` argument is used as given (no discovery needed); otherwise `resolve::resolve` with the `repository` check's `Discovery`, and `skip` if that check failed (a launch would fail on the same error, so resolving as "not in a repository" would show a profile no launch uses). Invalid name argument -> `fail` (the `InvalidProfileName` text); nothing resolves -> `warn` ("a launch here needs an explicit profile"); resolved -> `info` naming profile and source. |
| `profile` | agent form | the profile | `presence`: `Conflict` -> `fail` naming the twin; `Absent` -> `info` ("created on first launch"); `Materialized` -> `pass`; `Known` -> `pass` (reserved and unconstructed by all twelve adapters, ROADMAP.md; mapped so the match stays exhaustive) |
| `profile_path` | agent form | each path | one record per `Adapter::paths` entry, in that order, decided on `fs::metadata` of the path, which follows symbolic links exactly as `ensure_paths` and `has_kind` do (`adapter/mod.rs:281-285`; measured 2026-10-20: a launch through a symlinked `profiles/work` exits 0): exists with the expected kind -> `pass`, with `data.mode` on Unix (octal string) and `null` elsewhere; exists with the wrong kind (a file where a directory goes, or the reverse) -> `fail` (`ensure_paths` refuses it, `adapter/mod.rs:294-297`); Unix only, exists with the expected kind but no write bit at all -> `warn` (the launch is not refused, but the agent cannot write its state there); any error other than not-found -> `fail` carrying the error. Not found and `symlink_metadata` finds an entry -> `fail` ("<path> is a dangling symbolic link"; measured 2026-10-20: a launch through a dangling `profiles/dang` exits 4 with "File exists"). Not found -> walk up to the nearest existing ancestor, continuing above the application root when the root itself is missing (as `app_root_writable` does, and as `create_dir_all` would create the whole chain), applying the same two rules to each ancestor (`fs::metadata`, and a dangling link is a `fail`): that ancestor is not a directory -> `fail` ("<ancestor> is not a directory, so a launch cannot create <path>"; reproduced 2026-10-20: a file at `profiles/work` makes `agent-profile fake work` exit 4 with "Not a directory"); Unix only, the ancestor has no write bit at all -> `fail` (a launch cannot create the path; this covers a read-only `profiles/` directory); otherwise `info` ("created on first launch"). |
| `launch_environment` | agent form | the variable or option | exactly one `info` record: the adapter's `Mechanism`, matched on its four variants, with the message `mechanism.sentence_for(<target>)` where the target is the profile directory (`Env`, `FlagDir`) or file (`EnvFile`, `FlagFile`) from `Adapter::paths`. For the two variable mechanisms, when the current environment already sets that variable the message adds that the launch replaces it. The current value is never read into the output, so there is nothing to redact. For the two variable mechanisms the message is `sets <NAME> to <path>`; `sentence_for` is used for the flag mechanisms. |
| `conflicting_option` | agent form | the option spelling | one `info` per `ConflictOption` ("a launch refuses this option because the profile already sets it") |
| `capability` | agent form | the capability | one record per `Capability::ALL` entry whose state is not `supported`: `info`, message = state label + basis. These are the credential-isolation caveats; they are facts the evidence documents, never `warn`. |

### 5.4 V3 §29 traceability

| V3 §29 item | check(s) |
|---|---|
| executable availability | `executable` |
| executable path | `executable`, `repository_local_executable` |
| platform | `platform` |
| adapter support | `support` |
| profile validity | `resolution`, `profile`, `case_twins` |
| profile path | `profile_path` |
| writability | `app_root_writable`, `profile_path` |
| expected environment overrides | `launch_environment` |
| expected arguments | `launch_environment` (flag mechanisms), `conflicting_option` |
| safe version detection | `support` (evidence version; installed version deliberately not probed, owner ruling F2) |
| compatibility | `support`, `executable` (`Unsupported`) |
| credential-isolation caveats | `capability`, `support` |
| repository mapping | `mapping`, `mapping_agent` |
| effective resolution | `repository`, `resolution` |
| known configuration conflicts | `config`, `config_agent`, `case_twins`, `mapping_agent` |
| mechanism-version diagnostics | `support` (`mechanism.id` and evidence; no stored per-profile mechanism exists to compare, so nothing can report a change in v0.1) |
| relevant path permissions | `app_root_writable`, `profile_path` (`data.mode`) |

V3 §20's three states map to `executable` (`NotOnPath`, `ConfiguredMissing`) and `repository_local_executable`.
V3 §29's four capability words map to the `capability` and `support` records' state tokens.

## 6. JSON document

```json
{
  "schema": "agent-profile/doctor/1",
  "tool_version": "0.0.5",
  "agent": "claude",
  "profile": "work",
  "repository": "/home/me/src/acme",
  "result": "ok",
  "counts": { "pass": 1, "info": 1, "warn": 0, "fail": 0, "skip": 0 },
  "checks": [
    { "id": "platform", "agent": null, "subject": null, "status": "info",
      "message": "linux x86_64, agent-profile 0.0.5",
      "data": { "os": "linux", "arch": "x86_64" } },
    { "id": "executable", "agent": "claude", "subject": "/usr/local/bin/claude", "status": "pass",
      "message": "found on PATH: /usr/local/bin/claude",
      "data": { "state": "found", "path": "/usr/local/bin/claude", "origin": "path" } }
  ]
}
```

- Envelope and conventions exactly as SP5b design §3.1: one pretty object, `schema` and `tool_version` first,
  `snake_case`, every member always present, `null` for absent, closed token sets for `agent-profile/doctor/1`.
- `agent`: `null` for bare doctor. `profile`: the profile the agent form examined, or `null`. `repository`: the
  discovered root or `null`.
- `result`: `fail` iff any record is `fail`, otherwise `ok`. It always agrees with the exit code (5 or 0).
- `counts`: all five keys always present.
- `checks[].id` and `status` are the closed sets of §4 and §5. `data` per id:
  `platform` {os, arch}; `app_root` / `app_root_writable` / `config` / `profiles_dir` {path, exists};
  `profiles_dir` adds {profiles, ignored}; `case_twins` {names: [a, b]}; `mapping` = the SP5b `repositories`
  element; `executable` = the SP5b `agents` element's `executable` object; `support` = {mechanism, support,
  evidence, capabilities, installed_version, version_probe}; `profile` {state, existing} as in SP5b `status`
  `presence`; `profile_path` {path, kind (`dir` or `file`), exists, mode (Unix octal string, else `null`)};
  `launch_environment` {kind (`env`, `env_file`, `flag_dir`, `flag_file`: a closed set of its own, one token per
  `Mechanism` variant; NOT `Mechanism::probe_token`, whose strings carry the name after a colon),
  name (the variable or the option spelling), value (the target path the launch passes; a profile path, never a
  secret), already_set (`true`/`false` for the two variable kinds, `null` for the two flag kinds)};
  every other id `null`. No member ever carries a value read from the environment.
  `executable` data is `null` only on an unexpected discovery error, which `exe::discover` cannot produce today.
- Under `--json`, stderr carries nothing doctor-specific; exit codes are those of §4.

## 7. Text output

One line per record, in record order: a padded status word (`PASS`, `INFO`, `WARN`, `FAIL`, `SKIP`), the
agent id in brackets when present, then the message. Records group under three headings, `host`,
`repositories`, and `agent <id>` once per agent, with a blank line between groups. The last line is a summary:
`doctor: <n> failed, <n> warnings` (exit 5) or `doctor: no failures, <n> warnings` (exit 0). Text goes to
stdout; nothing doctor-specific goes to stderr.

## 8. Constraints

- Read-only. No file, directory or lock is created, opened for writing, or removed, on any path. Proven by a
  test that snapshots the application root and a repository before and after a run (§9).
- No process is spawned. No network.
- No secret: §5.3's redaction rule; no `AdapterEvidence.notes` beyond what `agents --json` already prints.
- Every check tolerates every earlier failure: no panic, no early return, `skip` instead.
- Platform-dependent facts live in `data` values, never in which keys exist.

## 9. Testing (oracles)

- One integration test per status of every check id in §5, through the real binary with a temporary
  application root (the `tests/repositories.rs` and `tests/json.rs` style): each asserts the record's `id`,
  `status` and `subject` in the JSON document and the matching text line.
- Exit-code tests: a host with only `info`/`pass`/`warn` exits 0; one `fail` exits 5; a usage error exits 2 with
  empty stdout.
- `doctor` with no agents installed, an empty root and no repository exits 0 (the anti-goal "permanently red").
- Targeted doctor for an uninstalled agent exits 5, for an installed one with an absent profile exits 0.
- Read-only proof: a snapshot (paths, kinds, sizes, mtimes) of the root and the repository before and after
  doctor in both forms, with an existing profile and with a missing one, is identical.
- No environment value leaks: with the agent's own variable set to a sentinel string in doctor's environment,
  neither output contains the sentinel, and `already_set` is `true`.
- Every row of §4's dependency table: the failed check produces exactly the listed `skip` records.
- A regular file at `profiles/<profile>`: `<agent> doctor <profile>` reports `profile_path` `fail` and exits 5,
  and the same launch exits 4 (the pair is the oracle; both are asserted in one test).
- Unix: a `profiles/` directory with mode `0555` and a missing profile: `profile_path` `fail`, exit 5.
- A config error: `mapping` produces exactly one `skip` record with `subject: null`, whatever the file held.
- A symlink at `profiles/<profile>` to a real directory: `profile_path` is not `fail` and the launch exits 0; a
  dangling symlink there: `profile_path` `fail` and the launch exits 4.
- A fresh host: no application root at all, `<agent> doctor work` for an installed agent exits 0 and
  `profile_path` is `info`.
- Windows CI: a relative `PATH` entry `.` holding `fake.exe` gives `repository_local_executable` `warn`; an
  extensionless `fake` there gives no record.
- `<agent> doctor` with nothing resolving: `resolution` `warn`, then exactly one `skip` each for `profile`,
  `profile_path`, `launch_environment`.
- Case twins: on a case-sensitive filesystem (Linux CI), `work` and `Work` produce one `fail` naming both; on
  macOS and Windows CI the test creates only one and asserts no record.
- The JSON document parses, its key set is identical across two runs with different installed agents, and
  `result` agrees with the exit code.
- `doctor_stays_unimplemented` is replaced by grammar tests for both forms.
- Debt: the `cfg(windows)` unit test in `repositories.rs` asserting
  `classify(r"\\?\C:\agent-profile-nonexistent\x") == (State::Missing, None)`, measured by the Windows CI leg.

## 10. Left to the builder

- Module layout (a `doctor.rs` producing records, renderers in `output.rs` and `json.rs` is the obvious shape).
- Exact message wording, within §4's rule that a `fail` names what to change.
- How `repository_local_executable` reuses `exe.rs`'s `search_dir` (it must, so the two cannot disagree on
  what counts as an executable).

## 11. Decisions

Owner, 2026-10-20 (AskUserQuestion after the agy-first consult):
- F1: bare `doctor` is the global host audit; `<agent> doctor [<profile>]` is the targeted form.
- F2: never execute the agent in v0.1; report the evidence version and say the installed one is not probed.
- F3: doctor exits only 0, 2 or 5 (plus 1 when stdout cannot be written); 5 iff any `fail`.

My calls (owner: please check)
- C1: F4, one document of ordered check records with a summary, rather than nested per-agent documents. agy
  agreed in round 0. The `data` objects reuse the SP5b element shapes so nothing is restated differently.
- C2: an uninstalled agent is `info` in bare doctor and `fail` in the targeted form (agy round 1).
- C3: an invalid profile argument is a `resolution` `fail` record, not a usage error, so the host report still
  prints.
- C4: capability caveats are `info`, never `warn`, so a proven adapter with a documented caveat does not turn
  every run yellow.
- C5: `app_root_writable` and `profile_path` read permission bits only. They can say "read-only" reliably and
  cannot prove writability (ACLs, ownership); messages say so.
- C7: the builder check's five questions (2026-10-20) are settled in §4 (dependency table, `fail` = a command of
  this tool refuses), §5.1 (`config` takes any load error; `profiles_dir` names the failing path), §5.3
  (`resolution` skips on a discovery error; repository-local lookup uses the real cwd; `launch_environment`
  never reads a value) and §5.1/§5.3 (writability is Unix-only).
- C6: a profile directory's mode is reported as data, never judged, because the tool itself creates them with
  the default umask (§2) and judging would flag every profile it made.

Owner questions
- Q1 (answered 2026-10-20: tracked as ROADMAP debt, out of this slice): profile directories are created with the default umask (commonly `0755`), so other local users can list
  them; agents usually protect their own credential files. Should `create` and lazy initialization use `0700`
  on Unix? That changes SP5a behaviour, so it is out of this slice either way; the question is whether to track
  it as debt.

## Consults

- agy-first, 2026-10-20: brief `.clavity/seams/sp5c-forks.md`, reply `.clavity/scratch/sp5c-forks/reply.md`
  (NEGOTIATE on exit codes and scope); negotiation round 1 `.clavity/seams/sp5c-forks-negotiate-r1.md`, reply
  `.clavity/scratch/sp5c-forks/reply-r1.md` (ALIGNED). Verified by measurement before folding: V3 §30 makes the
  version command optional (spec:1100-1110); `AgentNotInstalled` is exit 3 and nothing maps to 5
  (error.rs:83-100); lazy initialization is V3 §9 (spec:378-405); relative PATH entries are counted, never run
  (exe.rs:76-93). The peer reversed itself on the targeted not-installed exit code between rounds; the owner
  decided F3. Tree unchanged after both rounds.
- Spec panel round 1 (agy escalation), 2026-10-20: brief `.clavity/seams/sp5c-spec-panel-r1.md`, reply
  `.clavity/scratch/sp5c-spec-panel/reply-r1.md`, six findings, all verified and FOLDED: the invalid-profile
  fail belongs to `resolution`; `app_root` also skips `launch_environment`; `profiles_dir` gets a dependency
  row; `launch_environment.kind` is its own token set (`metadata.rs:170-177` prints `env:<name>`); one `skip`
  record per skipped check; `profile_path` walks to the nearest existing ancestor (reproduced: a file at
  `profiles/work` makes a launch exit 4). The peer's open-question point that `profiles/` itself was never
  checked for writability is folded into the same ancestor walk.
- Spec panel round 2, 2026-10-20: brief `.clavity/seams/sp5c-spec-panel-r2.md`, reply
  `.clavity/scratch/sp5c-spec-panel/reply-r2.md`, three findings, all verified and FOLDED: the dependency table
  now names the `resolution` `warn` as a trigger; `profile_path` follows symbolic links like `ensure_paths`
  (measured: a symlinked profile directory launches, exit 0) and fails on a dangling one (measured: exit 4);
  every `message` names its subject, so `case_twins` names both profiles in text.
- Spec panel round 3, 2026-10-20: brief `.clavity/seams/sp5c-spec-panel-r3.md`, reply
  `.clavity/scratch/sp5c-spec-panel/reply-r3.md`, two findings, both verified and FOLDED: the ancestor walk
  continues above a missing application root (it found nothing on a fresh host and could only fail);
  `repository_local_executable` reuses `search_dir`, so Windows sees `claude.exe` and ignores an extensionless
  `claude` (`exe.rs:105-123`). Its open-question remark that version detection is nominal is the owner's F2
  ruling, recorded under Stand-downs.
- Spec panel round 4, 2026-10-20: brief `.clavity/seams/sp5c-spec-panel-r4.md`, reply
  `.clavity/scratch/sp5c-spec-panel/reply-r4.md`: PANEL VERDICT GREEN, every seat "no new findings" (State
  Corruptor new this round). Its one concrete claim was checked and stood down below. Tree unchanged after
  every round. Seats used across the review: Axiom Breaker, Cascade Analyst, Protocol Pedant, Literal
  Implementer, Mechanism Gamer, Boundary Smuggler, Blindspot Auditor, Fold Auditor, Platform Skeptic
  (bespoke), State Corruptor.

## Stand-downs

- UNVERIFIED-ACCEPTED: version detection reports only the evidence version, never the installed one (panel
  round 3, open question 1). The owner chose this in ruling F2 (§11); `support` says so in every run.
- DISCARDED-BELOW-FLOOR: a profile directory deleted while `list_profiles` runs turns `profiles_dir` into a
  `fail` (panel round 4, State Corruptor) - unreachable because `unrecognized_children` returns 0 on
  `NotFound` (`lifecycle.rs:298-299`, "Gone between listing the profile and reading it").
