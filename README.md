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
| Claude Code 2.1.270 (`claude`) | `CLAUDE_CONFIG_DIR` | Proven | Supported (project `.claude/` settings still layer on top) | Conditional (`ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `CLAUDE_CODE_OAUTH_TOKEN` and similar variables bypass it) | NotGuaranteed (history and project state moving with the directory is community-sourced only) |
| Codex CLI 0.153.4 (`codex`) | `CODEX_HOME` | Proven | Supported (project-level configuration layering is not measured) | Conditional (`OPENAI_API_KEY`, `CODEX_API_KEY`, `CODEX_ACCESS_TOKEN` bypass it) | Conditional (`CODEX_SQLITE_HOME`) |
| Aider 0.86.2 (`aider`) | `--config <profile>/.aider.conf.yml` | Proven | NotGuaranteed (home, repository and working-directory `.aider.conf.yml`, `.env` and `AIDER_*` still apply) | NotSupported | NotSupported |

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

| Agent | Primary mechanism | Intended semantic tier |
|---|---|---|
| Gemini CLI | `GEMINI_CLI_HOME` | Home/state isolation |
| GitHub Copilot CLI | `COPILOT_HOME` | Home/config isolation |
| OpenCode | `OPENCODE_CONFIG_DIR` / `OPENCODE_CONFIG` | Config/home isolation |
| Cline CLI | `--config`, `--data-dir` | Config/state selection |
| Pi | `PI_CODING_AGENT_DIR` | Agent home/state isolation |
| Kiro CLI | `KIRO_HOME` | Independent home/profile |
| Cursor Agent CLI | `CURSOR_CONFIG_DIR` | Configuration selection |
| Continue CLI | `--config` | Configuration selection |
| Amp | `--settings-file` | Settings selection |

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
