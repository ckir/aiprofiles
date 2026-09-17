# Agent Profile dev tooling

The toolchain is carried over from `flux` (`E:\Rust\flux`) and audited against what Agent Profile
needs. Agent Profile is an offline cross-platform CLI wrapper, so flux's TLA+ model-checking and
benchmarking tooling is dropped (see "Deliberately not carried over").

Every tool runs through a `just` recipe, so the local gate, the git hook and CI cannot drift apart.
The same list is machine-readable in `.claude/recommended-tools.json`.

## Tools

| Tool | Config | `just` recipe | Gates |
|---|---|---|---|
| `rustup` / `rustc` / `cargo` | `rust-toolchain.toml` (`stable`, with rustfmt + clippy) | `build` | everything |
| `cargo-nextest` | — | `test`, `test-verbose` (`cargo nextest run --workspace --no-tests=pass` + `cargo test --doc --workspace`) | `just check`; CI Test job on ubuntu, macos, windows |
| `lefthook` | `lefthook.yml` (pre-push: fmt-check, clippy, typos in parallel; no pre-commit) | `hooks` | local pre-push |
| `release-plz` (GitHub Action) | `release-plz.toml` (`git_only`, release PR, `v{{ version }}` tags, pre-releases until SP5) | — | merging the release PR releases |
| `rustfmt` | `rustfmt.toml` (edition 2024, width 100) | `fmt`, `fmt-check` | `just check`, pre-push, CI Format |
| `clippy` | `clippy.toml` (msrv 1.98) | `clippy` (`--workspace --all-targets -- -D warnings`) | `just check`, pre-push, CI Clippy |
| `typos` (typos-cli) | `_typos.toml` (excludes the V3 spec) | `typos` | `just check`, pre-push, CI Typos |
| `cargo-deny` | `deny.toml` (licence allow-list, `openssl-sys` ban per V3 §36, 7 target triples) | `deny` | CI Cargo deny |
| `bacon` | `bacon.toml` (default job `check-all`) | `watch` | local |
| `cargo-mutants` | — | `mutants` (`--package agent-profile`) | on demand |
| Podman or Docker | `sandbox/Containerfile`, `sandbox/run.sh`, `sandbox/probes/` | `sandbox-test`, `probe <agent>`, `sandbox-shell` | on demand; `.github/workflows/sandbox.yml` runs the same on a GitHub runner with **podman** (probes by hand; `test` on PRs that change the harness) |
| `actionlint` + `shellcheck` | — | — | workflow linting before pushing `.github/` changes |
| `cargo-binstall` | — | — | installs the cargo tools above |
| GitHub Actions | `.github/workflows/ci.yml` | — | Format, Typos, Clippy, Cargo deny, Docs build, Test ×3 OS — all required checks on `main` |
| GitHub Actions | `.github/workflows/docs.yml` | — | publishes rustdoc to Pages after merge |
| GitHub Actions | `.github/workflows/release-plz.yml`, `.github/workflows/release.yml` | — | release PR on push to `main`; on its merge, tag, GitHub release and cross-platform `agent-profile` binaries |
| GitHub Actions | `.github/workflows/pr-title.yml` | — | Conventional PR title (required check; PRs are squash-merged) |
| Dependabot | `.github/dependabot.yml`, `.github/workflows/dependabot-automerge.yml` | — | weekly grouped minor/patch PRs, auto-merged once required checks pass |

## Install

```bash
cargo binstall -y cargo-nextest just lefthook cargo-deny typos-cli bacon cargo-mutants
winget install rhysd.actionlint
winget install koalaman.shellcheck
lefthook install
```

On Windows, winget installs `actionlint` and `shellcheck` under
`%LOCALAPPDATA%\Microsoft\WinGet\Packages\`. If Git Bash cannot find them, add those package
directories to `PATH`.

A container engine is not in that list because nothing in `just check` needs one — only the sandbox
does. Install one when you touch `sandbox/`:

```bash
winget install RedHat.Podman     # or Docker Desktop
```

### Podman and Docker are not interchangeable here

`sandbox/run.sh` picks podman when it is installed and falls back to Docker, and either runs a probe.
But `run.sh` sets `--userns=keep-id:uid=1000,gid=1000` **only under podman**, and the sandbox workflow
sets `AGENT_PROFILE_SANDBOX_ENGINE: podman` in both jobs. That mapping admits one id, while the
container runs two unprivileged users — the harness and the measured agent. So anything about uid
mapping across the `/out` bind mount behaves differently under Docker and cannot be reproduced there.

Use Docker for ordinary sandbox work. For a uid-mapping question, prefer dispatching the Sandbox
workflow, which is the environment CI actually uses; a Windows podman machine is a third environment
again, not the Linux runner. Reach for a local podman at the point where you would otherwise be
re-pushing to CI to iterate.

## Gate commands (the contract)

```
just check          # fmt-check + clippy + typos + test — the local gate
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
typos
cargo nextest run --workspace --no-tests=pass
cargo test --doc --workspace    # nextest does not run doctests
cargo deny check
```

## Deliberately not carried over from flux

- `models/`, `.github/workflows/model.yml`, and the `java` and `python3` tool entries: flux model-checks
  a lock protocol with TLA+. Nothing comparable exists here.
- `benches/` and `criterion`: the V3 spec asks for no performance work.

## Added beyond flux

- The `Docs build` CI job: flux builds docs only after merge, so a PR could merge with a broken docs
  build. Here it is a required PR check.
