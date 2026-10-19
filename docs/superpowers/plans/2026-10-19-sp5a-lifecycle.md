# SP5a Profile Lifecycle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `create`, `delete`, `profiles`, `list` and `agents` so a profile can be made, seen and
removed on purpose, and a removal can never leave a mapping or the global default pointing at nothing.

**Architecture:** The filesystem logic lives in a new `lifecycle` module (pure functions over `AppRoot`,
returning outcome values, like `config::link`/`unlink`); `output` renders outcomes into lines; `cli` parses
the grammar and dispatches, the only place that prints or decides an exit code. `delete` confirms before
taking the configuration lock, then re-checks everything under it. No adapter changes: `create` reuses
`adapter::ensure_paths`, presence reuses `Adapter::presence`.

**Tech Stack:** Rust (edition 2024, `rust-version = "1.98"`), std only — no new dependencies.

**Spec:** `docs/superpowers/specs/2026-10-19-sp5a-lifecycle-design.md` (the design; panel GREEN
2026-10-19). V3 `agent-profile-implementation-spec-v3.md` §5.3, §10, §16, §17, §31, §33 wins over it.

## Global Constraints

- Gate is the repository's `just check` as-is (fmt, clippy `-D warnings`, typos, shellcheck,
  line-citations, nextest, probe-tests, tools-doc-check, doc-check). Never add flags, never edit a test to
  match code.
- `scripts/check-line-citations.sh` fails a comment that cites another file by line number: write no
  `file:line` into code comments.
- No new exit code numbers (V3 §33): usage 2, profile/config 4, declined confirmation 1.
- No output path ever shows a support level without the full capability matrix (owner ruling 3).
- Nothing here reads, prints or copies a credential; no secret appears in any message (V3 §36).
- The read-only commands (`profiles`, `list`, `agents`) never create the root, `config.toml` or the lock.
- Every commit ends with `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`. Commits use
  Conventional Commits.
- Every task: Step 0 — `git status --short` prints nothing and `HEAD` is the previous task's commit; else
  STOP and report `STATE_MISMATCH: <what>`. If the gate fails with a diagnostic in code written from this
  plan, STOP and report it rather than silence it.
- Spelling in user-visible text follows the code (`materialized`, `unrecognized`), not the design's prose.

## Review Focus

1. A fresh install: `AGENT_PROFILE_HOME` names a directory that does not exist yet — `create` must create
   it and succeed; `profiles` must print `no profiles` and create nothing. (Task 3, Task 5)
2. A stray file under `<root>/profiles/` (a note, a tarball) — `profiles` must not panic or list it; it is
   counted as unrecognized. (Task 5)
3. `delete` while `config.toml` is unparsable — refused with exit 4 before any prompt, nothing removed
   (V3 §17). (Task 4)
4. `agents` with no `PATH` at all — every executable reads `not found on PATH`, exit 0. (Task 6)
5. `create` for a configuration-file adapter (`aider`) — the initial file is written once, and a second
   `create` leaves a user-edited file byte-identical. (Task 3)

---

## File structure

| File | Tasks | Responsibility |
|---|---|---|
| `crates/agent-profile/src/cli.rs` | 1, 3–6 | grammar (`CommandWord`, clap `Command`, `validate_command`), dispatch, prompt, stdout/stderr |
| `crates/agent-profile/src/config.rs` | 2 | `locked`: a lock-holding read scope |
| `crates/agent-profile/src/lifecycle.rs` (new) | 3–5 | create/delete/listing over the filesystem, outcome types |
| `crates/agent-profile/src/lib.rs` | 3 | `pub mod lifecycle;` |
| `crates/agent-profile/src/error.rs` | 4 | `Error::ProfileReferenced` |
| `crates/agent-profile/src/output.rs` | 3–6 | one render function per outcome; `capability_lines` extraction |
| `crates/agent-profile/tests/config.rs` | 2 | `locked` tests |
| `crates/agent-profile/tests/lifecycle.rs` (new) | 3–6 | end-to-end through the binary, plus library-level race tests |
| `README.md`, `ROADMAP.md` | 7 | usage block, SP row |

Verified anchors (read 2026-10-19 at `14bd5b1`), named here once so tasks can refer to them:
`cli.rs` — `enum Command` (clap) at 58–77 with `reserved_name()` at 87–98; `enum CommandWord` 104–110 with
`parse` 114, `as_str` 125, `usage` 136; `run` 169; `Invocation::Command` 221–226; `split_top_level` 321;
`split` 339 (reserved-word branch 369–377); `validate_command` 423–477; `execute` 479; `run_command` 624;
`write_lines`/`write_out` 752/756; unit-test helpers `command()` 817, `top()` 840.
`config.rs` — `LOCK_FILE` 21, `lock_path` 79, `default_profile()` 150, `mappings_referencing` 165,
`update`/`update_with` 349/361 (lock taken at 383), `check_case_twins` 595 (`pub(crate)`), `acquire` 612.
`adapter/mod.rs` — `Adapter` trait 57–83 (`paths`, `presence`), `ProfilePath` 107, `REAL_ADAPTERS` 131,
`registry` 137, `known_agents` 146, `lookup` 151, `profile_dir` 176, `ensure_paths` 289, private
`has_kind` 281. `exe.rs` — `Origin` 11, `Found` 18, `discover` 33. `error.rs` — `Error` 21–76,
`exit_code` 79, test `exit_codes_follow_spec_33` 188. `output.rs` — `LABEL_WIDTH` 21, `support_label`
33, `report_lines` 87 (matrix loop 110–125), private `labeled` 165, `note` 170, `status_lines` 190.
`tests/support/mod.rs` — `Root::{new, empty, path, write_config, profile_dir, agent_profile}`.

---

### Task 1: Grammar — the five words parse, bind and route

**Files:**
- Modify: `crates/agent-profile/src/cli.rs` (anchors above)

**Interfaces:**
- Produces: `CommandWord::{Create, Delete, List, Agents, Profiles}`; `Invocation::Command { agent, command,
  profile, repo, yes: bool }`; `run_command(agent, command, profile, repo, yes)`. Until Tasks 3–6 land,
  `run_command` answers the five words with `Error::NotYetImplemented { command }` where `command` is
  `"<agent> <word>"` for an agent-scoped word and `"<word>"` for a top-level one — the strings the current
  code already produces, so observable behaviour is unchanged.

Grammar rules to implement in `validate_command` (one table, applied in this order after the existing
`has_cut` check):

| word | agent word | `--repo` | bare words | `--yes` | `--json` |
|---|---|---|---|---|---|
| `create` | required | no | exactly one profile | no | NotYetImplemented |
| `delete` | required | no | exactly one profile | yes | NotYetImplemented |
| `list` | optional | no | none | no | NotYetImplemented |
| `agents`, `profiles` | forbidden | no | none | no | NotYetImplemented |
| `current`, `resolve` | required (unchanged) | yes | none | no | unknown option (unchanged) |
| `status` | optional | yes | none | no | NotYetImplemented (unchanged) |
| `link` | optional | yes | exactly one profile | no | unknown option (unchanged) |
| `unlink` | optional | yes | none | no | unknown option (unchanged) |

Exact messages (new): `--repo` where not allowed → `unknown option "--repo" for \`<word>\``; `--yes` where
not allowed → `unknown option "--yes" for \`<word>\``; `create`/`delete` with no profile →
`` `create` needs a profile: agent-profile <agent> create <profile> `` (same shape for `delete`); more than
one bare word → `` `create` takes one profile `` / `` `delete` takes one profile ``; `agents`/`profiles` after an
agent → `` `agents` takes no agent: agent-profile agents ``; top-level `create`/`delete` (in `split_top_level`,
beside the `resolve`/`current` branch) → `` `create` needs an agent: agent-profile <agent> create <profile> ``.
`--yes` is exact (`--yes=anything` is an unknown option, matched like `--dry-run=yes` is today).

- [ ] **Step 1: Extend the unit tests in `cli.rs`'s `mod tests`.** Update the `command()` helper to take
      `yes: bool` as a fifth parameter (every existing call passes `false`). Change
      `reserved_first_bare_word_routes_by_exact_spelling`'s first assertion: `split_ok(&["fake", "create",
      "work"])` now equals `command(Some("fake"), CommandWord::Create, Some("work"), None, false)`. In
      `help_and_version_win_first`, `split_ok(&["fake", "create", "-h"])` now equals
      `Invocation::CommandHelp(CommandWord::Create)` (a command word's help names the word, as `status
      --help` already does). Add:

```rust
#[test]
fn lifecycle_words_bind_agent_profile_and_yes() {
    assert_eq!(split_ok(&["fake", "create", "work"]), command(Some("fake"), CommandWord::Create, Some("work"), None, false));
    assert_eq!(split_ok(&["fake", "delete", "work", "--yes"]), command(Some("fake"), CommandWord::Delete, Some("work"), None, true));
    assert_eq!(split_ok(&["fake", "--yes", "delete", "work"]), command(Some("fake"), CommandWord::Delete, Some("work"), None, true));
    assert_eq!(split_ok(&["fake", "list"]), command(Some("fake"), CommandWord::List, None, None, false));
    assert_eq!(top(&["list"]).unwrap(), command(None, CommandWord::List, None, None, false));
    assert_eq!(top(&["agents"]).unwrap(), command(None, CommandWord::Agents, None, None, false));
    assert_eq!(top(&["profiles"]).unwrap(), command(None, CommandWord::Profiles, None, None, false));
    assert_eq!(top(&["create", "-h"]).unwrap(), Invocation::CommandHelp(CommandWord::Create));
}

#[test]
fn lifecycle_usage_errors() {
    for (items, message) in [
        (&["create"][..], "`create` needs an agent: agent-profile <agent> create <profile>"),
        (&["delete", "work", "--yes"], "`delete` needs an agent: agent-profile <agent> delete <profile>"),
        (&["agents", "--repo", "x"], "unknown option \"--repo\" for `agents`"),
        (&["profiles", "extra"], "`profiles` takes no arguments"),
        (&["list", "--yes"], "unknown option \"--yes\" for `list`"),
    ] {
        assert_eq!(usage_message(top(items)), message, "{items:?}");
    }
    for (items, message) in [
        (&["fake", "create"][..], "`create` needs a profile: agent-profile <agent> create <profile>"),
        (&["fake", "delete"], "`delete` needs a profile: agent-profile <agent> delete <profile>"),
        (&["fake", "create", "a", "b"], "`create` takes one profile"),
        (&["fake", "create", "work", "--repo", "x"], "unknown option \"--repo\" for `create`"),
        (&["fake", "create", "work", "--yes"], "unknown option \"--yes\" for `create`"),
        (&["fake", "delete", "work", "--yes=1"], "unknown option \"--yes\" for `delete`"),
        (&["fake", "agents"], "`agents` takes no agent: agent-profile agents"),
        (&["fake", "profiles"], "`profiles` takes no agent: agent-profile profiles"),
        (&["fake", "list", "extra"], "`list` takes no arguments"),
        (&["fake", "create", "work", "--", "x"], "`create` takes no agent arguments; remove `--`"),
    ] {
        assert_eq!(usage_message(split(args(items), KNOWN)), message, "{items:?}");
    }
    for items in [&["fake", "create", "work", "--json"][..], &["fake", "list", "--json"]] {
        assert!(matches!(split_err(items), Error::NotYetImplemented { .. }), "{items:?}");
    }
    assert!(matches!(top(&["agents", "--json"]), Err(Error::NotYetImplemented { .. })));
}

#[test]
fn doctor_repositories_completions_stay_unimplemented() {
    match split_err(&["fake", "doctor"]) {
        Error::NotYetImplemented { command } => assert_eq!(command, "fake doctor"),
        other => panic!("{other:?}"),
    }
}
```

- [ ] **Step 2: Run the new tests, expect compile failure** — `cargo nextest run -p agent-profile
      -E 'test(lifecycle_)'`. Expected: FAIL (no `CommandWord::Create`, `command()` arity).

- [ ] **Step 3: Implement the grammar.** In `cli.rs`: add the five variants to `CommandWord` with
      `parse`/`as_str`; add a `usage()` text per word in the style of the existing ones (one line of
      purpose, then options — `delete` lists `--yes  Delete without asking`); remove `Agents`, `Profiles`,
      `List`, `Create`, `Delete` from the clap `enum Command` and from `reserved_name()` (its doc comment
      now lists the eight dispatched-before-clap words); add `yes: bool` to `Invocation::Command`;
      in `split_top_level` extend the "needs an agent" branch to `Create | Delete`; in `validate_command`
      implement the table above (a `--yes` token is recognised only for `Delete`; `Token::Repo` is refused
      for the five new words before the existing count checks); extend `COMMANDS_HELP` with a
      `Profile commands:` block listing the five forms; pass `yes` through `execute` into `run_command`,
      which for the five words returns the `NotYetImplemented` strings named in Interfaces.

- [ ] **Step 4: Run the whole crate's tests** — `cargo nextest run --workspace --no-tests=pass`.
      Expected: all pass, including `tests/smoke.rs::help_exits_zero_and_shows_launch_usage`.

- [ ] **Step 5: Gate** — `just check`. Expected: exit 0.

- [ ] **Step 6: Commit** — `git add crates/agent-profile/src/cli.rs` ·
      `feat(cli): parse create, delete, list, agents and profiles`.

---

### Task 2: `config::locked` — read the configuration under the write lock

**Files:**
- Modify: `crates/agent-profile/src/config.rs` (beside `update`, anchor 349)
- Test: `crates/agent-profile/tests/config.rs`

**Interfaces:**
- Produces: `pub fn locked<T>(root: &AppRoot, f: impl FnOnce(&Config) -> Result<T>) -> Result<T>` —
  opens `<root>/config.toml.lock` exactly as `update_with` step 2 does (`OpenOptions` read+write+create,
  no truncate), `acquire`s it with the same bounded wait, loads `Config::load(root)` under it, runs `f`,
  releases on every path. It does **not** create the root directory and does not sweep temp files: a
  missing root is `Error::ConfigWrite { path: lock_path, source }` from the open.

- [ ] **Step 1: Write the failing tests** in `tests/config.rs`:

```rust
#[test]
fn locked_reads_the_latest_configuration_and_releases_the_lock() {
    let (_dir, root) = root();
    std::fs::write(root.config_path(), "default_profile = \"work\"\n").unwrap();
    let seen = config::locked(&root, |config| Ok(config.default_profile().map(|p| p.to_string()))).unwrap();
    assert_eq!(seen.as_deref(), Some("work"));
    // Released: an update right after must not wait out the 10 s bound.
    let started = Instant::now();
    config::update(&root, |doc| { set_agent(doc, "fake", &absolute("x")); Ok(()) }).unwrap();
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn locked_times_out_while_another_handle_holds_the_lock() {
    // Same shape as update_times_out_while_another_handle_holds_the_lock, calling
    // config::locked(&root, |_| Ok(())) on the thread; asserts Err(ConfigWrite) whose source mentions
    // "another agent-profile process holds the configuration lock", elapsed >= 9 s.
}

#[test]
fn locked_never_creates_the_root() {
    let dir = tempfile::tempdir().unwrap();
    let root = AppRoot::from_path(dir.path().join("absent"));
    assert!(matches!(config::locked(&root, |_| Ok(())), Err(Error::ConfigWrite { .. })));
    assert!(!dir.path().join("absent").exists());
}
```

- [ ] **Step 2: Run them, expect failure** — `cargo nextest run -p agent-profile -E 'test(locked_)'`.
      Expected: FAIL, `locked` not found.
- [ ] **Step 3: Implement `locked`** in `config.rs`. Reuse `acquire` and the `write_error` closure shape
      from `update_with`; the lock `File` is dropped at the end of the function.
- [ ] **Step 4: Run them, expect pass** — same command. Expected: 3 passed.
- [ ] **Step 5: Gate** — `just check`. Expected: exit 0.
- [ ] **Step 6: Commit** — `feat(config): read the configuration under the write lock`.

---

### Task 3: `create`

**Files:**
- Create: `crates/agent-profile/src/lifecycle.rs`
- Modify: `crates/agent-profile/src/lib.rs` (add `pub mod lifecycle;`), `src/output.rs`, `src/cli.rs`
- Test: `crates/agent-profile/tests/lifecycle.rs` (new; `mod support;` + `use support::Root;` + the
  `stdout`/`stderr` helpers copied from `tests/launch.rs`)

**Interfaces:**
- Produces in `lifecycle.rs`:
  - `pub enum CreateOutcome { Created, AlreadyExists }`
  - `pub fn create(root: &AppRoot, adapter: &dyn Adapter, profile: &ProfileName) -> Result<(CreateOutcome, PathBuf)>`
    — `check_case_twins(root, profile)?`; outcome is `AlreadyExists` iff `adapter.presence(root, profile)
    == ProfilePresence::Materialized` before anything is written; then `ensure_paths` over
    `adapter.paths(root, profile)` mapped into `ProfilePath { path, kind, existed: false }` (`existed` is
    ignored by `ensure_paths`); the returned path is the first of `adapter.paths`, the profile directory.
- Produces in `output.rs`: `pub fn create_line(outcome: &CreateOutcome, agent: &str, profile: &ProfileName, path: &Path) -> String`
  — `created profile 'work' for fake: <path>` / `profile 'work' for fake already exists: <path>`.
- Produces in `cli.rs`: `fn run_create(root: &AppRoot, agent: &AgentId, profile: &ProfileName) -> Result<i32>`,
  dispatched from `run_command` right after `Config::load` (before `base_dir`): the config is loaded only
  so an invalid file is refused (V3 §17) — `create` needs nothing from it. Exit 0 on both outcomes.
  The agent word is validated by `split` already; `lookup(agent)` cannot fail.

- [ ] **Step 1: Write the failing tests** in `tests/lifecycle.rs`:

```rust
#[test]
fn create_materializes_once_and_then_reports_already_exists() {
    let root = Root::empty();
    let out = root.agent_profile(["fake", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), format!("created profile 'work' for fake: {}\n", root.profile_dir("work").display()));
    assert!(root.profile_dir("work").is_dir());
    let out = root.agent_profile(["fake", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), format!("profile 'work' for fake already exists: {}\n", root.profile_dir("work").display()));
}

#[test]
fn create_on_a_fresh_root_creates_the_root() {
    let dir = tempfile::tempdir().unwrap();
    support::assert_outside_any_repository(dir.path());
    let home = dir.path().join("brand-new");
    let out = Root::empty().agent_profile(["fake", "create", "work"]).env("AGENT_PROFILE_HOME", &home).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(home.join("profiles").join("work").join("fake").is_dir());
    assert!(!home.join("config.toml").exists());
}

#[test]
fn create_for_a_file_adapter_writes_the_initial_file_once_and_never_overwrites() {
    // aider owns a configuration file (SP2 design; adapter/aider.rs). create needs no executable.
    let root = Root::empty();
    let out = root.agent_profile(["aider", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let dir = root.path().join("profiles").join("work").join("aider");
    let file = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.path()).find(|p| p.is_file()).expect("aider's config file");
    std::fs::write(&file, "# edited by the user\n").unwrap();
    let out = root.agent_profile(["aider", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("already exists"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "# edited by the user\n");
}

#[test]
fn create_completes_a_half_materialized_profile_and_reports_created() {
    let root = Root::empty();
    std::fs::create_dir_all(root.path().join("profiles").join("work")).unwrap(); // no fake/ below it
    let out = root.agent_profile(["fake", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).starts_with("created profile 'work' for fake:"));
}

#[test]
fn create_refuses_a_case_twin_a_reserved_name_and_an_invalid_config() {
    let root = Root::empty();
    std::fs::create_dir_all(root.profile_dir("Work")).unwrap();
    let out = root.agent_profile(["fake", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(stderr(&out).contains("differs only in letter case"), "{}", stderr(&out));
    let out = root.agent_profile(["fake", "create", "status"]).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(stderr(&out).contains("invalid profile name"), "{}", stderr(&out));
    root.write_config("this is not toml = = =\n");
    let out = root.agent_profile(["fake", "create", "other"]).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(!root.profile_dir("other").exists());
}
```

- [ ] **Step 2: Run, expect failure** — `cargo nextest run -p agent-profile --test lifecycle`. Expected:
      every test FAILS with exit code 2 and `not yet implemented` in stderr.
- [ ] **Step 3: Implement** `lifecycle::create`, `output::create_line`, `run_create` and the dispatch.
      Add a unit test in `lifecycle.rs` for the outcome rule using the `Fake` adapter: absent → `Created`,
      materialized → `AlreadyExists`, directory present but a declared path missing → `Created`.
- [ ] **Step 4: Run, expect pass** — same command. Expected: 5 passed (+ the unit test).
- [ ] **Step 5: Gate** — `just check`. Expected: exit 0.
- [ ] **Step 6: Commit** — `feat(lifecycle): add create` (lifecycle.rs, lib.rs, output.rs, cli.rs,
      tests/lifecycle.rs).
- [ ] **Step 7: Mutant** — in `lifecycle::create`, make the outcome always `Created`. Run
      `cargo nextest run -p agent-profile -E 'test(create_materializes_once)'`. Expected: RED. Restore
      with `git checkout -- crates/agent-profile/src/lifecycle.rs`; `git status --short` prints nothing.

---

### Task 4: `delete`

**Files:**
- Modify: `src/error.rs`, `src/lifecycle.rs`, `src/output.rs`, `src/cli.rs`
- Test: `tests/lifecycle.rs`, `src/error.rs` test `exit_codes_follow_spec_33`, unit tests in `cli.rs`

**Interfaces:**
- `error.rs`: `Error::ProfileReferenced { agent: String, profile: String, references: Vec<String> }`,
  exit 4, Display exactly:
  `cannot delete profile 'work' for fake: it is referenced by one or more repository mappings (<references joined by ", ">); unlink the profile first, then retry deletion`
  where a reference renders as `<stored key>` for a repository-wide `profile` field, `<stored key>
  (agents.fake)` for an agent field, and the literal `default_profile` for the global default. When
  `default_profile` is among them, append ` — default_profile is set in config.toml; edit it to change the
  default` (ASCII hyphen-minus `-` not an em dash: the repo has hit non-ASCII corruption).
- `lifecycle.rs`:
  - `pub fn blocking_references(config: &Config, agent: &AgentId, profile: &ProfileName) -> Vec<String>`
    — `config.mappings_referencing(profile)` filtered to `reference.agent.is_none() ||
    reference.agent.as_ref() == Some(agent)`, rendered as above, plus `"default_profile"` when
    `config.default_profile()` matches `profile` ignoring ASCII case. Order: mappings in iteration order,
    then `default_profile`.
  - `pub fn delete_target(root: &AppRoot, agent: &AgentId, profile: &ProfileName) -> Result<Option<PathBuf>>`
    — `check_case_twins`; `symlink_metadata` on `<root>/profiles/<P>/<agent>`: `NotFound` → `Ok(None)`;
    a symlink there or at `<root>/profiles/<P>` → `Error::ProfileDir { path, source:
    io::Error::other("is a symbolic link or junction; refusing to delete through it") }`; not a directory
    → `ProfileDir` "is not a directory"; then `canonicalize` both the target and `root.profiles_dir()` and
    require `starts_with`, else `ProfileDir` "resolves outside the profiles directory". Returns the
    un-canonicalized target path (the one shown to the user).
  - `pub enum DeleteOutcome { Deleted { path: PathBuf, parent_note: Option<String> }, Vanished { path: PathBuf } }`
  - `pub fn delete(root: &AppRoot, agent: &AgentId, profile: &ProfileName) -> Result<DeleteOutcome>` —
    inside `config::locked`: `delete_target` again (`None` → `Vanished`), `blocking_references` again
    (non-empty → `Err(ProfileReferenced)`), `fs::remove_dir_all(target)` (error → `ProfileDir` with the
    source wrapped as `"<error>; the profile may be partially removed, retry the deletion"`), then if
    `read_dir(parent)` is empty try `fs::remove_dir(parent)`; a failure there becomes `parent_note =
    Some(format!("could not remove empty directory {}: {error}", parent.display()))`.
- `cli.rs`:
  - `fn confirm(question: &str, input: &mut dyn BufRead) -> Result<bool>` — writes `question` to stderr
    without a newline, flushes, reads one line; `Ok(true)` iff the trimmed line equals `y` or `yes`
    ignoring ASCII case; EOF or anything else → `Ok(false)`; a read error → `Error::Io`.
  - `fn run_delete(root: &AppRoot, config: &Config, agent: &AgentId, profile: &ProfileName, yes: bool) -> Result<i32>`,
    dispatched like `run_create`. Order: `delete_target` → `None`: print
    `profile 'work' for fake does not exist; nothing to delete` to stdout, and if `blocking_references`
    is non-empty print `agent-profile: note: still referenced by <refs>; unlink it before deleting` to
    stderr, return `Ok(0)` → `Some(path)`: advisory `blocking_references` non-empty →
    `Err(ProfileReferenced)`; if `!yes`: `!io::stdin().is_terminal()` → `Err(usage("--yes is required
    when input is not a terminal"))`, else `confirm("delete profile 'work' for fake at <path>? [y/N] ",
    &mut io::stdin().lock())?` false → stderr `agent-profile: not deleted`, `Ok(1)`; then
    `lifecycle::delete` → `Deleted` prints `deleted profile 'work' for fake: <path>` (stdout) and the
    `parent_note` as `agent-profile: note: <note>` (stderr), `Ok(0)`; `Vanished` prints the
    does-not-exist line, `Ok(0)`.
- `output.rs`: `pub fn delete_line(outcome: &DeleteOutcome, agent: &str, profile: &ProfileName) -> String`
  for the stdout line of each outcome, and `pub fn missing_profile_line(agent: &str, profile: &ProfileName) -> String`.
  stderr text is written in `cli.rs` with the `agent-profile: ` prefix the launch hedge uses.

- [ ] **Step 1: Write the failing tests.** In `error.rs::exit_codes_follow_spec_33` add
      `(Error::ProfileReferenced { agent: "a".into(), profile: "p".into(), references: vec![] }, 4)`.
      In `cli.rs` `mod tests`:

```rust
#[test]
fn confirm_accepts_only_y_or_yes_and_eof_is_no() {
    use std::io::Cursor;
    for (input, expected) in [("y\n", true), ("YES\n", true), (" yes \n", true), ("n\n", false), ("yes please\n", false), ("", false), ("\n", false)] {
        assert_eq!(confirm("q? ", &mut Cursor::new(input)).unwrap(), expected, "{input:?}");
    }
}
```

      In `tests/lifecycle.rs` (helpers copied from `tests/resolution.rs`: `scratch()`, `git_dir`, `key`, `mapping`; plus
      `fn created(root: &Root, agent: &str, profile: &str)` that runs `<agent> create <profile>` through
      the binary and asserts exit 0):

```rust
#[test]
fn delete_with_yes_removes_the_profile_and_the_empty_parent() {
    let root = Root::empty();
    created(&root, "fake", "work");
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), format!("deleted profile 'work' for fake: {}\n", root.profile_dir("work").display()));
    assert!(!root.path().join("profiles").join("work").exists());
}

#[test]
fn delete_keeps_sibling_agents_and_the_parent() {
    let root = Root::empty();
    created(&root, "fake", "work");
    created(&root, "aider", "work");
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(!root.profile_dir("work").exists());
    assert!(root.path().join("profiles").join("work").join("aider").is_dir());
}

#[test]
fn delete_without_a_terminal_needs_yes() {
    let root = Root::empty();
    created(&root, "fake", "work");
    let out = root.agent_profile(["fake", "delete", "work"]).stdin(std::process::Stdio::null()).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("--yes is required when input is not a terminal"), "{}", stderr(&out));
    assert!(root.profile_dir("work").is_dir());
}

#[test]
fn delete_of_a_missing_profile_exits_zero_and_notes_a_dangling_reference() {
    let root = Root::empty();
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "profile 'work' for fake does not exist; nothing to delete\n");
    assert_eq!(stderr(&out), "");
    root.write_config("default_profile = \"work\"\n");
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(stderr(&out).contains("still referenced by default_profile"), "{}", stderr(&out));
}

#[test]
fn delete_refuses_each_blocking_reference_and_allows_another_agents_mapping() {
    let root = Root::empty();
    let (_dir, base) = scratch(); let repo = base.join("acme"); git_dir(&repo);
    for (config, blocked) in [
        (mapping(&repo, "profile = \"work\""), true),
        (mapping(&repo, "agents = { fake = \"work\" }"), true),
        (mapping(&repo, "agents = { fake = \"WORK\" }"), true),
        ("default_profile = \"Work\"\n".to_owned(), true),
        (mapping(&repo, "agents = { aider = \"work\" }"), false),
    ] {
        created(&root, "fake", "work");
        root.write_config(&config);
        let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
        if blocked {
            assert_eq!(out.status.code(), Some(4), "{config}");
            assert!(stderr(&out).contains("cannot delete profile 'work' for fake: it is referenced"), "{}", stderr(&out));
            assert!(root.profile_dir("work").is_dir(), "{config}");
            std::fs::remove_dir_all(root.path().join("profiles")).unwrap();
        } else {
            assert_eq!(out.status.code(), Some(0), "{config}: {}", stderr(&out));
            assert!(!root.profile_dir("work").exists());
        }
    }
}

#[test]
fn delete_refuses_an_unparsable_config_before_touching_anything() {
    let root = Root::empty();
    created(&root, "fake", "work");
    root.write_config("= = =\n");
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(root.profile_dir("work").is_dir());
}

#[test]
fn delete_refuses_a_case_twin() {
    let root = Root::empty();
    created(&root, "fake", "Work");
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(root.profile_dir("Work").is_dir());
}

#[cfg(unix)]
#[test]
fn delete_refuses_symlinks_inside_and_outside_the_root_and_touches_nothing() {
    use std::os::unix::fs::symlink;
    let root = Root::empty();
    created(&root, "fake", "real");
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("keep"), "x").unwrap();
    // target is a symlink out of the root
    std::fs::create_dir_all(root.path().join("profiles").join("out")).unwrap();
    symlink(outside.path(), root.profile_dir("out")).unwrap();
    // parent is a symlink back inside the root
    symlink(root.path().join("profiles").join("real"), root.path().join("profiles").join("alias")).unwrap();
    for name in ["out", "alias"] {
        let out = root.agent_profile(["fake", "delete", name, "--yes"]).output().unwrap();
        assert_eq!(out.status.code(), Some(4), "{name}: {}", stderr(&out));
        assert!(stderr(&out).contains("symbolic link"), "{}", stderr(&out));
    }
    assert!(outside.path().join("keep").is_file());
    assert!(root.profile_dir("real").is_dir());
}

#[test]
fn a_mapping_added_after_the_advisory_check_still_blocks_the_locked_delete() {
    use agent_profile::config::{self, AppRoot};
    use agent_profile::lifecycle;
    use agent_profile::name::{AgentId, Platform, ProfileName};
    let root = Root::empty();
    created(&root, "fake", "work");
    let app = AppRoot::from_path(root.path().to_path_buf());
    let agent = AgentId::parse("fake").unwrap();
    let profile = ProfileName::parse("work", Platform::host()).unwrap();
    assert!(lifecycle::delete_target(&app, &agent, &profile).unwrap().is_some());
    assert!(lifecycle::blocking_references(&config::Config::load(&app).unwrap(), &agent, &profile).is_empty());
    let (_dir, base) = scratch(); let repo = base.join("acme"); git_dir(&repo);
    config::link(&app, &repo, None, &profile).unwrap();
    assert!(matches!(lifecycle::delete(&app, &agent, &profile), Err(agent_profile::error::Error::ProfileReferenced { .. })));
    assert!(root.profile_dir("work").is_dir());
}

#[test]
fn a_profile_that_vanishes_before_the_locked_delete_is_not_an_error() {
    // Same setup; remove root.profile_dir("work") with std::fs before lifecycle::delete; expect
    // Ok(DeleteOutcome::Vanished { .. }).
}

#[test]
fn delete_removes_a_half_materialized_profile() {
    let root = Root::empty();
    std::fs::create_dir_all(root.path().join("profiles").join("work").join("aider")).unwrap(); // no config file
    let out = root.agent_profile(["aider", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(!root.path().join("profiles").join("work").exists());
}
```

- [ ] **Step 2: Run, expect failure** — `cargo nextest run -p agent-profile -E 'test(delete) or
      test(confirm_) or test(vanishes) or test(advisory) or test(exit_codes_follow_spec_33)'`. Expected:
      compile failure (`ProfileReferenced`, `lifecycle::delete`, `confirm` missing).
- [ ] **Step 3: Implement** in the order `error.rs` → `lifecycle.rs` → `output.rs` → `cli.rs`.
- [ ] **Step 4: Run, expect pass** — same command. Expected: all pass (the symlink test only on Unix).
- [ ] **Step 5: Gate** — `just check`. Expected: exit 0.
- [ ] **Step 6: Commit** — `feat(lifecycle): add delete with the referenced-profile invariant`.
- [ ] **Step 7: Mutants** — apply each alone, run the named test, expect RED, restore with
      `git checkout -- crates/agent-profile/src/lifecycle.rs`, confirm a clean tree:
      1. drop the `default_profile` clause from `blocking_references` →
         `delete_refuses_each_blocking_reference_and_allows_another_agents_mapping` RED;
      2. make the agent filter `true` → the same test RED (the `aider` row);
      3. in `delete`, skip the second `blocking_references` →
         `a_mapping_added_after_the_advisory_check_still_blocks_the_locked_delete` RED;
      4. replace `remove_dir(parent)` with `remove_dir_all(parent)` →
         `delete_keeps_sibling_agents_and_the_parent` RED;
      5. (Unix) skip the parent symlink check in `delete_target` →
         `delete_refuses_symlinks_inside_and_outside_the_root_and_touches_nothing` RED.

---

### Task 5: `profiles` and `list`

**Files:**
- Modify: `src/lifecycle.rs`, `src/output.rs`, `src/cli.rs`
- Test: `tests/lifecycle.rs`

**Interfaces:**
- `lifecycle.rs`: `pub struct Listing { pub rows: Vec<(ProfileName, Vec<String>)>, pub ignored: usize }`
  and `pub fn list_profiles(root: &AppRoot, adapters: &[&dyn Adapter]) -> Result<Listing>` — `read_dir(root.profiles_dir())`:
  `NotFound` → empty listing; other errors → `Error::Io { context: "could not read the profiles
  directory", source }`. Each entry: name starting with `.` → skipped and not counted; not a directory or
  not a valid `ProfileName` (`Platform::host()`) → `ignored += 1`; otherwise one row whose agents are
  the ids of `adapters` for which `adapter.presence(root, profile) == Materialized`, in `adapters` order;
  child directories under the profile that are not a known agent id → `ignored += 1` each. Rows sorted by
  profile name (byte order).
- `output.rs`:
  - `pub fn profiles_lines(listing: &Listing) -> Vec<String>` — empty rows → `["no profiles"]`; else one
    line per row, `format!("{:<width$}  {}", name, agents)` with `width` = longest name, agents joined
    by `", "`, or `(none materialized)` when the row has no agent.
  - `pub fn list_lines(listing: &Listing, agent: &str) -> Vec<String>` — the names of rows whose agents
    contain `agent`, one per line, possibly empty.
- `cli.rs`: `fn run_profiles(root: &AppRoot, agent: Option<&AgentId>) -> Result<i32>` for
  `Profiles`, `List` (no agent → same as `Profiles`) and `List` with an agent. After the stdout lines: if
  `listing.ignored > 0`, stderr `agent-profile: ignored N unrecognized entries`; for the filtered form with
  no match, stderr `agent-profile: no profiles for fake`. Exit 0 always. Uses `adapter::registry()`.
  Read-only: `Config::load` is still called first (V3 §17), nothing is created.

- [ ] **Step 1: Write the failing tests**:

```rust
#[test]
fn profiles_lists_materialized_agents_per_profile_sorted() {
    let root = Root::empty();
    created(&root, "fake", "work"); created(&root, "aider", "work"); created(&root, "fake", "home");
    std::fs::create_dir_all(root.path().join("profiles").join("half").join("aider")).unwrap(); // no file: not materialized
    let out = root.agent_profile(["profiles"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), "half  (none materialized)\nhome  fake\nwork  aider, fake\n");
    assert_eq!(stderr(&out), "");
    let out = root.agent_profile(["list"]).output().unwrap();
    assert_eq!(stdout(&out), "half  (none materialized)\nhome  fake\nwork  aider, fake\n");
}

#[test]
fn profiles_on_an_empty_or_missing_root_prints_no_profiles_and_creates_nothing() {
    let root = Root::empty();
    let out = root.agent_profile(["profiles"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "no profiles\n");
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn profiles_counts_unrecognized_entries_on_stderr_and_ignores_dot_names() {
    let root = Root::empty();
    created(&root, "fake", "work");
    std::fs::write(root.path().join("profiles").join("notes.txt"), "x").unwrap();
    std::fs::create_dir_all(root.path().join("profiles").join("status")).unwrap(); // reserved word
    std::fs::create_dir_all(root.path().join("profiles").join("work").join("zzz")).unwrap(); // unknown agent
    std::fs::write(root.path().join("profiles").join(".DS_Store"), "x").unwrap();
    let out = root.agent_profile(["profiles"]).output().unwrap();
    assert_eq!(stdout(&out), "work  fake\n");
    assert_eq!(stderr(&out), "agent-profile: ignored 3 unrecognized entries\n");
}

#[test]
fn agent_list_prints_names_only_and_says_so_when_empty() {
    let root = Root::empty();
    created(&root, "fake", "work"); created(&root, "fake", "home"); created(&root, "aider", "other");
    let out = root.agent_profile(["fake", "list"]).output().unwrap();
    assert_eq!(stdout(&out), "home\nwork\n");
    assert_eq!(stderr(&out), "");
    let out = root.agent_profile(["claude", "list"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
    assert_eq!(stderr(&out), "agent-profile: no profiles for claude\n");
}
```

- [ ] **Step 2: Run, expect failure** — `cargo nextest run -p agent-profile -E 'test(profiles_) or
      test(agent_list_)'`. Expected: FAIL (`not yet implemented`).
- [ ] **Step 3: Implement.** - [ ] **Step 4: Run, expect pass.** - [ ] **Step 5: Gate** — `just check`.
- [ ] **Step 6: Commit** — `feat(lifecycle): add profiles and list`.
- [ ] **Step 7: Mutant** — in `list_profiles`, treat `Absent` as present. Expected:
      `profiles_lists_materialized_agents_per_profile_sorted` RED. Restore; clean tree.

---

### Task 6: `agents`

**Files:**
- Modify: `src/output.rs`, `src/cli.rs`
- Test: `tests/lifecycle.rs`, unit test in `output.rs`

**Interfaces:**
- `output.rs`:
  - `pub fn capability_lines(metadata: &AdapterMetadata) -> Vec<String>` — the matrix block extracted
    verbatim from `report_lines` (the `isolation:` line, continuation lines and each `basis`); `report_lines`
    calls it. Byte-identical output.
  - `pub enum ExecutableStatus { Found(Found), NotOnPath, ConfiguredMissing(PathBuf), Unsupported(PathBuf) }`
  - `pub fn agents_lines(entries: &[(&AdapterMetadata, ExecutableStatus)]) -> Vec<String>` — per entry:
    `agent:`, `executable:` (`<path> (PATH|configured)` / `not found on PATH` / `<path> (configured, missing)`
    / `<path> (unsupported: shell script or shim)`), `mechanism:` (`metadata.mechanism.to_string()`),
    `support:` (`support_label`), `version:` (`format!("{} (verified {})", evidence.upstream_version,
    evidence.verified_at)` — the measured upstream version, not a probe of the installed one),
    then `capability_lines`; an empty line between entries, none after the last.
- `cli.rs`: `fn run_agents(root: &AppRoot, config: &Config) -> Result<i32>` — for each adapter in
  `adapter::registry()`, `exe::discover(id, metadata.executable, config.agent_executable(id),
  std::env::var_os("PATH").as_deref(), &root.config_path())` mapped: `Ok` → `Found`;
  `Err(AgentNotInstalled { reason: NotOnPath { .. }, .. })` → `NotOnPath`;
  `Err(AgentNotInstalled { reason: ExplicitMissing(p), .. })` → `ConfiguredMissing(p)`;
  `Err(UnsupportedExecutable { path, .. })` → `Unsupported(path)`; any other `Err` propagates. Exit 0.

- [ ] **Step 1: Write the failing tests.** Unit test in `output.rs`:

```rust
#[test]
fn report_lines_embed_capability_lines_unchanged() {
    // Build a PlannedLaunch + Resolution the way the existing output tests do, render report_lines in
    // DryRun mode, and assert that capability_lines(metadata) occurs as a contiguous slice of it.
}
```

      In `tests/lifecycle.rs`:

```rust
#[test]
fn agents_lists_every_registered_agent_with_the_full_matrix() {
    let root = Root::new(); // config points fake at the fixture
    let out = root.agent_profile(["agents"]).env_remove("PATH").output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    let ids = agent_profile::adapter::known_agents();
    for id in &ids { assert!(text.contains(&format!("agent:{}{id}\n", " ".repeat(8))), "{id} missing"); }
    assert_eq!(text.matches("support:").count(), ids.len());
    assert_eq!(text.matches("version:").count(), ids.len());
    assert_eq!(text.matches("config:").count(), ids.len());
    assert_eq!(text.matches("credentials:").count(), ids.len());
    assert_eq!(text.matches("state:").count(), ids.len());
    assert!(text.contains(&format!("executable:{}{} (configured)", " ".repeat(3), env!("CARGO_BIN_EXE_fake-agent"))));
    assert_eq!(text.matches("not found on PATH").count(), ids.len() - 1);
    assert_eq!(text.matches("\n\n").count(), ids.len() - 1);
}

#[test]
fn agents_reports_a_configured_but_missing_executable() {
    let root = Root::empty();
    // Config::load requires an absolute path, and `/x` is not absolute on Windows.
    let missing = if cfg!(windows) { r"C:\nonexistent\fake-agent" } else { "/nonexistent/fake-agent" };
    root.write_config(&format!("[agents.fake]\nexecutable = {missing:?}\n"));
    let out = root.agent_profile(["agents"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(stdout(&out).contains(&format!("{missing} (configured, missing)")));
}
```

      (Padding follows `labeled`: `{:<14}` on `label:`, so `agent:` is followed by 8 spaces and
      `executable:` by 3.)

- [ ] **Step 2: Run, expect failure.** - [ ] **Step 3: Implement**, extracting `capability_lines` first
      and running the whole suite before adding `agents_lines`.
- [ ] **Step 4: Run, expect pass.** - [ ] **Step 5: Gate** — `just check`.
- [ ] **Step 6: Commit** — `feat(cli): add agents, rendering the full capability matrix per agent`.
- [ ] **Step 7: Mutant** — in `agents_lines`, drop the `capability_lines` call. Expected:
      `agents_lists_every_registered_agent_with_the_full_matrix` RED. Restore; clean tree.

---

### Task 7: Documentation and closeout

**Files:**
- Modify: `README.md` (Usage block at lines 39–46 and the paragraph after it), `ROADMAP.md` (SP5 row,
  line 14).

- [ ] **Step 1:** Add to the README usage block:
      `agent-profile <agent> create <profile>`, `agent-profile <agent> delete <profile> [--yes]`,
      `agent-profile [<agent>] list`, `agent-profile profiles`, `agent-profile agents`; one paragraph:
      profiles are created on first launch or with `create`; `delete` asks unless `--yes`, and refuses a
      profile any mapping or `default_profile` still names; `agents` shows every agent's executable,
      support level and capability matrix.
- [ ] **Step 2:** ROADMAP SP5 row: state becomes `SP5a done (#<PR>): create, list, delete, profiles,
      agents; SP5b–d next` (fill the PR number at merge time; until then write `in progress`).
- [ ] **Step 3: Gate** — `just check` (typos and doc-check cover the prose). Expected: exit 0.
- [ ] **Step 4: Commit** — `docs: document the profile lifecycle commands`.

---

## Self-review (done 2026-10-19 before handoff)

- Spec coverage: design §3.1 → Task 3; §3.2 (all nine steps, the no-entry branch, the parent-removal
  note, non-TTY, EOF) → Task 4; §3.3 → Task 5; §3.4 → Task 6; §4 → Tasks 1, 4; §5 → Task 1; §6 oracles →
  every test above; §6 "prompt does not block a concurrent link" is pinned structurally (the only lock
  holder is `lifecycle::delete`, called after `confirm`) rather than by a timing test — a plan call.
- Design §3.4 amended at plan time (recorded in its §9): `exe::discover` has a fourth reachable outcome
  (an unsupported shell script or shim), which `agents` shows rather than hides; and a `version:` line
  carries the evidence's upstream version so a reader knows which build the matrix was measured on.
- Not covered on purpose: a Windows junction test (needs `mklink /J` via `cmd`); `symlink_metadata`'s
  `is_symlink()` on Windows reports name-surrogate reparse points, which junctions are — the implementer
  verifies that against the rustdoc of `std::fs::FileType::is_symlink` and notes it in the code comment.
- Names used consistently: `lifecycle::{create, delete_target, blocking_references, delete,
  list_profiles}`, `CreateOutcome`, `DeleteOutcome`, `Listing`, `output::{create_line, delete_line,
  missing_profile_line, profiles_lines, list_lines, capability_lines, agents_lines}`, `cli::{confirm,
  run_create, run_delete, run_profiles, run_agents}`, `config::locked`, `Error::ProfileReferenced`.
