# SP5d — shell completions, README refresh, CI pinning and clippy matrix

Status: DRAFT 2026-10-21, awaiting owner approval. The owner skipped the brainstorming questions and decided four
forks on 2026-10-21 (§9); agy-first ALIGNED on both consults (`.clavity/seams/sp5d-forks.md`,
`.clavity/seams/sp5d-clippy-matrix.md`).
V3 (`agent-profile-implementation-spec-v3.md`) wins over this document. Fourth and last slice of SP5 as the owner
split it on 2026-10-19: SP5a lifecycle (`9039ee1`) -> SP5b `--json` + `repositories` (`e099b01`) -> SP5c
`doctor` (`c0d5caf`) -> **SP5d**. Branch `sp5d-completions-docs-ci` off `c0d5caf`.

## 1. Intent

Three loose ends stand between the implemented command set and V3 §35 Phase 6 ("shell completion", "README",
"cross-platform CI"):

1. `completions` is a reserved word (V3 §5.3) that still answers "not yet implemented".
2. The README describes the project as of SP3. Its status line names three adapters, while twelve now exist.
   Its usage block omits `doctor` and `completions`.
3. Two tracked-debt items in `ROADMAP.md`: "Two contradictory action-pinning idioms" and "Clippy runs on Linux
   only, so a platform-gated warning never fails CI".

Goals
- `agent-profile completions <shell>` prints a completion script for bash, elvish, fish, powershell or zsh
  (owner: static script, §9 D1).
- The completion script and the command dispatcher derive from ONE description of the grammar, so they cannot
  drift apart. A test pins that description to the dispatcher's behaviour.
- Every external `uses:` in `.github/workflows/` is pinned to a 40-hex commit SHA with a version comment.
  `just check` and CI enforce this with our own script (owner, §9 D2).
- Clippy `-D warnings` runs on ubuntu, macos and windows. The required status check `Clippy` keeps its name, so
  branch protection is unchanged (agy-first ALIGNED on option A, §9 D5).
- The README is refreshed in full for the v0.1 command set (owner, §9 D3).
- `ROADMAP.md` closes both debt items and marks SP5 done.

Non-goals
- Completing profile names. The static script stops at the profile position, where the shell falls back to
  its own default (owner, §9 D1). Recorded as a future enhancement in `ROADMAP.md` (§7).
- Runtime ("dynamic") completion, `clap_complete`'s `unstable-dynamic` feature, or a hidden `__complete` entry.
- Installing the script anywhere. `completions` prints it to stdout; the user puts it in place (README §6).
- A `doctor` check about completions (agy-first F3: there is nothing on the host to diagnose).
- Changing branch protection or any repository setting.

Anti-goals (a failure even if everything compiles)
- `completions` that reads the configuration, the application root or the environment. It must work with
  `AGENT_PROFILE_HOME` unset, relative, or pointing at a missing directory. It writes nothing to disk and
  prints no environment values (V3 §36).
- A completion script that offers something the dispatcher then rejects, for example `--json` after `create`
  (which answers "`--json` is not yet implemented") or `agents` after an agent.
- A pin check that a floating tag can pass, for example `@v7 # 0123456789abcdef0123456789abcdef01234567`.
- A `Clippy` aggregator that reports success when a leg failed, was cancelled or was skipped.

## 2. The `completions` command

### 2.1 Grammar and behaviour (the contract)

`agent-profile completions <shell>`, where `<shell>` is exactly one of `bash`, `elvish`, `fish`, `powershell`,
`zsh`, spelt in lower case.

| Invocation | Exit | stdout | stderr |
|---|---|---|---|
| `completions bash` (and each other shell) | 0 | the script | empty |
| `completions --help`, `completions -h` | 0 | the command's usage (§2.4) | empty |
| `completions` | 2 | empty | `agent-profile: error: \`completions\` needs a shell: agent-profile completions <bash\|elvish\|fish\|powershell\|zsh>` |
| `completions nope`, `completions Bash` | 2 | empty | `agent-profile: error: unknown shell \`nope\` (shells: \`bash\`, \`elvish\`, \`fish\`, \`powershell\`, \`zsh\`)` |
| `completions bash zsh` | 2 | empty | `agent-profile: error: \`completions\` takes one shell` |
| `completions bash --json` (any option) | 2 | empty | the existing unknown-option message, as other command words emit it |
| `completions bash --repo x` | 2 | empty | `agent-profile: error: unknown option "--repo" for \`completions\`` (the existing lifecycle form) |
| `completions bash -- x` | 2 | empty | the existing "takes no agent arguments; remove `--`" message |
| `<agent> completions` (with or without a shell) | 2 | empty | `agent-profile: error: \`completions\` takes no agent: agent-profile completions` (the existing top-level-only form used for `agents`, `profiles` and `repositories`) |

The exact wording of the rows marked "existing" is whatever the dispatcher emits today for the same mistake on
`agents`. The test asserts it by comparison with `agents`, not by a copied literal.

If writing to stdout fails, the command behaves like the other commands that print through `write_out`. The
script is therefore generated into an in-memory `Vec<u8>` and then written with `write_out`, never generated
straight into `io::stdout()`. `clap_complete::generate` returns `()`, and its bash generator calls
`try_generate(..).expect("failed to write completion file")` (clap_complete 4.6.11 `src/aot/shells/bash.rs`),
so a closed pipe would panic instead of exiting 1.

`completions` must not construct an `AppRoot`, read `config.toml`, or touch the filesystem: it is executed
before `run_command`'s root and configuration setup. The shell word is never passed through `ProfileName`
validation; it is matched against the five names only, so `completions Bash` reports an unknown shell and
nothing else.

### 2.2 Dispatch

`completions` joins the hand dispatcher as a `CommandWord` variant, `CommandWord::Completions`. It is top-level
only, like `Agents`, and takes one bare argument, its shell.
- The clap `Command::Completions(Rest)` stub and `Command::reserved_name` are removed. After this change no
  reserved word reaches clap except through `external_subcommand`.
- `split_top_level`/`validate_command` turn a valid `completions <shell>` into a new
  `Invocation::Completions(clap_complete::Shell)`, which `execute` handles itself and never passes to
  `run_command`. `run_command` resolves `AppRoot`, loads the configuration and parses `profile` as a
  `ProfileName`, all of which `completions` must avoid (§2.1). The shell word therefore never travels in
  `Invocation::Command`'s `profile` field.
- `RESERVED_WORDS` in `name.rs` is unchanged: `completions` stays reserved.
- `COMMANDS_HELP` gains the line `agent-profile completions <shell>` under a new heading "Shell completion:".
- `Error::NotYetImplemented` stays, because `--json` on `create`/`delete` still uses it.

### 2.3 One grammar description (the anti-drift mechanism)

`validate_command` currently decides inline, with `matches!` sets, which command accepts `--repo`, `--json`
and `--yes`, which takes a profile, and which may or must have an agent. SP5d moves each of those decisions into
a `CommandWord` method that returns one fact:
- agent scope: `TopLevelOnly` (agents, profiles, repositories, completions), `AgentRequired` (current, resolve,
  create, delete), or `Either` (status, link, unlink, list, doctor);
- `accepts_repo`;
- `accepts_json` (true only where `--json` is implemented; create and delete answer "not yet implemented"
  today, so they are false). `validate_command` keeps its separate `create`/`delete` check that answers
  `--json` with `Error::NotYetImplemented`. That check runs before the generic unknown-option check, exactly
  as today, so `create --json` still exits 2 with "`--json` is not yet implemented". The completion model
  offers `--json` only where `accepts_json` is true;
- `accepts_yes`;
- the bare positional it takes, as a function of whether an agent was given: none, a required profile, an
  optional profile, or a shell. Measured at `c0d5caf`: `doctor` takes an optional profile only with an agent
  (`agent-profile doctor work` exits 2 "`doctor` takes no arguments"); `link`, `create` and `delete` take a
  required profile; `completions` takes a required shell; every other word takes none.

`validate_command` then calls these methods. The completion model is built from the same methods, iterating an
exhaustive `CommandWord::ALL` array that a `match` with no wildcard keeps in sync with the enum. Because both
sides read one table, a new flag on a command reaches the completion script automatically. The test in §2.5
pins the table itself.

This refactor must change no behaviour. The oracle is the whole existing test suite (398 tests at `c0d5caf`,
in particular `tests/launch.rs` `behaviour_table_rows_with_non_zero_exits` and the `cli.rs` unit tests). Only
the two `completions` rows change, which §2.5 rewrites.

### 2.4 The completion model and the script

A private function builds a `clap::Command` named `agent-profile` purely for `clap_complete::generate`. It is
never used to parse input. Its content comes from §2.3's methods and `adapter::known_agents()`:

- Root: one subcommand per command word whose scope is `TopLevelOnly` or `Either`; one subcommand per id in
  `known_agents()`; the flags `-h`/`--help` and `-V`/`--version`.
- Under each agent: one subcommand per command word whose scope is `AgentRequired` or `Either`, and the
  launch options `--dry-run` and `--verbose`. There is no `profile` positional at the agent level.
  clap_complete's zsh generator places subcommands at `$line[positionals + 1]` (`src/aot/shells/zsh.rs`), so
  an empty positional would push subcommand completion to the second word. The profile is not completed
  anyway (§1), so it is simply left out.
- `disable_help_subcommand(true)` on every level. Measured in a throwaway build: without it clap adds a
  `help` subcommand, which the script then offers although the dispatcher has no such command.
- Under each command: `--repo <path>` with `ValueHint::DirPath` if `accepts_repo`; `--json` if `accepts_json`;
  `--yes` if `accepts_yes`; the positional from §2.3 (`completions` gets the five shell names as possible
  values; a profile positional has no candidates).
- Every subcommand carries `-h`/`--help`.

`known_agents()` includes `fake` in debug builds and not in release builds. The script follows the build, like
the dispatcher's "unknown agent" message.

`completions <shell>` runs `clap_complete::generate(shell, &mut model, "agent-profile", &mut buffer)` and
then writes the buffer with `write_out`. The buffer becomes the `&str` that `write_out` takes through
`String::from_utf8_lossy`. clap_complete writes only `str` data, so nothing is lost, and the conversion has no
panic path.

**Upstream bash defect and its workaround (measured 2026-10-21).** In clap_complete 4.6.11, and in clap's
`main` branch on that date, the bash generator names states inconsistently when the binary name contains a
`-`:
- the traversal derives its names from `bin_name.replace('-', "__")` (`agent__profile__subcmd__claude`);
- the `case` labels derive theirs from `sc.replace('-', "__subcmd__")` over a path that starts with the binary
  name (`agent__subcmd__profile__subcmd__claude`).

No label ever matches, so bash completes the first word and nothing after it. A throwaway build confirmed
both the failure and the fix: rewriting every `agent__subcmd__profile` to `agent__profile` in the generated
script makes `agent-profile claude <TAB>` offer the agent's commands and flags. The fish, zsh and PowerShell
scripts from the same model name their states consistently.

So, for bash only, the generated text has every occurrence of `agent__subcmd__profile` replaced with
`agent__profile` before it is written. If a future clap_complete fixes the bug, the string no longer occurs
and the replacement does nothing. §2.5 test 7 is the oracle in both cases. The ROADMAP records the upstream
defect, so the workaround can be removed once it is fixed (§7).

New dependency: `clap_complete` at the newest 4.x release at implementation time (4.6.11 on 2026-10-21),
default features only, added under `[workspace.dependencies]` like `clap`. Only `clap_complete` enters
`Cargo.lock`: `clap` stays at 4.6.7, which satisfies clap_complete's `clap >= 4.6.6`, and no `cargo update`
runs. `cargo deny check` must stay green.
clap_complete's licence (MIT OR Apache-2.0) and MSRV (1.85) fit the repository's.

The command's usage text (`CommandWord::Completions.usage()`):
```
Usage: agent-profile completions <shell>
Print a shell completion script to stdout. <shell> is bash, elvish, fish, powershell or zsh.
```

### 2.5 Tests (each must fail under a logic mutant of the code it guards)

1. **Contract table.** One integration test per §2.1 row. stdout, stderr and exit code are asserted exactly,
   except the script body.
2. **Rewritten rows.** In `tests/launch.rs` `behaviour_table_rows_with_non_zero_exits`, the two rows
   `(["completions"], 2, "not yet implemented")` and `(["fake","completions"], 2, "not yet implemented")`
   become the §2.1 messages.
3. **No filesystem or root access.** `completions bash` exits 0 with `AGENT_PROFILE_HOME` set to a relative
   path, and again set to a missing directory. A temp root's directory tree is identical before and after.
4. **Grammar table pinned.** A `cli.rs` unit test lists every `CommandWord` with its expected scope,
   `accepts_repo`, `accepts_json`, `accepts_yes` and positional, written out in full. It asserts the methods
   return exactly that. A second test drives the dispatcher (`split`) for every word, with and without an
   agent, and for every flag. It compares the outcome against that same hand-written table, NOT against the
   methods, because comparing against the methods would only test the dispatcher's delegation to them. A
   flag the table accepts must not be rejected. A flag the table rejects must be rejected with the message
   the dispatcher gives today. This includes `create --json` and `delete --json` answering "not yet
   implemented".
5. **Model mirrors the table.** For the `clap::Command` after `build()`, so that anything clap adds
   implicitly, such as a `help` subcommand, is visible:
   - the root's subcommand names equal the expected set exactly, so nothing is extra and nothing is missing;
   - each agent's subcommand names do the same;
   - each command's argument ids do the same.
   Assert set equality, not counts.
6. **Script per shell.** For each of the five shells, `completions <shell>` exits 0 with non-empty stdout,
   and stdout contains every agent id and every command word. Distractor: the script for `agents` (top-level)
   must not appear under an agent subcommand in the model; that is asserted on the model in test 5, not on
   script text.
7. **Bash behaviour (Linux only, `cfg(target_os = "linux")`).** Source the bash script in `bash --norc` and
   drive the completion function with `COMP_WORDS`/`COMP_CWORD`. Assert that:
   - `agent-profile <TAB>` offers `claude` and `agents`;
   - `agent-profile claude <TAB>` offers `create` and `--dry-run`, and not `agents`;
   - `agent-profile claude create <TAB>` offers `--help`, and offers neither `--json` nor `--repo`. The positive
     half is what stops an empty, broken completion from passing.
   If `bash` is absent on Linux, the test fails rather than skips.

## 3. Action pinning

### 3.1 The rule

Every `uses:` in `.github/workflows/*.yml` whose value does not start with `./` must have this form:
```
uses: <owner>/<repo>[/<path>]@<40 lower-case hex> # <ref>
```
- `<ref>` is the release tag the SHA belongs to (`v7.0.1`).
- For an action whose version is chosen by a branch, `<ref>` is that branch name. Today this applies only to
  `dtolnay/rust-toolchain`, whose `stable` branch's `action.yml` defaults `toolchain: stable`, measured at
  `6bed076` and `b725eda`.
- Local reusable workflows (`uses: ./.github/workflows/release.yml`) are exempt.

### 3.2 The conversion

Every floating reference listed below becomes SHA + comment. The SHA is the commit of the action's newest
release when the change is implemented, resolved from GitHub (`gh api repos/<o>/<r>/git/ref/tags/<tag>`, with
annotated tags dereferenced to the commit). It is never copied from a review; agy's `rust-cache` SHA was
measured wrong. The same action has the same SHA in every workflow, so the already-pinned workflows
(`release-plz.yml`, `release.yml`, `sandbox.yml`, `pr-title.yml`, the typos step) move to the same newest SHAs.

The floating references at `c0d5caf`, complete:
- `ci.yml`:
  - `actions/checkout@v7` ×11;
  - `dtolnay/rust-toolchain@stable` ×5;
  - `Swatinem/rust-cache@v2` ×4;
  - `taiki-e/install-action@nextest` ×1;
  - `EmbarkStudios/cargo-deny-action@v2` ×1.
- `docs.yml`:
  - `actions/checkout@v7`;
  - `dtolnay/rust-toolchain@stable`;
  - `Swatinem/rust-cache@v2`;
  - `actions/upload-pages-artifact@v5`;
  - `actions/deploy-pages@v5`.
- `dependabot-automerge.yml`: `dependabot/fetch-metadata@v3`.

The implementer re-counts these with `grep -n 'uses:' .github/workflows/*.yml` and reports any difference as
`STATE_MISMATCH`.

Inputs that a branch or ref name used to supply implicitly become explicit:
- `taiki-e/install-action` gets `with: tool: nextest`. On a release tag `tool` is `required: true`, measured
  at `v2.87.29`.
- `dtolnay/rust-toolchain` is pinned to the `stable` branch tip, with `# stable`. Every use also states
  `toolchain: stable`, so the intent does not rest on the branch's default.

Known limit, recorded and not fixed: the `dtolnay/rust-toolchain` pin in `release-plz.yml` stayed at
`6bed076` from 2026-09-15 while its branch moved to `b725eda`. That is consistent with Dependabot not bumping a
SHA pinned from a branch, but it is not proven. The ROADMAP line (§7) states it as unverified.

### 3.3 The check

`scripts/check-action-pins.sh`, POSIX `sh`, shellcheck-clean like the other scripts:
- It scans every `uses:` line in `.github/workflows/*.yml`, with or without a leading `- `.
- It skips comment lines and `./` values.
- It requires the whole value after `uses:` to match
  `^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(/[^@ ]+)?@[0-9a-f]{40} # [A-Za-z0-9_.-]+$`.
- On a violation it prints `file:line: <offending text>` for each one and exits 1. Otherwise it exits 0.
- It takes an optional directory argument, so its test can point it at fixtures.

`scripts/tests/check-action-pins.sh`, with fixtures written to a temp dir and removed by `trap`. Each case
asserts an exit code and that the offending line is named:
- **accepted:** a SHA + tag pin; `- uses:` and bare `uses:` forms; a `./` reusable workflow; a commented-out
  floating `uses:`;
- **rejected:** `@v7`; `@stable`; a 39-hex SHA; a 40-hex SHA with upper case; a SHA with no comment;
  `@v7 # <40 hex>`, where the SHA is only in the comment; `docker://` (no such use exists, and the rule
  rejects it until someone decides otherwise).

Wiring:
- a `just action-pins` recipe runs both, and is added to `check`;
- `ci.yml` gains a job `name: Action pins` on `ubuntu-latest`. Its first step is `actions/checkout`, pinned
  like every other use. Its next two steps run the recipe's two commands with `sh`. This job adds one more
  pinned checkout; the floating-reference counts in §3.2 describe `c0d5caf` and are unchanged.
Making `Action pins` a required check is the owner's call, outside this branch, and is mentioned in the PR body.

## 4. Clippy on three operating systems

As agy-first ALIGNED on option A (`.clavity/seams/sp5d-clippy-matrix.md`):
- Job `clippy-matrix`, `name: Clippy (${{ matrix.os }})`, `strategy: { fail-fast: false, matrix: { os:
  [ubuntu-latest, macos-latest, windows-latest] } }`, `runs-on: ${{ matrix.os }}`. Same steps as today:
  checkout, rust-toolchain with `components: clippy`, rust-cache, and
  `cargo clippy --workspace --all-targets -- -D warnings`.
- Job `clippy`, `name: Clippy`, `needs: [clippy-matrix]`, `if: ${{ !cancelled() }}`, `runs-on: ubuntu-latest`, one step
  that exits 1 unless `needs.clippy-matrix.result == 'success'`. The value is passed through `env:`, not
  interpolated into the script. On failure the step prints an `::error::` line naming the result and pointing
  at the `Clippy (<os>)` legs.
- Any warning the new legs surface is fixed in this branch. Locally, `cargo clippy --workspace --all-targets
  --target x86_64-pc-windows-gnu -- -D warnings` lints the Windows `cfg` code (the target is installed on the
  maintainer host). macOS has no local equivalent, so CI is its oracle.
- The `just clippy` recipe is unchanged.

## 5. README refresh

Rewrite the stale parts and keep the parts that are current. Current means the agent table, "Agents that
cannot be isolated", Architecture, Workspace and Licence; their content stays as it is.
- Status paragraph: all v0.1 commands are implemented, there are twelve adapters, and a pointer to
  `ROADMAP.md`. No claim that v0.1 is released.
- `## Usage` block: split the combined `agent-profile <agent> resolve|status ...` line into the two lines
  `COMMANDS_HELP` uses, and add `agent-profile doctor [--repo <path>] [--json]`,
  `agent-profile <agent> doctor [<profile>] [--repo <path>] [--json]` and `agent-profile completions <shell>`.
  A test (my call, §10) reads `README.md` with `include_str!` and asserts that every `agent-profile ...` line
  of `COMMANDS_HELP` appears verbatim in the README's usage block. The README may carry more lines (the launch
  forms), not fewer.
- New prose on `doctor`: what it checks (host, configuration, mappings, the agents, and with `<agent>` one
  profile's paths and launch environment names), that it never launches an agent and never prints a secret
  or an environment value, and its exit codes. The exit codes come from SP5c's spec: 5 when a check fails
  (returned by `run_doctor` in `cli.rs`, not an `Error` variant), 0 otherwise, and the usual `error.rs` codes
  for usage and I/O errors. They are checked against the code at implementation time.
- New `## Shell completion` section, with one install snippet per shell. Each snippet loads
  `agent-profile completions <shell>` the way that shell expects, and the section notes that profile names are
  not completed.
- `## Building`: the `just check` comment lists what the gate really runs (the justfile's `check` line).
- `## Planned adapters`: delete the empty section, whose only content is "No adapters are currently planned".
  The decision is my call, marked in §10.

## 6. Shell snippets (README content; the implementer verifies each against clap_complete's docs or source)

- bash: `source <(agent-profile completions bash)` in `~/.bashrc`.
- zsh: write the script to a directory on `fpath` as `_agent-profile`, then `compinit`.
- fish: `mkdir -p ~/.config/fish/completions && agent-profile completions fish > ~/.config/fish/completions/agent-profile.fish`.
- powershell: `agent-profile completions powershell | Out-String | Invoke-Expression` in `$PROFILE`.
- elvish: `eval (agent-profile completions elvish | slurp)` in `rc.elv`.

## 7. ROADMAP and TODO

`ROADMAP.md`:
- The SP5 row becomes "SP5d done". This is the last SP5 slice.
- Delete the two debt entries "Two contradictory action-pinning idioms" and "Clippy runs on Linux only".
- Add the unverified `dtolnay/rust-toolchain` branch-pin limit (§3.2).
- Add profile-name completion as a future enhancement, outside v0.1.
- Add the upstream clap_complete bash defect for binary names containing `-`, with the workaround's location,
  so the workaround is removed once upstream fixes it. Reporting it upstream is an outward action; it is done
  only with the owner's OK.

`TODO.md` is updated to match.

## 8. Delivery

One branch and one PR (owner, §9 D4), with commits in this order:
1. CI: SHA pins, the pin check and its test, the clippy matrix, and any lint fixes it needs.
2. The grammar-table refactor (no behaviour change).
3. `completions`.
4. README, ROADMAP and TODO.

`just check` stays green after every commit. The usual gates apply before the PR: AGY-CAPSTONE GREEN, then
AGY-TEST-AUDIT. Pushing and opening the PR need the owner's OK.

## 9. Decisions

- D1 (owner 2026-10-21): static completion script, without profile names. agy and I both recommended it.
- D2 (owner): our own pin-check script, not zizmor, which agy raised as an alternative.
- D3 (owner): a full README refresh.
- D4 (owner): one PR.
- D5 (agy-first ALIGNED, within the owner's ruling that clippy runs on three OSes): matrix plus a `Clippy`
  aggregator, with no branch-protection change, `fail-fast: false`, and a strict `== success` check.
- D6 (agy-first ALIGNED): all five shells; no `doctor` check for completions.

## 10. Calls I made (the owner may overturn any)

- The exact error wording in §2.1.
- `completions` moves into the hand dispatcher (§2.2), and the clap stub is removed.
- The flag rules move into `CommandWord` methods (§2.3), rather than a separate completion table guarded only
  by a test.
- The bash behaviour test (§2.5 test 7) is Linux-only, and fails if bash is absent.
- The README usage block is pinned to `COMMANDS_HELP` by a test (§5).
- All workflows move to the newest release SHAs, including the already-pinned ones (§3.2).
- `rust-toolchain` is pinned to its `stable` branch tip, with an explicit `toolchain: stable`.
- The `Action pins` job is not required by branch protection unless the owner adds it.
- The empty "Planned adapters" README section is deleted.
- The bash workaround (§2.4) is a string replacement on the generated script, not a fork or a hand-written
  bash script.

## Panel ledger

Round 1 (solo + agy):
- folded: the positional depends on whether an agent is given; completions runs before root/config and
  bypasses profile validation; README is pinned to COMMANDS_HELP by a test; the pin counts;
  AB-1 (no agent-level profile positional, because of zsh); CA-1 (buffer + write_out); LI-1
  (`Invocation::Completions`); MG-1 (keep create/delete `--json` NotYetImplemented; test 4's second half
  checks against the table, not the methods); PP-1 (split `resolve|status`); DC-1 (no clap bump); the help
  subcommand; the bash hyphen defect workaround.
- REJECTED (round 1): agy's Q4 claim that bash completes subcommands under an agent. A throwaway build at
  clap_complete 4.6.11 showed `agent-profile claude <TAB>` offering nothing until the state names were corrected.
Round 2 (agy; seats Axiom Breaker, Cascade Analyst, Activation Auditor, Blindspot Auditor, Mechanism Gamer):
- folded: AA-1 (the Action pins job has a pinned checkout); AA-2 (the aggregator uses `!cancelled()`, so a
  cancelled run does not turn red; a leg that times out or fails still runs it and fails it, and a cancelled
  run's required Test checks block the merge regardless); MG-2 (the `create <TAB>` assertion gains a positive
  half); BA-1 (fish needs `mkdir -p`); CA-2 (`from_utf8_lossy`).
Round 3 (agy; seats Axiom Breaker, Cascade Analyst, Docs Truth Auditor (bespoke), Dependency Cynic, Literal
Implementer): no live challenge, PANEL VERDICT GREEN. agy ran the zsh, fish and PowerShell snippets on this host;
elvish is not installed. Folded one clarification: doctor's exit 5 comes from `run_doctor`, not `error.rs`.

## Stand-downs

- UNVERIFIED-ACCEPTED (pending owner acceptance at spec approval): the elvish install snippet (§6) is checked only
  against clap_complete's elvish generator source, not run, because elvish is not installed on the maintainer host
  or the CI runners.
