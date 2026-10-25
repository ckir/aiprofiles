# Agent Profile Roadmap

v0.1 scope is fixed by the V3 specification. The build order follows spec §35, split into sub-projects.
Each sub-project has its own design spec and implementation plan under `docs/superpowers/`, and ends
green on Linux, macOS and Windows.

| SP | Deliverable | V3 §35 phases | State |
|---|---|---|---|
| SP0 | Scaffold — workspace, tooling, licence, community docs, CI, `fake-agent` fixture | — | done (#1) |
| SP1 | Core + explicit launch — profile-name validation, application root, TOML configuration with locked atomic writes, structured errors, `LaunchPlan`, Unix `exec` / Windows child launcher, passthrough, environment overrides, dry run, exit codes | 1, 3 | done (#8) |
| SP2 | Architecture gate — adapter model, capability and evidence metadata, common contract suite, Claude Code, Codex CLI, Aider | 4A | done (#11) |
| SP3 | Repository resolution — discovery, canonical identity, mapping storage, precedence, `resolve`, `current`, `status`, `link`, `unlink` | 2 + part of 5 | done (#19) |
| SP4 | Remaining adapters — Gemini CLI, GitHub Copilot CLI, OpenCode, Cline CLI, Pi, Kiro CLI, Cursor Agent CLI, Continue CLI, Amp | 4B | done (branch `sp4b-adapters`) |
| SP5 | Lifecycle and quality — `create`, `list`, `delete`, `repositories`, `doctor`, JSON output, shell completions, docs | rest of 5, 6 | done — SP5a lifecycle, SP5b --json + repositories, SP5c doctor, SP5d completions + docs + CI |

`link`/`unlink` land with resolution (SP3), earlier than §35's Phase 5, so resolution never ships
without a way to create the mappings it resolves. SP4 is done: the nine adapters named above are
registered on branch `sp4b-adapters`, each backed by a `custody: ci` transcript committed byte-for-byte
and measured in the Sandbox workflow (run 39024332187), so the repository carries twelve adapters, each
with a checkable transcript. SP5 picks up lifecycle and quality.

`ProfilePresence::Known` was settled in SP4: it is **reserved, not dead**. Spec §8 defines a profile as
*known* when it can be identified from the adapter's documented profile mechanism rather than a
materialized directory. None of the twelve adapters is native-profile, so nothing constructs it — but
removing it would make the first such adapter a change to a public enum, and until then `presence()`
would have to report a profile the agent itself lists as `Absent`. The variant is documented at its
declaration and stays unconstructed.

## Tracked debt

**Antigravity CLI has no isolation mechanism, and the README says so.** Measured on 2026-09-22 against
`agy` 1.2.7: nothing relocates `~/.gemini/antigravity-cli/`, and credentials live in the OS keyring. It
is documented under "Agents that cannot be isolated" rather than shipped as an adapter, because the
`Adapter` trait materialises a profile directory and one that nothing reads would imply isolation the
user does not have. Re-measure if the vendor ships a configuration-directory variable or flag; at that
point it is an ordinary adapter candidate and a specification change, since V3's roster is closed at
twelve.

**`dtolnay/rust-toolchain` is pinned to a branch tip, and whether Dependabot bumps it is unverified.** `release-plz.yml` sat at `6bed076` from 2026-09-15 while the `stable` branch moved to `b725eda`; SP5d moved every use to the tip with an explicit `toolchain: stable`. If it stays put again, bump it by hand with `gh api repos/dtolnay/rust-toolchain/git/ref/heads/stable`.

**Mechanism changes are not detected.** `doctor` shows each adapter's declared mechanism id but nothing compares it to an earlier one (spec section 28: `doctor` "may" report a change). Do not store a per-profile snapshot: section 28 makes mechanism ids adapter metadata, not part of the profile model, and a wrapper file would sit inside the agent's own directory. If an adapter ever moves to a `-v2` mechanism, declare the retired mechanism's sentinel path in `AdapterMetadata` so `doctor` can spot old profiles read-only.

**Profile names are not completed.** The completion script is static (owner's choice, SP5d D1); completing profile names needs a runtime hook. A future enhancement outside v0.1.

**clap_complete's bash script breaks for a binary name containing `-`.** Measured at 4.6.11 and on clap's `main` on 2026-10-21: the traversal names states from `bin_name.replace('-', "__")`, the `case` labels from `replace('-', "__subcmd__")`, so no label matches and bash completes only the first word. `cli/completions.rs` rewrites `agent__subcmd__profile` to `agent__profile` in the generated bash; `bash_script_has_no_mismatched_state_label` is the oracle. Remove the workaround once upstream fixes it. Not reported upstream (an outward action the owner has not OK'd).

v0.1 was done when every item of spec §37 (Final Definition of Done) was checked, and shipped as `v0.1.0` on
2026-10-25.
