# SP3 — Repository resolution: design

**Status:** draft, 2026-09-15; design sections approved by the owner in brainstorming; panel rounds 1-6 folded (round 6 GREEN);
awaiting the owner's review of this document.
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
| D1 | Discovery is a hand-written walk that stops at the first `.git` and validates Git's `gitdir`/`commondir` metadata; an invalid `.git` is an error, never a reason to continue upward. `GIT_DIR`, `GIT_WORK_TREE`, `GIT_CEILING_DIRECTORIES` and `core.worktree` are ignored. No new dependency. | walk (an ownership check was added after panel round 2 and deferred by the owner after round 3; §10) | measured: `git rev-parse --show-toplevel` honours a repository's own `core.worktree` and reported a different directory as the top level (reproduced by the driver); with `GIT_DIR` set git reported the cwd as the top level. measured: `gix-discover` 0.55.0 needs `-F sha1` to compile, pulls 66 transitive crates, and for a nested `.git` that is `gitdir: ../nowhere`, garbage, or an empty directory returns the PARENT repository's work dir (git also climbs past an empty `.git` directory). measured: `git rev-parse` did not run a `core.fsmonitor` script (`git status` did). V3 §13 forbids applying a parent's mapping to a nested repository; V3 §36 forbids trusting repository-controlled configuration. |
| D2 | Mappings and the global default live in `config.toml`: `default_profile`, and `[repositories.'<root>']` tables with `profile` and `agents`. | config.toml tables | measured: `toml_edit` 0.25.15 wrote a `\\?\C:\…\it's a repo` key with correct escaping and `toml` 1.1.6 read it back byte-equal; an exact duplicate key is already a parse error. reasoned: one file and one lock let SP5's `delete` check references under the lock `link` takes. SP1 test `schema_rejects_every_error_class_naming_the_key` pins `default` as an unknown key, hence `default_profile`. |
| D3 | The global default is a hand-edited `default_profile` key; SP3 reads, validates and shows it; no setter command. | hand-edited | reasoned: V3 §34 tests the global default, so it cannot be deferred; V3 §15 defines `link` as "current repository → profile" and the V3 §5.3 command words are fixed. |
| D4 | Command forms, outputs and exit codes of §7; `--repo` on the non-launch commands only; `link` accepts any valid profile name. | as proposed | reasoned: refusing an unmaterialized profile would contradict lazy initialization (§9) before `create` exists (SP5); `unlink` of nothing is idempotent like `create` (§10). |
| D5 | A mapping applies only when its root equals the discovered root. Discovery or canonicalization failure is exit 4, except for a launch with an explicit profile. `unlink --repo <path>` removes a mapping by key without discovery, so an orphan mapping can always be removed (§6.2). | as proposed; the unlink rule refined by panel round 1 (C1) | reasoned: discovery always yields the innermost root, so V3 §14.1's "longest applicable mapping" has exactly one candidate and §13's nested rule holds; a silent fallback could select a different account. Without key-based unlink a deleted or broken repository's mapping could never be removed and §16.1 would block deleting its profile forever, while §27 forbids pruning. Panel round 1 showed that running discovery for `unlink --repo` removes an enclosing repository's mapping when the path was recreated without `.git`. |

## 4. Architecture

### 4.1 Module layout

```text
crates/agent-profile/src/
  repo.rs      Discovery, discover(), canonical path identity (canonical, strip_verbatim)   [was empty]
  resolve.rs   resolve() signature and body replaced; Resolution and ResolutionSource unchanged
  config.rs    schema: default_profile, repositories; queries; link/unlink edits;
               check_case_twins moved here from adapter/mod.rs as pub(crate)
  cli.rs       argument binding and routing for current/resolve/status/link/unlink; resolved launch
  output.rs    resolve and status renderers; report `repository:` line
  error.rs     Error::Repository; NoProfile gains config_file
```

No new dependency. Module dependencies stay as in SP1 design §5: `config` → `error`, `name`; `repo` → `error`;
`resolve` → `config`, `repo`, `name`; `adapter` → `config` (for `check_case_twins`).

### 4.2 Types and functions

```rust
// repo.rs
pub enum Discovery {
    Repository(PathBuf),   // canonical working-tree root (after strip_verbatim)
    NotInRepository,
}
pub fn discover(start: &Path) -> Result<Discovery>;
pub fn canonical(path: &Path) -> io::Result<PathBuf>;   // fs::canonicalize, then strip_verbatim on Windows
pub fn strip_verbatim(path: &Path) -> PathBuf;          // identity on Unix
pub fn target_allowed(target: &Path, repository_dir: &Path) -> bool;   // §5.3
pub fn unlink_keys(cwd: &Path, repo: &OsStr) -> Vec<PathBuf>;           // §6.2 candidates, in order, deduplicated

// resolve.rs (types from SP1 unchanged: Resolution, ResolutionSource)
pub fn resolve(agent: AgentId, explicit: Option<ProfileName>, config: &Config,
               discovery: &Discovery) -> Resolution;

// config.rs
pub struct Mapping { pub profile: Option<ProfileName>, pub agents: BTreeMap<AgentId, ProfileName> }
impl Config {
    pub fn default_profile(&self) -> Option<&ProfileName>;
    pub fn mapping(&self, root: &Path) -> Option<&Mapping>;
    pub fn mappings(&self) -> impl Iterator<Item = (&Path, &Mapping)>;
    pub fn mappings_referencing(&self, profile: &ProfileName) -> Vec<Reference>;
}
pub struct Reference { pub root: PathBuf, pub agent: Option<AgentId>, pub profile: ProfileName }
pub enum LinkOutcome { Linked, Changed { old: ProfileName }, AlreadyLinked }
pub enum UnlinkOutcome { Unlinked { root: PathBuf, old: ProfileName }, NothingToRemove { shown: PathBuf } }
pub fn link(root: &AppRoot, repository: &Path, agent: Option<&AgentId>, profile: &ProfileName)
    -> Result<LinkOutcome>;
/// Tries `keys` in order under ONE lock; removes at the first key with a mapping at that field.
/// `NothingToRemove::shown` is `keys[0]`. `keys` is never empty.
pub fn unlink(root: &AppRoot, keys: &[PathBuf], agent: Option<&AgentId>) -> Result<UnlinkOutcome>;
pub(crate) fn check_case_twins(root: &AppRoot, profile: &ProfileName) -> Result<()>;

// error.rs
Error::Repository { path: PathBuf, reason: String }
    // exit 4; Display: "repository <path>: <reason>"; every reason and path in §5.3.1
Error::NoProfile { agent: String, config_file: PathBuf, in_repository: bool }
    // exit 4; Display: "no profile selected for `<agent>`; name one (agent-profile <agent> <profile>),
    //  [link this repository (agent-profile link <profile>), ]or set default_profile in <config_file>"
    //  (the bracketed hint only when in_repository)
```

The plan may add derives. Key comparison (`mapping`, `unlink`) uses `Path` equality, which compares
components: separators and a trailing separator do not matter, letter case does. A stored key is a UTF-8
string, so a non-UTF-8 root never has a mapping. `mappings_referencing` matches a stored profile name equal
to `profile` ignoring ASCII case, and returns the stored spelling in `Reference::profile`, so SP5 `delete`
cannot miss a case-only twin that names the same directory on a case-insensitive filesystem (SP1 D9).

### 4.3 Resolver (V3 §12)

`resolve` is pure; callers run discovery first.

1. `explicit` is `Some` → `Explicit`.
2. `discovery` is `Repository(r)` and `config.mapping(r)` has `agents[agent]` → `RepositoryAgent`.
3. … and has `profile` → `RepositoryDefault`.
4. `config.default_profile()` is `Some` → `GlobalDefault`.
5. otherwise → `None` (profile `None`).

`Resolution.repository` is `Some(r)` whenever the discovery passed in is `Repository(r)`, whatever the
source; callers that skip or ignore discovery pass `NotInRepository` (§7.6).

Existing callers of the SP1 `resolve(agent, explicit)` that change with the signature: `cli.rs` (launch),
`resolve.rs` test `explicit_profile_resolves_as_explicit`, the `output.rs` test helper `resolution()`, and the
`adapter/mod.rs` test that builds a `Resolution`.

## 5. Discovery (`repo::discover`)

### 5.1 Start

The start is the current working directory, or `--repo <path>` (a relative path is joined to the cwd). It is
canonicalized with `repo::canonical`, so a symlink or junction into a repository resolves to the real root
(V3 §14 example). A start that does not exist, cannot be canonicalized, or is not a directory →
`Error::Repository` (exit 4).

### 5.2 Walk

From the canonical start up to the filesystem root, at each directory `D` examine `D/.git` with
`fs::metadata` (following symlinks):

| `D/.git` | Outcome |
|---|---|
| `NotFound`, and `symlink_metadata` is also `NotFound` | continue with the parent of `D` |
| `NotFound` while `symlink_metadata` succeeds (a dangling symlink) | `Error::Repository`, exit 4 |
| any other I/O error (permission, …) | `Error::Repository`, exit 4 |
| a directory containing a regular file `HEAD` | root = `D` |
| a directory without a regular file `HEAD` | `Error::Repository` "invalid .git directory", exit 4 (never continue upward) |
| a regular file of at most 64 KiB whose content is UTF-8 and whose first line is `gitdir: <p>` (trailing CR/LF trimmed) | resolve `<p>` against `D` (an absolute `<p>` as is); refuse a network or device target (§5.3); canonicalize it; it must be a directory containing a regular file `HEAD`; if it contains an entry `commondir`, that entry must be a regular file of at most 64 KiB whose UTF-8 first line, resolved against the gitdir, passes §5.3 and is an existing directory; then root = `D` |
| a regular file failing any rule in the row above, or a `gitdir`/`commondir` that is missing, stale or not as described | `Error::Repository` naming the file and the reason, exit 4 |
| any other file type | `Error::Repository`, exit 4 |
| the walk passes the filesystem root without a `.git` | `NotInRepository` |

Consequences:
- A submodule, a nested repository and a linked worktree each carry their own `.git` at their root, so
  the innermost wins (V3 §13 "the submodule is the current repository").
- A bare repository has no `.git` entry: `NotInRepository` unless an enclosing directory is a repository.
- A start inside a `.git` directory (for example `repo/.git/objects`) reaches `repo`: root = `repo`.
- Nothing is executed and no Git configuration is read; only the `.git` entry, the existence of `HEAD`, and
  the `.git` file and `commondir` contents (both capped) are read.
- Every read target must be a regular file when checked, and on Windows `\\.\` device paths are refused by
  §5.3. A local user who swaps a checked file for a FIFO before it is opened can still block the read (§10).

### 5.3 No network or device targets

The target classified is exactly `D.join(<p>)` for a `gitdir` and `gitdir.join(<first line>)` for a `commondir`
(Rust's `join` replaces the base when the value carries a prefix or root). It is checked before any filesystem call
on it, as an allow-list; anything not allowed is refused with `Error::Repository` (exit 4):
- on Windows, allowed only when the first component (`Path::components().next()`) is a `Prefix` of kind `Disk`
  or `VerbatimDisk`, or of kind `UNC`/`VerbatimUNC` with the same server and share (compared ignoring ASCII case)
  as the prefix of `D` (a repository checked out on that network share). Refused therefore: `UNC` and
  `VerbatimUNC` on another share, `DeviceNS` (`\\.\…`), other `Verbatim` (`\\?\Volume{…}`, `\\?\GLOBALROOT`),
  and a first component that is not a prefix at all (malformed spellings such as `\\evil\\share\x`, which
  Windows still normalizes to a UNC path). Measured by panel rounds 4 and 5 (rustc 1.98, Windows 11, `Path::
  components` and `std::path::absolute`): `\\server\share\x`, `//server/share/x` and `\/server/share/x` parse as
  `UNC`, `//./pipe/x` as `DeviceNS`, `\\?\GLOBALROOT\…` as `Verbatim`; `\\evil\\share\x` has no prefix (first
  component `RootDir`) and `absolute` turns it into `\\evil\share\x`;
- on every platform, a target containing a NUL is refused (on Unix this is the only rule).

The check is a pure classifier `repo::target_allowed(target: &Path, repository_dir: &Path) -> bool`. `std::path`
parses Windows prefixes only when compiled for Windows, so the prefix rows are unit-tested on the Windows CI
runner and the NUL row on every platform.

So a `.git` file from an archive cannot make discovery open an SMB session, connect a named pipe, or wait on a
network timeout (V3 §36 "no hidden network requests"). Not detected (§10): Unix automount paths
(`/net/host/…`), Windows mapped network drive letters, and a `.git` or a gitdir path component that is itself a
symlink to a network location (followed by `fs::metadata`/`canonicalize`; creating one on Windows needs symlink
privilege or Developer Mode), and a DOS device name as the last component of a local path (`C:\repo\NUL`
normalizes to `\\.\NUL`; it reaches a local device, not a network). Refused although legitimate (§10), and only
through a `.git` file's `gitdir` or a `commondir` (a plain `.git` directory is never classified): a worktree or
submodule on a volume without a drive letter (its paths stay `\\?\Volume{…}`), a local worktree whose main
repository is on a network share, and one share spelled with two server names (host name and IP).

### 5.3.1 `Error::Repository` reasons

Every reason text, with the path the error carries. `<target>` is the joined path that was classified or checked
(§5.3), shown with `Path::display`:

| Condition | `path` | `reason` |
|---|---|---|
| start does not exist or cannot be canonicalized | the start as given | `cannot resolve the directory: <io error>` |
| start is not a directory | the start | `not a directory` |
| `link` (with or without `--repo`) or `unlink` (without `--repo`) outside a repository | the canonical start | `not inside a Git repository` |
| dangling `.git` symlink | `D/.git` | `.git is a broken symbolic link` |
| other I/O error on `.git` | `D/.git` | `cannot read .git: <io error>` |
| `.git` directory without a regular file `HEAD` | `D/.git` | `invalid .git directory: no HEAD file` |
| `.git` of another file type | `D/.git` | `.git is neither a directory nor a file` |
| `.git` file over 64 KiB, not UTF-8, or without a `gitdir: ` first line | `D/.git` | `invalid .git file: <too large \| not UTF-8 \| no gitdir line>` |
| gitdir target refused (§5.3) | `D/.git` | `gitdir points to a network or device path: <target>` |
| gitdir missing, not a directory, or without `HEAD` | `D/.git` | `gitdir <target> is missing or is not a Git directory` |
| `commondir` over 64 KiB, not UTF-8, not a regular file, unreadable, or with an empty first line | the `commondir` file | `invalid commondir file: <too large \| not UTF-8 \| not a regular file \| cannot read: <io error> \| empty>` |
| `commondir` target refused (§5.3) | the `commondir` file | `commondir points to a network or device path: <target>` |
| `commondir` target missing or not a directory | the `commondir` file | `commondir <target> is missing or is not a directory` |
| `link` on a non-UTF-8 root | the root | `a repository path that is not valid UTF-8 cannot be linked` |

Display: `repository <path>: <reason>`.

### 5.4 Path identity

- `repo::canonical` = `fs::canonicalize`, then on Windows `strip_verbatim`: `\\?\C:\x` → `C:\x`,
  `\\?\UNC\server\share\x` → `\\server\share\x`; any other verbatim form (for example `\\?\Volume{…}`) is kept
  unchanged. Measured on Windows by the panel: canonicalization normalizes letter case, 8.3 short names
  (`C:\PROGRA~1` → `C:\Program Files`), trailing separators, and junctions, so after `strip_verbatim` one
  directory yields one byte string.
- macOS case (unmeasured): a case-insensitive APFS volume may return different spellings of one directory
  from `canonicalize`. Before the plan is written, a throwaway prototype PR measures on the macOS runner
  whether two case spellings of one directory, and a symlinked start, canonicalize to identical bytes. If
  they do not, `repo::canonical` on macOS adds `fcntl(F_GETPATH)` on the opened directory, and this section
  is amended before planning.
- Linux case-insensitive mounts (WSL `/mnt/c` drvfs, ext4 casefold, CIFS, exFAT) keep the typed case through
  `realpath`; two spellings are two identities there (§10).
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
| `repositories.<key>` | the key is absolute in Unix form (starts with `/`) or in Windows form (a drive letter followed by `:\` or `:/`, or starts with `\\`), whatever the host; the value is a table. A key in the other platform's form is valid but never matches, so a configuration synced between machines keeps working. |
| `repositories.<key>.profile` | optional; a valid profile name |
| `repositories.<key>.agents` | optional; a table whose keys pass `AgentId::parse` (syntax, not "known agent") and whose values are valid profile names |
| any other field in an entry | unknown key |

Two keys that are component-equal under the host's `Path` rules (`'/a'` and `'/a/'`; on Windows `'C:\x'`,
`'C:/x'` and `'c:\x\'`) are `ConfigInvalid` naming both keys, so at most one entry can ever apply to a root
(V3 §37 "applicable mapping selection is deterministic").

An empty entry is accepted. A key that is not in canonical form is accepted but never matches unless it is
component-equal to a canonical root; it still counts in `mappings_referencing`. Profile names read from
configuration become `ProfileName` values, so every profile reaching the planner is valid by type (SP1
design §5.1 step 4). The SP1 test case `default = "work"` stays an unknown-key case.

### 6.2 Writes

All writes use `config::update` (locked, atomic, re-read and re-validated under the lock, formatting and
comments preserved; SP1 design §6.4).

`link(root, repository, agent, profile)` (the caller has discovered `repository`):
1. `repository` must be valid UTF-8, else `Error::Repository` "a repository path that is not valid UTF-8
   cannot be linked" (exit 4).
2. `check_case_twins(root, profile)`: a profile name that differs only in ASCII case from an existing
   `<root>/profiles/` entry is refused with `ProfileCaseConflict` (exit 4), the SP1 design §7.3 rule, so a
   link cannot point at a profile every launch would refuse.
3. Under the lock, find the entry whose key is component-equal to `repository` (else create one keyed by
   `repository`'s string): same profile at that field → `AlreadyLinked`, no write; a different one → set it,
   `Changed { old }`; none → set it, `Linked`.

`unlink(root, keys, agent)`, under one lock:
1. For each key in order, find the entry whose stored key is component-equal to it and that has a mapping at
   that field (`profile` with no agent, `agents.<id>` with an agent). The first such entry wins.
2. None → `NothingToRemove { shown: keys[0] }`, no write.
3. Otherwise remove that field; remove an `agents` table that becomes empty and an entry that becomes empty →
   `Unlinked { root: <stored key>, old }`.

How the CLI chooses `keys` for `unlink` (§7.2):
- Without `--repo`: discovery from the cwd. `Repository(r)` → `keys = [r]`. `NotInRepository` →
  `Error::Repository` "not inside a Git repository" (exit 4). A discovery error → exit 4.
- With `--repo <p>`: no discovery; `keys = repo::unlink_keys(cwd, p)`. An empty `p` is a usage error (exit 2).
  `p` is joined to the cwd if relative. The candidates, in order, duplicates dropped:
  1. `repo::canonical(p)`, when canonicalization succeeds;
  2. the resolved path: `repo::canonical` of the deepest existing ancestor of `p`, followed by the remaining
     components of `p` with `.` dropped and `..` applied lexically, so `../gone`, or a deleted directory under
     an 8.3 or symlinked ancestor (Windows `RUNNER~1`, macOS `/var` → `/private/var`), still matches the key
     `link` stored;
  3. `strip_verbatim` of `p` after joining it to the cwd, with no canonicalization or lexical normalization.

  So a mapping for a deleted directory, a directory recreated without `.git`, or a worktree with a stale
  `gitdir` is always removable, and an enclosing repository's mapping is never touched. If no key matches →
  `NothingToRemove`, reported with the first candidate (§7.5).

`link --repo <p>` and every other command use discovery from `p`; an empty `p` is a usage error there too.

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

### 7.2 Argument binding and routing

This replaces SP1 design §4.2 rule 4 and extends `cli::split`; the other SP1 rules keep their order.

**Top-level dispatch before Clap.** Clap removes a `--` in the first position of a subcommand's trailing
arguments (measured with clap 4.6.6: `status --` yields no tokens and `unlink -- --repo x` yields
`["--repo", "x"]`). So `cli::run` inspects the raw first argument before Clap: exactly `status`, `link`,
`unlink`, `resolve` or `current` hands every remaining raw token to the command parser below with no agent. The
other top-level reserved words stay Clap subcommands (not yet implemented), and `status`, `link`, `unlink`,
`resolve`, `current` are removed from the Clap `Command` enum and from `Command::reserved_name`; the top-level help
lists them through the `Cli` parser's `after_help` text, one usage line each (§7.3). A top-level word in another
letter case (`LINK`) is not a command and reaches the agent path: "unknown agent `LINK`" (exit 2), as today.

For the top-level five, the dispatched word IS the command word: steps 3 and 4 below are skipped, and step 5
applies to the remaining tokens, so `agent-profile link status` links a profile named `status` (refused as a
reserved profile name, V3 §6, exit 4) and `agent-profile link Create` likewise, exactly as `claude link Create`.

**Agent-scoped** (`agent-profile <agent> …`), and the top-level five, on the tokens before the first `--`:
1. Token binding: `--repo=<v>` carries its value; a bare `--repo` consumes the next token whatever its first
   byte (a missing next token → usage error "`--repo` needs a path", exit 2). Every other token starting with
   `-` is an option; every remaining token is a bare word. A token consumed as a `--repo` value is never a help
   flag, an option or a command word.
2. `-h`/`--help`, then `-V`/`--version`, among the bound options, wherever they appear. The command word for this
   step is the dispatched word for the top-level five, and otherwise the first bare word. When it is exactly one
   of the five lower-case command words, help prints that command's usage (§7.3) to stdout and exits 0
   (`agent-profile link -h` → `link` usage; `agent-profile link status -h` → `link` usage; `agent-profile
   resolve -h` → `resolve` usage); otherwise the launch usage (unchanged, including for `claude create -h`).
3. Unknown agent (unchanged; agent-scoped only).
4. The first bare word, compared ignoring ASCII case, against all 13 reserved words (V3 §5.3):
   - the exact lower-case spelling of `current`, `resolve`, `status`, `link` or `unlink` → that command,
     validated by step 5;
   - the exact lower-case spelling of any other reserved word → not yet implemented (unchanged);
   - any reserved word in another letter case → usage error "command words are lower case: `create`" (exit 2).
5. Command validation, in order, each a usage error (exit 2) with the text in the table below: a `--` anywhere;
   `--repo` more than once; an empty `--repo` value; `--json` on `status` or `resolve` → not yet implemented
   (SP5, V3 §32), and on `current`, `link`, `unlink` → unknown option; any other option (including
   `--dry-run`, `--verbose`); `link` without a profile word; an extra bare word; a `link` profile word that is
   not UTF-8.
6. No reserved bare word: a launch, with the SP1/SP2 rules; `--repo` here is an unknown option (exit 2).

| Condition | Usage error text |
|---|---|
| bare `--repo` with no next token | ``--repo` needs a path`` |
| `--repo` given twice | ``--repo` may be given only once`` |
| empty `--repo` value | ``--repo` needs a non-empty path`` |
| reserved word in another letter case | ``command words are lower case: `<lower-case word>` `` |
| `--` inside a command | `` `<command>` takes no agent arguments; remove `--` `` |
| unknown option on a command | `` unknown option "<name>" for `<command>` `` (only the part before `=`; `<command>` is the command word alone, without the agent) |
| `--json` on `status` or `resolve` | `` `--json` is not yet implemented `` (the SP1 not-yet-implemented error, exit 2) |
| `link` without a profile | `` `link` needs a profile: agent-profile [<agent>] link <profile> `` |
| extra bare word (`link` second word, or any word after `current`/`resolve`/`status`/`unlink`) | `` `link` takes one profile `` / `` `<command>` takes no arguments `` |
| non-UTF-8 `link` profile word | `the profile name is not valid UTF-8` (as a launch) |
| top-level `resolve`/`current` | `` `<command>` needs an agent: agent-profile <agent> <command> `` |

Token binding (step 1) runs for launches too. Changed launch behaviour, all previously unusual: a bare `--repo`
in a launch consumes the next token, so `claude work --repo --help` becomes an unknown-option error (exit 2)
instead of help, `claude -h --repo` becomes "`--repo` needs a path" instead of help, and `claude --repo create`
becomes an unknown-option error instead of not yet implemented.

Top-level `resolve` and `current` → the "needs an agent" usage error (exit 2), after the help check.

`--repo` values are OS strings (may be non-UTF-8).

SP1 tests whose expectations change: `reserved_first_bare_word_ignores_everything_else` (`["fake", "CREATE",
"--bogus"]` and `["fake", "Create", "x"]` become the lower-case usage error; `["fake", "--bogus", "link"]`
becomes an unknown-option usage error) and `behaviour_table_rows_with_non_zero_exits` (`["link", "work",
"extra"]` becomes "`link` takes one profile"; `["fake"]` keeps exit 4 with the new `NoProfile` text), and the
`error.rs` test `exit_codes_follow_spec_33` (the `NoProfile` row gains `config_file` and `in_repository`; a
`Repository` row is added).

### 7.3 Command usage text

Printed for `-h`/`--help` (stdout, exit 0):

```text
Usage: agent-profile <agent> current [--repo <path>]
Print the profile agent-profile would select for <agent> here.

Usage: agent-profile <agent> resolve [--repo <path>]
Show the profile, where it comes from, and the repository.

Usage: agent-profile [<agent>] status [--repo <path>]
Show the repository, its mappings, the default profile and what each agent resolves to.

Usage: agent-profile [<agent>] link <profile> [--repo <path>]
Map this repository (or only <agent> in it) to <profile>.

Usage: agent-profile [<agent>] unlink [--repo <path>]
Remove this repository's mapping (or only <agent>'s). With --repo, removes the mapping stored for that path.
```

Each block is one command's text; `--repo <path>` is described on a following line as
`  --repo <path>  Use the repository at <path> instead of the current directory`.

### 7.4 Check order for the new commands

1. Usage (exit 2).
2. The `link` profile name is validated (V3 §6, exit 4).
3. Application root and configuration load (exit 4).
4. Discovery from the cwd or `--repo` (a discovery error is exit 4). `NotInRepository` is a normal result
   for `status`, `resolve` and `current` (no mapping applies) and an error for `link` (§7.5). `unlink`
   chooses its key as in §6.2.
5. The command.

### 7.5 Output

Reports go to stdout; labels use the dry-run report's 14-column alignment (`LABEL_WIDTH` in `output.rs`, where
the new renderers live). Paths are shown with `Path::display`.

`<agent> current`: the profile name and a newline. No profile → the `NoProfile` error on stderr, exit 4.

`<agent> resolve`:
```text
agent:        claude
profile:      personal
source:       repository agent mapping
repository:   C:\src\acme
```
`repository:` is `none` outside a repository. No profile → `profile:` and `source:` show `none`, the report is
printed to stdout, then the same `NoProfile` error is written to stderr, exit 4 (V3 §11 "same no-profile
condition").

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
- `repository:` is `not in a repository` for `NotInRepository`; `mapping:`, `agents:` and `default:` show
  `none` when absent; `agents:` lists `id=profile` pairs sorted by id, separated by `, `.
- One line per known agent (`adapter::known_agents()`; debug builds include `fake`), `none` when nothing
  resolves.
- One `note:` line per stored key that is a proper ancestor (by components) of the discovered root, ordered by the stored key string.
- Exit 0. A discovery error is exit 4 like every command.

`<agent> status`: the same report with only that agent's line, followed by a `presence:` line for the resolved
profile: `materialized`, `known`, `absent`, or `conflicts with <existing>` when `check_case_twins` refuses the
profile. No `presence:` line when nothing resolves. Exit 0.

`link` (exit 0), one line:
```text
linked C:\src\acme -> work
linked claude: C:\src\acme -> personal
changed C:\src\acme: personal -> work
changed claude: C:\src\acme: personal -> work
already linked C:\src\acme -> work
already linked claude: C:\src\acme -> personal
```
`<agent> link` adds `note:         profile personal has not been launched with claude yet` when the adapter's
presence is `Absent`. `link` with `NotInRepository` → `Error::Repository` "not inside a Git repository"
(exit 4); a non-UTF-8 root → `Error::Repository` "a repository path that is not valid UTF-8 cannot be linked"
(exit 4, §6.2).

`unlink` (exit 0):
```text
unlinked C:\src\acme (was work)
unlinked claude: C:\src\acme (was personal)
no mapping to remove for C:\src\acme
no mapping to remove for claude: C:\src\acme
```
The path shown is the stored key when a mapping was removed, otherwise the first candidate (`NothingToRemove::shown`,
§6.2). After
`no mapping to remove`, one `note:` line follows, ordered by the stored key string, for each stored key that is a proper ancestor
of that path (`note:         C:\src has a mapping; remove it with agent-profile unlink --repo C:\src`), and a
top-level `unlink` whose entry holds only agent mappings adds
`note:         agent mappings remain: claude=personal; remove them with agent-profile <agent> unlink`.

### 7.6 Launch (SP1 design §5.1 step 4, SP2 design §4.3)

- No profile word: discovery from the cwd runs after configuration load; a discovery error exits 4. No profile
  resolves → `NoProfile` (exit 4).
- Explicit profile: discovery runs only with `--dry-run` or `--verbose`, to fill the report; its errors are
  ignored (the report shows `repository:   none`). Without those options no discovery runs, so an unreachable
  or hostile working directory can never slow or block an explicit launch.
- The dry-run and verbose reports show `repository:` from the resolution and the real source label.
- The SP2 order after resolution is unchanged: conflict scan, plan (executable discovery, case twin), dry run,
  initialization, launch.

## 8. Testing (V3 §34 "Resolution")

Every expected path in a test is passed through `repo::canonical` first (Windows runner temp directories use
8.3 names such as `RUNNER~1`; macOS temp directories live under `/var`, a symlink to `/private/var`).

### 8.1 `repo` unit tests (no `git` needed)

Temp-directory layouts: a `.git` directory with `HEAD` from its root and a deep subdirectory; a worktree `.git`
file with `gitdir` and `commondir`; a submodule `.git` file into `.git/modules/x`; a nested repository inside
another; a start inside `.git/objects`; no `.git` anywhere; a non-existent start; a start that is a file.
Refusals, each asserting `Error::Repository` and never the parent root: an empty `.git` directory; `HEAD` as a
directory; garbage in a `.git` file; a missing or stale `gitdir`; a `gitdir` without `HEAD`; a missing
`commondir` target; a `.git` file or `commondir` over 64 KiB; a `.git` file with CRLF line endings (accepted); a `commondir` that is a directory; non-UTF-8
content; a `gitdir` or `commondir` naming a UNC, `\\.\` or non-drive `\\?\` target, and one containing a NUL (both
refused before any filesystem call, so the tests never touch a network). Unix only: a symlinked start, a dangling
`.git` symlink, and a FIFO `commondir` (created with `mkfifo` through `Command`; `discover` runs on a thread and
the test fails after a bounded wait instead of hanging CI). Windows only: a junction start (`mklink /J` needs no
privilege); `strip_verbatim` for drive, UNC and other verbatim forms.

### 8.2 `repo` against real `git`

A test helper runs `git` hermetically: `env_remove` of every `GIT_*` variable, `GIT_CONFIG_GLOBAL` set to an
empty temp file, `GIT_CONFIG_NOSYSTEM=1`, and `-c user.name=t -c user.email=t@t -c commit.gpgsign=false`. Each
layout makes an empty commit. Layouts: `git init`; `git worktree add`; `git submodule add` with
`-c protocol.file.allow=always` on that command. For each clean layout `discover` equals `repo::canonical` of
the helper's `git rev-parse --show-toplevel`. For the broken nested layouts of D1, `discover` returns
`Error::Repository`.

### 8.3 Resolver and configuration

- `resolve` table: each precedence step winning over the ones below it; exact-root applicability; a nested
  repository not inheriting its parent's mapping (the V3 §34 "longest applicable mapping" row); `NotInRepository`
  with and without `default_profile`; `repository` populated for every source.
- `repo::target_allowed` table (Windows only; the NUL row on every platform): `\\server\share\x`, `//server/share/x`,
  `\/server/share/x`, `//./pipe/x`, `\\?\UNC\s\x`, `\\?\Volume{…}\x`, `\\?\GLOBALROOT\x`, the malformed
  `\\evil\\share\x` (no prefix) refused; `C:\repo` joined with `\??\UNC\s\x` classified as the joined path; `C:\x`
  and `\\?\C:\x` allowed; a
  UNC target on the same server and share as the repository directory allowed, a different share refused; a NUL
  refused.
- Schema accept/reject rows for every rule of §6.1, each naming the key, including keys in the other platform's
  absolute form.
- `link`/`unlink`: every outcome; comments and formatting preserved; empty `agents` and empty entry removed;
  component-equal keys (trailing separator, `/` versus `\` on Windows); two component-equal keys in one file
  rejected naming both (`'/a'` and `'/a/'` on Unix; `'C:\x'`, `'C:/x'`, `'c:\x\'` on Windows); the case-twin
  refusal; `mappings_referencing` with a case-only twin, a non-canonical stored key, and an agent-field reference.
- `unlink --repo` key choice (§6.2), as a pure function over (cwd, `p`, filesystem state): canonical key; a
  deleted directory given relative with `..`; a deleted directory under a non-canonical temp path; the literal
  fallback; an empty value rejected.
- CLI routing as pure functions: every token sequence of §7.2 (including `status --`, `unlink -- --repo x`,
  `claude link work --repo -h`, `claude LINK`, `claude Create`, `claude --repo`, `link status`, `link Create`,
  top-level `link -h`, `link status -h`, `resolve -h`, `claude -h --repo`),
  and the decision whether a
  launch needs discovery (explicit profile without `--dry-run`/`--verbose` → no).

### 8.4 End to end (`fake-agent`)

`tests/support` `Root::agent_profile` sets `current_dir` to a guarded temp directory by default, so no e2e test
discovers this project's own checkout. Through the binary: a resolved launch from each source; `current`,
`resolve` (stdout report plus stderr error on none), `status` (ancestor note), `<agent> status` (each presence
line); `link`/`unlink` with and without `--repo`; `link` and `unlink` outside a repository (exit 4); `unlink
--repo` for a deleted directory, a directory recreated without `.git` inside an enclosing mapped repository
(the enclosing mapping survives), and a worktree with a stale `gitdir`; a worktree and a submodule each
resolving their own mapping; an explicit launch inside a broken repository launches, with and without
`--dry-run`; a resolved launch inside it exits 4; every usage error of §7.2 with its text; `link` through a
symlinked (Unix) or junction (Windows) `--repo` stores the canonical root, and a resolved launch from the real
path finds it; `link --repo <missing>` and `<agent> resolve --repo <missing>` exit 4; for each of the five sources,
`current`, `resolve` and `status` agree on the profile; the `<agent> link` "not launched yet" note and both
`unlink` notes; the top-level help listing the five command words; a non-UTF-8 `--repo` value (Unix).

### 8.5 Environment guard

Every test that expects `NotInRepository`, and every e2e temp directory, first checks that no ancestor of the
directory has a `.git` entry (`Path::ancestors` with `symlink_metadata`, not `discover`), and fails with that
reason if one does.

## 9. Documentation

- README: the resolution order, the new commands, a `config.toml` example with `default_profile` and a
  mapping.
- TODO.md: close "SP3 open decisions"; add the known limits below.
- ROADMAP.md: SP3 done at merge.

## 10. Known limits

- Git layouts that rely on `core.worktree`, `GIT_DIR` or `GIT_WORK_TREE` are not honoured (D1); `--repo` is
  the explicit override.
- A bare repository is not a repository for resolution.
- A repository root that is not valid UTF-8 cannot be linked.
- A moved repository's mapping stays under the old path; resolution then falls back to `default_profile`
  until the repository is linked again. The SP5 `repositories` report shows the orphan.
- On a Linux case-insensitive mount, two letter-case spellings of one directory are two repository identities.
- The global default is set only by editing `default_profile`.
- Profile names in `config.toml` are validated with the host's rules (V3 §6), so a configuration written on Unix
  with a name Windows forbids (for example `aux` or `work.`) is invalid on Windows and every command there exits
  4 until the name is edited.
- An agent mapping for an agent this build does not know (for example `agents.gemini` from a newer version, or
  `agents.fake` written by a debug build) cannot be removed with `<agent> unlink`, which refuses unknown agents;
  it is removed by editing `config.toml`, and SP5's `delete` refusal must name the key and field to edit.
- Shared multi-user machines (deferred by the owner after panel round 3, to be revisited with `doctor` in SP5
  together with a `safe.directory`-style escape hatch): discovery does not check who owns a `.git`. Another
  local user who can create `C:\.git` or `/tmp/.git` can make commands in non-repository directories below it
  exit 4, or turn that directory into a repository root; resolution never silently selects an unmapped profile,
  and `link` prints the root it maps. The same local user can swap a checked file for a FIFO before it is opened
  and block discovery.
- Unix automount paths and Windows mapped network drive letters in a `gitdir`/`commondir` (for example
  `/net/host/…` or `Z:\…`) are not detected and may start a mount or a network connection, and a DOS device name
  as the last component of a local target reaches that local device (§5.3).
- On Windows the network refusal (§5.3) also refuses legitimate layouts: a repository on a volume mounted without a
  drive letter, a local worktree whose main repository is on a network share, and a share reached through two
  server spellings (host name versus IP address). Such repositories exit 4 until `--repo` or a drive letter is
  used.
- A mapping made in the main checkout does not apply in its linked worktrees, which are separate repositories
  (V3 §14.2); `status` in a worktree does not mention the main checkout's mapping.
- The `repositories` report, orphan output, JSON and `delete` are SP5.
- macOS case identity is unmeasured until the prototype PR (§5.4).
