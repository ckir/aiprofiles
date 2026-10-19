# SP5b — `--json` output and the `repositories` report

Status: APPROVED by the owner on 2026-10-19 with closed token sets (§10 call 11 resolved: closed). agy-first
ALIGNED; adversarial panel GREEN at round 3 (see "Consults").
V3 (`agent-profile-implementation-spec-v3.md`) wins over this document; §1
records every place this document goes past it. Second slice of SP5 as the owner split it on 2026-10-19:
SP5a lifecycle (merged, `9039ee1`) → **SP5b `--json` + `repositories`** → SP5c `doctor` → SP5d completions,
docs and CI-pinning debt. The ordering consult is `.clavity/seams/sp5-order.md` (agy-first, aligned).

## 1. Intent

Every report the tool prints today is text for a person. V3 §32 requires machine-readable JSON for `agents`,
`profiles`, `status`, `resolve` and `doctor`, with schemas stable within a release series, and V3 §27 requires a
read-only `repositories` report of the stored mappings. SP5b delivers the JSON for the four commands that exist
(`doctor` is SP5c and will join the same contract), and the `repositories` report in both forms, so that a
script, an editor integration or a shell prompt can ask the tool what it would do and trust the answer's shape.

Goals
- `--json` on `status`, `<agent> status`, `<agent> resolve`, `agents`, `profiles`, `list`, `<agent> list` and
  the new `repositories`: one JSON document on stdout, the same facts the text report carries, no fewer.
- One envelope and one set of conventions for every document, written down once (§3), so SP5c's `doctor`
  document is a new schema under the same rules rather than a new style.
- `repositories`: every stored mapping with what it points at and whether that still exists, never exiting
  non-zero because of what it finds (V3 §27: reported, never silently removed).

Non-goals
- `--json` on a launch or `--dry-run` (V3 §32 does not list it; the dry-run report is a different document
  with secrets to redact, and gets its own slice if ever). It stays `NotYetImplemented`, as today.
- `--json` on `create`, `delete`, `link`, `unlink`, `current` (writes and the one-word answer have an exit code
  and a line; nothing to structure). `current --json` stays an unknown option; the others stay as today.
- `repositories prune` (V3 §27 forbids it in V0.1). `doctor` (SP5c). Any change to the text reports.
- Fixing mapping keys that never match (TODO: verbatim `\\?\` keys): the report shows them; it does not repair.

Anti-goals — a result that "works" but is a failure
- A JSON document that names a support level without the full capability matrix in the same object (TODO.md,
  SP4 design §6.1; owner ruling 3).
- Any secret or environment value in a document (V3 §36). None of the five documents carries an environment.
- A document whose shape depends on the host platform or on which agents are installed (keys must not come
  and go; absent is `null`).
- `--json` changing an exit code, or stderr, or writing a document that is not valid JSON on a failure path.

## 2. Ground truth in the repository (read 2026-10-19 at main `9039ee1`)

- `--json` is parsed and refused today in two places: a launch (`cli.rs:442`, `Error::NotYetImplemented
  { command: "--json" }`, exit 2) and every `CommandWord` in `validate_command` (`cli.rs:504-513`: `status`,
  `resolve`, `create`, `delete`, `list`, `agents`, `profiles`); `link`/`unlink`/`current` report it as an
  unknown option. Tests pin both (`cli.rs:1311`, `:1324`, `:1394-1397`, `:1505`).
- `repositories` is a clap stub (`cli.rs:72`, `Command::Repositories(Rest)`) turned into `NotYetImplemented`
  by `reserved_name()` (`cli.rs:87-94`). SP3 and SP5a moved every implemented word out of that enum into
  `CommandWord` (`cli.rs:99-110`) and dispatch them before clap in `run` (`cli.rs:204`); `repositories` follows.
- The text renderers and the data they take: `output::resolve_lines(&Resolution)` (`output.rs:201`),
  `output::status_lines(&Discovery, &Config, &[(Resolution, Option<String>)])` (`output.rs:212`),
  `output::agents_lines(&[(&AdapterMetadata, ExecutableStatus)])` (`output.rs:401`),
  `output::profiles_lines(&Listing)` / `list_lines(&Listing, agent)` (`output.rs:352`, `:373`). `run_command`
  (`cli.rs:700`) builds those inputs; `write_lines` / `write_out` (`cli.rs:1011-1022`) are the only stdout
  writers and turn a write failure into `Error::Io` (exit 1).
- Data types: `Resolution { agent, profile: Option<ProfileName>, source: ResolutionSource, repository:
  Option<PathBuf> }` (`resolve.rs:34`) with `ResolutionSource::{Explicit, RepositoryAgent, RepositoryDefault,
  GlobalDefault, None}` (`resolve.rs:21-29` gives the text labels); `Discovery::{Repository(PathBuf),
  NotInRepository}` (`repo.rs:18`); `Mapping { profile: Option<ProfileName>, agents: BTreeMap<AgentId,
  ProfileName> }` (`config.rs:95`); `Config::mappings()` ordered by the stored key string (`config.rs:160`),
  `Config::mapping(root)` (`:155`), `default_profile()` (`:150`); `AdapterMetadata { id, executable, mechanism,
  support, evidence, capabilities, env, conflicts }` (`metadata.rs:221`), `AdapterEvidence { mechanism_id,
  verified_at, upstream_version, source_url, notes }` (`:18`), `Capability::ALL` (`:41`), `CapabilityState`
  (`:47`, five states), `CapabilityClaim { capability, state, basis }` (`:57`); `ExecutableStatus::{Found(Found),
  NotOnPath, ConfiguredMissing(PathBuf), Unsupported(PathBuf)}` (`output.rs:385`) with `Found { path, origin:
  Origin::{Configured, Path} }` (`exe.rs:11-21`); `Listing { rows: Vec<(ProfileName, Vec<String>)>, ignored }`
  (`lifecycle.rs:210`); the `presence:` value is a `String` built in `cli.rs:996` (`materialized`, `known`,
  `absent`, `conflicts with <existing>`).
- Labels already fixed for text: `support_label`, `capability_label`, `state_label`, `origin_word`
  (`output.rs:34-58`, `:153`).
- A mapping key is any absolute path in Unix or Windows form, whatever the host (`config.rs:236`
  `is_absolute_key`); a key in the other platform's form is valid and never matches (SP3 design §6.1).
- `serde_json` is already a dependency of the crate (`crates/agent-profile/Cargo.toml`); `serde` with
  `derive` is a workspace dependency (`Cargo.toml:25`) not yet used by this crate.
- Exit codes: `Error::exit_code` (`error.rs:82-101`). `resolve` with no profile prints its text report and
  then the `NoProfile` error, exit 4 (`cli.rs:756-764`; SP3 design §7.5).

## 3. The JSON contract (every document)

### 3.1 Envelope and conventions
- Exactly one JSON object on stdout, pretty-printed with two-space indentation, followed by one newline.
  Nothing else on stdout. (Pretty rather than one line: a person reads it while debugging a script, and every
  consumer parses it either way.)
- The first two members of every document are `"schema"` and `"tool_version"`:
  - `schema`: `"agent-profile/<command>/<major>"`, one per command (`status`, `resolve`, `agents`,
    `profiles`, `repositories`); `<major>` starts at `1`. Adding a member is NOT a bump; renaming, removing,
    retyping or changing the meaning of a member is, and so is adding a token to an enumeration: every
    token set in this document is CLOSED for its `<major>` (a consumer may match exhaustively; agy-first
    round 1, Axiom Breaker: a strict deserializer breaks on an unknown token exactly as on a renamed
    member). V3 §32 "stable within a release series" is read as: `<major>` cannot change within a 0.x
    minor series. Consequence, stated so nobody is surprised: the token sets below are final for the
    0.x series; a state that does not fit them waits for 1.0 or lives in a new schema (as `doctor`'s
    will). A consumer checks the prefix it knows and ignores members it does not.
  - `tool_version`: the crate version (`CARGO_PKG_VERSION`, the string `--version` prints).
- Member names are `snake_case`. Enumerations are lower-case `snake_case` tokens, never the text labels
  (`"repository_agent_mapping"`, not `"repository agent mapping"`), so the text can be reworded without a
  schema bump.
- Every member is always present; an absent value is `null`, never an omitted key. Lists are `[]` when empty.
- Paths are strings as `Path::display` renders them (the same spelling the text report shows and
  `config.toml` stores; a non-UTF-8 component on Unix is rendered lossily, as in the text). No `\\?\` prefix
  is added or removed.
- Documents carry what the text report carries, structured. Where the text shows a derived word (`none`,
  `(none materialized)`, `not in a repository`), the document has the underlying value (`null`, `[]`).
- No document contains an environment, an argument list, a credential, or anything `plan()` computes (V3 §36).

### 3.2 Errors, exit codes and stderr under `--json`
- `--json` changes stdout only. Exit codes are identical to the text form. stderr is identical to the text form:
  the `agent-profile: error: …` line on failure, the `profiles` ignored-entries note, the support hedge where
  one exists. A consumer that wants only the document reads stdout; a human reads stderr.
- On a failure that happens before the report is built (usage error, invalid configuration, discovery error,
  unknown agent) stdout is empty. There is no JSON error envelope: the exit code is the machine-readable
  signal (V3 §33) and the stderr line the human one. Reason: a JSON error object would be a sixth schema
  that every consumer must handle on every call, for information the exit code already carries. (Owner
  question 1 in §10.)
- `<agent> resolve --json` with no profile keeps the text form's shape: the document (with `"profile": null`,
  `"source": "none"`) is written to stdout, then the `NoProfile` error to stderr, exit 4.
- A stdout write failure is `Error::Io`, exit 1, as for text. The document is serialised to a `String`
  first and written with one `write_out` call, never streamed to stdout: a serialisation failure can then
  never leave half a document on stdout, and a write failure (a closed pipe included) is one `Error::Io`.
  What a consumer that closes the pipe early reads is its own doing, not the tool's (agy-first round 1,
  Cascade Analyst; corrected in panel round 1, S2-1).
- Consumers read stdout alone. stderr keeps its notes under `--json` (§3.2 first bullet), so a capture
  that merges the two streams (`2>&1`) is not a JSON document; the README says so (panel round 1, Q3).

### 3.3 `agent-profile <agent> resolve --json`
```json
{
  "schema": "agent-profile/resolve/1",
  "tool_version": "0.0.5",
  "agent": "claude",
  "profile": "personal",
  "source": "repository_agent_mapping",
  "repository": "/home/me/src/acme"
}
```
- `source` ∈ `explicit` | `repository_agent_mapping` | `repository_mapping` | `global_default` | `none`
  (the five `ResolutionSource` variants; `explicit` is unreachable from `resolve`, which passes no explicit
  profile, and is listed so the token set equals the type). `profile` is `null` iff `source` is `none`.
- `repository`: the canonical root, or `null` outside a repository.

### 3.4 `agent-profile [<agent>] status --json`
```json
{
  "schema": "agent-profile/status/1",
  "tool_version": "0.0.5",
  "agent": null,
  "repository": "/home/me/src/acme",
  "mapping": { "profile": "work", "agents": { "claude": "personal" } },
  "default_profile": "work",
  "agents": [
    { "agent": "claude", "profile": "personal", "source": "repository_agent_mapping", "presence": null },
    { "agent": "codex", "profile": "work", "source": "repository_mapping", "presence": null }
  ],
  "inapplicable_mappings": ["/home/me/src"]
}
```
The same root through `agent-profile claude status --json` (only the members that differ are shown):
```json
{
  "agent": "claude",
  "agents": [
    { "agent": "claude", "profile": "personal", "source": "repository_agent_mapping",
      "presence": { "state": "materialized", "existing": null } }
  ]
}
```
- `agent`: `null` for bare `status`, the id for `<agent> status`, so the two forms are told apart from the
  document alone (solo panel round 1, Protocol Pedant; the same member `profiles` carries).
- `repository`: `null` for `NotInRepository`. `mapping`: the entry whose key equals the root, or `null`;
  inside it `profile` may be `null` and `agents` is an object keyed by agent id (sorted, as `BTreeMap`
  iterates), `{}` when empty.
- `agents`: one element per known agent in `adapter::known_agents()` order for bare `status`; exactly one
  for `<agent> status`. `profile`/`source` as in §3.3.
- `presence`: `null` for bare `status` and whenever nothing resolves; for `<agent> status` with a profile, an
  object `{ "state": "materialized" | "known" | "absent" | "conflict", "existing": <profile> | null }` where
  `existing` is non-null iff `state` is `conflict` (the text's `conflicts with <existing>`). An object, not a
  string, so a future detail does not change the member's type.
- `inapplicable_mappings`: the stored keys that are proper ancestors of the root, in stored-key order (the
  text's `note:` lines); `[]` outside a repository or when there are none.
- For an integration that shows "which profile applies here, and does it exist": use `<agent> status
  --json`, which exits 0 whether or not anything resolves and carries `presence`; `<agent> resolve --json`
  exits 4 when nothing resolves and has no presence (panel round 3, S3-2; the README says so).

### 3.5 `agent-profile agents --json`
```json
{
  "schema": "agent-profile/agents/1",
  "tool_version": "0.0.5",
  "agents": [
    {
      "agent": "claude",
      "executable": { "state": "found", "path": "/usr/local/bin/claude", "origin": "path" },
      "mechanism": { "id": "claude-config-dir-env", "description": "environment variable CLAUDE_CONFIG_DIR" },
      "support": "proven",
      "evidence": { "upstream_version": "2.0.1", "verified_at": "2026-09-16",
                    "source_url": "https://…", "notes": "…" },
      "capabilities": [
        { "capability": "config", "state": "supported", "basis": "…" },
        { "capability": "credentials", "state": "supported", "basis": "…" },
        { "capability": "state", "state": "supported", "basis": "…" }
      ]
    }
  ]
}
```
- One element per registered agent, in `agents` text order (registry order). The four text facts map
  one-to-one: `executable.state` ∈ `found` | `not_on_path` | `configured_missing` | `unsupported`; `path` is
  `null` only for `not_on_path`; `origin` ∈ `configured` | `path` and is `null` unless `state` is `found`.
- `mechanism.id` is `evidence.mechanism_id`; `mechanism.description` is the `Mechanism` display sentence the
  text shows. `support` ∈ `proven` | `experimental`.
- `capabilities`: exactly `Capability::ALL` in that order, `capability` ∈ `config` | `credentials` | `state`,
  `state` ∈ `supported` | `not_supported` | `not_guaranteed` | `conditional` | `unknown`; `basis` is the claim's
  basis string. A registry adapter declares every capability (`metadata_invariants`), so `not_declared` never
  appears in this document; the text's `not declared` exists for a fixture path `agents` cannot reach.
- `support` and `capabilities` are members of the same object (TODO.md: never separable).

### 3.6 `agent-profile profiles --json`, `agent-profile [<agent>] list --json`
```json
{
  "schema": "agent-profile/profiles/1",
  "tool_version": "0.0.5",
  "agent": null,
  "profiles": [
    { "name": "personal", "agents": ["claude"] },
    { "name": "work", "agents": ["claude", "codex"] }
  ],
  "ignored_entries": 0
}
```
- `profiles`: one element per listed profile directory, sorted by name as `Listing.rows` is; `agents` lists
  the ids materialized for it in adapter order, `[]` for `(none materialized)`.
- Bare `list --json` is identical to `profiles --json` (bare `list` is the alias, SP5a §9.4). `<agent> list
  --json` sets `agent` to the id and keeps only the elements whose `agents` contains it; each kept element
  still lists all of its agents (it is a filter of the same data, not a different document). An empty result
  is `"profiles": []` with exit 0; the text form's `no profiles for <agent>` stderr line is unchanged.
- `ignored_entries` is `Listing.ignored`; the stderr note stays (§3.2).

### 3.7 `agent-profile repositories --json`
```json
{
  "schema": "agent-profile/repositories/1",
  "tool_version": "0.0.5",
  "repositories": [
    {
      "repository": "/home/me/src/acme",
      "state": "active",
      "detail": null,
      "profile": { "name": "work", "exists": true },
      "agents": { "claude": { "profile": "personal", "exists": true, "known_agent": true } }
    },
    {
      "repository": "/home/me/old/acme",
      "state": "moved",
      "detail": "/home/me/src/monorepo",
      "profile": { "name": "work", "exists": true },
      "agents": {}
    },
    {
      "repository": "/mnt/gone/acme",
      "state": "unreadable",
      "detail": "cannot read .git: Permission denied (os error 13)",
      "profile": null,
      "agents": { "gemini": { "profile": "work", "exists": false, "known_agent": false } }
    }
  ]
}
```
Members follow §4. `repository` is the stored key string verbatim, as `Config::mappings()` yields it (its
casing and separators are the user's; on Windows a `C:/src/acme` key stays with forward slashes), which
is also what `Path::display` renders for it (panel round 1, S4-1). `state` is one of the six tokens of
§4. `detail` is `null` for `active`, `missing`,
`not_a_repository` and `foreign`; for `moved` it is the discovered root as a path string (what the text shows
after `now`); for `unreadable` it is the reason string exactly as `Error::Repository.reason` or the metadata
error carries it (what the text shows in parentheses). Two moved entries or two unreadable entries are
therefore distinguishable from the document alone (agy-first round 1, Blindspot Auditor). `profile` is
`null` when the entry has no `profile` field; `agents` is `{}` when it has none. `exists` is whether
`<app root>/profiles/<name>` (`AppRoot::profiles_dir()`, the tool's own root, never the repository) is a
directory: any agent counts, because the mapping names a profile, not an agent's materialization.
`known_agent` is whether this build's registry has the id.

## 4. The `repositories` command

`agent-profile repositories [--json]`: top-level only, like `agents` and `profiles` (an agent word before it
is a usage error); takes no arguments; `--repo` is an unknown option (it reports every mapping; there is nothing
to point it at). Read-only: creates nothing, never writes `config.toml`, takes no lock. Loads the configuration
like every command (V3 §17: an invalid file is exit 4). Runs no discovery of the working directory.

One block per stored entry, in `Config::mappings()` order (the stored key string), blank line between blocks:
```text
repository:   /home/me/src/acme
state:        active
profile:      work
agents:       claude=personal
```
- `state:` is the classification of the stored key against the filesystem (every non-`foreign` key is
  probed, exactly as `status --repo <key>` would probe it, so a key on an unreachable network share takes
  as long to classify as it would to resolve; the probes run one after another with no timeout of their
  own, so a tool that calls `repositories` on a configuration with several stale network keys should
  run it under its own subprocess timeout, and the README says so; panel round 1, S6-1), decided per
  entry in this order;
  the first rule that applies wins, and a failure inside one entry never stops the loop (a report that stops
  at the first broken entry hides the rest; where another command would exit 4, this one records the reason):
  1. `foreign` — the key is in the other platform's form (`is_absolute_key` accepts both; a Windows-form key
     on Unix, or on Windows a Unix-form key such as `/srv/acme`, which would otherwise be read relative to
     the current drive). Never discovered; never applies on this host; kept so a synced configuration keeps
     working (SP3 design §6.1). The filesystem is not consulted.
  1a. `unreadable` with the reason `device path refused`, on Windows only, unless the key's first path
     component IS a `Component::Prefix` of kind `Disk`, `VerbatimDisk`, `UNC` or `VerbatimUNC` (an
     allow-list, as `repo::prefix_allowed` is written: a key whose first component is anything else, a
     device prefix or no prefix at all, is refused; panel round 2, S2-2): a `\\.\pipe\…` or `\\.\COM1`
     device namespace passes `is_absolute_key` (it starts
     with `\\`, `config.rs:236`) and a `fs::metadata` on it can block or reach a driver. The same
     allow-list `repo::target_allowed` applies to `gitdir` targets (`repo.rs:61-98`); the report applies it
     to the key itself before touching the filesystem (panel round 1, S5-1). Unix has no such prefix: a
     `/dev/...` key is an ordinary entry and `stat` does not open it.
  2. `missing` — `fs::metadata(key)` (which follows symlinks) fails with `NotFound`: the directory is gone, or
     the key is a symlink whose target is gone (a dangling link has the same remedy, `unlink --repo <key>`;
     agy-first round 1, question 2).
  3. `unreadable` — `fs::metadata(key)` fails for another reason, or succeeds on something that is not a
     directory, or `repo::discover(key)` returns `Error::Repository` (a broken `.git` file, a permission
     refusal, a network path refusal under §5.3 of the SP3 design, a key that is a file). The text adds the
     reason: `unreadable (<reason>)`, where `<reason>` is the error's reason string (`not a directory` for a
     non-directory key).
  4. `not_a_repository` — discovery yields `NotInRepository`: the checkout was deleted but the directory
     remains, or `.git` was removed.
  5. `active` — discovery yields `Discovery::Repository(root)` with `root == key` component-equal (the same
     test `Config::mapping` applies). This mapping applies today.
  6. `moved` — discovery yields a repository whose root is a different path (the key is now inside a
     repository, or resolves through a symlink to a root with another spelling). The text adds
     `(now <root>)`. This mapping never applies: nothing discovers this key as a root.
- `profile:` is the entry's `profile` or `none`; it gains the suffix ` (no profile directory)` when
  `<app root>/profiles/<name>` (`AppRoot::profiles_dir()`, never the repository) is not a directory.
  `agents:` lists `id=profile` pairs in key order or `none`; a pair gains ` (no profile directory)` under
  the same rule and ` (unknown agent)` when this build's registry does not know the id (SP3 known limit:
  such a mapping is removable only by editing `config.toml`); when both apply, `(no profile directory)`
  comes first, then `(unknown agent)` (panel round 2, S6-1).
- An entry with neither a `profile` nor an `agents` field (valid under SP3 design §6.1, reachable by hand
  editing) maps nothing; its block gains `note:         maps nothing; remove it by editing config.toml`.
  Not `unlink --repo <key>`: `config::unlink` removes a field it finds mapped (`mapped`, `config.rs:584`,
  reads `profile` or `agents.<id>`), so on an empty table it reports nothing to remove and leaves the table
  (panel round 2, S2-1; the same limit SP3 records for unknown-agent fields). The document needs no
  member for it: a consumer reads `"profile": null` with `"agents": {}` (panel round 1, Q2).
- No entries: the single line `no repository mappings`, exit 0.
- Exit 0 for every content; the only failures are the usual ones before the report: usage (2), invalid
  configuration (4), root resolution (4), stdout write (1).
- V3 §27 names the states `active/missing/orphaned`; this document splits `orphaned` into `moved`,
  `not_a_repository`, `unreadable` and `foreign` because each has a different remedy (removing the
  mapping for the first two, fixing the checkout for the third, nothing for the fourth). Recorded in §10.
  Removing a mapping completely takes one `unlink` per stored field: `agent-profile unlink --repo <key>`
  removes the entry's `profile` and `agent-profile <agent> unlink --repo <key>` one `agents.<id>` field;
  `config::unlink` drops the table only once it is empty (`config.rs:556-558`), so an entry whose
  `agents` object is not `{}` is still reported after the top-level `unlink` alone (panel round 3, S3-1).
  The README's `repositories` paragraph says so.

## 5. Grammar changes

- `CommandWord` gains `Repositories` with usage text `Usage: agent-profile repositories [--json]` /
  `Show every stored repository mapping and whether it still applies.`; `Command::Repositories` leaves the clap
  enum and `reserved_name()` (only `doctor` and `completions` remain stubs), so `--help` no longer lists it as
  unimplemented. `validate_command` rejects an agent word before it (as for `agents`/`profiles`), any bare
  word, `--repo`, and `--yes`.
- `--json` becomes a recognised option for `status`, `resolve`, `agents`, `profiles`, `list`, `repositories`:
  exact spelling only (`--json=1` stays an unknown option, like `--yes=1`), at most once (a repeat is a usage
  error `` `--json` may be given only once ``, as `--repo`), position-free among the options AFTER the
  command word (`agent-profile --json status` reaches clap and is refused as today, like `--repo` there;
  solo panel round 1, Literal Implementer). `COMMANDS_HELP` (`cli.rs:33`) gains the `repositories` line
  and `[--json]` on the six commands. For `create`,
  `delete` it stays `NotYetImplemented`; for a launch it stays `NotYetImplemented` (`cli.rs:442`); for `link`,
  `unlink`, `current` it stays an unknown option. The usage texts of the six commands gain `[--json]` and a
  `--json  Print a JSON document instead of the report` line.
- `Invocation::Command` gains `json: bool` (the enum's shape is the builder's).
- README usage block: `agent-profile repositories` added; `[--json]` shown on `status`, `resolve`, `agents`,
  `profiles`, `list`, `repositories`; a short paragraph on the JSON contract (schema member, stability rule).
  ROADMAP: SP5 row updated. SP5d's documentation pass is not anticipated beyond those lines.

## 6. Errors and exit codes

No new `Error` variant and no new exit code. The `repositories` states are report content, not errors.
`--json` on a command that does not take it: unchanged from today (`NotYetImplemented` exit 2, or unknown
option exit 2, per §5).

## 7. Testing (oracles)

- End to end through the binary (`tests/json.rs`, new; helpers from `tests/support`): for each of the six
  `--json` forms, parse stdout with `serde_json::from_slice::<Value>` and compare against the full expected
  document built in the test from the tempdir's paths (every member, including the nulls), then assert the
  exit code and stderr equal the text form's for the same root. Cases: `status` in and outside a repository,
  with and without a mapping and default, with an inapplicable ancestor mapping; `<agent> status` with each
  presence state including `conflict`; `resolve` with each reachable source and the no-profile exit 4 with
  the document still on stdout; `agents` with the fixture configured, not on PATH, configured-missing and
  unsupported (`fake.cmd`), and that every element's `capabilities` has exactly three entries in order;
  `profiles`/`list`/`<agent> list` with an empty root, one row with no agents, the filtered form, and
  `ignored_entries` with its stderr note; `repositories` with one entry per state (`active`, `moved` via a
  key that is now a subdirectory of a repository, `not_a_repository`, `missing`, `unreadable` via a
  `.git` file that is a directory-less bogus `gitdir`, `foreign` via a Windows-form key on Unix and the
  reverse on Windows), `no profile directory` and `unknown agent` both in text and JSON, and the empty
  report.
- Schema pinning: one test per schema asserts the exact member list of the top-level object and of each
  nested object (a sorted key list compared to a literal written by hand in the test, never derived from
  the view type or from the document under test; solo panel round 1, Mechanism Gamer), so an accidental rename or omission is red, and
  that `schema` and `tool_version` are the first two members. The comparison is structural (a parsed
  `serde_json::Value` against values built in the test), never a byte-exact golden string, so a
  `serde_json` release that changes whitespace cannot turn the suite red while the contract holds; the
  one textual assertion is the member order, checked on the raw stdout prefix (`{\n  "schema": …`, then
  `"tool_version"`), because a parsed object does not keep it (panel round 2, Q3). Each such test runs on a fixture in which
  EVERY nullable member is null (`status` outside a repository with no default, `<agent> status` with
  nothing resolved, `resolve` with no profile, `profiles` on an empty root, `repositories` with an entry
  that has no `profile` field and a `detail` of `null`) and asserts the key is present with
  `serde_json::Value::Null`; a `skip_serializing_none`-style regression passes a fully populated fixture and
  only this one catches it (agy-first round 1, Mechanism Gamer).
- `repositories` isolation: a test with one `unreadable` entry sorted BEFORE a healthy one asserts both
  are reported and the exit is 0, so a per-entry failure is proven not to abort the loop.
- Grammar: `cli.rs` unit tests for `--json` on each command (accepted, `--json=1` rejected, repeated rejected,
  `--json` still `NotYetImplemented` for `create`/`delete`/launch, unknown for `current`/`link`/`unlink`), and
  `repositories` with an agent word, a bare word, `--repo`.
- Non-vacuity: each new assertion is proven by a logic mutant per the repository's test discipline (a token
  misspelled, a member dropped, `exists` inverted, a state misclassified) during the plan's test steps.
- Secrets: a test asserts `agents --json` and `status --json` documents contain no member named like an
  environment variable or holding an `=`-joined pair (V3 §36; cheap, structural).

## 8. Constraints

- New dependency: `serde` (workspace, `derive`) added to the crate; `serde_json` already present. No other.
- Linux, macOS, Windows green; the `foreign` state test runs on every OS with the opposite key form.
- No text report changes. No `config.toml` write anywhere in this slice. No lock taken by `repositories`.
- The documents are built by dedicated view types (one `struct` per schema with `#[derive(Serialize)]`,
  in a new module beside `output`), filled from the same inputs the text renderers take. Each schema of
  §3.3-§3.7 maps one-to-one to a view struct whose field names are the JSON member names verbatim, with
  no `skip_serializing_if` or `skip_serializing_none` anywhere (panel round 3, Q1). Domain types do not
  derive `Serialize`: the wire names are a contract and must not follow a field rename.

## 9. Left to the builder

The module name and file layout for the view types; whether the per-command builders take the text
renderers' inputs or a shared intermediate; how `Invocation` carries `json`; the exact usage-text wording
within §5's shapes; how the component-equal compare for `active` is shared with `Config::mapping`.

## 10. Calls I made (owner: please check)

1. No JSON error envelope: a failing command leaves stdout empty and reports on stderr with the usual exit
   code (§3.2). The alternative is `{"schema": "agent-profile/error/1", "exit_code": 4, "message": …}` on
   stdout for every failure under `--json`. I chose none because the exit code already carries the machine
   signal and a sixth schema taxes every consumer; a consumer that needs the message has stderr.
2. Schema identity is one string, `agent-profile/<command>/<major>`, with the stability rule in §3.1; no
   separate numeric member, no per-release-series string.
3. Pretty-printed documents (two-space indent, trailing newline), not one line.
4. `list --json` and `<agent> list --json` reuse the `profiles` schema with an `agent` member (filter, not a
   new document); `repositories --json` exists from day one although V3 §32 does not list the command.
5. `repositories` splits V3 §27's `orphaned` into `moved`, `not_a_repository`, `unreadable`, `foreign`
   (§4), reports a discovery failure as a state instead of exiting 4, and adds `no profile directory` and
   `unknown agent` annotations.
6. `presence` in `status --json` is an object (`state` + `existing`), the other enumerations are strings.
7. Enumeration tokens are `snake_case` words derived from the type's variant names, not the text labels.
8. Dedicated serialisable view types rather than deriving on the domain types (§8).
9. Enumeration token sets are closed within a schema major (§3.1): adding a state later is a major bump.
   Folded from agy-first round 1 (Axiom Breaker); the alternative, "consumers must treat every token field
   as an open string", pushes the cost onto every consumer.
10. `detail` carries the moved-to root or the unreadable reason, and is `null` otherwise (§3.7); `missing`
    is decided by `fs::metadata`, so a dangling symlink is `missing`, not `unreadable` (§4). Folded from
    agy-first round 1 (Literal Implementer, Blindspot Auditor, question 2).
11. OPEN FOR THE OWNER: closed token sets (call 9) versus open ones. The agy-first peer asked for closed
    sets (a strict consumer breaks on an unknown token); the panel peer, one round later, called the same
    rule the contract's weakest point (no new state can be added before 1.0 without breaking V3 §32).
    Both are right about their consumer. My recommendation stands with closed sets for 0.x: the five
    token sets are small and derived from types this slice does not expect to grow, `doctor` gets its
    own schema, and a consumer that wants to be forward-compatible can still write a fallback arm. The
    alternative is to declare every token field open and require consumers to tolerate unknown tokens.

## Consults
- agy-first round 1 (2026-10-19, `.clavity/seams/sp5b-design-forks.md`, reply
  `.clavity/scratch/sp5b-design-forks/round1-reply.md`): `[VERDICT: ALIGNED]`. Folded: closed token sets;
  `detail` semantics; `<app root>` wording; serialise-then-write; null-bearing schema fixtures; per-entry
  isolation test; dangling symlink is `missing`. Verified before folding: `repo::discover` canonicalises the
  start first and a missing or dangling path fails there with `cannot resolve the directory: …`
  (`repo.rs:105-114`), so `fs::metadata` ahead of discovery is what makes `missing` precise. Peer answers:
  F1 (empty stdout on failure) confirmed with the shell-capture scenario, F2 (per-command schema string)
  confirmed, F3 taxonomy confirmed as exhaustive for the remedies. Discarded by the peer, agreed: a
  `--compact` flag; naming the member `$schema`.

- Panel round 1, solo (2026-10-19; persona relentless-adversarial-auditor; seats Axiom Breaker, Cascade
  Analyst, Protocol Pedant, Literal Implementer, Blindspot Auditor, Mechanism Gamer, Activation Auditor;
  dropped State Corruptor (nothing writes), Boundary Smuggler (the only input is the validated
  `config.toml`), Resource Vampire (one probe per stored entry, bounded by the file), Dependency Cynic
  (`serde` is already a pinned workspace dependency)). Folded: the 0.x consequence of closed token sets
  (§3.1); the `agent` member in the `status` document (§3.4); `--json` only after the command word and the
  `COMMANDS_HELP` lines (§5); the hand-written key-list literal (§7); the filesystem-probing note (§4).
  PANEL VERDICT: five folds, no open finding.
- Panel round 1, agy escalation (2026-10-19; brief `.clavity/seams/sp5b-panel.md`, reply
  `.clavity/scratch/sp5b-panel/round1-reply.md`; seats as above plus Boundary Smuggler and Resource
  Vampire): `NEGOTIATE` on two blocking findings, both verified and folded. S1-1 the bare `status` example
  showed a populated `presence` against its own rule (verified against `cli.rs:777-780`, which sets no
  presence for bare `status`): example corrected, a second example for `<agent> status` added (§3.4).
  S5-1 a Windows device-namespace key (`\\.\pipe\…`) passes `is_absolute_key` and would reach
  `fs::metadata` (verified: `config.rs:236` accepts any `\\` prefix; `repo::target_allowed`'s allow-list
  at `repo.rs:61-98` covers only `gitdir` targets): rule 1a added (§4). Debt folded: `<app root>` wording
  in §4 (S1-2); the `| head` rationale was wrong, `head` truncates on its own (S2-1); `repository` is the
  stored key verbatim (S4-1); synchronous probes and the caller-side timeout advice (S6-1). Answers: Q2 the
  empty mapping entry gets a `note:` (§4); Q3 stderr parity stands, consumers read stdout alone (§3.2).
  Q1 reopens the closed-token-set rule: see §10 call 11. Discarded by the peer, agreed: alphabetical
  `agents` order; renaming `exists`. PANEL VERDICT: two blocking folded, four debt folded; round 2 owed.
- Panel round 2, agy escalation (2026-10-19; brief `.clavity/seams/sp5b-panel-r2.md`, reply
  `.clavity/scratch/sp5b-panel/round2-reply.md`; seats Axiom Breaker, Cascade Analyst, State Corruptor
  (new), Dependency Cynic (new), Protocol Pedant, Literal Implementer; four of six returned "no new
  findings"): one blocking, verified and folded. S2-1 the empty-mapping note named a remedy that cannot
  work (verified: `config::unlink` finds a field through `mapped`, `config.rs:584-593`, which reads
  `profile` or `agents.<id>`, so an empty table yields `NothingToRemove`): the note now says edit
  `config.toml` (§4). Debt folded: rule 1a phrased as an allow-list (S2-2); annotation order when both
  apply (S6-1). Answers: Q2 no missing member for a clean-up script; Q3 structural comparison, with the
  member-order check on raw text (§7). Discarded by the peer, agreed: `agents: null` for an empty list;
  rewording `device path refused`. PANEL VERDICT: one blocking folded, two debt folded; round 3 owed.
- Panel round 3, agy escalation (2026-10-19; brief `.clavity/seams/sp5b-panel-r3.md`, reply
  `.clavity/scratch/sp5b-panel/round3-reply.md`; seats Axiom Breaker, Cascade Analyst, Integration
  Author (bespoke, new), Protocol Pedant, Literal Implementer; four of five "no new findings"): GREEN,
  two debt findings folded. S3-1 removing an entry that also holds `agents` fields takes one `unlink` per
  field (verified: `config::unlink` removes one field and drops the table only when empty,
  `config.rs:545-558`) (§4). S3-2 integrations wanting presence and exit 0 use `<agent> status --json`,
  not `resolve --json` (§3.4). Q1 the view-struct rule made explicit (§8). Q2 the peer could not
  construct a consumer that acts wrongly while honouring the document. Q3 `repositories` stays a separate
  command: `status` is the hot path and must not probe every stored key. Discarded by the peer, agreed:
  mandating sorted `agents` keys; renaming `inapplicable_mappings`.
  PANEL VERDICT: GREEN; no live challenge.

## Stand-downs
None discarded below the floor by the driver and none accepted unverified: every agy-first, solo-panel and
escalation finding was folded. Peer-discarded items are listed under each round above.
