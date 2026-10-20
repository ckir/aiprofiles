# SP5b `--json` output and `repositories` report Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `--json` on `status`, `<agent> status`, `<agent> resolve`, `agents`, `profiles`, `list`, `<agent> list`
and the new `repositories` command, each a document under one contract, plus the text `repositories` report.

**Architecture:** A new `json` module holds one serialisable view struct per schema and a builder per command
that takes the same inputs the text renderers take; `cli.rs` chooses text or document after building the same
data. A new `repositories` module classifies every stored mapping key against the filesystem and returns plain
entries that both the text renderer in `output.rs` and the JSON builder consume. No domain type derives
`Serialize`; nothing writes `config.toml` or takes the lock.

**Tech Stack:** Rust 1.98 (edition 2024), `serde` 1 with `derive` (workspace dependency, added to the crate),
`serde_json` 1 (already a dependency), `cargo nextest`, `just check`.

**Spec:** `docs/superpowers/specs/2026-10-19-sp5b-json-repositories-design.md` (approved 2026-10-19, closed token
sets). V3 `agent-profile-implementation-spec-v3.md` §27, §32, §33, §36 win over both.

## Global Constraints

- Every document: `"schema": "agent-profile/<command>/1"` then `"tool_version"` first; `snake_case` members;
  every member always present, `null` for absent, `[]`/`{}` for empty; tokens are `snake_case` and the sets are
  CLOSED (spec §3.1); pretty-printed, two-space indent, one trailing newline; serialised to a `String` and
  written with ONE `write_out` call (spec §3.2).
- `--json` changes stdout only: exit codes and stderr identical to the text form; on a failure before the
  report is built stdout is empty; `resolve` with no profile writes the document, then the error, exit 4.
- No environment, argument, credential or `plan()` output in any document (V3 §36).
- Text reports do not change. `repositories` never writes, never locks, never exits non-zero for what it finds.
- View structs carry no `skip_serializing_if` / `skip_serializing_none`; field names are the JSON member names.
- Linux, macOS, Windows green; the `just check` gate (fmt, clippy `-D warnings`, typos, shellcheck, nextest, docs)
  passes at every commit. New dependency: `serde` only. Commit trailer: `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`.
- Every new test is proven non-vacuous by a logic mutant that turns exactly that test red (repository discipline;
  the test-audit will re-run them). Each task's last verification step names the mutant. The implementer
  appends each mutant run's nextest summary and failing-test line to
  `.clavity/scratch/sp5b-mutants/task-<N>.log` (gitignored runtime state) before restoring, so a reviewer can
  check the proof without re-running it (plan panel round 3, Q3).

## Review Focus

1. `status --json` outside any repository with no `config.toml` at all: a full document with `repository`,
   `mapping`, `default_profile` all `null`, every agent `profile: null`, `source: "none"`, exit 0 (Task 3).
2. `<agent> status --json` where the resolved profile has an on-disk case twin: `presence` is
   `{ "state": "conflict", "existing": "<twin>" }`, exit 0 (Task 3).
3. `repositories --json` where a stored key is a regular FILE: `state: "unreadable"`, `detail: "not a directory"`,
   the loop continues, exit 0 (Task 5).
4. `profiles --json` when `<app root>/profiles` itself is unreadable: exit 1, stdout EMPTY, stderr the `Io` line
   (Task 4).
5. `agents --json` with `PATH` removed and nothing configured: every element `executable.state: "not_on_path"`
   with `path: null`, `origin: null`, exit 0 (Task 4).

---

## File structure

- Create `crates/agent-profile/src/json.rs`: the five view-struct families, their builders, `render`.
- Create `crates/agent-profile/src/repositories.rs`: `State`, `Entry`, `ProfileRef`, `AgentRef`, `report`,
  `classify`.
- Create `crates/agent-profile/tests/json.rs`: end-to-end `--json` tests and schema pinning.
- Create `crates/agent-profile/tests/repositories.rs`: end-to-end `repositories` text and JSON tests.
- Modify `crates/agent-profile/src/lib.rs:10-20` (two `pub mod` lines), `src/cli.rs` (grammar, dispatch, usage
  texts, `COMMANDS_HELP`), `src/output.rs` (header comment, `repositories_lines`), `src/lifecycle.rs`
  (`Presence`), `crates/agent-profile/Cargo.toml` (`serde`), `README.md`, `ROADMAP.md`.

### Task 1: `json` module: envelope, `render`, `resolve` and `status` documents

**Files:**
- Create: `crates/agent-profile/src/json.rs`
- Modify: `crates/agent-profile/Cargo.toml` (`[dependencies]`, after `clap`): `serde = { workspace = true }`
- Modify: `crates/agent-profile/src/lib.rs:16` (insert `pub mod json;` after `pub mod exe;`, alphabetical)
- Modify: `crates/agent-profile/src/lifecycle.rs` (append `Presence` and `presence`)
- Modify: `crates/agent-profile/src/cli.rs:996-1009` (`presence` moves to `lifecycle`; `cli` keeps a one-line
  wrapper returning `presence.label()` for `status_lines`)
- Modify: `crates/agent-profile/src/output.rs:259` (`is_proper_ancestor` becomes `pub(crate)`)
- Test: unit tests inside `src/json.rs` and `src/lifecycle.rs`

**Interfaces:**
- Consumes: `resolve::Resolution` (`resolve.rs:34`), `resolve::ResolutionSource` (variants
  `Explicit | RepositoryAgent | RepositoryDefault | GlobalDefault | None`, `resolve.rs:21`), `repo::Discovery`
  (`repo.rs:18`), `config::Config::{mapping, mappings, default_profile}` (`config.rs:150-160`),
  `config::check_case_twins`, `adapter::lookup`, `adapter::ProfilePresence`.
- Produces (all `pub`, in `json.rs`):
  - `pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");`
  - `pub fn render<T: serde::Serialize>(document: &T) -> String` — `serde_json::to_string_pretty` plus `"\n"`;
    these structs cannot fail to serialise, so a failure is an `expect` with the message `view types serialise`.
  - `pub fn source_token(source: ResolutionSource) -> &'static str` —
    `explicit | repository_agent_mapping | repository_mapping | global_default | none`.
  - `pub struct ResolveDocument { schema: &'static str, tool_version: &'static str, agent: String, profile:
    Option<String>, source: &'static str, repository: Option<String> }` (`#[derive(Debug, Serialize)]`; field
    order is member order) and `pub fn resolve(resolution: &Resolution) -> ResolveDocument` with
    `schema = "agent-profile/resolve/1"`.
  - `pub struct PresenceView { state: &'static str, existing: Option<String> }`,
    `pub struct AgentResolutionView { agent: String, profile: Option<String>, source: &'static str, presence:
    Option<PresenceView> }`, `pub struct MappingView { profile: Option<String>, agents: BTreeMap<String,
    String> }`, `pub struct StatusDocument { schema, tool_version, agent: Option<String>, repository:
    Option<String>, mapping: Option<MappingView>, default_profile: Option<String>, agents:
    Vec<AgentResolutionView>, inapplicable_mappings: Vec<String> }` and
    `pub fn status(agent: Option<&AgentId>, discovery: &Discovery, config: &Config, agents: &[(Resolution,
    Option<Presence>)]) -> StatusDocument` with `schema = "agent-profile/status/1"`; `inapplicable_mappings`
    uses the same proper-ancestor test as `output::is_proper_ancestor` (`output.rs:259`; make it `pub(crate)`).
  - In `lifecycle.rs`: `pub enum Presence { Materialized, Known, Absent, Conflict { existing: String } }` with
    `pub fn label(&self) -> String` (`materialized`, `known`, `absent`, `conflicts with <existing>`, the strings
    `cli.rs:1000-1006` print today) and `pub fn token(&self) -> &'static str`
    (`materialized | known | absent | conflict`), and `pub fn presence(root: &AppRoot, agent: &AgentId,
    profile: &ProfileName) -> Presence` (the body of `cli.rs:996-1009`, returning the enum).

- [ ] **Step 1: Write the failing unit tests in `src/json.rs`**

```rust
#[test]
fn resolve_document_carries_every_member_with_null_for_absent() {
    // Resolution { agent: fake, profile: None, source: ResolutionSource::None, repository: None }
    let text = render(&resolve(&resolution));
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["schema"], "agent-profile/resolve/1");
    assert_eq!(value["tool_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(value["agent"], "fake");
    assert_eq!(value["profile"], serde_json::Value::Null);
    assert_eq!(value["source"], "none");
    assert_eq!(value["repository"], serde_json::Value::Null);
    assert_eq!(value.as_object().unwrap().len(), 6);
    assert!(text.starts_with("{\n  \"schema\": \"agent-profile/resolve/1\",\n  \"tool_version\": "));
    assert!(text.ends_with("}\n"));
}

#[test]
fn source_tokens_are_snake_case_and_closed() {
    assert_eq!(source_token(ResolutionSource::RepositoryAgent), "repository_agent_mapping");
    // ... one assertion per variant: explicit, repository_mapping, global_default, none
}

#[test]
fn status_document_outside_a_repository_is_all_null_and_empty() {
    // Discovery::NotInRepository, Config::default(), agents = [(fake resolution with no profile, None)]
    let value = parsed(&status(None, &Discovery::NotInRepository, &Config::default(), &rows));
    assert_eq!(value["agent"], Null); assert_eq!(value["repository"], Null);
    assert_eq!(value["mapping"], Null); assert_eq!(value["default_profile"], Null);
    assert_eq!(value["agents"][0]["presence"], Null);
    assert_eq!(value["inapplicable_mappings"], serde_json::json!([]));
    assert_eq!(keys(&value), ["agent", "agents", "default_profile", "inapplicable_mappings", "mapping",
                               "repository", "schema", "tool_version"]); // sorted, hand-written
}

#[test]
fn status_document_inside_a_repository_carries_the_mapping_presence_and_ancestors() {
    // config parsed from text with default_profile = "work", [repositories.'<outer>'] profile = "outer",
    // [repositories.'<inner>'] profile = "inner", agents = { fake = "mine" }; discovery = Repository(inner);
    // rows = [(fake -> mine via RepositoryAgent, Some(Presence::Conflict { existing: "Mine".into() }))]
    // asserts: mapping.profile == "inner", mapping.agents == {"fake": "mine"}, default_profile == "work",
    // agents[0].source == "repository_agent_mapping", agents[0].presence == {"state":"conflict","existing":"Mine"},
    // inapplicable_mappings == [outer]; agent == "fake" when `Some(fake)` is passed
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo nextest run -p agent-profile --lib json`
Expected: FAIL to compile (`json` module missing)

- [ ] **Step 3: Add `serde` to the crate, create `src/json.rs` with the items in Interfaces, add `Presence` to
  `lifecycle.rs`, and make `cli.rs:996` a wrapper over `lifecycle::presence(...).label()`**

`presence` in `lifecycle.rs` keeps the case-twin check first (`config::check_case_twins`), then
`adapter::lookup(agent).expect("known agents have an adapter").presence(root, profile)`.

- [ ] **Step 4: Run the unit tests and the existing suites**

Run: `cargo nextest run -p agent-profile`
Expected: all pass (295 existing plus the new unit tests)

- [ ] **Step 5: Mutant proof**

Temporarily change `source_token` to return `"global default"` (the text label) for `GlobalDefault`, a variant
no status fixture in this task resolves through: `source_tokens_are_snake_case_and_closed` is red and nothing
else (mutating `RepositoryAgent` would also redden the status fixture test; panel round 1, S4-1). Restore.

- [ ] **Step 6: Commit**

```bash
git add crates/agent-profile/Cargo.toml crates/agent-profile/src/lib.rs crates/agent-profile/src/json.rs crates/agent-profile/src/lifecycle.rs crates/agent-profile/src/cli.rs crates/agent-profile/src/output.rs
git commit -m "feat(json): resolve and status documents behind a serialisable view layer"
```

### Task 2: `--json` grammar and the `resolve`/`status` wiring

**Files:**
- Modify: `crates/agent-profile/src/cli.rs:33-46` (`COMMANDS_HELP`), `:146-200` (usage texts), `:236-253`
  (`Invocation::Command` gains `json: bool`), `:461-553` (`validate_command`), `:555-575` (`execute`),
  `:700-822` (`run_command`), `:1076` (test helper `command(...)` gains `json`), `:1311`, `:1324` (the
  `--json` expectations), `tests/resolution.rs:505` (the `--json is not yet implemented` row)
- Test: `crates/agent-profile/tests/json.rs` (new), unit tests in `cli.rs`

**Interfaces:**
- Consumes: `json::{resolve, status, render}`, `lifecycle::presence` (Task 1).
- Produces: `Invocation::Command { agent, command, profile, repo, yes, json }`; `run_command(..., json: bool)`;
  `validate_command` accepts `--json` for `Status | Resolve | List | Agents | Profiles` (Task 6 adds
  `Repositories`), exact spelling, at most once; `--json` on `Create | Delete` stays
  `Error::NotYetImplemented { command: "--json" }`; launches unchanged (`cli.rs:442`); a repeat is
  `` `--json` may be given only once `` (usage, exit 2); `--json=1` falls through to
  `unknown option "--json" for `<name>``.

- [ ] **Step 1: Write the failing grammar unit tests in `cli.rs`**

```rust
#[test]
fn json_binds_once_after_the_command_word_for_the_report_commands() {
    for items in [&["status", "--json"][..], &["profiles", "--json"], &["agents", "--json"], &["list", "--json"]] {
        assert!(matches!(top(items), Ok(Invocation::Command { json: true, .. })), "{items:?}");
    }
    for items in [&["fake", "resolve", "--json"][..], &["fake", "status", "--json", "--repo", "x"], &["fake", "list", "--json"]] {
        assert!(matches!(split(args(items), KNOWN), Ok(Invocation::Command { json: true, .. })), "{items:?}");
    }
    assert_eq!(usage_message(top(&["status", "--json", "--json"])), "`--json` may be given only once");
    assert_eq!(usage_message(top(&["status", "--json=1"])), "unknown option \"--json\" for `status`");
    for items in [&["fake", "create", "work", "--json"][..], &["fake", "delete", "work", "--json"]] {
        assert!(matches!(split_err(items), Error::NotYetImplemented { .. }), "{items:?}");
    }
    assert_eq!(usage_message(split(args(&["fake", "current", "--json"]), KNOWN)), "unknown option \"--json\" for `current`");
}
```
Update `lifecycle_usage_errors` (`cli.rs:1394-1397`) and `json_is_not_yet_implemented` (`:1505`): `fake list
--json` and `agents --json` are no longer `NotYetImplemented`; the launch rows stay.

- [ ] **Step 2: Write the failing end-to-end tests in `tests/json.rs`**

Helpers: copy `scratch`, `git_dir`, `key`, `configure`, `mapping`, `run`, `stdout`, `stderr` from
`tests/resolution.rs:14-55`; add `fn document(out: &Output) -> serde_json::Value` (parses stdout, panics
with the text on failure) and `fn keys(value: &Value) -> Vec<String>` (sorted member names).

```rust
#[test]
fn resolve_json_matches_the_text_form_and_keeps_exit_4_without_a_profile() {
    // Root::empty(); scratch repo `acme` with git_dir; configure(mapping(acme, "agents = { fake = \"mine\" }"))
    // run fake resolve --json in acme: exit 0, document == {schema, tool_version, agent:"fake", profile:"mine",
    //   source:"repository_agent_mapping", repository: acme.display()}; stderr == ""
    // configure(&root, None, ""); run again: exit 4, document has profile null + source "none",
    //   stderr == the same single line `fake resolve` prints without --json (run both, compare)
}

#[test]
fn status_json_bare_and_per_agent_agree_with_the_text_report() {
    // the fixture of tests/resolution.rs:195-213 (outer/inner, default work)
    // bare `status --json` in inner: agent null; repository inner; mapping {profile:"inner", agents:{fake:"mine"}};
    //   default_profile "work"; agents has one element per adapter::known_agents() in that order, every
    //   presence null; inapplicable_mappings == [outer]
    // `fake status --json` in inner: agent "fake"; agents.len() == 1; presence == {state:"absent", existing:null}
    // after `fake` launch: presence.state == "materialized"
}

#[test]
fn status_json_outside_a_repository_with_no_config_is_all_null() {   // Review Focus 1
    // Root::empty(), no config file, run `status --json` from scratch base: exit 0,
    // repository/mapping/default_profile null, every agents[i].profile null, source "none", presence null,
    // inapplicable_mappings []; keys(top) == hand-written sorted list
}

#[test]
fn agent_status_json_reports_a_case_twin_as_a_conflict() {          // Review Focus 2
    // the fixture of tests/resolution.rs:259 (a_case_twin_profile_directory_is_reported_as_a_conflict):
    // presence == {state:"conflict", existing:"<twin spelling>"}, exit 0
}

#[test]
fn json_leaves_stderr_and_exit_codes_as_the_text_form() {
    // for each of: ["status"], ["fake","status"], ["fake","resolve"] with and without a resolvable profile:
    // run with and without --json; assert equal exit codes and byte-equal stderr
}

#[test]
fn json_members_are_in_order_and_documents_end_with_one_newline() {
    // raw stdout of `status --json` starts with "{\n  \"schema\": \"agent-profile/status/1\",\n  \"tool_version\": \""
    // and ends with "}\n" exactly once
}
```

- [ ] **Step 3: Run the new tests to verify they fail**

Run: `cargo nextest run -p agent-profile --lib cli --test json`
Expected: the grammar test fails on `json` (field does not exist); `tests/json.rs` fails with exit 2 (`--json is
not yet implemented`)

- [ ] **Step 4: Implement the grammar and wiring**

- `validate_command`: after the `--repo` refusal block (`cli.rs:483-500`), replace the `--json`
  `NotYetImplemented` block (`:501-513`) with: count `options(tokens).filter(|o| *o == "--json")`; for
  `Create | Delete` and a count > 0 return `NotYetImplemented`; for the report commands a count > 1 is the
  usage error above; `json = count == 1`. The unknown-option loop (`:517-519`) must then skip `--json` for the
  report commands exactly as it skips `--yes` for `delete`.
- Usage texts gain ` [--json]` in the `Usage:` line and a `--json  Print a JSON document instead of the report`
  option line for `resolve`, `status`, `list`, `agents`, `profiles`; `COMMANDS_HELP` the same `[--json]`.
- `run_command`: `Resolve` builds the resolution, then `if json { write_out(&json::render(&json::resolve(&r)))? }
  else { write_lines(&output::resolve_lines(&r))? }`, then the existing `no_profile` error path; `Status` builds
  `rows: Vec<(Resolution, Option<Presence>)>` and renders `json::status(agent.as_ref(), &discovery, &config,
  &rows)` or `output::status_lines(&discovery, &config, &labelled)` where `labelled` maps each presence to
  `label()`.

- [ ] **Step 5: Run the whole crate**

Run: `cargo nextest run -p agent-profile`
Expected: all pass

- [ ] **Step 6: Mutant proofs**

(a) Make the bare `status` arm of `run_command` compute a presence for every agent (as `<agent> status` does):
`status_json_bare_and_per_agent_agree_with_the_text_report` red (it asserts `presence: null` for every bare
row) and nothing else, since the case-twin test runs `<agent> status` only (panel round 2, S6-1). (b) Make
`render` return the document without the trailing newline:
`json_members_are_in_order_and_documents_end_with_one_newline` red. (c) Let `resolve` under `--json` return
`Ok(0)` when no profile resolves: `resolve_json_matches_the_text_form_and_keeps_exit_4_without_a_profile` red.
Restore after each.

- [ ] **Step 7: Commit**

```bash
git add crates/agent-profile/src/cli.rs crates/agent-profile/tests/json.rs crates/agent-profile/tests/resolution.rs
git commit -m "feat(cli): --json on resolve and status"
```

### Task 3: `agents` and `profiles`/`list` documents

**Files:**
- Create/extend: `crates/agent-profile/src/json.rs` (agents and profiles families)
- Modify: `crates/agent-profile/src/cli.rs:823-882` (`run_profiles`, `run_agents` take `json: bool`), the
  dispatch in `run_command` (`:710-717`)
- Test: `crates/agent-profile/tests/json.rs`, unit tests in `src/json.rs`

**Interfaces:**
- Consumes: `output::ExecutableStatus` (`output.rs:385`), `exe::{Found, Origin}` (`exe.rs:11-21`),
  `adapter::AdapterMetadata` / `AdapterEvidence` / `Capability::ALL` / `CapabilityClaim` / `CapabilityState` /
  `SupportLevel` (`adapter/metadata.rs:18-61, :221`), `output::{support_label, capability_label}` (reuse; they
  already produce the spec's tokens `proven|experimental`, `config|credentials|state`), `lifecycle::Listing`
  (`lifecycle.rs:210`).
- Produces: `pub fn state_token(state: CapabilityState) -> &'static str` (`supported | not_supported |
  not_guaranteed | conditional | unknown`); `pub struct ExecutableView { state: &'static str, path:
  Option<String>, origin: Option<&'static str> }` (`found | not_on_path | configured_missing | unsupported`;
  origin `configured | path`); `MechanismView { id: &'static str, description: String }` (`id` =
  `metadata.evidence.mechanism_id`, `description` = `metadata.mechanism.to_string()`); `EvidenceView {
  upstream_version, verified_at, source_url, notes }` (all `&'static str`); `CapabilityView { capability:
  &'static str, state: &'static str, basis: &'static str }`; `AgentView { agent, executable, mechanism, support,
  evidence, capabilities: Vec<CapabilityView> }`; `AgentsDocument { schema, tool_version, agents:
  Vec<AgentView> }`; `pub fn agents(entries: &[(&AdapterMetadata, ExecutableStatus)]) -> AgentsDocument`
  (`schema = "agent-profile/agents/1"`, capabilities in `Capability::ALL` order, a missing claim is impossible
  for a registry adapter: `expect("registry adapters declare every capability")`). `ProfileView { name: String,
  agents: Vec<String> }`; `ProfilesDocument { schema, tool_version, agent: Option<String>, profiles:
  Vec<ProfileView>, ignored_entries: usize }`; `pub fn profiles(listing: &Listing, agent: Option<&AgentId>)
  -> ProfilesDocument` (`schema = "agent-profile/profiles/1"`; with an agent, keep only rows whose agents
  contain it, rows unchanged otherwise).

- [ ] **Step 1: Write the failing unit tests in `src/json.rs`**

```rust
#[test]
fn agents_document_renders_each_executable_state_and_the_full_matrix() {
    // one entry per ExecutableStatus variant using a REGISTRY adapter's metadata
    // (`adapter::lookup("fake").unwrap().metadata()`, which declares every capability; the hand-built fixture
    // at output.rs:557 declares none and would hit the builder's expect):
    // Found{configured} -> {state:"found", path:Some, origin:"configured"}; NotOnPath -> {state:"not_on_path", path:null, origin:null};
    // ConfiguredMissing(p) -> {"configured_missing", Some(p), null}; Unsupported(p) -> {"unsupported", Some(p), null};
    // every element: capabilities.len() == 3 in order config, credentials, state; support in {"proven","experimental"};
    // mechanism.id == evidence.mechanism_id; keys(element) == hand-written sorted list
}

#[test]
fn profiles_document_filters_by_agent_without_changing_rows() {
    // Listing { rows: [("personal", ["claude"]), ("work", ["claude","codex"])], ignored: 2 }
    // profiles(&listing, None): agent null, two rows, ignored_entries 2
    // profiles(&listing, Some(codex)): agent "codex", one row "work" whose agents is still ["claude","codex"]
}
```

- [ ] **Step 2: Write the failing end-to-end tests in `tests/json.rs`**

```rust
#[test]
fn agents_json_lists_every_registered_agent_with_executable_states() {
    // Root::new() (fake configured), env_remove("PATH"): fake is found/configured, every other agent
    // not_on_path with path null (Review Focus 5); agents.len() == adapter::known_agents().len() and ids in order
    // Root::empty() + config executable = <root>/fake.cmd (file written): fake is "unsupported" with that path
    // Root::empty() + config executable = /nonexistent/fake-agent (Windows: C:\nonexistent\fake-agent): "configured_missing"
}

#[test]
fn profiles_and_list_json_share_the_schema_and_keep_the_stderr_notes() {
    // Root::empty(); fake create work; aider create work; fake create personal; write profiles/notes.txt
    // `profiles --json`: agent null; profiles == [{personal,[fake]},{work,[fake,aider]}] (adapter order);
    //   ignored_entries 1; stderr == "agent-profile: ignored 1 unrecognized entries\n"
    // `list --json` document == `profiles --json` document (byte-equal stdout)
    // `aider list --json`: agent "aider"; profiles == [{work,[fake,aider]}]
    // `codex list --json`: profiles == [], exit 0, stderr == "agent-profile: no profiles for codex\n"
    // `profiles --json` on an empty root: profiles [], ignored_entries 0, exit 0
}

#[cfg(unix)]
#[test]
fn profiles_json_on_an_unreadable_profiles_directory_leaves_stdout_empty() {   // Review Focus 4
    // profiles dir mode 000 (skip when still readable, as tests/lifecycle.rs:346 does): exit 1, stdout "",
    // stderr contains "could not read the profiles directory"
}
```

- [ ] **Step 3: Run them to verify they fail**

Run: `cargo nextest run -p agent-profile --lib json --test json`
Expected: compile failure on the missing builders, then exit-2 failures end to end

- [ ] **Step 4: Implement the builders and wire `run_agents` / `run_profiles`**

`run_profiles(root, agent, json)`: build the listing as today; with `json` write `render(&profiles(&listing,
agent))` instead of the lines; the two stderr notes stay exactly as they are (`cli.rs:831-842`), and `no profiles
for <agent>` is decided on the filtered rows being empty. `run_agents(root, config, json)`: build `entries` as
today (`cli.rs:852-879`); write `render(&agents(&entries))` or the lines.

- [ ] **Step 5: Run the whole crate**

Run: `cargo nextest run -p agent-profile`
Expected: all pass

- [ ] **Step 6: Mutant proofs**

(a) Make `profiles` drop rows' other agents when filtering: `profiles_document_filters_by_agent_without_changing_rows`
red. (b) Swap `not_on_path` and `configured_missing` tokens:
`agents_json_lists_every_registered_agent_with_executable_states` red. (c) Have `run_profiles` print the
document AND the lines: `profiles_and_list_json_share_the_schema_and_keep_the_stderr_notes` red.

- [ ] **Step 7: Commit**

```bash
git add crates/agent-profile/src/json.rs crates/agent-profile/src/cli.rs crates/agent-profile/tests/json.rs
git commit -m "feat(cli): --json on agents, profiles and list"
```

### Task 4: schema pinning and the contract tests

**Files:**
- Modify: `crates/agent-profile/tests/json.rs`

**Interfaces:**
- Consumes: Tasks 1-3. Produces nothing new.

- [ ] **Step 1: Write the pinning tests (they pass immediately if Tasks 1-3 are right; the mutants below prove them)**

```rust
#[test]
fn every_schema_pins_its_member_lists_on_a_null_bearing_fixture() {
    // resolve (no profile): keys == ["agent","profile","repository","schema","source","tool_version"]
    // status outside a repository, no config: top-level keys as in Task 2; agents[0] keys ==
    //   ["agent","presence","profile","source"]
    // agents: element keys == ["agent","capabilities","evidence","executable","mechanism","support"];
    //   executable keys == ["origin","path","state"]; mechanism ["description","id"];
    //   evidence ["notes","source_url","upstream_version","verified_at"]; capability ["basis","capability","state"]
    // profiles on an empty root: ["agent","ignored_entries","profiles","schema","tool_version"]
    // every nullable member above is asserted == Value::Null, never absent
}

#[test]
fn documents_carry_no_environment_or_secret_shaped_member() {
    // for agents --json and status --json: walk the Value tree; no member name contains '=' and no string value
    // of a member other than `basis`, `notes`, `description` contains "TOKEN", "SECRET", "PASSWORD" (V3 §36)
}
```
`repositories` joins this test in Task 6.

- [ ] **Step 2: Run them**

Run: `cargo nextest run -p agent-profile --test json`
Expected: pass

- [ ] **Step 3: Mutant proofs**

(a) Add `#[serde(skip_serializing_if = "Option::is_none")]` to `StatusDocument.mapping`:
`every_schema_pins_its_member_lists_on_a_null_bearing_fixture` red. (b) Rename `ignored_entries` to
`ignored` in `ProfilesDocument`: the same test red. (c) Add a temporary member `api_token: &'static str`
holding `"TOKEN=x"` to `StatusDocument`: `documents_carry_no_environment_or_secret_shaped_member` red; an
implementer who widened that test's exemption list to make it pass would fail this proof (panel round 2,
Q2). Restore.

- [ ] **Step 4: Commit**

```bash
git add crates/agent-profile/tests/json.rs
git commit -m "test(json): pin every schema's member list on null-bearing fixtures"
```

### Task 5: `repositories` classification, text report and the command

**Files:**
- Create: `crates/agent-profile/src/repositories.rs`
- Modify: `crates/agent-profile/src/lib.rs` (`pub mod repositories;` after `repo`), `src/output.rs` (header
  comment lines 1-3: JSON is SP5b now; append `repositories_lines`), `src/config.rs:236`
  (`is_absolute_key` stays private; add `pub fn is_host_form_key(root: &str) -> bool` beside it)
- Modify: `crates/agent-profile/src/cli.rs:33-46` (`COMMANDS_HELP`: add `  agent-profile repositories` under
  `Repository commands`), `:68-76` (remove `Repositories(Rest)` from `enum Command`), `:87-94`
  (`reserved_name` without `Repositories`), `:99-143` (`CommandWord::Repositories`, `parse`, `as_str`), `:146`
  (usage text: `Usage: agent-profile repositories` / `Show every stored repository mapping and whether it still
  applies.`), `validate_command` (`:461`: `Repositories` joins `Agents | Profiles` in the no-agent check and
  the `--repo` refusal list; takes no bare word; `--json` on it stays `NotYetImplemented` until Task 6),
  `run_command` (`:700`: `if command == CommandWord::Repositories { return run_repositories(&root, &config); }`
  right after the `Agents` return), new `fn run_repositories(root: &AppRoot, config: &Config) -> Result<i32>`;
  `tests/resolution.rs:529` (`command_help_and_the_top_level_help`, if it pins `COMMANDS_HELP`); `cli.rs:1401`
  (the stub test keeps `doctor`)
- Test: unit tests in `src/repositories.rs`; `crates/agent-profile/tests/repositories.rs` (text form; the JSON
  tests join it in Task 6). The command is wired IN THIS TASK so the file is green at this task's commit
  (panel round 1, S2-1: a committed red test file fails `just check`).

**Interfaces:**
- Consumes: `config::Config::mappings` (`config.rs:160`), `config::Mapping` (`:95`), `repo::discover`
  (`repo.rs:105`), `repo::Discovery`, `adapter::lookup` (`adapter/mod.rs:151`), `AppRoot::profiles_dir`
  (`config.rs:75`), `output::{labeled, note, shell_word}` (`output.rs:187-194, :254`; make `labeled` and
  `shell_word` `pub(crate)`).
- Produces (in `repositories.rs`):
  - `pub enum State { Active, Moved, NotARepository, Missing, Unreadable, Foreign }` (`#[derive(Debug, Clone,
    Copy, PartialEq, Eq)]`, so the unit tests can `assert_eq!` on it) with `pub fn token(self) -> &'static str`
    (`active | moved | not_a_repository | missing | unreadable | foreign`).
  - `pub struct ProfileRef { pub name: ProfileName, pub exists: bool }`, `pub struct AgentRef { pub profile:
    ProfileName, pub exists: bool, pub known: bool }`, `pub struct Entry { pub key: String, pub state: State,
    pub detail: Option<String>, pub profile: Option<ProfileRef>, pub agents: BTreeMap<AgentId, AgentRef> }`.
  - `pub fn classify(key: &str) -> (State, Option<String>)` — the spec §4 order: `is_host_form_key` false →
    `Foreign`; (Windows only) first component not a `Prefix` of kind `Disk | VerbatimDisk | UNC | VerbatimUNC`
    → `Unreadable`, `Some("device path refused")`; `fs::metadata(key)` `NotFound` → `Missing`; other error →
    `Unreadable`, `Some(error.to_string())`; not a directory → `Unreadable`, `Some("not a directory")`;
    `repo::discover` `Err(Error::Repository { reason, .. })` → `Unreadable`, `Some(reason)`; any other `Err` →
    `Unreadable`, `Some(error.to_string())`; `NotInRepository` → `NotARepository`; `Repository(root)` with
    `root == Path::new(key)` → `Active`; otherwise `Moved`, `Some(root.display().to_string())`.
  - `pub fn report(root: &AppRoot, config: &Config) -> Vec<Entry>` — one entry per `config.mappings()` in that
    order; `exists` = `root.profiles_dir().join(name.as_str()).is_dir()`; `known` = `adapter::lookup(id).is_some()`.
  - In `config.rs`: `pub fn is_host_form_key(root: &str) -> bool` — Unix: starts with `/`; Windows: does NOT
    start with `/` (the drive and `\\` forms are the host's).
  - In `output.rs`: `pub fn repositories_lines(entries: &[Entry]) -> Vec<String>` — `no repository mappings`
    when empty; otherwise per entry, blank line between entries: `repository:   <key>`; `state:        <token>`
    with ` (now <detail>)` for `Moved` and ` (<detail>)` for `Unreadable`; `profile:      <name>[ (no profile
    directory)]` or `none`; `agents:       id=profile[ (no profile directory)][ (unknown agent)], ...` in key
    order or `none`; and when `profile` is `None` and `agents` is empty, `note:         maps nothing; remove it
    by editing config.toml`.

- [ ] **Step 1: Write the failing unit tests in `src/repositories.rs`**

```rust
#[test]
fn classify_orders_foreign_missing_unreadable_not_a_repository_active_and_moved() {
    // tempdir; a repository `acme` (dir + .git/HEAD) canonicalised through repo::canonical:
    //   classify(acme) == (Active, None)
    //   classify(acme/sub) (a plain subdirectory) == (Moved, Some(acme.display()))
    //   classify(tempdir/plain) (dir, no .git) == (NotARepository, None)
    //   classify(tempdir/none) == (Missing, None)
    //   classify(tempdir/file.txt) (a regular file) == (Unreadable, Some("not a directory"))
    //   classify(the other platform's form: unix "C:\\src\\acme", windows "/srv/acme") == (Foreign, None)
}

#[cfg(unix)] #[test]
fn classify_treats_a_dangling_symlink_as_missing_and_a_broken_git_as_unreadable() {
    // symlink tempdir/dangling -> tempdir/gone: (Missing, None)
    // dir `broken` whose `.git` is a FILE containing "gitdir: /nonexistent/x": (Unreadable, Some(reason)) with
    //   reason non-empty (the exact text is repo::discover's; assert state and detail.is_some())
}

#[cfg(windows)] #[test]
fn classify_refuses_a_device_namespace_key_before_touching_it() {
    assert_eq!(classify(r"\\.\pipe\agent-profile-test"), (State::Unreadable, Some("device path refused".into())));
}

#[test]
fn report_marks_missing_profile_directories_unknown_agents_and_keeps_stored_order() {
    // config text: [repositories.'<acme>'] profile = "work", agents = { fake = "mine", gemini = "work" }
    //              [repositories.'<zzz-missing>'] profile = "gone"
    // AppRoot with profiles/work/fake materialised (create_dir_all), nothing for mine or gone
    // entries[0]: key acme, profile {work, exists true}, agents {fake: {mine, exists false, known true},
    //   gemini: {work, exists true, known false}}; entries[1]: state Missing, profile {gone, exists false}
    // order == stored key order (config.mappings() is sorted by key string)
}
```

- [ ] **Step 2: Write the failing text-form tests in `tests/repositories.rs`**

```rust
#[test]
fn repositories_prints_one_block_per_mapping_with_states_and_annotations() {
    // Root::empty(); scratch: acme (git), acme/sub, plain (dir), gone (absent), file.txt; configure mappings for
    // all five plus an empty table for acme/empty (git_dir'd) and a foreign key; `fake create work` for one profile
    // run `repositories`: exit 0; stdout split on "\n\n" has one block per key in sorted-key order; assert the
    // exact lines of the acme block, the `state:        moved (now <acme>)`, `state:        missing`,
    // `state:        unreadable (not a directory)`, `state:        foreign` lines, `(no profile directory)`,
    // `(unknown agent)` in that order on one pair, and the `note:         maps nothing; ...` line only on the empty table
    // stderr == ""
}

#[test]
fn repositories_with_no_mappings_prints_the_empty_line_and_creates_nothing() {
    // Root::empty(): stdout == "no repository mappings\n", exit 0, root dir still empty (no config.toml, no lock)
}

#[test]
fn repositories_continues_past_an_unreadable_entry_sorted_first() {      // Review Focus 3 and spec §7
    // key "aaa" is a regular FILE (sorts first), key "bbb" a repository: both blocks present, exit 0
}

#[test]
fn repositories_refuses_an_agent_word_arguments_and_repo() {
    // ["fake","repositories"] -> 2 "`repositories` takes no agent: agent-profile repositories"
    // ["repositories","x"] -> 2 "`repositories` takes no arguments"
    // ["repositories","--repo","x"] -> 2 "unknown option \"--repo\" for `repositories`"
    // ["repositories","--yes"] -> 2 "unknown option \"--yes\" for `repositories`"
    // ["repositories","--json"] -> 2 "`--json` is not yet implemented"   (until Task 6)
}
```
In `cli.rs` unit tests: `top(&["repositories"])` is `Ok(Invocation::Command { command: CommandWord::Repositories,
json: false, .. })`; `split_err(&["fake", "repositories"])` is the usage error above; `top(&["repositories",
"--json"])` is `Err(Error::NotYetImplemented { .. })` for now.

- [ ] **Step 3: Run them to verify they fail**

Run: `cargo nextest run -p agent-profile --lib repositories --lib cli --test repositories`
Expected: compile failure (module missing; `CommandWord::Repositories` missing); end-to-end rows exit 2
(`not yet implemented`)

- [ ] **Step 4: Implement `repositories.rs`, `is_host_form_key`, `repositories_lines`, then the grammar and
  `run_repositories`**

`classify` uses `std::path::Component::Prefix` and `Prefix::kind()` under `#[cfg(windows)]` for rule 1a; the
`Active` compare is `root == Path::new(key)` (component equality, the test `Config::mapping` uses at
`config.rs:155`). `run_repositories`: `let entries = repositories::report(root, config);
write_lines(&output::repositories_lines(&entries))?; Ok(0)`. `repositories` leaves the clap `enum Command`
exactly as `status` and the lifecycle words did (SP3 design §7.2, SP5a design §5).

- [ ] **Step 5: Run the whole crate**

Run: `cargo nextest run -p agent-profile`
Expected: all pass, `tests/repositories.rs` included

- [ ] **Step 6: Mutant proofs**

(a) Swap the `Missing` and `NotARepository` arms:
`classify_orders_foreign_missing_unreadable_not_a_repository_active_and_moved` red. (b) Compute `exists`
against the repository root instead of `profiles_dir`:
`report_marks_missing_profile_directories_unknown_agents_and_keeps_stored_order` red. (c) Drop the
`Repositories` arm from the no-agent check: `repositories_refuses_an_agent_word_arguments_and_repo` red.
(d) Let `report` stop at the first `Unreadable` entry:
`repositories_continues_past_an_unreadable_entry_sorted_first` red.

- [ ] **Step 7: Commit**

```bash
git add crates/agent-profile/src/repositories.rs crates/agent-profile/src/lib.rs crates/agent-profile/src/output.rs crates/agent-profile/src/config.rs crates/agent-profile/src/cli.rs crates/agent-profile/tests/repositories.rs crates/agent-profile/tests/resolution.rs
git commit -m "feat(repositories): classify every stored mapping and report it"
```

### Task 6: the `repositories` document, `--json` on it, and the docs

**Files:**
- Modify: `crates/agent-profile/src/cli.rs` (`COMMANDS_HELP` and the `repositories` usage text gain
  `[--json]` and the `--json` option line; `validate_command`: `Repositories` joins the `--json` list of Task 2;
  `run_command` passes `json` to `run_repositories(root, config, json: bool)`, which renders the document
  when set).
- Modify: `crates/agent-profile/src/json.rs` (repositories family), `tests/repositories.rs` (JSON tests),
  `tests/json.rs` (pinning rows and the secrets walk), `README.md:40-52` (usage block), `README.md:65-70` (a
  paragraph on `repositories` and the JSON contract), `ROADMAP.md:14` (SP5 row: SP5b done).

**Interfaces:**
- Consumes: `repositories::{report, Entry, State}` and `run_repositories` (Task 5), `json::render`, the
  `--json` grammar of Task 2.
- Produces: `pub struct ProfileRefView { name: String, exists: bool }`, `AgentRefView { profile: String,
  exists: bool, known_agent: bool }`, `RepositoryView { repository: String, state: &'static str, detail:
  Option<String>, profile: Option<ProfileRefView>, agents: BTreeMap<String, AgentRefView> }`,
  `RepositoriesDocument { schema, tool_version, repositories: Vec<RepositoryView> }`, `pub fn
  repositories(entries: &[Entry]) -> RepositoriesDocument` (`schema = "agent-profile/repositories/1"`).

- [ ] **Step 1: Write the failing tests**

In `tests/repositories.rs`:
```rust
#[test]
fn repositories_json_carries_every_state_with_detail_and_references() {
    // the fixture of the text test: for each key assert state token, detail (null / moved root / "not a directory"),
    // profile {name, exists}, agents {id: {profile, exists, known_agent}}; the empty table has profile null and
    // agents {}; order == sorted keys; stderr == ""; `repositories --json` on an empty root: repositories []
}
#[test]
fn repositories_json_members_are_pinned_on_a_null_bearing_entry() {
    // keys(top) == ["repositories","schema","tool_version"]; entry keys == ["agents","detail","profile","repository","state"];
    // the missing-key entry has detail == Null and profile == Null when its table is empty
}
```
In `cli.rs` unit tests: `top(&["repositories", "--json"])` now sets `json: true` (replace Task 5's
`NotYetImplemented` expectation), and the Task 5 end-to-end row `["repositories","--json"] -> 2` is removed.

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo nextest run -p agent-profile --lib cli --test repositories`
Expected: FAIL (`json::repositories` does not exist; `repositories --json` still exits 2)

- [ ] **Step 3: Implement the JSON family, add `Repositories` to the `--json` list, then update the docs**

`run_repositories(root, config, json)`: `let entries = repositories::report(root, config);` then
`write_out(&json::render(&json::repositories(&entries)))` when `json`, else the Task 5 lines; `Ok(0)`.
README usage block: add `agent-profile repositories [--json]` and `[--json]` to the six commands; one paragraph
after line 70: what `repositories` reports (the six states), that removing an entry takes one `unlink` per stored
field, that `--json` documents carry `schema` and `tool_version`, that consumers read stdout only and must run
`repositories` under their own timeout, and that a status-bar integration uses `<agent> status --json`.
ROADMAP row 14: `SP5b done: --json on status/resolve/agents/profiles/list, repositories; SP5c-d next`.

- [ ] **Step 4: Run the gate**

Run: `just check`
Expected: exit 0; nextest reports every test passing (295 + the new ones; no skips other than platform `cfg`s)

- [ ] **Step 5: Mutant proofs**

(a) Make `json::repositories` emit `detail` as the state token for `Moved`:
`repositories_json_carries_every_state_with_detail_and_references` red. (b) Add
`#[serde(skip_serializing_if = "Option::is_none")]` to `RepositoryView.detail`:
`repositories_json_members_are_pinned_on_a_null_bearing_entry` red.

- [ ] **Step 6: Commit**

```bash
git add crates/agent-profile/src/cli.rs crates/agent-profile/src/json.rs crates/agent-profile/tests/repositories.rs crates/agent-profile/tests/json.rs README.md ROADMAP.md
git commit -m "feat(cli): repositories --json; document the JSON contract"
```

---

## Self-review (done 2026-10-19 before handoff)

1. Spec coverage: §3.1-§3.2 Task 1 (`render`, order, newline) and Task 2 (stdout-only, exit parity); §3.3-§3.4
   Tasks 1-2; §3.5-§3.6 Task 3; §3.7 and §4 Tasks 5-6; §6 nothing to build; §7
   oracles Tasks 2-6 plus Task 4; §5 grammar Tasks 2, 5 and 6; §8 constraints in Global Constraints;
   README/ROADMAP Task 6.
2. Step scan: every code step names its signature and file; bodies appear only for `classify`'s rule order
   (spec §4) and the text block (spec §4), which the tests do not fully determine.
3. Type consistency: `Presence` (Task 1) is what Task 2's rows carry; `Entry`/`State` (Task 5) are what Task 6's
   builder and renderer consume; `json: bool` threads `Invocation::Command` → `run_command` → the three runners.
4. Review Focus: each of the five lines has a named test in Tasks 2, 3, 5.
5. Proportion: six tasks, test names plus assertions, signatures without bodies.

Exhaustiveness audit: no TBD; the only values left to the implementer are message wording already fixed by the
spec and the module-internal helper names. Not covered by any test on this host and left to CI: the Windows
device-namespace test (`cfg(windows)`), the `foreign` key form on Windows (`/srv/acme`), and NTFS case folding of
a `moved` key; all three are stated in the spec's §4.

## Panel

- Round 1, solo (2026-10-19; persona relentless-adversarial-auditor; seats Axiom Breaker, Cascade Analyst,
  Literal Implementer, Mechanism Gamer, Protocol Pedant, Activation Auditor; dropped State Corruptor, Boundary
  Smuggler, Resource Vampire, Dependency Cynic, Blindspot Auditor as covered by the spec's panel). Folded: the
  Task 3 unit test must build its entries from a registry adapter's metadata, because the hand-built fixture
  declares no capability and the builder's `expect` would fire (Literal Implementer); `State` derives
  `PartialEq`/`Debug` for the unit tests' `assert_eq!` (Literal Implementer). PANEL VERDICT: two folds, no
  open finding.
- Round 1, agy escalation (2026-10-19; brief `.clavity/seams/sp5b-plan-panel.md`, reply
  `.clavity/scratch/sp5b-plan-panel/round1-reply.md`; seats Axiom Breaker, Cascade Analyst, Literal
  Implementer, Mechanism Gamer, Protocol Pedant, Activation Auditor; three of six "no new findings"):
  `NEGOTIATE` on one blocking finding, verified and folded. S2-1 Task 5 committed a red end-to-end test file
  (verified: `just check` runs `cargo nextest run --workspace`, `justfile:13`, so the Task 5 commit would fail
  the gate): the `repositories` command's grammar and text wiring moved from Task 6 into Task 5, Task 6 keeps
  the document, `--json` on it, and the docs (the peer's option 2; its question 2 asked for exactly this
  boundary). Debt folded: `output.rs:259` added to Task 1's files (S3-1); the Task 1 mutant now flips
  `GlobalDefault`, which no fixture in that task resolves through, so exactly one test turns red (S4-1).
  Answers: Q3 the plan's oracle (hand-written key lists on null-bearing fixtures plus raw prefix and suffix
  checks) is the right one for this slice; a golden file is brittle, a schema file needs a validator, a
  round-trip struct misses order and the newline. Discarded by the peer, agreed: mandating sorted map keys;
  renaming the view structs. PANEL VERDICT: one blocking folded, two debt folded; round 2 owed.
- Round 2, agy escalation (2026-10-19; brief `.clavity/seams/sp5b-plan-panel-r2.md`, reply
  `.clavity/scratch/sp5b-plan-panel/round2-reply.md`; seats Axiom Breaker, Cascade Analyst, Dependency Cynic
  (new), Blindspot Auditor (new), Literal Implementer, Mechanism Gamer; four of six "no new findings"):
  `NEGOTIATE` on one blocking finding, verified and folded. S2-1 Task 1's commit line omitted `output.rs`,
  the file whose `is_proper_ancestor` the task makes `pub(crate)`, so the committed tree would not compile
  (verified against the Files list and the commit line): the file is in the `git add` now. Debt folded: Task
  2's mutant (a) would have reddened the case-twin test too; it now mutates the bare `status` arm only
  (S6-1). Q2: the secrets walk gained a mutant proof (Task 4, c) so widening its exemption list cannot pass
  unnoticed. Q3 the six-task grain confirmed. Discarded by the peer, agreed: a hyphenation slip (fixed
  anyway); runner parameter order. PANEL VERDICT: one blocking folded, one debt folded, one guard added;
  round 3 owed.
- Round 3, agy escalation (2026-10-19; brief `.clavity/seams/sp5b-plan-panel-r3.md`, reply
  `.clavity/scratch/sp5b-plan-panel/round3-reply.md`; seats Axiom Breaker, Cascade Analyst, Cold Executor
  (bespoke, new), Literal Implementer, Mechanism Gamer): GREEN, no finding. The Cascade Analyst walked all
  six commits and found each green under `just check`; the Cold Executor found Tasks 2 and 5 executable
  from their own text. Q1 named the single-write constraint as the line no test would miss if deleted (it
  stays, and the spec pins it). Q2: no design requirement without a task. Q3 folded: mutant proofs are logged
  under `.clavity/scratch/sp5b-mutants/` (Global Constraints). Discarded by the peer, agreed: `use`
  boilerplate; checklist phrasing. PANEL VERDICT: GREEN; no live challenge.

## Stand-downs
None.
