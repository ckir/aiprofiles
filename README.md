# Agent Profile

A local, privacy-first Rust CLI for selecting and launching profiles for multiple coding agents.

> `agent-profile` owns profile selection. The coding agent owns authentication and agent-specific
> configuration. The evidence determines what `agent-profile` is allowed to claim.

Agent Profile never copies credentials, extracts tokens, sends telemetry or trusts
repository-controlled profile selection.

**Status: repository resolution (SP3).** Profile-name validation, configuration, the launch path (`exec` on
Unix, a supervised child on Windows), three evidence-backed adapters (Claude Code, Codex CLI, Aider) and
repository mappings exist; see [ROADMAP.md](ROADMAP.md). The authoritative design is
[`agent-profile-implementation-spec-v3.md`](agent-profile-implementation-spec-v3.md).

## Architecture

```text
CLI
  ↓
Configuration
  ↓
Repository Discovery
  ↓
Profile Resolution
  ↓
Agent Adapter
  ↓
LaunchPlan
  ↓
Process Launcher
  ↓
Coding Agent
```

Adapters produce a `LaunchPlan`; they never spawn processes.

## Usage

```text
agent-profile <agent> <profile> [WRAPPER OPTIONS] [-- <agent args...>]
agent-profile <agent> [WRAPPER OPTIONS] [-- <agent args...>]
agent-profile <agent> current|resolve|status [--repo <path>]
agent-profile [<agent>] link <profile> [--repo <path>]
agent-profile [<agent>] unlink [--repo <path>]
agent-profile status [--repo <path>]
```

Everything after `--` is passed to the agent untouched. `--dry-run` shows what would be launched without
launching or creating anything.

Configuration and profiles live in `~/.agent-profile/` (`%USERPROFILE%\.agent-profile\` on Windows). Set
`AGENT_PROFILE_HOME` to an absolute path to use another directory.

Without a profile word, the profile comes from the first of: the agent's mapping for the current Git repository,
the repository's mapping, `default_profile`, and otherwise an error. `link` stores a mapping for the repository
you are in (its canonical root; a subdirectory, symlink or junction resolves to it), `unlink` removes it, and
`unlink --repo <path>` removes the mapping stored for a path even after the repository was deleted or moved. A
mapping applies only to that repository: not to a nested repository, a submodule or a linked worktree, which are
repositories of their own. `<agent> resolve` and `status` show what would be selected and why.

```toml
default_profile = "work"

[repositories.'C:\src\acme']
profile = "work"
agents = { claude = "personal" }
```

The global default is set by editing `default_profile`. Discovery reads only the `.git` entry and Git's
`gitdir`/`commondir` files; it never runs Git, never reads Git configuration, and ignores `GIT_DIR` and
`core.worktree` (use `--repo`).

## Supported agents

Every profile lives under `<root>/profiles/<profile>/<agent>/` and is created on first launch. Each row is
backed by a transcript in [`docs/evidence/`](docs/evidence/) recording the measurement it rests on; a state
weaker than `Supported` means exactly what its reason says.

**Read the whole row, not the support column.** `Proven` is a statement about the mechanism, not about the
account: an adapter can honestly be `Proven` while its credential isolation is `Unknown`, because proving
that stored credentials separate requires logging in to each vendor and that cannot be measured in CI. The
capability states are never hidden behind a flag for this reason.

| Agent | Mechanism | Support | Config isolation | Credential isolation | State isolation |
|---|---|---|---|---|---|
| Claude Code 2.1.292 (`claude`) | `CLAUDE_CONFIG_DIR` | Proven | Supported (project `.claude/` settings still layer on top) | Conditional (`ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `CLAUDE_CODE_OAUTH_TOKEN` and similar variables bypass it) | NotGuaranteed (history and project state moving with the directory is community-sourced only) |
| Codex CLI 0.160.1 (`codex`) | `CODEX_HOME` | Proven | Supported (project-level configuration layering is not measured) | Conditional (`OPENAI_API_KEY`, `CODEX_API_KEY`, `CODEX_ACCESS_TOKEN` bypass it) | Conditional (`CODEX_SQLITE_HOME`) |
| Aider 0.86.2 (`aider`) | `--config <profile>/.aider.conf.yml` | Proven | NotGuaranteed (home, repository and working-directory `.aider.conf.yml`, `.env` and `AIDER_*` still apply) | NotSupported | NotSupported |
| Gemini CLI 0.64.0 (`gemini`) | `GEMINI_CLI_HOME` | Proven | Supported (`GEMINI_CLI_HOME` names a directory *containing* `.gemini`; the agent creates the nested directory itself) | Unknown (requires a vendor session; `GEMINI_API_KEY`, `GOOGLE_API_KEY` and `GOOGLE_APPLICATION_CREDENTIALS` are the bypass variables) | Supported (history and tmp state moved into the target's `.gemini`) |
| GitHub Copilot CLI 1.0.98 (`copilot`) | `COPILOT_HOME` | Experimental | Unknown (the measured delta was empty at both watched locations, so the variable's effect is undetermined) | Unknown (storage not located; `GH_TOKEN` and `GITHUB_TOKEN` are the documented bypass variables found in the binary's strings — `COPILOT_GITHUB_TOKEN` is documented upstream but was absent from this version's strings) | Unknown (nothing separated: the measured delta was empty at both the target and the default `~/.copilot`) |
| OpenCode v2.0.32 (`opencode`) | `OPENCODE_CONFIG_DIR` | Proven | Supported (`service.json` followed the variable into the target; `OPENCODE_CONFIG` names a config *file* to read, not the directory the adapter sets) | Unknown (`auth.json` was never written by these unauthenticated launches, so where stored credentials live was not established; `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` are the bypass variables) | NotSupported (`opencode.db`, `shell/` and `opencode.log` stayed at `.local/share/opencode` under both mechanisms; no data-directory variable exists) |
| Pi 1.3.0 (`pi`) | `PI_CODING_AGENT_DIR` | Proven | Supported (`auth.json` and `models-store.json` moved into the target with no `.pi/agent` prefix — the variable names the config directory itself) | NotSupported (stored credentials are `auth.json` inside the config directory, which is exactly what the variable moves, so they travel with it and no separate credential store exists; `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` bypass the file entirely) | Supported (`sessions/` and `models-store.json` appeared under the mechanism, with nothing left at the default `.pi/agent`) |
| Kiro CLI 2.32.0 (`kiro`) | `KIRO_HOME` (launches `kiro-cli`) | Proven | Supported (`settings/cli.json` moved into the target; `~/.kiro` and `.local/share/kiro-cli` were absent at baseline) | Unknown (no credential file appeared in the delta; `KIRO_API_KEY` is the documented bypass variable, and `AWS_ACCESS_KEY_ID`/`AWS_PROFILE` are also read) | NotGuaranteed (the data subsystem ignores the variable: `data.sqlite3`, `run-receipts/` and `telemetry-export-drops.lock` were written under `.local/share/kiro-cli` anyway) |
| Cursor Agent CLI 2026.10.16-e3cb207 (`cursor`) | `CURSOR_CONFIG_DIR` (launches `cursor-agent`) | Experimental | Supported (`cli-config.json` moved into the target; the watched XDG fallback `.config/cursor` stayed absent, so the fallback was not chosen) | Unknown (no login step ran and no credential-bearing file surfaced anywhere in the delta, so where stored credentials live was not established; `CURSOR_API_KEY` is the documented bypass variable) | Unknown (the agent wrote only `cli-config.json` before it needed network access, so no state was observable) |
| Continue CLI 1.5.47 (`continue`, launches `cn`) | `--config <profile>/config.yaml` | Experimental | Unknown (the measured delta at both watched locations was `(no change)` — the agent writes nothing before it needs the network — so whether the agent's own config writes would follow the configured file was not observed) | Unknown (no login step ran and no credential-bearing file surfaced anywhere in the delta; `CONTINUE_API_KEY`, `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` are the documented bypass variables) | Unknown (the same empty delta: the only pre-network write, `logs/cn.log` under the default `~/.continue`, predated the launch) |
| Amp 0.0.1792224035-gabd4f6 (`amp`) | `--settings-file <profile>/settings.json` | Proven | NotGuaranteed (the flag was accepted, but the agent's own writes did not follow it: `device-id.json` was written at the default `.local/share/amp`, the only observed write target, and `.config/amp` was absent at baseline and stayed so) | Unknown (no credential file was observed anywhere in the delta and no login step ran; `AMP_API_KEY` is the documented bypass variable) | NotGuaranteed (`accounts.json` and `device-id.json` were written at the default `.local/share/amp` while pointed at the flag's target — the device identity and account store do not move with `--settings-file`) |
| Cline CLI 3.0.73 (`cline`) | `CLINE_DATA_DIR` | Experimental | Unknown (the probe's delta at both watched locations — the target and the default `~/.cline` — was `(no change)`, so whether the agent's own config writes would follow the variable was not observable while it writes nothing before it needs the network) | Unknown (no login step ran and no credential-bearing file surfaced anywhere in the delta; `CLINE_API_KEY`, `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` and `OPENROUTER_API_KEY` are the documented bypass variables) | Unknown (the only pre-network write, `cli-node-extra-ca-certs.pem`, predated the launch, and the measured delta was `(no change)`, so no state write was observable to place) |

A new Codex profile starts logged out. Arguments that select the same mechanism (Aider's `-c`, `--config`
and its abbreviations) are refused before launch. On Windows an agent must be a native executable (`.exe`, or a configured `.com`): an npm or pnpm
`.cmd` shim is refused, and the error names the `[agents.<id>] executable` setting to use instead.

**On Windows, two limits compound for the npm-installed agents.** The executable is a shim, and a shim is
refused with a hint rather than parsed, so `executable` must be set in the configuration. Meanwhile the
probes run in a Linux container, so the evidence behind every claim above is Linux evidence. Neither fact
is hidden by the other: an agent can be both refused by default on Windows *and* unmeasured there.

## Planned adapters — not yet implemented or evidence-verified

The mechanisms below are the specification's starting point, not a claim about isolation. Each is cited
from first-party documentation and none is yet measured; where a probe contradicts a row, the probe wins.

_No adapters are currently planned: SP4b measured and registered the nine candidates from the design's §8 table, and every one landed._

## Agents that cannot be isolated

An agent belongs here when it offers no mechanism to relocate what it stores, so this tool cannot give it
a profile. The distinction matters: an adapter that isolates *little* is useful and honest — `aider`
ships one, declaring two of its three capabilities `NotSupported` — because its mechanism still moves
something. An agent with no mechanism at all would get a profile directory nothing ever reads, and a user
who sees a profile directory reasonably concludes they have isolation they do not have.

| Agent | Why not | Measured |
|---|---|---|
| Antigravity CLI (`agy`) | No environment variable, flag or settings key relocates its configuration, and its credentials are in the OS keyring rather than a file | 2026-09-22, `agy` 1.2.7 |

Antigravity stores settings, session state and skills under `~/.gemini/antigravity-cli/`, inside the
directory Gemini CLI uses. Sharing that parent is deliberate: the Antigravity IDE, SDK and CLI use it for
global rules, plugins, skills and authentication, so isolating one of them would cut it off from the
others by design rather than by oversight.

Measured in a disposable container, each with a positive and a negative control: `AGY_HOME`,
`ANTIGRAVITY_HOME`, `AGY_CONFIG`, `ANTIGRAVITY_CONFIG_DIR` and `GEMINI_CLI_HOME` are absent from the
binary; `XDG_CONFIG_HOME` and `XDG_DATA_HOME` are present but relocate nothing. Only overriding `HOME`
moves its files, and that is not a mechanism this tool will use for a coding agent: with `HOME`
relocated, `git commit` exits 128 with no author identity and `ssh` exits 255 on host-key verification,
so the agent loses the ability to commit or reach a remote. Credentials would remain shared regardless,
because the keyring is not a directory.

This row is a statement about a measured version, not a permanent judgement. If Antigravity gains a
configuration-directory mechanism, it becomes an ordinary adapter candidate.

## Workspace

| Crate | Responsibility |
|---|---|
| `agent-profile` | Library (naming, configuration, repository discovery, resolution, adapters, launcher, CLI, output) and the `agent-profile` binary |
| `agent-profile` → `fake-agent` binary | Test-only stand-in for a coding agent (`src/bin/fake-agent.rs`); never shipped |

## Building

Requires Rust 1.98+ (edition 2024).

```
cargo build --workspace
just check              # the local gate: fmt + clippy + typos + test
```

Tooling is listed in [`docs/dev-tooling.md`](docs/dev-tooling.md); see
[`CONTRIBUTING.md`](CONTRIBUTING.md) to get set up.

## Roadmap

See [`ROADMAP.md`](ROADMAP.md) for the sub-project plan and [`TODO.md`](TODO.md) for what is
immediately next.

## Licence

[PolyForm Noncommercial License 1.0.0](LICENSE). Noncommercial use only.
