# SP3 — Repository resolution: design

**Status:** draft, 2026-09-15; design sections approved by the owner in brainstorming; awaiting the panel
review and the owner's review of this document.
**Branch:** `sp3-resolution` (from `main` at `7f62a7d`).
**Oracle:** `agent-profile-implementation-spec-v3.md` ("V3" below). Where this document and V3 disagree, V3
wins; report the conflict instead of resolving it silently.
**Previous sub-projects:** SP1 `docs/superpowers/specs/2026-09-14-sp1-core-launch-design.md` (merged
`316980b`), SP2 `docs/superpowers/specs/2026-09-15-sp2-adapters-design.md` (merged `314c92d`).

## 1. Goal

SP3 delivers V3 §35 phase 2 (repository discovery, canonical repository identity, mapping storage,
precedence, `resolve`, `current`, `status`) and pulls `link`/`unlink` forward from phase 5 (ROADMAP), so
resolution never ships without a way to create the mappings it resolves. It ends with
`agent-profile <agent>` launching the profile the V3 §12 precedence selects for the current directory.

## 2. Scope

### 2.1 In scope

| Area | V3 |
|---|---|
| Git repository and worktree discovery, submodules and nested repositories as independent repositories | §13, §14.2 |
| Canonical repository identity | §14 |
| Mapping and global-default storage in `config.toml` | §12, §14, §17, §18 |
| The single resolver with the full precedence | §12 |
| `<agent> current`, `<agent> resolve`, `status`, `<agent> status` | §11, §12, §35 phase 2 |
| `link`, `<agent> link`, `unlink`, `<agent> unlink`, `--repo` | §13, §15 |
| Repository-resolved launch (no profile word) | §5.1 |
| `mappings_referencing(profile)` query for SP5 `delete` | §16.1, §31 |
| Resolution contract tests | §34 "Resolution" |

### 2.2 Out of scope

| Area | Lands in |
|---|---|
| `repositories` report, orphan-mapping report output | SP5 (§27) |
| JSON output for `status` and `resolve` | SP5 (§32) |
| `create`, `delete` (the referenced-profile refusal uses SP3's query), `list`, `profiles`, `agents`, `doctor`, completions | SP5 |
| A command that sets the global default | not planned (D3) |
| Pruning mappings | never in v0.1 (§27) |

## 3. Decisions and their evidence

Each fork was consulted with an independent subagent first (AGY-FIRST on subagents, owner's standing
instruction); the owner decided. Brief: `.clavity/seams/sp3-forks.md`. Evidence marks: **measured** (run
on this machine on 2026-09-15: git 2.55.0.windows.5, rustc 1.98.0, Windows 11 NTFS), **reasoned**,
**unmeasured**.

| # | Decision | Owner choice | Evidence |
|---|---|---|---|
| D1 | Discovery is a hand-written walk that stops at the first `.git` and validates Git's `gitdir`/`commondir` metadata; an invalid `.git` is an error, never a reason to continue upward. `GIT_DIR`, `GIT_WORK_TREE`, `GIT_CEILING_DIRECTORIES` and `core.worktree` are ignored. No new dependency. | walk | measured: `git rev-parse --show-toplevel` honours a repository's own `core.worktree` and reported a different directory as the top level (reproduced by the driver); with `GIT_DIR` set git reported the cwd as the top level. measured: `gix-discover` 0.55.0 needs `-F sha1` to compile, pulls 66 transitive crates, and for a nested `.git` that is `gitdir: ../nowhere`, garbage, or an empty directory returns the PARENT repository's work dir (git also climbs past an empty `.git` directory). measured: `git rev-parse` did not run a `core.fsmonitor` script (`git status` did). V3 §13 forbids applying a parent's mapping to a nested repository; V3 §36 forbids trusting repository-controlled configuration. |
| D2 | Mappings and the global default live in `config.toml`: `default_profile`, and `[repositories.'<root>']` tables with `profile` and `agents`. | config.toml tables | measured: `toml_edit` 0.25.15 wrote a `\\?\C:\…\it's a repo` key with correct escaping and `toml` 1.1.6 read it back byte-equal; an exact duplicate key is already a parse error. reasoned: one file and one lock let SP5's `delete` check references under the lock `link` takes. SP1 test `schema_rejects_every_error_class_naming_the_key` pins `default` as an unknown key, hence `default_profile`. |
| D3 | The global default is a hand-edited `default_profile` key; SP3 reads, validates and shows it; no setter command. | hand-edited | reasoned: V3 §34 tests the global default, so it cannot be deferred; V3 §15 defines `link` as "current repository → profile" and the §5.3 command words are fixed. |
| D4 | Command forms, outputs and exit codes of §7; `--repo` on the non-launch commands only; `link` accepts any valid profile name. | as proposed | reasoned: refusing an unmaterialized profile would contradict lazy initialization (§9) before `create` exists (SP5); `unlink` of nothing is idempotent like `create` (§10). |
| D5 | A mapping applies only when its root equals the discovered root. Discovery or canonicalization failure is exit 4, except for a launch with an explicit profile. An orphan mapping can be removed with `unlink --repo <absolute path>` matched literally. | as proposed | reasoned: discovery always yields the innermost root, so §14.1's longest match is satisfied and §13's nested rule holds; a silent fallback could select a different account. Without literal unlink a deleted repository's mapping could never be removed and §16.1 would block deleting its profile forever, while §27 forbids pruning. |

## 4. Architecture

### 4.1 Module layout

```text
crates/agent-profile/src/
  repo.rs      Discovery, discover(), canonical path identity (strip_verbatim)   [was empty]
  resolve.rs   resolve() body replaced; types unchanged
  config.rs    schema: default_profile, repositories; queries; link/unlink edits
  cli.rs       routing for current/resolve/status/link/unlink; --repo; resolved launch
  output.rs    resolve and status renderers; report `repository:` line
  error.rs     Error::Repository; NoProfile message names the fixes
```

No new dependency.

### 4.2 Types and functions

```rust
// repo.rs
pub enum Discovery {
    Repository(PathBuf),   // canonical working-tree root (after strip_verbatim)
    NotInRepository,
}
pub fn discover(start: &Path) -> Result<Discovery>;
pub fn canonical(path: &Path) -> io::Result<PathBuf>;   // fs::canonicalize + strip_verbatim

// resolve.rs (types from SP1 unchanged: Resolution, ResolutionSource)
pub fn resolve(agent: AgentId, explicit: Option<ProfileName>, config: &Config,
               discovery: &Discovery) -> Resolution;

// config.rs
pub struct Mapping { pub profile: Option<ProfileName>, pub agents: BTreeMap<AgentId, ProfileName> }
impl Config {
    pub fn default_profile(&self) -> Option<&ProfileName>;
    pub fn mapping(&self, root: &Path) -> Option<&Mapping>;            // exact byte match
    pub fn mappings(&self) -> impl Iterator<Item = (&Path, &Mapping)>;
    pub fn mappings_referencing(&self, profile: &ProfileName) -> Vec<(PathBuf, Option<AgentId>)>;
}
pub fn link(root: &AppRoot, repository: &Path, agent: Option<&AgentId>, profile: &ProfileName)
    -> Result<LinkOutcome>;          // Linked | Changed { old } | AlreadyLinked
pub fn unlink(root: &AppRoot, repository: &Path, agent: Option<&AgentId>)
    -> Result<UnlinkOutcome>;        // Unlinked { old } | NothingToRemove

// error.rs
Error::Repository { path: PathBuf, reason: String }   // exit 4
```

`Mapping` and outcome field names are fixed here; the plan may add derives. Repository keys are stored as
UTF-8 strings; `mapping(root)` compares `root.to_str()` with the key bytes, so a non-UTF-8 root never
matches.

### 4.3 Resolver (V3 §12)

`resolve` is pure; callers run discovery first.

1. `explicit` is `Some` → `Explicit`.
2. `discovery` is `Repository(r)` and `config.mapping(r)` has `agents[agent]` → `RepositoryAgent`.
3. … and has `profile` → `RepositoryDefault`.
4. `config.default_profile()` is `Some` → `GlobalDefault`.
5. otherwise → `None` (profile `None`).

`Resolution.repository` is `Some(r)` whenever discovery found `r`, whatever the source.

## 5. Discovery (`repo::discover`)

### 5.1 Start

The start is the current working directory, or `--repo <path>` (relative to the cwd). It is canonicalized
first (`repo::canonical`), so a symlink into a repository resolves to the real root (V3 §14 example). A
start that does not exist or cannot be canonicalized → `Error::Repository` (exit 4).

### 5.2 Walk

From the canonical start up to the filesystem root, at each directory `D` examine `D/.git` with
`fs::metadata` (following symlinks):

| `D/.git` | Outcome |
|---|---|
| `NotFound` | continue with the parent of `D` |
| any other I/O error (permission, …) | `Error::Repository`, exit 4 |
| a directory containing a regular file `HEAD` | root = `D` |
| a directory without `HEAD` | `Error::Repository` "invalid .git directory", exit 4 |
| a regular file whose first line is `gitdir: <p>` (trailing CR/LF trimmed) | resolve `<p>` against `D` (absolute `<p>` as is), canonicalize; it must be a directory containing `HEAD`; if it contains a `commondir` file, that file's first line resolved against the gitdir must be an existing directory; then root = `D` |
| a regular file with any other content, content that is not UTF-8, more than 64 KiB, or a `gitdir`/`commondir` that is missing, stale or lacks `HEAD` | `Error::Repository` naming the file and the reason, exit 4 |
| a dangling symlink (`metadata` fails with `NotFound` while `symlink_metadata` succeeds) or another file type | `Error::Repository`, exit 4 |
| the walk passes the filesystem root without a `.git` | `NotInRepository` |

Consequences:
- A submodule, a nested repository and a linked worktree each carry their own `.git` at their root, so
  the innermost wins (V3 §13 "the submodule is the current repository").
- A bare repository has no `.git` entry: `NotInRepository` unless an enclosing directory is a repository.
- A start inside a `.git` directory (for example `repo/.git/objects`) reaches `repo`: root = `repo`.
- Nothing is executed and no Git configuration is read; only `.git`, `HEAD` (existence), `commondir` and the
  `.git` file content are read.

### 5.3 Path identity

- `repo::canonical` = `fs::canonicalize` followed by `strip_verbatim` on Windows: `\\?\C:\x` → `C:\x`,
  `\\?\UNC\server\share\x` → `\\server\share\x`; any other verbatim form (for example `\\?\Volume{…}`) is
  kept unchanged. Every path used as a mapping key or looked up goes through this one function, so matching
  is byte-exact.
- macOS case (unmeasured): a case-insensitive APFS volume may return different spellings of one directory
  from `canonicalize`. Before the plan is written, a throwaway prototype PR measures on the macOS runner
  whether two case spellings of one directory, and a symlinked start, canonicalize to identical bytes. If
  they do not, `repo::canonical` on macOS adds `fcntl(F_GETPATH)` on the opened directory, and this section
  is amended before planning.
- A root that is not valid UTF-8 is discovered normally but can never match a mapping; `link` refuses it
  (exit 4). Resolution falls back to `default_profile`, which is not inheritance from a parent.

## 6. Configuration

### 6.1 Schema

The SP1 strict schema (SP1 design §6.2) gains two top-level keys; every other unknown key stays an error.

```toml
default_profile = "work"

[agents.codex]
executable = "C:\\tools\\codex.exe"

[repositories.'C:\src\acme']
profile = "work"
agents = { claude = "personal" }
```

Validation on every read (each failure is `ConfigInvalid` naming the key, exit 4):

| Key | Rule |
|---|---|
| `default_profile` | a string; `ProfileName::parse(_, Platform::host())` (V3 §6 incl. reserved words) |
| `repositories` | a table |
| `repositories.<key>` | the key is an absolute path (`Path::is_absolute`); the value is a table |
| `repositories.<key>.profile` | optional; a valid profile name |
| `repositories.<key>.agents` | optional; a table whose keys pass `AgentId::parse` (syntax, not "known agent") and whose values are valid profile names |
| any other field in an entry | unknown key |

An empty entry is accepted. A key that is not in canonical form is accepted but never matches; it still
counts in `mappings_referencing`. Profile names read from configuration become `ProfileName` values, so
every profile reaching the planner is valid by type (SP1 design §5.1 step 4).

The SP1 test case `default = "work"` stays an unknown-key case.

### 6.2 Writes

All writes use `config::update` (locked, atomic, re-read and re-validated under the lock, formatting and
comments preserved; SP1 design §6.4).

`link(root, repository, agent, profile)`:
1. `repository` must be valid UTF-8, else `Error::Repository` (exit 4).
2. A profile name that differs only in ASCII case from an existing `<root>/profiles/` entry is refused with
   `ProfileCaseConflict` (exit 4), the SP1 design §7.3 rule, reusing the same check as the planner.
3. Under the lock: if the same profile is already mapped at that field → `AlreadyLinked`, no write; if a
   different one → set it, `Changed { old }`; otherwise create the entry or field → `Linked`.

`unlink(root, repository, agent)`:
1. Under the lock: remove `profile` (no agent) or `agents.<id>`; remove an `agents` table that becomes
   empty and an entry that becomes empty → `Unlinked { old }`.
2. Nothing mapped at that field → `NothingToRemove`, no write.

Orphan unlink: when `--repo <path>` is given to `unlink`/`<agent> unlink`, `<path>` is absolute, and
canonicalization fails with `NotFound`, the key used is `strip_verbatim(<path>)` as given, without discovery.
A relative non-existent path is `Error::Repository`. `link`, `resolve`, `current` and `status` never use
the literal form.

## 7. Commands

### 7.1 Grammar

```text
agent-profile <agent> [<profile>] [--dry-run] [--verbose] [-- <agent args>...]
agent-profile <agent> current  [--repo <path>]
agent-profile <agent> resolve  [--repo <path>]
agent-profile <agent> status   [--repo <path>]
agent-profile <agent> link <profile> [--repo <path>]
agent-profile <agent> unlink   [--repo <path>]
agent-profile status [--repo <path>]
agent-profile link <profile> [--repo <path>]
agent-profile unlink [--repo <path>]
```

- `--repo <path>` and `--repo=<path>` are both accepted; the value is an OS string (may be non-UTF-8).
- Top-level `current` or `resolve` → usage error (exit 2): "`resolve` needs an agent: agent-profile
  <agent> resolve".
- On these commands a `--`, an extra bare word, a missing `<profile>` for `link`, or any other option →
  usage error (exit 2); an unknown option's error echoes only its name (SP2).
- `--json` → not yet implemented (SP5), as today. `--dry-run`/`--verbose` on these commands → usage error.
- `--repo` on a launch → unknown option (exit 2).
- The reserved words `agents`, `profiles`, `list`, `create`, `delete`, `doctor`, `repositories`,
  `completions` (top-level or after an agent) stay not yet implemented.

### 7.2 Check order for the new commands

1. Usage (exit 2).
2. The `link` profile name is validated (V3 §6, exit 4).
3. Application root and configuration load (exit 4).
4. Discovery from the cwd or `--repo` (exit 4), except the orphan-unlink literal form (§6.2).
5. The command.

### 7.3 Output

Reports go to stdout; labels use the dry-run report's 14-column alignment (`output::LABEL_WIDTH`). Paths are
shown with `Path::display`.

`<agent> current`: the profile name and a newline. No profile → the `NoProfile` error on stderr, exit 4.

`<agent> resolve`:
```text
agent:        claude
profile:      personal
source:       repository agent mapping
repository:   C:\src\acme
```
`repository:` is `none` outside a repository. No profile → `profile:` and `source:` show `none`, the report
is printed, then exit 4.

`status`:
```text
repository:   C:\src\acme
mapping:      work
agents:       claude=personal
default:      work
claude:       personal (repository agent mapping)
codex:        work (repository mapping)
aider:        work (repository mapping)
note:         C:\src has a mapping that does not apply to this repository
```
- `repository:` is `not in a repository` when there is none; `mapping:`, `agents:` and `default:` show
  `none` when absent; `agents:` lists `id=profile` pairs sorted by id, separated by `, `.
- One line per known agent (`adapter::known_agents()`; debug builds include `fake`), each `none` when
  nothing resolves.
- One `note:` line per mapped root that is a proper ancestor of the discovered root.
- Exit 0.

`<agent> status`: the same report with only that agent's line, followed by
`presence:     materialized` or `absent` for the resolved profile (`Adapter::presence`); no `presence:` line
when nothing resolves. Exit 0.

`link`: `linked C:\src\acme -> work`; an agent link prefixes the agent: `linked claude: C:\src\acme ->
personal`. `Changed` prints `changed <old> -> <new>` instead of `linked`; `AlreadyLinked` prints
`already linked`. `<agent> link` adds `note: profile work has not been launched with claude yet` when the
adapter's presence is `Absent`. Exit 0.

`unlink`: `unlinked C:\src\acme (was work)` (agent form `unlinked claude: …`), or `no mapping to remove`.
Exit 0.

### 7.4 Launch (SP1 design §5.1 step 4, SP2 design §4.3)

- No profile word: discovery from the cwd runs after configuration load; a discovery error exits 4. No
  profile resolves → `NoProfile`, whose message becomes: "no profile selected for `<agent>`; name one
  (agent-profile <agent> <profile>), link this repository (agent-profile link <profile>), or set
  default_profile in <config path>".
- Explicit profile: discovery still runs; its errors are ignored and `Resolution.repository` is `None`.
- The dry-run and verbose reports show `repository:` from the resolution and the real source label.
- The SP2 order after resolution is unchanged: conflict scan, plan (discovery of the executable, case twin),
  dry run, initialization, launch.

## 8. Testing (V3 §34 "Resolution")

### 8.1 `repo` unit tests (no `git` needed)

Temp-directory layouts: a `.git` directory with `HEAD` from its root and a deep subdirectory; a worktree
`.git` file with `gitdir` and `commondir`; a submodule `.git` file into `.git/modules/x`; a nested repository
inside another; a start inside `.git/objects`; no `.git` anywhere; a non-existent start. Refusals, each
asserting `Error::Repository` and never the parent root: an empty `.git` directory; garbage in a `.git` file;
a missing or stale `gitdir`; a `gitdir` without `HEAD`; a missing `commondir` target; a `.git` file over
64 KiB; non-UTF-8 content. Unix only: a symlinked start and a dangling `.git` symlink. Windows: `strip_verbatim`
for drive, UNC and other verbatim forms.

### 8.2 `repo` against real `git`

Tests build layouts with the `git` executable (present on CI runners and this machine): `git init`,
`git worktree add`, `git submodule add` with `-c protocol.file.allow=always` on that command only. For each
clean layout, `discover` equals `repo::canonical` of `git rev-parse --show-toplevel`. For the broken nested
layouts of D1, `discover` returns `Error::Repository`.

### 8.3 Resolver and configuration

- `resolve` table: each precedence step winning over the ones below it; exact-root applicability; a nested
  repository not inheriting its parent's mapping; `NotInRepository` with and without `default_profile`;
  `repository` populated for every source.
- Schema accept/reject rows for every rule of §6.1, each naming the key.
- `link`/`unlink`: all outcomes; comments and formatting preserved; empty `agents` and empty entry removed;
  the case-twin refusal; orphan literal-key unlink; `mappings_referencing`.

### 8.4 End to end (`fake-agent`)

Through the binary: a resolved launch from each source; `current`, `resolve`, `status`, `<agent> status`;
`link`/`unlink` with and without `--repo`; a worktree and a submodule each resolving their own mapping; an
explicit launch inside a broken repository launches; a resolved launch inside it exits 4; every exit code and
usage error of §7.

### 8.5 Environment guard

Every test that expects `NotInRepository` first asserts that its temp directory is not inside a Git
repository and fails with that reason if it is.

## 9. Documentation

- README: the resolution order, the new commands, a `config.toml` example with `default_profile` and a
  mapping.
- TODO.md: close "SP3 open decisions"; add the known limits below.
- ROADMAP.md: SP3 done at merge.

## 10. Known limits

- Git layouts that rely on `core.worktree`, `GIT_DIR` or `GIT_WORK_TREE` are not honoured (D1); `--repo`
  is the explicit override.
- A bare repository is not a repository for resolution.
- A repository root that is not valid UTF-8 cannot be linked.
- The global default is set only by editing `default_profile`.
- The `repositories` report, orphan output, JSON and `delete` are SP5.
- macOS case identity is unmeasured until the prototype PR (§5.3).
