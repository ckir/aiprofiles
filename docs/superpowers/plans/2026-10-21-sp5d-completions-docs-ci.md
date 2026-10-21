# SP5d — completions, README, CI pinning and clippy matrix: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `agent-profile completions <shell>`, pin every GitHub Action to a commit SHA with an enforcing check, run clippy on three operating systems behind the unchanged `Clippy` status check, and refresh the README, ROADMAP and TODO.

**Architecture:** The hand dispatcher in `cli.rs` gains a grammar table (methods on `CommandWord`) that both `validate_command` and a new completion model read, so the script cannot drift from the dispatcher. The model is a `clap::Command` built only for `clap_complete::generate`, in a child module `cli/completions.rs`. CI work is three small commits (pin check, pin conversion, clippy matrix) that precede the Rust work.

**Tech Stack:** Rust 1.98 / edition 2024, clap 4.6.7 (unchanged), clap_complete 4.6.11 (new), POSIX `sh` + shellcheck for the pin check, GitHub Actions YAML.

**Spec:** `docs/superpowers/specs/2026-10-21-sp5d-completions-docs-ci-design.md` (approved by the owner 2026-10-21). The plan argues from it; executors read both.

## Global Constraints

- `just check` (fmt-check, clippy `-D warnings`, typos, shellcheck `-s sh`, line-citations, nextest, probe-tests, tools-doc-check, doc-check) is green after EVERY commit. Run it as `just check`; do not invent stricter flags.
- `clap` stays at 4.6.7; `clap_complete` is `4.6` under `[workspace.dependencies]`, default features only; only `clap_complete` (4.6.11) is added to `Cargo.lock`. No `cargo update`. `cargo deny check` stays green.
- `completions` never constructs an `AppRoot`, reads `config.toml`, touches the filesystem, or prints an environment value (spec §1 anti-goals, V3 §36).
- Comments in `*.rs`, `*.sh` and `*.yml` under `crates/`, `scripts/`, `sandbox/`, `.github/` must not cite another file by line number (`scripts/check-line-citations.sh`). Cite symbols.
- Every `uses:` in `.github/workflows/*.yml` matches `^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(/[^@ ]+)?@[0-9a-f]{40} # [A-Za-z0-9_.-]+$` unless it starts with `./`.
- The required status-check name `Clippy` keeps existing (spec §4). Branch protection is not touched.
- Commit order: CI (Tasks 1–3) → grammar refactor (Task 4) → completions (Tasks 5–6) → docs (Task 7). Commits end with `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`. Nothing is pushed and no PR is opened without the owner's OK.
- Error text printed by the binary is `agent-profile: error: <message>\n` on stderr, exit code from `Error::exit_code` (`Usage`/`NotYetImplemented` → 2, `Io` → 1).
- SHAs below were resolved on 2026-10-21 with `gh api repos/<o>/<r>/git/ref/tags/<tag>` (annotated tags dereferenced via `git/tags/<sha>`), script at the session scratchpad `resolve-pins.sh`. The implementer re-resolves before editing and reports any difference as `STATE_MISMATCH` rather than picking one.

## Review Focus

Inputs the spec implies but no spec test names. Each is pinned in the task that owns the code.

1. `completions bash extra --help` — help wins over the arity error, exit 0 with the usage text (Task 6, `help_wins_before_any_arity_check`).
2. A stdout that cannot be written (`> /dev/full`) — exit 1 with `could not write to stdout`, no panic from `clap_complete::generate`'s `expect` (Task 6, `a_failed_stdout_write_exits_one`).
3. `completions bash --repo=x` (the `=` form bound by `bind`) — the same `unknown option "--repo"` as the two-token form (Task 6, contract table row).
4. A workflow directory with no `uses:` at all, or a nonexistent directory — the pin check must FAIL, not pass on an empty scan (`scripts/lib/scan-guard.sh` idiom) (Task 1, test cases `empty directory is a broken scan` and `missing directory is a broken scan`).
5. A job-level `uses: ./.github/workflows/release.yml` with four-space indent and no `- ` (release-plz.yml has one) — accepted as local (Task 1, test case `job-level local workflow is exempt`).

---

### Task 1: The action-pin check script, its test, and the `action-pins` recipe

**Files:**
- Create: `scripts/check-action-pins.sh`
- Create: `scripts/tests/check-action-pins.sh`
- Modify: `justfile` (add recipe `action-pins`; add it to `check`)
- Modify: `.claude/recommended-tools.json` (the `just` entry's `why` text lists `action-pins`)
- Modify: `docs/dev-tooling.md` (regenerated, never hand-edited)

**Interfaces:**
- Produces: `sh scripts/check-action-pins.sh [dir]` — scans every `*.yml` in `dir` (default `<repo>/.github/workflows`); prints one `<file>:<line>: <offending text>` line per violation and exits 1; on success prints `check-action-pins: <n> uses checked` and exits 0; exits non-zero through `scan_empty_failure` when there is nothing to scan. Task 2 and the CI job depend on this exit contract.

- [ ] **Step 1: Write the failing test `scripts/tests/check-action-pins.sh`**

Same harness shape as `sandbox/tests/resolve-agents.sh`: `set -eu`, `root=$(cd "$(dirname "$0")/../.." && pwd)`, a `check <name> <got> <want>` function counting `failures`, `exit 1` at the end if any failed. Fixtures live under `tmp=$(mktemp -d)` with `trap 'rm -rf "$tmp"' EXIT INT TERM`; each case is its own subdirectory holding one `w.yml`. A `run <dir>` helper captures `out=$(sh "$root/scripts/check-action-pins.sh" "$dir" 2>&1)` and `status=$?` under `set +e`/`set -e`.

Cases, each asserting the status and (for a rejection) that `w.yml:<line>:` appears in `out` exactly once (`grep -c`):

| case | `w.yml` content (one `uses:` line each, inside a minimal `jobs:` skeleton) | status | named line |
|---|---|---|---|
| sha plus tag is accepted | `      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1` | 0 | — |
| bare uses form is accepted | `        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1` | 0 | — |
| sub-path action is accepted | `      - uses: owner/repo/sub/dir@3d3c42e5aac5ba805825da76410c181273ba90b1 # v1.2.3` | 0 | — |
| job-level local workflow is exempt | `    uses: ./.github/workflows/release.yml` | 0 | — |
| commented-out floating uses is ignored | `      # - uses: actions/checkout@v7` plus one accepted line | 0 | — |
| floating tag is rejected | `      - uses: actions/checkout@v7` | 1 | yes |
| branch name is rejected | `      - uses: dtolnay/rust-toolchain@stable` | 1 | yes |
| 39-hex sha is rejected | `@3d3c42e5aac5ba805825da76410c181273ba90b # v7.0.1` | 1 | yes |
| upper-case sha is rejected | `@3D3C42E5AAC5BA805825DA76410C181273BA90B1 # v7.0.1` | 1 | yes |
| sha without a comment is rejected | `@3d3c42e5aac5ba805825da76410c181273ba90b1` | 1 | yes |
| sha only in the comment is rejected | `@v7 # 3d3c42e5aac5ba805825da76410c181273ba90b1` | 1 | yes |
| docker reference is rejected | `      - uses: docker://alpine:3.20` | 1 | yes |
| every violation in a file is named | two floating lines in one file | 1 | both lines, each once |
| empty directory is a broken scan | directory with no `*.yml` | non-zero | `out` contains `found nothing to scan` |
| missing directory is a broken scan | `"$tmp/absent"` | non-zero | `out` contains `found nothing to scan` |
| a file with no uses lines is a broken scan | `w.yml` holding only `name: x` | non-zero | `out` contains `found nothing to scan` |
| success reports the count | two accepted lines | 0 | `out` is `check-action-pins: 2 uses checked` |

- [ ] **Step 2: Run it to verify it fails**

Run: `sh scripts/tests/check-action-pins.sh`
Expected: every case prints `FAIL -` (the checker does not exist, `sh` exits 127), final exit 1.

- [ ] **Step 3: Write `scripts/check-action-pins.sh`**

POSIX `sh`, `set -eu`, shellcheck-clean under `-s sh`. The header comment states the rule (spec §3.1) and why the SHA must be the ref, not the comment. Algorithm, which the tests pin but the repository's scan-guard idiom determines:

```sh
repo_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
. "$repo_root/scripts/lib/scan-guard.sh"
dir=${1:-$repo_root/.github/workflows}
pin='^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(/[^@ ]+)?@[0-9a-f]{40} # [A-Za-z0-9_.-]+$'
files=$(mktemp); trap 'rm -f "$files"' EXIT INT TERM
scan_expand "$files" must-find "workflow files under $dir" "$dir"/*.yml
# For each file: scan_grep -n -E '^[[:space:]]*(- )?uses:' "$file" > "$hits"  → lines "N:text". Read $hits with
# `while IFS= read -r hit; do …; done < "$hits"` — never through a pipe, which runs the loop in a subshell and
# loses the counters (POSIX sh).
# value = text with the leading `[[:space:]]*(- )?uses:[[:space:]]*` and trailing whitespace removed.
# count every hit; skip a value starting with `./`; otherwise test the value against $pin with
# `printf '%s\n' "$value" | grep -qE "$pin"`; on failure print "$file:$N: $text" (text trimmed of leading
# whitespace) and set violations=1.
# scan_require_count "$checked" "uses: lines under $dir"   # the "nothing to scan" refusal
# exit 1 if violations; else printf 'check-action-pins: %s uses checked\n' "$checked".
```

The grep anchor already excludes comment lines (they start with `#`). File paths in violation lines are printed as given (the argument-relative path), which is what the test greps for.

- [ ] **Step 4: Run the test and shellcheck**

Run: `sh scripts/tests/check-action-pins.sh && shellcheck -s sh scripts/check-action-pins.sh scripts/tests/check-action-pins.sh`
Expected: every case `ok   -`, exit 0; shellcheck silent.

- [ ] **Step 5: Run the checker on the real tree**

Run: `sh scripts/check-action-pins.sh; echo "exit=$?"`
Expected: exit 1 and exactly 28 violation lines (the floating references at `c0d5caf`, spec §3.2): `ci.yml` ×22 (checkout 11, rust-toolchain 5, rust-cache 4, install-action 1, cargo-deny-action 1), `docs.yml` ×5, `dependabot-automerge.yml` ×1. Measured on 2026-10-21: 45 `uses:` lines in all, 28 floating. A different count is `STATE_MISMATCH`.

- [ ] **Step 6: Wire `just action-pins` and the tools doc**

In `justfile`, after the `line-citations` recipe:

```
# Fails when a `uses:` in .github/workflows/ is not pinned to a 40-hex commit SHA with a `# <ref>` comment,
# or when the SHA is only in the comment. Both the check and its own tests.
action-pins:
    sh scripts/check-action-pins.sh
    sh scripts/tests/check-action-pins.sh
```

and change the `check` line to `check: fmt-check clippy typos shellcheck line-citations action-pins test probe-tests tools-doc-check doc-check`, updating the comment above it to list `action-pins`. In `.claude/recommended-tools.json`, the `just` entry's `why` gains `+ action-pins` after `line-citations`. Then `sh scripts/gen-tool-table.sh` regenerates `docs/dev-tooling.md`; `sh scripts/gen-tool-table.sh --check` must pass.

- [ ] **Step 7: Commit (the gate is red by design until Task 2 — commit only the script, its test, the justfile and the tools files; run `just fmt-check typos shellcheck line-citations tools-doc-check` and the two new scripts instead of the full `check`)**

```bash
git add scripts/check-action-pins.sh scripts/tests/check-action-pins.sh justfile .claude/recommended-tools.json docs/dev-tooling.md
git commit -m "ci: add the action-pin check and its tests"
```

Note in the commit body that `just action-pins` fails on this commit's own workflows until the next commit pins them.

---

### Task 2: Pin every action and add the `Action pins` job

**Files:**
- Modify: `.github/workflows/ci.yml`, `.github/workflows/docs.yml`, `.github/workflows/dependabot-automerge.yml`, `.github/workflows/release-plz.yml`, `.github/workflows/release.yml`

**Interfaces:**
- Consumes: `sh scripts/check-action-pins.sh` (Task 1) as the oracle: it exits 0 when this task is done.

- [ ] **Step 1: Re-resolve the SHAs and compare with this table**

Run the resolver (`gh api repos/<o>/<r>/releases/latest --jq .tag_name`, then `git/ref/tags/<tag>`, dereferencing `object.type == "tag"` through `git/tags/<sha>`; for `dtolnay/rust-toolchain` `git/ref/heads/stable`). Expected, resolved 2026-10-21:

| action | pinned value |
|---|---|
| actions/checkout | `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1` |
| dtolnay/rust-toolchain | `dtolnay/rust-toolchain@b725eda525362e7cf34af9a7243cac1e3f01a7ec # stable` |
| Swatinem/rust-cache | `Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2.9.3` |
| taiki-e/install-action | `taiki-e/install-action@8cca0918f8471c33515202b73f56ab27f2bed283 # v2.87.29` |
| EmbarkStudios/cargo-deny-action | `EmbarkStudios/cargo-deny-action@3c6349835b2b7b196a839186cb8b78e02f7b5f25 # v2.1.1` |
| actions/upload-pages-artifact | `actions/upload-pages-artifact@fc324d3547104276b827a68afc52ff2a11cc49c9 # v5.0.0` |
| actions/deploy-pages | `actions/deploy-pages@368f82528645a54fb793d4d04e342629a3f51346 # v5.0.1` |
| dependabot/fetch-metadata | `dependabot/fetch-metadata@25dd0e34f4fe68f24cc83900b1fe3fe149efef98 # v3.1.0` |

If a newer release exists at execution time, use it and record the new value in the commit body; if a resolved SHA differs for the SAME tag, stop and report `STATE_MISMATCH`.

Already at the newest SHA and left unchanged: `crate-ci/typos` (v1.51.1), `amannn/action-semantic-pull-request` (v6.1.1), `actions/upload-artifact` (v7.0.2), `actions/checkout` in release-plz.yml, release.yml and sandbox.yml (already v7.0.1). **Also left unchanged, my call:** `release-plz/action@…# v0.5.140` — Dependabot PR #45 (0.5.140 → 0.5.142) is open with auto-merge enabled, and that action appears in only one workflow, so the "same SHA everywhere" rule does not reach it.

- [ ] **Step 2: Replace every floating reference**

Edit by hand, no `sed -i` on the whole directory. In `ci.yml`: 11 × checkout, 5 × rust-toolchain, 4 × rust-cache, 1 × install-action, 1 × cargo-deny-action. In `docs.yml`: checkout, rust-toolchain, rust-cache, upload-pages-artifact, deploy-pages. In `dependabot-automerge.yml`: fetch-metadata. In `release-plz.yml` (×2) and `release.yml` (×1): rust-toolchain `6bed0761d98439e5a578e2877258200ad565ba87` → the `b725eda…` value, so the same action has the same SHA in every workflow.

Inputs that a ref used to supply implicitly become explicit (spec §3.2):
- every `dtolnay/rust-toolchain` use gets `with:` / `toolchain: stable` (merging with an existing `with: components: …` block; the `evidence` job's use keeps its `if:`);
- `taiki-e/install-action` gets `with:` / `tool: nextest`.

- [ ] **Step 3: Add the `Action pins` job to `ci.yml`**

After the `tools-doc` job:

```yaml
  action-pins:
    # Every `uses:` must be a 40-hex commit SHA with a `# <ref>` comment, so a tag that is moved or a
    # branch that is pushed cannot change what this repository runs. The script and its own tests, the
    # same two commands `just action-pins` runs.
    name: Action pins
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
      - run: sh scripts/check-action-pins.sh
      - run: sh scripts/tests/check-action-pins.sh
```

- [ ] **Step 4: Verify**

Run: `sh scripts/check-action-pins.sh && grep -nE '@(v[0-9]+|stable|nextest)$' .github/workflows/*.yml; just check`
Expected: `check-action-pins: 46 uses checked` (the 45 lines measured in Task 1 plus the new job's checkout; re-count with `grep -cE '^[[:space:]]*(- )?uses:' .github/workflows/*.yml` and report a different total as `STATE_MISMATCH`), no grep hits, `just check` green.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/ci.yml .github/workflows/docs.yml .github/workflows/dependabot-automerge.yml .github/workflows/release-plz.yml .github/workflows/release.yml
git commit -m "ci: pin every action to a commit SHA and gate it"
```

---

### Task 3: Clippy on three operating systems

**Files:**
- Modify: `.github/workflows/ci.yml` (the `clippy` job)
- Modify: any `crates/agent-profile/src/**` or `tests/**` file the Windows-target lint flags (expected: none or few; report each)

- [ ] **Step 1: Lint the Windows cfg code locally**

Run: `cargo clippy --workspace --all-targets --target x86_64-pc-windows-gnu -- -D warnings`
Expected: green. If it reports warnings, fix each minimally (the warning names the line) and re-run until green; list the fixes in the commit body. macOS has no local oracle; CI is (spec §4).

- [ ] **Step 2: Replace the `clippy` job**

```yaml
  clippy-matrix:
    # A warning that exists only under `cfg(windows)` or `cfg(target_os = "macos")` compiled cleanly on the
    # Linux-only job this replaces and was never linted (ROADMAP "Clippy runs on Linux only", closed here).
    name: Clippy (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
      - uses: dtolnay/rust-toolchain@b725eda525362e7cf34af9a7243cac1e3f01a7ec # stable
        with:
          toolchain: stable
          components: clippy
      - uses: Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2.9.3
      - run: cargo clippy --workspace --all-targets -- -D warnings

  clippy:
    # Branch protection requires a check named `Clippy`. The matrix legs are named `Clippy (<os>)`, so this
    # job keeps the required name and fails unless every leg succeeded. `!cancelled()` runs it after a
    # failed or timed-out leg (and fails it) but not after the run was cancelled, so a cancelled run does
    # not show a red Clippy. The result is read through `env`, never interpolated into the script.
    name: Clippy
    needs: [clippy-matrix]
    if: ${{ !cancelled() }}
    runs-on: ubuntu-latest
    steps:
      - name: Require every Clippy leg to succeed
        env:
          RESULT: ${{ needs.clippy-matrix.result }}
        run: |
          if [ "$RESULT" != "success" ]; then
            echo "::error::Clippy matrix result is '$RESULT'; see the 'Clippy (<os>)' jobs"
            exit 1
          fi
          echo "every Clippy leg succeeded"
```

Use the SHAs Task 2 settled on if they differ from the above.

- [ ] **Step 3: Verify and commit**

Run: `sh scripts/check-action-pins.sh && just check`
Expected: `check-action-pins: 46 uses checked` (unchanged: the three `uses:` moved from `clippy` to `clippy-matrix`, and the aggregator has none), gate green.

```bash
git add .github/workflows/ci.yml  # plus any lint fixes
git commit -m "ci: run clippy on ubuntu, macos and windows behind the Clippy check"
```

---

### Task 4: The grammar table (behaviour-preserving refactor)

**Files:**
- Modify: `crates/agent-profile/src/cli.rs` — `impl CommandWord` (after `fn usage`), `split_top_level`, `validate_command`, and `mod tests`
- Test: `crates/agent-profile/src/cli.rs` `mod tests`

**Interfaces:**
- Produces (all private to `cli`, reachable from a child module via `super::`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentScope { TopLevelOnly, AgentRequired, Either }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Positional { None, RequiredProfile, OptionalProfile }   // Task 6 adds `Shell`

impl CommandWord {
    const ALL: [CommandWord; 12];                 // Task 6 makes it 13
    fn scope(self) -> AgentScope;
    fn accepts_repo(self) -> bool;
    fn accepts_json(self) -> bool;
    fn accepts_yes(self) -> bool;
    fn positional(self, with_agent: bool) -> Positional;
}
```

- [ ] **Step 1: Write the two failing tests in `mod tests`**

A hand-written table, in full (this is the oracle for both tests; it is NOT derived from the methods):

```rust
struct Row {
    word: CommandWord, scope: AgentScope, repo: bool, json: bool, yes: bool,
    top_level: Positional, with_agent: Positional,
}
const TABLE: &[Row] = &[
    // word          scope          repo   json   yes    top_level                  with_agent
    Row { word: CommandWord::Current,      scope: AgentScope::AgentRequired, repo: true,  json: false, yes: false, top_level: Positional::None, with_agent: Positional::None },
    Row { word: CommandWord::Resolve,      scope: AgentScope::AgentRequired, repo: true,  json: true,  yes: false, top_level: Positional::None, with_agent: Positional::None },
    Row { word: CommandWord::Status,       scope: AgentScope::Either,        repo: true,  json: true,  yes: false, top_level: Positional::None, with_agent: Positional::None },
    Row { word: CommandWord::Link,         scope: AgentScope::Either,        repo: true,  json: false, yes: false, top_level: Positional::RequiredProfile, with_agent: Positional::RequiredProfile },
    Row { word: CommandWord::Unlink,       scope: AgentScope::Either,        repo: true,  json: false, yes: false, top_level: Positional::None, with_agent: Positional::None },
    Row { word: CommandWord::Create,       scope: AgentScope::AgentRequired, repo: false, json: false, yes: false, top_level: Positional::RequiredProfile, with_agent: Positional::RequiredProfile },
    Row { word: CommandWord::Delete,       scope: AgentScope::AgentRequired, repo: false, json: false, yes: true,  top_level: Positional::RequiredProfile, with_agent: Positional::RequiredProfile },
    Row { word: CommandWord::List,         scope: AgentScope::Either,        repo: false, json: true,  yes: false, top_level: Positional::None, with_agent: Positional::None },
    Row { word: CommandWord::Agents,       scope: AgentScope::TopLevelOnly,  repo: false, json: true,  yes: false, top_level: Positional::None, with_agent: Positional::None },
    Row { word: CommandWord::Profiles,     scope: AgentScope::TopLevelOnly,  repo: false, json: true,  yes: false, top_level: Positional::None, with_agent: Positional::None },
    Row { word: CommandWord::Repositories, scope: AgentScope::TopLevelOnly,  repo: false, json: true,  yes: false, top_level: Positional::None, with_agent: Positional::None },
    Row { word: CommandWord::Doctor,       scope: AgentScope::Either,        repo: true,  json: true,  yes: false, top_level: Positional::None, with_agent: Positional::OptionalProfile },
];
```

(`create`/`delete` have `json: false` because `--json` there answers `NotYetImplemented`, spec §2.3.)

`grammar_table_is_pinned`: asserts `CommandWord::ALL.len() == TABLE.len()`, that every `TABLE` word is in `ALL` and vice versa, and for each row that `scope()`, `accepts_repo()`, `accepts_json()`, `accepts_yes()`, `positional(false)`, `positional(true)` equal the row.

`dispatcher_follows_the_grammar_table`: for each row, drive `top(..)` and `split(["fake", ..], KNOWN)` and compare with the ROW, never with the methods. Let `prof` be `&["p"]` when the relevant positional is `RequiredProfile`, else `&[]`. Assert:
- top level, bare: `AgentRequired` → `Err(Usage)` whose message starts with `` `{word}` needs an agent``; otherwise `Ok(Invocation::Command { agent: None, profile: prof.first(), .. })`.
- with agent, bare: `TopLevelOnly` → `Err(Usage)` with message `` `{word}` takes no agent: agent-profile {word}``; otherwise `Ok(Invocation::Command { agent: Some("fake"), .. })`.
- flags, on whichever form is valid (top level for `TopLevelOnly`, `fake` for the others), with `prof` appended: `--repo x` → `repo: Some("x")` if `row.repo`, else `Err(Usage)` `` unknown option "--repo" for `{word}` ``; `--json` → `json: true` if `row.json`, else for `create`/`delete` `Err(NotYetImplemented { command: "--json" })`, else `Err(Usage)` `` unknown option "--json" for `{word}` ``; `--yes` → `yes: true` if `row.yes`, else `Err(Usage)` `` unknown option "--yes" for `{word}` ``.
- positional arity: for `RequiredProfile`, bare (no word) → `Err(Usage)` message starts with `` `{word}` needs a profile``, and two words → `` `{word}` takes one profile``; for `None`, one word → `` `{word}` takes no arguments``; for `OptionalProfile` (`fake doctor`), one word → `Ok` with `profile: Some("p")`, two → `` `doctor` takes one profile``.

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo nextest run -p agent-profile grammar_table dispatcher_follows`
Expected: compile error, `AgentScope`/`Positional`/`scope` not found.

- [ ] **Step 3: Implement the methods and rewrite the two functions**

Add the enums and methods after `fn usage` in `impl CommandWord`; every method is a `match self` with no wildcard, so adding a variant is a compile error until each table is extended. `ALL` lists the 12 variants in declaration order.

`split_top_level`: replace the two `matches!` blocks with one: if `command.scope() == AgentScope::AgentRequired`, the message is `` `{name}` needs an agent: agent-profile <agent> {name}`` plus `" <profile>"` when `command.positional(true) == Positional::RequiredProfile`.

`validate_command`, keeping the ORDER of checks exactly as today: `has_cut` → `scope() == TopLevelOnly && agent.is_some()` → repo refused when `!accepts_repo()` → repo count/empty → the literal `Create | Delete` `--json` `NotYetImplemented` check (kept) → `json_accepted = command.accepts_json()` → `yes = command.accepts_yes() && …` → unknown option → `allowed = usize::from(command.positional(agent.is_some()) != Positional::None)` → the `link` needs-a-profile check (kept literal) → the too-many check, whose message is chosen by the positional kind: `RequiredProfile` → `` `{name}` takes one profile``, `OptionalProfile` → `` `{name}` takes one profile``, `None` → `` `{name}` takes no arguments`` → the `create`/`delete` needs-a-profile check (kept literal) → profile → `Ok`.

- [ ] **Step 4: Run the whole suite**

Run: `cargo nextest run --workspace --no-tests=pass && cargo test --doc --workspace`
Expected: all pass; the count is the pre-task count (398 at `c0d5caf`) + 2. No existing test is edited.

- [ ] **Step 5: `just check` and commit**

```bash
git add crates/agent-profile/src/cli.rs
git commit -m "refactor(cli): describe the command grammar in one table"
```

---

### Task 5: The completion model and script (`cli/completions.rs`)

**Files:**
- Modify: `Cargo.toml` (`clap_complete = "4.6"` under `[workspace.dependencies]`, after `clap`)
- Modify: `crates/agent-profile/Cargo.toml` (`clap_complete = { workspace = true }`)
- Modify: `Cargo.lock` (gains `clap_complete 4.6.11` only)
- Create: `crates/agent-profile/src/cli/completions.rs`
- Modify: `crates/agent-profile/src/cli.rs` (`mod completions;` after the `use` block; `Positional::Shell` variant; in Task 4's `positional` matches nothing returns it yet — the variant is constructed only by Task 6, so mark it `#[allow(dead_code)]` here and remove the attribute in Task 6)

**Interfaces:**
- Consumes: `CommandWord::ALL`, `scope()`, `accepts_repo()`, `accepts_json()`, `accepts_yes()`, `positional()` (Task 4); `adapter::known_agents()`.
- Produces:

```rust
pub(super) const SHELL_NAMES: [&str; 5] = ["bash", "elvish", "fish", "powershell", "zsh"];
pub(super) fn parse_shell(word: &str) -> Option<clap_complete::Shell>;   // exact lower-case only
pub(super) fn model() -> clap::Command;                                   // not built; callers build
pub(super) fn script(shell: clap_complete::Shell) -> String;              // the text to print
```

- [ ] **Step 1: Add the dependency and check the lockfile**

Run: `cargo build --workspace && git diff --stat Cargo.lock && grep -A1 'name = "clap_complete"' Cargo.lock && cargo deny check`
Expected: the lock diff adds one package block, `clap_complete` `4.6.11`, and changes nothing else; `clap` remains `4.6.7`; deny green.

- [ ] **Step 2: Write the failing unit tests in `cli/completions.rs` `mod tests`**

Build the model once with `let mut root = model(); root.build();` so clap's implicit additions are visible (spec §2.5 test 5). Set equality everywhere via `BTreeSet<String>`.

- `root_subcommands_are_the_top_level_words_and_the_agents`: `root.get_subcommands()` names == every `CommandWord::ALL` word whose `scope() != AgentRequired` (as strings) ∪ `known_agents()`. (After Task 6 this includes `completions`.)
- `root_arguments_are_help_and_version`: root argument ids == `{"help", "version"}`.
- `each_agent_offers_the_agent_words_and_the_launch_flags`: for every agent subcommand, subcommand names == words whose `scope() != TopLevelOnly`; argument ids == `{"dry-run", "verbose", "help"}` (no `profile`).
- `each_command_mirrors_the_grammar_table`: for every (parent, word) pair present in the model, argument ids == `{"help"}` ∪ `{"repo"}` if `accepts_repo()` ∪ `{"json"}` if `accepts_json()` ∪ `{"yes"}` if `accepts_yes()` ∪ `{"profile"}` if `positional(with_agent)` is `RequiredProfile`/`OptionalProfile` ∪ `{"shell"}` if `Shell`; and its subcommands are empty. Also assert the `repo` arg's `get_value_hint() == ValueHint::DirPath`.
- `no_help_subcommand_anywhere`: walk the tree; no subcommand is named `help`.
- `script_for_each_shell_names_every_agent_and_word`: for each `Shell` in `[Bash, Elvish, Fish, PowerShell, Zsh]`, `script(shell)` is non-empty and contains every `known_agents()` id and every `ALL` word.
- `parse_shell_is_exact_lower_case`: each of `SHELL_NAMES` parses to its variant; `"Bash"`, `"BASH"`, `""`, `"nope"` → `None`.
- `bash_script_has_no_mismatched_state_label`: `!script(Shell::Bash).contains("agent__subcmd__profile")` and it contains `agent__profile__subcmd__claude)` (the corrected `case` label).
- `bash_script_completes_the_second_and_third_word` (`#[cfg(target_os = "linux")]`): write `script(Shell::Bash)` to a `tempfile::NamedTempFile`; a helper `fn complete(script: &Path, words: &[&str]) -> BTreeSet<String>` runs

  ```
  bash --norc --noprofile -c 'source "$1"; shift; COMP_WORDS=("$@"); COMP_CWORD=$((${#COMP_WORDS[@]} - 1)); COMP_LINE="$*"; COMP_POINT=${#COMP_LINE}; _agent__profile agent-profile "${COMP_WORDS[COMP_CWORD]}" "${COMP_WORDS[COMP_CWORD - 1]}"; printf "%s\n" "${COMPREPLY[@]}"' bash <script> <words...>
  ```

  with `.output().unwrap()` (so a missing `bash` FAILS the test) and splits stdout into lines. Assert: `complete(["agent-profile", ""])` ⊇ `{"claude", "agents"}`; `complete(["agent-profile", "claude", ""])` ⊇ `{"create", "--dry-run"}` and ∌ `"agents"`; `complete(["agent-profile", "claude", "create", ""])` ∋ `"--help"` and ∌ `"--json"`, ∌ `"--repo"`.

- [ ] **Step 3: Run them to verify they fail**

Run: `cargo nextest run -p agent-profile completions::`
Expected: compile error (module and functions missing).

- [ ] **Step 4: Implement `cli/completions.rs`**

Module doc: the model exists only for `clap_complete::generate`, never parses, and mirrors the grammar table; the bash workaround and its upstream cause (clap_complete 4.6.11 `aot/shells/bash.rs`: `fn_name` from `bin_name.replace('-', "__")` versus `case` labels from `replace('-', CMD_SEP)`) with "remove once upstream names them consistently".

- `model()`: `Command::new("agent-profile").version(env!("CARGO_PKG_VERSION")).disable_help_subcommand(true)`, then `.subcommand(command_model(word, false))` for each `ALL` word with `scope() != AgentRequired`, then for each `known_agents()` id `.subcommand(Command::new(id).disable_help_subcommand(true).arg(flag("dry-run")).arg(flag("verbose")).subcommands(words with scope() != TopLevelOnly, built with with_agent = true))`.
- `fn command_model(word: CommandWord, with_agent: bool) -> Command`: `Command::new(word.as_str()).disable_help_subcommand(true)`; `--repo` as `Arg::new("repo").long("repo").value_name("path").value_hint(ValueHint::DirPath)` if `accepts_repo()`; `flag("json")`, `flag("yes")` by the table; the positional by `positional(with_agent)`: `RequiredProfile`/`OptionalProfile` → `Arg::new("profile").value_name("profile")` (required only for `RequiredProfile`), `Shell` → `Arg::new("shell").value_name("shell").required(true).value_parser(SHELL_NAMES)`, `None` → nothing.
- `fn flag(name: &'static str) -> Arg`: `Arg::new(name).long(name).action(ArgAction::SetTrue)`.
- `script(shell)`: `let mut model = model(); let mut buffer = Vec::new(); clap_complete::generate(shell, &mut model, "agent-profile", &mut buffer); let text = String::from_utf8_lossy(&buffer).into_owned(); if shell == Shell::Bash { text.replace("agent__subcmd__profile", "agent__profile") } else { text }`.
- `parse_shell`: an explicit `match` over the five lower-case names.

- [ ] **Step 5: Run the tests**

Run: `cargo nextest run -p agent-profile completions::`
Expected: all 9 pass (8 on non-Linux).

- [ ] **Step 6: `just check` and commit**

```bash
git add Cargo.toml Cargo.lock crates/agent-profile/Cargo.toml crates/agent-profile/src/cli.rs crates/agent-profile/src/cli/completions.rs
git commit -m "feat(cli): build the shell completion model from the command grammar"
```

---

### Task 6: The `completions` command

**Files:**
- Modify: `crates/agent-profile/src/cli.rs` — `COMMANDS_HELP`, `enum Command` + `struct Rest` + `impl Command` (removed), `enum CommandWord` (+ `Completions`), `parse`/`as_str`/`usage`/`scope`/`accepts_*`/`positional`/`ALL`, `run`, `enum Invocation` (+ `Completions(clap_complete::Shell)`), `validate_command`, `execute`, `mod tests`
- Modify: `crates/agent-profile/tests/launch.rs` (`behaviour_table_rows_with_non_zero_exits`, the two `completions` rows)
- Create: `crates/agent-profile/tests/completions.rs`

**Interfaces:**
- Consumes: `completions::{SHELL_NAMES, parse_shell, script}` (Task 5), `write_out`.
- Produces: the §2.1 contract; `CommandWord::Completions` with `scope() == TopLevelOnly`, `accepts_repo/json/yes == false`, `positional(_) == Positional::Shell`; `Invocation::Completions(Shell)`.

- [ ] **Step 1: Write the failing integration tests `tests/completions.rs`**

`mod support;` + the `stderr`/`stdout` helpers as in `tests/launch.rs`; `Root::new()` for a configured root. Command words pinned as a literal list: `["current","resolve","status","link","unlink","create","delete","list","agents","profiles","repositories","doctor","completions"]`.

- `contract_rows_exit_two_with_exact_messages`: for each `(args, message)` assert exit 2, empty stdout, and `stderr == format!("agent-profile: error: {message}\n")`:
  - `["completions"]` → `` `completions` needs a shell: agent-profile completions <bash|elvish|fish|powershell|zsh> ``
  - `["completions", "nope"]` → `` unknown shell `nope` (shells: `bash`, `elvish`, `fish`, `powershell`, `zsh`) ``
  - `["completions", "Bash"]` → `` unknown shell `Bash` (shells: `bash`, `elvish`, `fish`, `powershell`, `zsh`) ``
  - `["completions", "bash", "zsh"]` → `` `completions` takes one shell ``
  - `["completions", "bash", "--json"]` → `` unknown option "--json" for `completions` ``
  - `["completions", "bash", "--repo=x"]` → `` unknown option "--repo" for `completions` ``
- `existing_messages_match_agents_word_for_word`: for each pair, run both and assert `stderr(completions) == stderr(agents).replace("agents", "completions")` and both exit 2: `(["completions","bash","--repo","x"], ["agents","--repo","x"])`, `(["completions","bash","--","x"], ["agents","--","x"])`, `(["completions","bash","--bogus"], ["agents","--bogus"])`, `(["fake","completions"], ["fake","agents"])`, `(["fake","completions","bash"], ["fake","agents"])`.
- `help_wins_before_any_arity_check`: `["completions","--help"]`, `["completions","-h"]`, `["completions","bash","extra","--help"]`, `["fake","completions","--help"]` all exit 0, empty stderr, stdout == `"Usage: agent-profile completions <shell>\nPrint a shell completion script to stdout. <shell> is bash, elvish, fish, powershell or zsh.\n"`.
- `each_shell_prints_a_script`: for each of the five names, exit 0, empty stderr, stdout non-empty and containing every id from `agent_profile::adapter::known_agents()` and every word of the pinned list.
- `completions_needs_no_root`: `root.agent_profile(["completions","zsh"]).env("AGENT_PROFILE_HOME", "relative")` exits 0; the same with `AGENT_PROFILE_HOME` = `root.path().join("absent")` exits 0 and that directory still does not exist; with `Root::empty()` (no config) `completions fish` exits 0 and `read_dir(root.path())` is still empty.
- `a_failed_stdout_write_exits_one` (`#[cfg(target_os = "linux")]`): `Command::new("sh").args(["-c", "\"$0\" completions bash > /dev/full", env!("CARGO_BIN_EXE_agent-profile")])` with the same env scrubbing as `Root::agent_profile` (reuse `root.agent_profile` for the env, but run through `sh`: build the command manually with `WRAPPER_VARS`/`FIXTURE_VARS` removed and `AGENT_PROFILE_HOME` set) → exit 1 and stderr contains `could not write to stdout`.

Rewrite the two rows in `tests/launch.rs`:
`(&["completions"], 2, "`completions` needs a shell")` and `(&["fake", "completions"], 2, "`completions` takes no agent: agent-profile completions")`.

Extend the Task 4 tests: add the row `Row { word: CommandWord::Completions, scope: AgentScope::TopLevelOnly, repo: false, json: false, yes: false, top_level: Positional::Shell, with_agent: Positional::Shell }` and, in `dispatcher_follows_the_grammar_table`, treat `Positional::Shell` like `RequiredProfile` with the word `bash` instead of `p`, expecting `Ok(Invocation::Completions(Shell::Bash))` and the messages `` `completions` needs a shell: … `` / `` `completions` takes one shell``. Add a unit test `completions_binds_the_shell_only_in_lower_case`: `top(["completions","zsh"])` → `Ok(Invocation::Completions(Shell::Zsh))`; `top(["completions","ZSH"])` → the unknown-shell usage message; `top(["completions", <non_utf8>])` → an `Error::Usage` whose message starts with `` unknown shell ` `` (lossy text, no panic).

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo nextest run -p agent-profile --test completions --test launch completions`
Expected: the new file fails to find the behaviour (exit 2 "not yet implemented" instead of the contract); the unit tests fail to compile (`Completions` missing).

- [ ] **Step 3: Implement in `cli.rs`**

1. `COMMANDS_HELP`: append `"\n\nShell completion:\n  agent-profile completions <shell>"` after the `profiles` line.
2. Delete `Command::Completions(Rest)`, `struct Rest`, `impl Command { fn reserved_name }`, the `Args` import if now unused, and in `run` the `if let Some(name) = cli.command.reserved_name()` block; the `let Command::Agent(argv) = cli.command else { unreachable!(..) }` becomes the irrefutable `let Command::Agent(argv) = cli.command;`. Update the doc comment on `enum Command` (every reserved word is now dispatched before clap). A `Subcommand` enum whose only variant is the `external_subcommand` compiles and parses (measured in a throwaway build on 2026-10-21: `completions bash` arrives as `Agent(["completions","bash"])`, bare invocation still prints help and exits 2).
3. `CommandWord::Completions`: `parse` `"completions"`, `as_str`, `usage` returning exactly `"Usage: agent-profile completions <shell>\nPrint a shell completion script to stdout. <shell> is bash, elvish, fish, powershell or zsh.\n"`, `scope` `TopLevelOnly`, `accepts_*` false, `positional` `Positional::Shell` for both, `ALL` now 13 (remove Task 5's `#[allow(dead_code)]`).
4. `Invocation::Completions(clap_complete::Shell)` (derive `PartialEq` holds: `Shell` is `PartialEq`).
5. `validate_command`: in the too-many check, `Positional::Shell` → `` `{name}` takes one shell``. After the `create`/`delete` needs-a-profile check and before building `profile`: `if command.positional(agent.is_some()) == Positional::Shell { let Some(word) = bare.first() else { return Err(usage(format!("`{name}` needs a shell: agent-profile {name} <{}>", SHELL_NAMES.join("|")))) }; let word = word.to_string_lossy(); return completions::parse_shell(&word).map(Invocation::Completions).ok_or_else(|| usage(format!("unknown shell `{word}` (shells: {})", SHELL_NAMES.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join(", ")))); }`.
6. `execute`: `Invocation::Completions(shell) => { write_out(&completions::script(shell))?; Ok(0) }`.

`run` needs no change: `CommandWord::parse("completions")` now succeeds at `args.get(1)`, so `completions …` goes through `split_top_level`; `<agent> completions` reaches `validate_command` through `split` step 4 and is refused by the scope check.

- [ ] **Step 4: Run everything**

Run: `cargo nextest run --workspace --no-tests=pass && cargo test --doc --workspace`
Expected: all pass (400 + 1 unit + 9 integration on Linux). `tests/launch.rs` and `tests/resolution.rs` `--json` rows unchanged and green.

- [ ] **Step 5: Try it by hand (measurement, not a test)**

Run: `cargo run -q -- completions zsh | head -3; cargo run -q -- completions bash | grep -c agent__subcmd__profile; cargo run -q -- completions bash extra; echo "exit=$?"`
Expected: a `#compdef agent-profile` header; `0`; `` agent-profile: error: `completions` takes one shell ``, exit 2.

- [ ] **Step 6: `just check` and commit**

```bash
git add crates/agent-profile/src/cli.rs crates/agent-profile/tests/launch.rs crates/agent-profile/tests/completions.rs
git commit -m "feat(cli): add the completions command"
```

---

### Task 7: README, ROADMAP, TODO and CONTRIBUTING

**Files:**
- Modify: `README.md`, `ROADMAP.md`, `TODO.md`, `CONTRIBUTING.md` (one comment line, my call)
- Modify: `crates/agent-profile/src/cli.rs` `mod tests` (the README-pinning test)

- [ ] **Step 1: Write the failing test `readme_usage_block_carries_every_command_help_line`**

In `cli.rs` `mod tests`: `let readme = include_str!("../../../README.md");` take the text after `"## Usage"`, then the first fenced block (between the first ```` ```text ```` and the next ```` ``` ````); for every line of `COMMANDS_HELP` that starts with `"  agent-profile "`, assert the block contains that line trimmed of leading whitespace, as a whole line (`block.lines().any(|l| l == line.trim_start())`), with the missing line in the panic message.

Run: `cargo nextest run -p agent-profile readme_usage_block`
Expected: FAIL naming `agent-profile <agent> resolve [--repo <path>] [--json]` (the README has the combined `resolve|status` line).

- [ ] **Step 2: Edit `README.md`** (current line numbers at `1b2ce4a`)

- Lines 11–14, the status paragraph, become: `**Status: v0.1 command set complete.** Every v0.1 command is implemented — launch, repository resolution (`current`, `resolve`, `status`, `link`, `unlink`, `repositories`), the profile lifecycle (`create`, `list`, `delete`, `profiles`, `agents`), `doctor`, `--json` on the report commands and `completions` — backed by twelve evidence-measured adapters; see [ROADMAP.md](ROADMAP.md) for what remains before a release. The authoritative design is …` (keep the spec link sentence).
- The usage block (lines 40–54): replace line 44 with the two lines `agent-profile <agent> resolve [--repo <path>] [--json]` and `agent-profile [<agent>] status [--repo <path>] [--json]`; delete line 47 (`agent-profile status …`, now covered); after the `repositories` line add `agent-profile doctor [--repo <path>] [--json]`, `agent-profile <agent> doctor [<profile>] [--repo <path>] [--json]` and `agent-profile completions <shell>`. Keep the two launch lines.
- After the `repositories` paragraph (ends line 84), a new paragraph: `doctor` checks the host (platform, application root and whether it is writable, `config.toml`, the profiles directory and case-twin names, the default profile), the current repository and every stored mapping, and each agent's executable and support level; `<agent> doctor [<profile>]` adds the profile that would resolve, its paths and the launch environment the adapter would set, by name only. It never launches an agent, and prints no secret or environment value. Exit codes: 0 when nothing failed, 5 when any check failed, 2 for a usage error and 1 when the report cannot be written (verified against `run_doctor` and `Error::exit_code` at implementation time).
- Before `## Supported agents` (line 98), a new section:

  ````markdown
  ## Shell completion

  `agent-profile completions <shell>` prints a completion script for `bash`, `elvish`, `fish`, `powershell` or
  `zsh`. It completes agents, commands and their options; profile names are not completed, so the shell falls
  back to its default after the command word.

  ```sh
  # bash — in ~/.bashrc
  source <(agent-profile completions bash)
  # zsh — once, into a directory on $fpath, then `compinit` in ~/.zshrc
  agent-profile completions zsh > ~/.zfunc/_agent-profile
  # fish
  mkdir -p ~/.config/fish/completions && agent-profile completions fish > ~/.config/fish/completions/agent-profile.fish
  # elvish — in ~/.config/elvish/rc.elv
  eval (agent-profile completions elvish | slurp)
  ```

  ```powershell
  # PowerShell — in $PROFILE
  agent-profile completions powershell | Out-String | Invoke-Expression
  ```
  ````

  (zsh: add `fpath=(~/.zfunc $fpath)` before `compinit` if that directory is new — say so in one sentence under the block. The elvish line is checked against clap_complete's elvish generator only; the owner accepted that stand-down.)
- Delete lines 133–139 (`## Planned adapters …` through the italic sentence).
- Line 182: `just check              # the local gate: fmt-check, clippy, typos, shellcheck, line-citations, action-pins, test, probe-tests, tools-doc-check, doc-check`.

- [ ] **Step 3: Run the README test and the shell snippets**

Run: `cargo nextest run -p agent-profile readme_usage_block`
Expected: PASS.

Run each snippet on this host with the built binary (zsh, fish and PowerShell are installed; elvish is not): `bash -c 'source <(target/debug/agent-profile completions bash) && complete -p agent-profile'`; `zsh -c 'fpath=(/tmp/zf $fpath); mkdir -p /tmp/zf; target/debug/agent-profile completions zsh > /tmp/zf/_agent-profile; autoload -Uz compinit; compinit -D; echo ok'`; `fish -c 'target/debug/agent-profile completions fish | source; echo ok'`; `pwsh -NoProfile -c 'target/debug/agent-profile completions powershell | Out-String | Invoke-Expression; "ok"'`.
Expected: each prints without error. Record the elvish line as unrun in the commit body.

- [ ] **Step 4: Edit `ROADMAP.md`, `TODO.md`, `CONTRIBUTING.md`**

`ROADMAP.md`:
- Line 14, the SP5 row's State: `done — SP5a lifecycle, SP5b --json + repositories, SP5c doctor, SP5d completions + docs + CI`.
- Delete the "Two contradictory action-pinning idioms" paragraph (lines 39–44) and the "Clippy runs on Linux only" paragraph (lines 74–80).
- Append three tracked-debt paragraphs in the existing bold-lead style:
  1. **`dtolnay/rust-toolchain` is pinned to a branch tip, and whether Dependabot bumps it is unverified.** `release-plz.yml` sat at `6bed076` from 2026-09-15 while the `stable` branch moved to `b725eda`; SP5d moved every use to the tip with an explicit `toolchain: stable`. If it stays put again, bump it by hand with `gh api repos/dtolnay/rust-toolchain/git/ref/heads/stable`.
  2. **Profile names are not completed.** The completion script is static (owner's choice, SP5d D1); completing profile names needs a runtime hook. A future enhancement outside v0.1.
  3. **clap_complete's bash script breaks for a binary name containing `-`.** Measured at 4.6.11 and on clap's `main` on 2026-10-21: the traversal names states from `bin_name.replace('-', "__")`, the `case` labels from `replace('-', "__subcmd__")`, so no label matches and bash completes only the first word. `cli/completions.rs` rewrites `agent__subcmd__profile` to `agent__profile` in the generated bash; `bash_script_has_no_mismatched_state_label` is the oracle. Remove the workaround once upstream fixes it. Not reported upstream (an outward action the owner has not OK'd).

`TODO.md`: delete the item at lines 138–140 (pin the actions in ci.yml). Nothing else references SP5d.

`CONTRIBUTING.md` line 33: `just check      # the gate: fmt-check, clippy, typos, shellcheck, line-citations, action-pins, test, probe-tests, tools-doc-check, doc-check`.

- [ ] **Step 5: `just check` and commit**

Run: `just check`
Expected: green; `typos` accepts `compinit`, `fpath`, `elvish`, `slurp` (if `typos` flags one, add it to `_typos.toml` `[default.extend-words]` and say so in the commit body).

```bash
git add README.md ROADMAP.md TODO.md CONTRIBUTING.md crates/agent-profile/src/cli.rs
git commit -m "docs: refresh the README for the v0.1 command set and close the CI debt entries"
```

---

## After the last task

Not part of the plan's tasks, required before the PR by the standing rules: AGY-CAPSTONE on the committed range `c0d5caf..HEAD` until GREEN, then AGY-TEST-AUDIT, then the owner's OK to push and open ONE PR (spec §8) whose body mentions that making `Action pins` a required check is the owner's call.

## Self-review

- Spec coverage: §2.1 → Task 6 tests; §2.2 → Task 6 step 3; §2.3 → Task 4; §2.4 → Task 5; §2.5 tests 1–7 → Tasks 5/6 (1: contract rows; 2: launch.rs rows; 3: `completions_needs_no_root`; 4: Task 4's two tests; 5: the four model tests; 6: `each_shell_prints_a_script`; 7: `bash_script_completes_the_second_and_third_word`); §3 → Tasks 1–2; §4 → Task 3; §5–§6 → Task 7; §7 → Task 7; §8 → commit order and "After the last task".
- Review Focus 1–5 each name their test and task.
- Types: `AgentScope`, `Positional`, `CommandWord::ALL`, `SHELL_NAMES`, `parse_shell`, `model`, `script`, `Invocation::Completions(Shell)` are spelled the same in Tasks 4, 5 and 6.
- Calls I made beyond the spec: `release-plz/action` left to Dependabot PR #45; the pin check also fails on an empty scan (repository idiom); the `CONTRIBUTING.md` gate comment fixed with the README's; the test harness file is `scripts/tests/` (new directory, already covered by the `shellcheck` recipe's `find scripts`).
