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
| `git-cliff` | `cliff.toml` (conventional commits; Dependabot uses `chore`/`ci` prefixes so its commits are kept) | `changelog` | release time |
| `cargo-release` | `[workspace.metadata.release]` in `Cargo.toml` (lockstep, `v{{version}}` tag, `publish = false`) | `release <patch\|minor\|major>` | pushed tag triggers `release.yml` |
| `rustfmt` | `rustfmt.toml` (edition 2024, width 100) | `fmt`, `fmt-check` | `just check`, pre-push, CI Format |
| `clippy` | `clippy.toml` (msrv 1.98) | `clippy` (`--workspace --all-targets -- -D warnings`) | `just check`, pre-push, CI Clippy |
| `typos` (typos-cli) | `_typos.toml` (excludes the V3 spec) | `typos` | `just check`, pre-push, CI Typos |
| `cargo-deny` | `deny.toml` (licence allow-list, `openssl-sys` ban per V3 §36, 7 target triples) | `deny` | CI Cargo deny |
| `bacon` | `bacon.toml` (default job `check-all`) | `watch` | local |
| `cargo-mutants` | — | `mutants` (`--package agent-profile`) | on demand |
| `actionlint` + `shellcheck` | — | — | workflow linting before pushing `.github/` changes |
| `cargo-binstall` | — | — | installs the cargo tools above |
| GitHub Actions | `.github/workflows/ci.yml` | — | Format, Typos, Clippy, Cargo deny, Docs build, Test ×3 OS — all required checks on `main` |
| GitHub Actions | `.github/workflows/docs.yml` | — | publishes rustdoc to Pages after merge |
| GitHub Actions | `.github/workflows/release.yml` | — | `v*` tag → cross-platform `agent-profile` binaries |
| Dependabot | `.github/dependabot.yml`, `.github/workflows/dependabot-automerge.yml` | — | weekly grouped minor/patch PRs, auto-merged once required checks pass |

## Install

```bash
cargo binstall -y cargo-nextest just lefthook cargo-deny typos-cli bacon git-cliff cargo-release cargo-mutants
winget install rhysd.actionlint
winget install koalaman.shellcheck
lefthook install
```

On Windows, winget installs `actionlint` and `shellcheck` under
`%LOCALAPPDATA%\Microsoft\WinGet\Packages\`. If Git Bash cannot find them, add those package
directories to `PATH`.

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
