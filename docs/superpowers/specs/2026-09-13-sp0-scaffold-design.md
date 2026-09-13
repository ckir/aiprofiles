# SP0 — Scaffold design

**Date:** 2026-09-13
**Status:** Approved in brainstorming; awaiting written-spec review
**Oracle:** [`agent-profile-implementation-spec-v3.md`](../../../agent-profile-implementation-spec-v3.md) (repo root)
**Template repo:** `E:\Rust\flux` — development tools, licence and community docs are ported from it.

## 1. Context: the sub-project split

The V3 spec is implemented as six sub-projects. Each gets its own spec → plan → implementation cycle and
ends with a green `just check` on Linux, macOS and Windows. A sub-project's plan is written only after the
previous one is merged.

| SP | Scope | V3 §35 phases |
|---|---|---|
| **SP0** | Scaffold: workspace, tooling, licence, community docs, CI, `fake-agent` fixture | — |
| SP1 | Core + explicit launch: name validation, app root, TOML config (lock + atomic replace), structured errors, `LaunchPlan`, Unix exec / Windows child launcher, passthrough, env overrides, dry-run, exit codes | 1, 3 |
| SP2 | Architecture gate: adapter trait, capability/evidence model, common contract suite, Claude Code, Codex CLI, Aider | 4A |
| SP3 | Repository resolution: discovery, canonical identity, mapping storage, precedence, `resolve`, `current`, `status`, `link`, `unlink` | 2 + part of 5 |
| SP4 | The remaining nine adapters, with evidence re-verified upstream | 4B |
| SP5 | `create`, `list`, `delete`, `repositories`, `doctor`, JSON output, completions, docs | rest of 5, 6 |

**Deviation from §35 order (deliberate, user-approved):** `link`/`unlink` move up from Phase 5 to land with
resolution (SP3). Otherwise resolution would ship with no CLI way to create the mappings it resolves. Every
adapter is still gated behind SP2, so §35's architecture gate holds.

This document covers **SP0 only**.

## 2. Goal

Create a compiling, CI-green workspace that carries flux's development workflow unchanged in spirit, so SP1
can start writing domain code at once. SP0 contains no domain logic.

## 3. Workspace layout

```text
aiprofiles/
├─ Cargo.toml                    virtual workspace (no root package)
├─ agent-profile-implementation-spec-v3.md
├─ crates/
│  └─ agent-profile/             lib + bins `agent-profile` and `fake-agent`
│     ├─ src/lib.rs
│     ├─ src/main.rs             bin `agent-profile` (the product)
│     ├─ src/bin/fake-agent.rs   bin `fake-agent` (test-only fixture)
│     ├─ src/{name,config,repo,resolve,cli,output}.rs
│     ├─ src/adapter/mod.rs
│     ├─ src/launch/mod.rs
│     └─ tests/                  integration + (later) contract suite
│        ├─ support/mod.rs       fake_agent() helper
│        └─ smoke.rs
├─ docs/ …                       see §6
└─ tooling configs               see §5
```

### 3.1 Root `Cargo.toml`

- `[workspace]` with `resolver = "3"` and the single member `crates/agent-profile`.
- `[workspace.package]`: `version = "0.1.0"`, `edition = "2024"`, `rust-version = "1.98"` (owner decision,
  2026-09-13; originally `1.85` as in flux),
  `license = "PolyForm-Noncommercial-1.0.0"`, `repository = "https://github.com/ckir/aiprofiles"`,
  `publish = false`.
- Every member `Cargo.toml` opts in to **each** of those keys: `version.workspace = true`,
  `edition.workspace = true`, `rust-version.workspace = true`, `license.workspace = true`,
  `repository.workspace = true`, `publish.workspace = true`. Cargo does not inherit them otherwise: a
  scratch probe gave `"publish": null, "license": null` in `cargo metadata` for members that did not opt in.
- `[workspace.dependencies]` uses flux's exact pins, limited to crates `agent-profile` will plausibly use:
  - `clap = { version = "4.6", features = ["derive"] }`
  - `serde = { version = "1", features = ["derive"] }`
  - `serde_json = "1"`
  - `toml = "1.1"`
  - `thiserror = "2"`
  - `tempfile = "3"`
  - `proptest = "1.11"`

  SP0 itself uses `clap` and `serde_json`, both as normal dependencies of `agent-profile`. A binary
  target cannot have dependencies of its own, so the `fake-agent` binary's `serde_json` is the package's;
  V3 §32 JSON output needs it in SP5 regardless. Each later sub-project adds its own crates in its own spec.
- `[workspace.metadata.release]` and `[profile.ci]` are copied verbatim from flux.
- `crates/agent-profile/Cargo.toml` sets `default-run = "agent-profile"`, so `cargo run` is unambiguous
  with two binaries.

**Why there is no root package:** the integration tests live in `crates/agent-profile/tests/`, which gives
them `CARGO_BIN_EXE_agent-profile` and `CARGO_BIN_EXE_fake-agent` directly.

### 3.2 `crates/agent-profile`

- **lib:** one documented, empty module per architecture layer. Each module doc cites its V3 sections:
  - `name` — §6
  - `config` — §17, §18
  - `repo` — §13, §14
  - `resolve` — §12
  - `adapter` — §2, §3, §21, §28
  - `launch` — §4, §22–§25
  - `cli` — §5
  - `output` — §32, §33
- **bin `agent-profile`:** a Clap stub.
  - `--version` and `--help` work; `--help` says the tool is a scaffold.
  - Any other invocation prints "not yet implemented" to stderr and exits with **2**, the §33 CLI usage
    error.
  - SP0 declares **no subcommands**. §5.3's reserved words are a mix of top-level forms (`link`) and
    agent-scoped forms (`<agent> create`), so the command tree is a grammar decision that belongs to SP1.

### 3.3 The `fake-agent` binary (`crates/agent-profile/src/bin/fake-agent.rs`)

A test fixture that later sub-projects use as the "agent" for launch, passthrough and environment contract
tests (§34).

- It prints one JSON object to stdout: `{"argv": [...], "cwd": "...", "env": {...}}`.
  - `argv` holds the arguments after the program name, exactly as received. They are read with
    `std::env::args_os`.
  - `env` contains only variables whose names appear in the comma-separated `FAKE_AGENT_ECHO_ENV`.
    An unset variable is omitted. Empty list entries are ignored. If `FAKE_AGENT_ECHO_ENV` is unset or
    empty, `env` is `{}` — the fixture never echoes the whole environment.
  - **UTF-8 only.** If any argument, the cwd, or an echoed env value is not valid UTF-8, the fixture prints
    nothing to stdout, writes a message to stderr and exits **125**. It never converts lossily, because
    that would make an exact-passthrough test pass on corrupted data.
- **Exit code:**
  - It exits with the integer in `FAKE_AGENT_EXIT`, which must be in `0..=255` (portable across Unix and
    Windows). The default is 0.
  - An empty, non-numeric or out-of-range value is a fixture error: stderr message, exit **125**. That
    code is reserved for fixture errors, so a test can always tell them apart from a requested code.
- The release workflow builds `--bin agent-profile` only, so the fixture never ships, and the package is
  `publish = false`. Known debt (tracked in `TODO.md`): `cargo install --path crates/agent-profile` would
  also install `fake-agent`. No gate or release path runs that.

### 3.4 Test helper

`crates/agent-profile/tests/support/mod.rs` exposes one function, `fake_agent() -> Command`, which every
fixture test uses.

- **Locating the fixture.** It starts from `env!("CARGO_BIN_EXE_fake-agent")`. The fixture is a binary of
  the same package, so cargo (and nextest) build it before any test runs. Tests never invoke cargo
  themselves.
- **Clean environment.** It removes `FAKE_AGENT_EXIT` and `FAKE_AGENT_ECHO_ENV` from the inherited
  environment, so a value exported in the developer's shell cannot change a test's outcome. Tests set the
  variables they mean to test on top of that baseline. Added after code review: before it, running the
  suite with `FAKE_AGENT_EXIT=9` exported failed 3 of 8 tests.

**History — why the fixture is not a separate crate.** The first design put `fake-agent` in its own crate
(`crates/fake-agent`). A helper ran `cargo build -p fake-agent --message-format=json` in each test process
and executed the path cargo reported.
- **Failure.** PR #1's first CI run (run `34765996082`) failed on `Test (macos-latest)` only:
  `fake_agent_env_is_empty_without_echo_list` got `Os { code: 2, kind: NotFound }` spawning that path.
- **Local reproduction attempts, all negative.** Windows stress (204 spawns during 60 no-op builds), Linux
  under WSL2 stress (2944 spawns during 80 builds), and 15 cold `cargo nextest run` iterations on WSL2.
- **Decision.** The macOS mechanism is unverified, so the design removes the whole class instead: no test
  may spawn a path that a concurrent cargo process could be writing. Options considered:
  - (A) a second binary of `agent-profile` — chosen;
  - (B) a copy with retries;
  - (C) prebuild and never build in tests;
  - (D) a cross-process lock around build and copy — agy's first proposal.
- **Convergence.** agy and the driver converged on A over two negotiation rounds, the second after the owner
  allowed a newer Rust, which makes `File::lock` available. Deciding reason: D still runs one nested cargo
  per test process and adds new locking code in exactly the area that failed.

## 4. Acceptance (definition of done)

1. `just check` passes locally: fmt-check, clippy `-D warnings`, typos, nextest, doctests.
2. `cargo deny check` passes.
3. Tests in `crates/agent-profile/tests/smoke.rs`:
   - `version_exits_zero` — `agent-profile --version` exits 0 and prints the crate version.
   - `unimplemented_invocation_is_usage_error` — `agent-profile doctor`, `agent-profile claude work` and
     `agent-profile zzz-unknown` each exit 2, with "not yet implemented" on stderr. This pins only the
     SP0 contract that everything except `--version`/`--help` is unimplemented. It deliberately makes no
     claim about reserved words, which SP1 designs and tests.
   - `fake_agent_echoes_argv_exactly` — args `["--foo", "bar", "--", "a b"]` come back verbatim.
   - `fake_agent_exit_code_is_controllable` — `FAKE_AGENT_EXIT=7` exits 7.
   - `fake_agent_echoes_only_listed_env` — only variables listed in `FAKE_AGENT_ECHO_ENV` and actually set
     appear in `env`; empty list entries are ignored.
   - `fake_agent_env_is_empty_without_echo_list` — `FAKE_AGENT_ECHO_ENV` unset gives `env: {}`.
   - `fake_agent_invalid_exit_code_is_fixture_error` — `FAKE_AGENT_EXIT` values `""`, `256`, `-1` and
     `seven` each exit 125 with empty stdout.
   - `fake_agent_non_utf8_argument_is_fixture_error` — a non-UTF-8 argument exits 125 with empty stdout
     (Unix: byte `0xff`; Windows: lone surrogate `0xD800`).
7. The workspace also compiles on the declared MSRV: `cargo +1.98 check --workspace --all-targets`.
4. Workflows pass `actionlint`.
5. SP0 lands through a pull request from branch `sp0-scaffold`, not a direct push to `main`.
   - The PR's CI jobs Format, Typos, Clippy, Cargo deny, Docs build and Test (ubuntu, macos, windows) are
     green.
   - After merge, the `push` run on `main` is green, including `docs.yml`.
   - The release workflow is not triggered.
6. The GitHub settings in §4.1 are applied in the stated order and read back through `gh api`.

### 4.1 GitHub repository settings

The copied workflows depend on repository settings that `ckir/aiprofiles` does not have.
Measured on 2026-09-13 with `gh api repos/ckir/aiprofiles`:
- `allow_auto_merge: false`
- `has_pages: false`
- branch protection on `main`: `404 Branch not protected`

Without them:
- **`docs.yml`** fails its deploy job on every push to `main`.
- **`dependabot-automerge.yml`**'s `gh pr merge --auto` errors on every Dependabot PR. It becomes unsafe
  once auto-merge is enabled while `main` has no required checks, because it would merge before CI runs
  (flux's own workflow comment calls branch protection load-bearing).

SP0 therefore includes these **outward-facing steps**. Each push and each settings write is confirmed with
the user before it runs, and they run in this order. The next step does not start until the current
step's gate holds; for steps 1 and 4 that means **waiting** for the CI runs to finish.

| # | Step | Gate before the next step |
|---|---|---|
| 1 | Push branch `sp0-scaffold` and open a PR to `main`. The PR's `pull_request` run makes the check names exist. | Wait until every §4 item 5 check has completed green on the PR |
| 2 | Protect `main`, requiring `Format`, `Typos`, `Clippy`, `Cargo deny`, `Docs build`, `Test (ubuntu-latest)`, `Test (macos-latest)`, `Test (windows-latest)`, with `strict: true`, so a PR must be up to date with `main` before merging and the checks cover the tree actually merged | `gh api repos/ckir/aiprofiles/branches/main/protection --jq '.required_status_checks'` shows `strict: true` and exactly those 8 contexts |
| 3 | Enable Pages with `build_type=workflow` | `gh api repos/ckir/aiprofiles/pages --jq .build_type` prints `workflow` |
| 4 | Merge the PR. This enables Dependabot, and `docs.yml`'s first run **publishes the docs site publicly** at the Pages URL; the repo is public. | Wait until `main`'s `push` runs (CI, Docs) complete green |
| 5 | Enable `allow_auto_merge` — last, so auto-merge never exists while `main` lacks required checks | `gh api repos/ckir/aiprofiles --jq .allow_auto_merge` prints `true` |

Between steps 4 and 5, a Dependabot PR's auto-merge step fails harmlessly: auto-merge is not yet allowed,
so nothing merges.

**Exact commands** (bash, repo root):

1. Open the PR and wait for its checks:
   ```bash
   git push -u origin sp0-scaffold
   gh pr create --base main --head sp0-scaffold --fill
   # Checks register asynchronously. Wait until all 8 gate checks exist before watching,
   # so --watch cannot return early on an empty set.
   until [ "$(gh pr checks sp0-scaffold --json name --jq '[.[] | select(.name == "Format" or .name == "Typos" or .name == "Clippy" or .name == "Cargo deny" or .name == "Docs build" or (.name | startswith("Test (")))] | length')" -ge 8 ]; do sleep 10; done
   gh pr checks sp0-scaffold --watch --fail-fast
   ```
   The gate holds when `gh pr checks` exits 0 with every check passing.
2. Protect `main`. The endpoint requires all four top-level keys; nullable ones are sent as `null`:
   ```bash
   gh api -X PUT repos/ckir/aiprofiles/branches/main/protection --input - <<'EOF'
   {
     "required_status_checks": {
       "strict": true,
       "contexts": ["Format", "Typos", "Clippy", "Cargo deny", "Docs build",
                    "Test (ubuntu-latest)", "Test (macos-latest)", "Test (windows-latest)"]
     },
     "enforce_admins": false,
     "required_pull_request_reviews": null,
     "restrictions": null
   }
   EOF
   ```
3. Enable Pages:
   ```bash
   gh api -X POST repos/ckir/aiprofiles/pages -f build_type=workflow
   ```
4. Merge, then wait for `main`'s runs:
   ```bash
   gh pr merge sp0-scaffold --squash --delete-branch
   git fetch origin main
   sha="$(git rev-parse origin/main)"
   # Runs for the merge commit spawn asynchronously; wait for both CI and Docs to exist for THIS sha,
   # so a watch can never latch onto an older green run.
   until [ "$(gh run list --commit "$sha" --event push --json workflowName --jq '[.[] | select(.workflowName == "CI" or .workflowName == "Docs")] | length')" -ge 2 ]; do sleep 10; done
   # A for loop's status is only its LAST iteration's, so collect failures explicitly.
   failed=0
   for id in $(gh run list --commit "$sha" --event push --json databaseId,workflowName --jq '.[] | select(.workflowName == "CI" or .workflowName == "Docs") | .databaseId'); do
     gh run watch "$id" --exit-status || failed=1
   done
   test "$failed" -eq 0 && echo MAIN-GREEN
   ```
5. Allow auto-merge:
   ```bash
   gh api -X PATCH repos/ckir/aiprofiles -F allow_auto_merge=true
   ```

If any command's actual API response contradicts the shape written here, the executor stops and reports it
instead of adapting.

If the user declines any of these, the workflow that depends on it is not added in SP0, and a TODO entry
records the gap.

SP0 has no runtime error model beyond Clap usage errors. SP1 designs the structured error types.

## 5. Development tools

Every tool runs through a `just` recipe, so the local gate, lefthook and CI cannot drift apart.

| Tool | Config | `just` recipe | Gates |
|---|---|---|---|
| cargo-nextest | — | `test`, `test-verbose` (`cargo nextest run --workspace --no-tests=pass` + `cargo test --doc --workspace`) | `just check`; CI Test job ×3 OS |
| lefthook | `lefthook.yml` (pre-push: fmt-check, clippy, typos in parallel; no pre-commit) | `hooks` | local pre-push |
| git-cliff | `cliff.toml` | `changelog` | release time |
| cargo-release | `[workspace.metadata.release]` (lockstep, `v{{version}}` tag, `publish = false`) | `release <patch\|minor\|major>` | pushed tag triggers `release.yml` |
| rustfmt | `rustfmt.toml` | `fmt`, `fmt-check` | gate, pre-push, CI |
| clippy | `clippy.toml` (msrv 1.98) | `clippy` (`--workspace --all-targets -- -D warnings`) | gate, pre-push, CI |
| typos | `_typos.toml` | `typos` | gate, pre-push, CI |
| cargo-deny | `deny.toml` | `deny` | CI |
| bacon | `bacon.toml` | `watch` | local |
| cargo-mutants | — | `mutants` (`--package agent-profile`) | on demand |
| actionlint + shellcheck | — | — | workflow linting |
| cargo-binstall | — | — | installs all of the above |

### 5.1 Files

**Copied verbatim:**
- `rust-toolchain.toml`
- `rustfmt.toml`
- `clippy.toml`
- `bacon.toml`
- `cliff.toml`
- `lefthook.yml`

**Adapted:**
- **`.gitignore`:** flux's file, minus the Python bytecode entry. It keeps `.serena/` and `.clavity/`.
- **`justfile`:** flux's recipes, minus `model`, `model-test`, `model-stamp` and `bench`. `mutants` targets
  `agent-profile`, and the header names Agent Profile.
- **`deny.toml`:**
  - Same targets, advisories, licence allow-list, `private = { ignore = true }` and sources.
  - The `openssl-sys` ban stays. Its reason now cites V3 §36: no hidden network requests and no telemetry.
- **`_typos.toml`:**
  - Exclude `target/`, `Cargo.lock` and `agent-profile-implementation-spec-v3.md` (a delivered document;
    do not rewrite it).
  - Keep the hex-digest ignore patterns.
  - `extend-words` holds domain terms as needed, plus `ckir`.
- **`.github/workflows/ci.yml`:** flux's job set (fmt, typos, clippy, deny, test ×3 OS) plus one added job:
  - Job id `docs-build`, `name: Docs build`, runs on `ubuntu-latest`.
  - Steps: checkout, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, then
    `cargo doc --workspace --no-deps --document-private-items` with `RUSTDOCFLAGS: -D warnings`.
    That is the same command as `docs.yml`'s build step.
  - This is a deliberate deviation from flux, chosen by the owner, so a PR cannot merge with a broken
    docs build. `docs.yml` stays the post-merge publisher.
- **`.github/workflows/release.yml`:** same matrix. It builds `--bin agent-profile` and names the archives
  `agent-profile-<target>.{tar.gz,zip}`.
- **`.github/workflows/docs.yml`:** same structure. The index page lists `agent-profile` and `fake-agent`.
- **`.github/dependabot.yml`, `.github/workflows/dependabot-automerge.yml`:** verbatim, with comments
  adjusted.
- **`.claude/recommended-tools.json`:** flux's entries, minus `java` and `python3`. The `why` texts are
  reworded for this project.

**Dropped from flux:**
- `models/` and `.github/workflows/model.yml` (TLA+ lock-protocol model; nothing comparable here).
- `benches/` and `criterion` (V3 asks for no performance work).
- The `java` and `python3` tool entries.

## 6. Licence and community docs

| File | Treatment |
|---|---|
| `LICENSE` | Verbatim flux copy: PolyForm Noncommercial 1.0.0. The user chose this knowingly; the agy consult flagged that a noncommercial licence limits corporate adoption. |
| `CODE_OF_CONDUCT.md` | Verbatim (Contributor Covenant). |
| `CONTRIBUTING.md` | flux's structure. The oracle is `agent-profile-implementation-spec-v3.md` (cite "spec §N"). Setup keeps the binstall line `cargo binstall -y cargo-nextest just lefthook cargo-deny typos-cli bacon git-cliff cargo-release cargo-mutants`. The gate is listed, and CI adds cross-OS tests with the rationale "process launch, signals and path canonicalization differ per OS". Testing points at spec §34. Conventional commits, the PR process and releasing are kept. Bug reports ask for OS + version, agent name + version, `agent-profile --version`, and `--dry-run` output with secrets redacted. |
| `SECURITY.md` | flux's reporting flow, pointed at `ckir/aiprofiles`. The threat model comes from V3 §1/§36 (see the list below). Out of scope: upstream agents' own authentication and network activity, and dependency vulnerabilities (report those upstream). |
| `README.md` | One-paragraph pitch (§1 core rule); "Status: scaffold"; the §4 architecture diagram; the §5.1 grammar; the §2 agent table, headed **"Planned adapters — not yet implemented or evidence-verified"**, since §2/§3 forbid claiming isolation before evidence exists; building; links to CONTRIBUTING/ROADMAP/TODO; licence. |
| `ROADMAP.md` | The SP0–SP5 table from §1, with a state column (SP0 in progress, others not started). |
| `TODO.md` | SP1 open decisions (listed below); housekeeping (enable private vulnerability reporting; branch protection itself is applied in §4.1); scaffold follow-ups (`lefthook install` per clone). |
| `docs/dev-tooling.md` | The §5 tool table, the gate commands, and a "dropped from flux" list. |

**`SECURITY.md` threat model:**
- Credential copying or token extraction.
- Secrets in human or JSON output.
- Repository-controlled profile selection being trusted.
- `delete` escaping the profile root.
- Configuration corruption or partial writes being consumed.
- Shell mediation in launches.
- Hidden network activity or telemetry.

**`TODO.md` — SP1 open decisions:**
- **Windows Ctrl-C mechanism (§24).** Verify against Microsoft docs whether `SetConsoleCtrlHandler(NULL, TRUE)`
  is inherited by child processes (suspected yes, which would make the agent ignore Ctrl-C). Candidate
  mechanism: a handler routine plus a job object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
- **Git discovery approach (§13–§14; SP3).** §14.2 requires Git worktree metadata, so a plain upward
  directory walk that ignores metadata is not enough.
- **Crate choices:** application-root location, file lock, and atomic replace on Windows.

## 7. Out of scope for SP0

- Any V3 domain behaviour: validation, config, resolution, adapters, launch.
- Choosing SP1 crates beyond the workspace version pins.
- Triggering a release, and publishing docs content beyond the generated index.

## Stand-downs

- REJECTED (at the time; the MSRV is now 1.98): the `rust-version = "1.85"` claim breaks on the pinned
  deps. Measured with `cargo info`: clap 4.6.6 → 1.85, toml 1.1.6 → 1.85, serde_json 1.0.151 → 1.71,
  thiserror 2.0.20 → 1.71.
- REJECTED: a lib and a bin both named `agent-profile` collide under `RUSTDOCFLAGS=-D warnings`.
  A scratch workspace probe documented `agent_profile` and `fake_agent` with no warning.
- OBSOLETE (the helper no longer builds anything): the §3.4 helper built `fake-agent` for the host under a
  cross `--target` test run. It was discarded below the floor because every gate runs tests natively.
- FOLDED (owner decision, 2026-09-13): `docs.yml` ran the docs build only on `push` to `main`, so a PR,
  including a Dependabot auto-merge whose push via `GITHUB_TOKEN` triggers no workflows, could merge
  with a broken docs build. Fixed by the `Docs build` CI job (§5.1), which is the 8th required check (§4.1).
- UNVERIFIED-ACCEPTED (owner decision, 2026-09-13): the pre-push hook keeps flux parity (fmt-check,
  clippy, typos; no tests). The agy panel suggested adding `just test`; the owner kept parity, and tests
  remain gated by `just check` and CI.
- UNVERIFIED-ACCEPTED (owner decision, 2026-09-13): the agy panel stopped at round 5 of 6, still finding
  command-level details in §4.1. The owner shipped the spec because the implementation plan re-verifies
  every command and each outward-facing step is confirmed before it runs. Not verified by a live call:
  `POST repos/{owner}/{repo}/pages` with `build_type=workflow` creates the site. The §4.1 stop-and-report
  rule covers a mismatch.
- REJECTED: the copied CI's action versions may not support `rust-version = "1.85"`. flux `ci.yml:19`
  uses `dtolnay/rust-toolchain@stable`, which installs current stable regardless; `rust-version` does not
  select the toolchain.
- WRONGLY REJECTED, then FOLDED: "the §3.4 helper's nested `cargo build` races under nextest".
  - The original rejection rested on a Windows scratch probe: 2 parallel tests each ran the nested build,
    and it passed.
  - That probe checked builds, not spawning the reported path while another build ran, and never ran on
    macOS.
  - macOS CI then failed with ENOENT exactly there.
  - Folded by making `fake-agent` a binary of `agent-profile` (§3.4, History).
