# SP5a — Profile lifecycle: `create`, `delete`, `profiles`, `agents`, `list`

Status: DRAFT for owner approval. V3 (`agent-profile-implementation-spec-v3.md`) wins over this document; §1
records every place this document goes past it. Part of SP5, which the owner split on 2026-10-19 into
SP5a lifecycle → SP5b `--json` + `repositories` → SP5c `doctor` → SP5d completions, docs and CI-pinning
debt. The ordering consult is `.clavity/seams/sp5-order.md` (agy-first, aligned).

## 1. Intent

Today a profile exists only as a side effect of a launch (lazy creation, V3 §9), and the commands that
should manage profiles answer `not yet implemented` (`cli.rs`: `Command::{Agents, Profiles, List, Create,
Delete}`, and `<agent> <word>` for the same words). SP5a makes the profile lifecycle explicit and safe:
create one on purpose, see what exists, remove one without ever leaving a mapping pointing at nothing.

Goals
- `create` and `delete` as V3 §10 and §16 define them, exactly.
- `agents` and `profiles` (and `list`) say what is available and what exists, human-readable.
- `delete` can never produce a dangling mapping (V3 §31), including under a concurrent `link`.

Non-goals
- `--json` anywhere (SP5b). Every `--json` stays `NotYetImplemented`, as today.
- `repositories`, `doctor`, `completions` (later slices). `hide_support_warning` stays deferred to SP5d.
- Copying, renaming or migrating profiles. Deleting mappings on the user's behalf. Pruning.
- Any credential handling (V3 §36). Nothing here reads or writes agent credentials.

Anti-goals — a result that "works" but is a failure
- `delete` removing anything outside `<root>/profiles/`, or following a symlink out of it.
- `delete` reporting success when files remain.
- `create` altering an existing profile in any way.
- Any surface printing a support level without the capability matrix (owner ruling 3).

## 2. Ground truth in the repository (read 2026-10-19 at main 14bd5b1)

- A profile for agent `A` named `P` lives at `<root>/profiles/P/A` (`adapter/mod.rs:176` `profile_dir`).
  `Adapter::paths(root, profile)` lists everything the adapter owns, the first entry being that directory.
- `ensure_paths` (`adapter/mod.rs:289`) is idempotent and never overwrites a file. `Adapter::presence`
  (`adapter/mod.rs:67`) says `Materialized` or `Absent` and never writes.
- `Config::mappings_referencing(profile)` (`config.rs:165`) returns every repository-mapping field naming
  the profile, case-insensitively, each `Reference { root, agent: Option<_>, profile }`. It does NOT look
  at `default_profile` (`config.rs:150`).
- `config::update` takes `config.toml.lock` (`config.rs:349`, `acquire` at `:612`); `link`/`unlink` use it.
  SP3 design D2 reserved this lock for `delete`'s reference check.
- `config::update` (`config.rs:349`) is the only lock-holding entry point and it is a read-modify-WRITE; it
  also creates the root and sweeps temp files. `delete` must hold the lock without rewriting `config.toml`, so
  SP5a adds a lock-holding read scope (shape left to the builder, §8) beside it, sharing `acquire`.
- Exit codes live in `Error::exit_code` (`error.rs:79`): usage 2, profile/config 4, `Io` 1.
- The capability matrix renderer is `output::report_lines` (`output.rs:87`); labels are
  `support_label`, `capability_label`, `state_label`.

## 3. Commands

### 3.1 `agent-profile <agent> create <profile>`
V3 §10. Steps: parse agent (known-agent check already in `split`) and profile name; load config (a parse
failure is an error, V3 §17); refuse a case-only twin of an existing profile directory
(`ProfileCaseConflict`, as launch does); read `presence()`; `ensure_paths(adapter.paths(..))`.
- Output (stdout, one line): `created profile 'work' for claude: <path>` when presence was `Absent`
  beforehand, `profile 'work' for claude already exists: <path>` when it was `Materialized`. Exit 0 both.
  A partially materialized profile is completed and reported as created. (My call, §9.)
- Takes no wrapper options. `--dry-run`, `--verbose`, `--repo` are usage errors; `--json` is
  `NotYetImplemented`. `create` never launches the agent and never needs the agent installed.
- A reserved or invalid name is `InvalidProfileName` (exit 4), same as `link`.

### 3.2 `agent-profile <agent> delete <profile> [--yes]`
V3 §16 and §31. The confirmation prompt happens BEFORE the lock, because `acquire` (`config.rs:612`) retries
for a bounded `LOCK_TIMEOUT` and then fails: a lock held across a prompt would make every concurrent `link`
and `unlink` time out while a user thinks. The reference check is therefore done twice: once for the
prompt text (advisory), and once under the lock after confirmation (authoritative).
1. Parse agent and profile name.
2. Resolve `<root>/profiles/<P>/<A>` and test existence with `symlink_metadata` ONLY (no `canonicalize`, which
   fails on a missing path). "Exists" means a directory entry exists there, NOT that `presence()`
   is `Materialized`. If it does not exist, branch NOW to the "no directory entry" outcome at the end of this
   section and stop: steps 3-9 never run, so the advisory check below is only a note there, never a failure.
   For an existing entry: a half-materialised profile is deleted too, so no command is ever unable to clean up
   its own leftovers. The name must match the directory entry exactly; a case-only twin is
   `ProfileCaseConflict`.
3. Containment: `symlink_metadata` on `<root>/profiles/P` and on the target. If either is a symlink (or a
   Windows junction/reparse point), refuse with `ProfileDir` and touch nothing, including a symlink that
   resolves back inside the root — `remove_dir_all` semantics on links differ per platform and there is no
   legitimate reason for a profile directory to be one. Then canonicalise and require the result to be inside
   the canonicalised `<root>/profiles/`.
4. Advisory reference check (below) → if referenced (entry exists), fail now with the same error as step 7.
5. Confirm unless `--yes`: prompt on stderr `delete profile 'work' for claude at <path>? [y/N] `, read one
   line from stdin, accept `y`/`yes` case-insensitively, anything else aborts with exit 1 and `not deleted`.
   If stdin is not a terminal and `--yes` is absent, fail immediately with a usage error (exit 2) without
   prompting; the message says `--yes is required when input is not a terminal`. An end-of-file at the prompt (Ctrl-D on a terminal) counts as "no": exit 1, `not deleted`.
6. Take the config lock; load the config under it. Re-run step 2's existence test and step 3's containment
   checks now, under the lock, immediately before step 8: the prompt can take unbounded time and the profile
   directory is agent-writable, so the earlier checks are stale. If the entry has vanished meanwhile, report
   the "does not exist" outcome and exit 0 (the desired state holds).
7. Authoritative reference check. Refuse if referenced (exit 4, error `ProfileReferenced`, V3 §16.1 text,
   extended to list each referencing repository path and field, and `default_profile`). References:
   - a repository-wide mapping `profile = P`, which applies to every agent, so it blocks `delete <any
     agent> P`;
   - an agent mapping `agents.<A> = P` for THIS agent only (owner decision, §9: an `agents.codex = P`
     mapping does not stop deleting `claude`'s `P`);
   - `default_profile = P`, which applies to every agent. New code: `mappings_referencing` ignores it, and a
     deleted default would make every unmapped repository fail to resolve. V3 names only mappings; this
     goes one step past it (owner-decided 2026-10-19) and the design ledger records the divergence.
   Matching is ASCII-case-insensitive for all three (`default_profile = "Work"` blocks deleting `work`).
8. `remove_dir_all` of `<root>/profiles/P/A` (std's implementation does not follow links it meets INSIDE the tree; it does
   resolve links in the path it is handed, so `<root>/profiles/P` being swapped for a symlink is covered only
   by the re-check in step 6. The residual window between that re-check and the removal is accepted and
   stated, not closed); then try to remove `<root>/profiles/P` only if it is now empty. A failure of that second removal is NOT
   a deletion failure: the agent's directory is gone, so it is a stderr `note: could not remove empty
   directory <path>: <error>` and exit 0 (a retry would exit 0 at step 2 and never reach it, so reporting
   "partially removed, retry" for it would be false). Only a failure of the `P/A` removal is `ProfileDir`. Any OS
   error (a Windows sharing violation included) is `ProfileDir` (exit 4), reports the path, says the profile
   may be PARTIALLY removed and can be retried (deletion is not atomic), and never says "deleted". The lock is released on every path.
9. Output `deleted profile 'work' for claude: <path>`.
- A profile with no directory entry: exit 0, `profile 'work' for claude does not exist; nothing to delete`,
  no prompt, no lock. If the advisory check finds a mapping or `default_profile` still naming it, a second
  stderr line says so (`note: still referenced by <list>; unlink it`), because the invariant of V3 §31 was
  already violated by hand-editing and silence would hide it. Exit stays 0. (My call, §9.)
- Known limit, stated not solved: launch takes no lock (SP2 §4.4), so a launch or a `create` racing a delete can
  re-materialise the directory. Mappings cannot dangle (the lock covers `link`); a freshly launched profile
  can reappear. V3 §16.2 accepts that deletion does not enumerate running processes.

### 3.3 `agent-profile profiles` and `agent-profile list`
Lists profiles that exist on disk: scan `<root>/profiles/*/`, group by profile name, show which agents have
a materialised directory (each agent's `presence()`, so a half-created one is not listed as present).
`list` (no agent) is an alias of `profiles`; `agent-profile <agent> list` is `profiles` filtered to one
agent and prints profile names only, one per line (script-friendly). When nothing matches, stdout is empty
and stderr says `no profiles for claude` (exit 0), so a human is never left with silence. Sorted by name then agent id. An entry under `<root>/profiles/` that is not a valid-profile-name directory, or a directory under a profile
that is not a known agent id, is left out of the list and counted in one trailing line `ignored N
unrecognised entries` on STDERR (stdout stays pipe-clean). Names starting with `.` (such as `.DS_Store`)
are ignored without counting. (My call, §9.) An empty or missing root prints `no profiles`. Needs no executable. Layout, one profile per line, agents
sorted and comma-joined: `work    claude, codex` (name column padded to the longest name). `profiles`,
`list` and `agents` are read-only: they never create the root, `config.toml` or the lock file (SP3 set
that precedent for `unlink` of nothing). They accept no arguments; `--json` is `NotYetImplemented`, any
other argument or option a usage error naming the command.

### 3.4 `agent-profile agents`
One block per agent in the registry (12), fixed order: id, whether an executable is currently found
(config override or `PATH`, no launch, no version probe; four states: found with its origin, not found on
`PATH`, configured path missing, and found but unsupported — a shell script or Windows shim, which
`exe::discover` already refuses at launch), the evidence's upstream version and verification date, mechanism sentence, support level and the
FULL capability matrix with bases, rendered by the same code as `--dry-run` (extract a shared function from
`report_lines`; do not copy it). Never conditional, never shortened (owner ruling 3).

## 4. Errors and exit codes
New: `Error::ProfileReferenced { agent, profile, references }` → 4. Reused: `InvalidProfileName` 4,
`ProfileCaseConflict` 4, `ProfileDir` 4, `Usage` 2, `ConfigInvalid` 4. Declined confirmation: exit 1.
No new exit code numbers (V3 §33).

## 5. Grammar changes
`create`, `delete`, `list` become valid after an agent word and (for `list`) alone. Top-level `create` and
`delete` without an agent stay usage errors with a message naming the form `agent-profile <agent> create
<profile>` (V3 §10/§16 define only the agent-first form). `agents`, `profiles` are top-level only: `validate_command` rejects an agent word before them
(`agent-profile claude agents` is a usage error), because `split` would otherwise route it there.
`--yes` is accepted by `delete` alone and has no short form. Today `validate_command` (`cli.rs`) accepts
`--repo` for every `CommandWord` and allows at most one bare word only for `link`; SP5a must reject `--repo`
for `create`, `delete`, `list`, `agents`, `profiles` (usage error naming the command), let `delete` take
`--yes`, and let `create`/`delete` take exactly one profile word (`list`, `agents`, `profiles` none). `CommandWord` gains `Create`, `Delete`, `List`
plus the top-level `Agents`/`Profiles`, each with usage text beside the existing ones (`cli.rs` help). `RESERVED_WORDS` in `name.rs` needs no change. In `cli.rs` the clap `enum Command` holds the five words as
"not yet implemented" stubs and `reserved_name()` turns them into `NotYetImplemented`; as `status` and
`link` did in SP3 (note above `enum Command`), SP5a removes `Agents`, `Profiles`, `List`, `Create`, `Delete`
from that enum and dispatches them before clap, so `--help` never lists them as unimplemented.
`Invocation::Command` gains a field carrying `--yes` (and the whole `Command` variant may be reshaped; left
to the builder).

## 6. Testing (oracles)
- Contract tests per command in `tests/` using `fake-agent`/the existing support harness: idempotent
  create, create leaves an existing file untouched (byte compare), partial profile completion, delete
  of each reference class (repo-wide, this-agent, other-agent allowed, default_profile), delete through a symlink (pointing outside the root AND pointing back inside it) refused with the tree
  unchanged, half-materialised profile deleted, delete of an in-use directory reported as failure
  (Unix: directory made unremovable), no-TTY without `--yes` fails fast (exit 2), EOF at the prompt aborts (exit 1). The prompt logic takes a
  reader so tests drive it without a pty; the TTY check is one small function covered by its own test.
- Siblings: deleting `claude`'s `P` leaves `<root>/profiles/P/codex` and its contents intact, and removes
  `<root>/profiles/P` only when empty.
- Prompt/lock: a `delete` waiting at its prompt must not block a concurrent `link` (assert `link` succeeds
  while the prompt is pending, via a reader the test controls); a mapping added between prompt and
  confirmation makes the `delete` fail `ProfileReferenced`.
- Concurrency: a `link` and a `delete` of the same profile raced under the lock never end with a mapping
  naming a missing profile (run both orders; assert the invariant, not an order).
- `agents` output contains every capability for every agent, no output path omits the matrix.
- One logic mutant per command, as in SP3/SP4a: drop the lock, drop the `default_profile` check, drop the
  containment check, make `create` overwrite, replace the empty-parent `remove_dir` with `remove_dir_all` (sibling test), skip
  the post-prompt re-check of containment (a test that swaps `P` for a symlink while the prompt waits), drop the re-check under the lock (the "mapping added between prompt and
  confirmation" test), widen the agent-precise filter to any agent — each must turn a named test red.
- The gate is the repository's own `just check`, unchanged.

## 7. Constraints
No new dependencies expected (std + existing `tempfile`, `toml_edit`). Linux, macOS, Windows green. No
secret in any output. `delete`'s containment check and lock are the security-relevant parts: higher
verification bar (own mutants, capstone round on the delta).

## 8. Left to the builder
Module layout, function names, the shape of the lock-holding read scope, exact prompt and message wording within §3's shapes, how the TTY check is
done per platform, and the internal API for scanning the profiles directory.

## 9. Calls I made (owner: please check)
1. `agents.<other> = P` does NOT block deleting this agent's `P` (agent-precise). V3 §16.1 says "any
   repository mapping"; the stricter reading blocks on any reference. I chose precision because the other
   agent's directory is untouched. Flip to strict by dropping one filter.
2. `delete` of a nonexistent profile exits 0 (idempotent). The alternative, exit 4 "no such profile",
   catches typos but breaks the idempotence `create` and `unlink` set.
3. `create` on a partial profile reports "created" (it did create something).
4. `list` bare = alias of `profiles`.
5. `delete` also blocks on `default_profile` (owner-decided 2026-10-19 as part of the slicing answer).
6. `profiles` skips unrecognised entries but counts them on one line.
7. Panel round 1 (agy): a vanished or never-existing entry exits 0 on every path, containment is re-checked
   under the lock, list output streams and the filtered-list shape are fixed, `--repo` is rejected for all
   new commands.
8. Plan-time amendment to §3.4 (2026-10-19): a fourth executable state (unsupported shim, already an outcome
   of `exe::discover`) and a `version:` line showing the evidence's upstream version. Flagged to the owner
   with the plan.
9. Builder-check calls: prompt before lock with a re-check under the lock; "exists" is a directory entry,
   not `Materialized`; any symlink or junction at the target is refused; a still-referenced nonexistent
   profile exits 0 with a stderr note.

## Stand-downs
None. Every panel finding to date was folded; none was discarded below the floor or accepted unverified.
