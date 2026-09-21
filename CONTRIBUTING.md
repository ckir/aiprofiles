# Contributing to Agent Profile

Thanks for your interest. This document covers the basics.

Agent Profile is built against an authoritative implementation specification,
[`agent-profile-implementation-spec-v3.md`](agent-profile-implementation-spec-v3.md). **The spec is the
oracle.** If the code and the spec disagree, that is a bug in the code — or a change that needs to be
made to the spec first, deliberately. Please cite the relevant section (e.g. "spec §12") in issues and
pull requests.

## Getting started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/aiprofiles.git`
3. Install the tooling (see below) and run `lefthook install`
4. Create a branch: `git checkout -b my-feature`
5. Make your changes
6. Run the gate: `just check`
7. Commit and push: `git commit -m "feat: my feature" && git push origin my-feature`
8. Open a pull request

## Development setup

Requires Rust 1.98+ (edition 2024). The toolchain is pinned by `rust-toolchain.toml`.

```bash
# One-time: install the dev tools
cargo binstall -y cargo-nextest just lefthook cargo-deny typos-cli bacon cargo-mutants
lefthook install

# Everyday
just build      # cargo build --workspace
just check      # the gate: fmt-check + clippy + typos + test
just watch      # bacon, in the background, while you work
just test       # nextest + doctests
just doc        # build and open the API docs
```

Every tool, what it gates and which config file drives it is documented in
[`docs/dev-tooling.md`](docs/dev-tooling.md). The same list is machine-checked from
`.claude/recommended-tools.json`.

## The gate

`just check` is exactly what CI runs. It must be green before you open a PR:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
typos
cargo nextest run --workspace --no-tests=pass
cargo test --doc --workspace       # nextest does not run doctests
```

CI additionally runs `cargo deny check`, builds the docs with warnings as errors, and runs the test job
on Linux, macOS **and** Windows. Agent Profile launches processes, handles console signals and
canonicalizes repository paths — all of which behave differently per platform — so a change that
passes on one OS is not evidence it passes on the others.

## Code style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- `cargo fmt --all` before committing
- Fix every clippy warning; the gate treats them as errors
- All public items get `///` documentation
- Cite the spec section in doc comments for anything implementing a normative rule

## Testing

Spec §34 defines the mandatory contract suite: profile names, resolution precedence, configuration
corruption and atomicity, LaunchPlan per adapter, opaque passthrough, environment overrides and
process behaviour. New behaviour should land with the matching test from that list, and
platform-specific behaviour needs the test on the platform it concerns.

Tests that launch an "agent" use the `fake-agent` fixture (`crates/agent-profile/src/bin/fake-agent.rs`),
launched through `crates/agent-profile/tests/support`. Never launch a real coding agent from the test suite;
real agents run only in the disposable sandbox described under "Measuring agent behaviour".

Every adapter has a row in `crates/agent-profile/tests/adapter_contract.rs` and an end-to-end launch in
`crates/agent-profile/tests/adapters_e2e.rs`; a new adapter without its row fails the suite.

## Measuring agent behaviour

Adapter evidence (`AdapterEvidence` in `crates/agent-profile/src/adapter/`) is refreshed only from measurements
of the real agent, and those measurements run **inside a disposable sandbox, never on your own machine**.
Nobody wants their machine filling up with coding agents they do not use, and a sandbox also behaves like a
clean install. Agents you already use may be probed read-only on the host (`--help`, `--version`).

**Windows: Sandboxie-Plus** (`winget install Sandboxie.Plus`).

```powershell
$sbie = 'C:\Program Files\Sandboxie-Plus'
& "$sbie\SbieIni.exe" set AgentProbe Enabled y
# Hide your real agent homes so the box behaves like a clean machine; add any other agent home you have.
& "$sbie\SbieIni.exe" append AgentProbe ClosedFilePath '%USERPROFILE%\.claude'
& "$sbie\SbieIni.exe" append AgentProbe ClosedFilePath '%USERPROFILE%\.codex'
& "$sbie\Start.exe" /reload
# Run a script inside the box; its exit code comes back, its writes stay in the box.
& "$sbie\Start.exe" /box:AgentProbe /wait cmd /c "C:\path\to\probe.cmd > C:\m\out.txt 2>&1"
Get-Content 'C:\Sandbox\<user>\AgentProbe\drive\C\m\out.txt'
# Delete everything the box installed, then remove the box.
& "$sbie\Start.exe" /box:AgentProbe /terminate
& "$sbie\Start.exe" /box:AgentProbe delete_sandbox_silent
& "$sbie\SbieIni.exe" set AgentProbe '*' ''
& "$sbie\Start.exe" /reload
```

Pin interpreter versions an agent supports (for example `uv tool install --python 3.12 aider-chat`): an
unsupported interpreter can start a long source build of native dependencies.

**Linux (and anywhere Podman or Docker runs): the container harness.** `sandbox/run.sh` builds a disposable
image (`sandbox/Containerfile`: Rust, Node/npm, Python/uv, no agent), runs one workload in it, and removes the
container and the image afterwards, also after Ctrl-C or a closed terminal. With local rootless Podman (Linux)
each run also uses its own temporary image store under `$TMPDIR` or `/var/tmp`, deleted at the end, so no image,
layer or cache survives; only a `kill -9` of the script can leave a store behind (remove it with
`podman unshare rm -rf /var/tmp/agent-profile-sandbox.*`). With Docker, and with remote Podman such as Podman
Desktop on macOS, the pulled base image and the build cache (toolchains, never agents) stay in the engine's store;
remove them with `docker builder prune` and `docker image rm` on the pinned `debian@sha256:…` base from
`sandbox/Containerfile` (or the `podman` equivalents).
Your checkout is mounted read-only and copied inside.

```bash
just probe claude     # sandbox/run.sh probe claude: install the agent inside, record version, help and a dry run
just sandbox-test     # sandbox/run.sh test: cargo nextest run --workspace in a clean container
just sandbox-shell    # sandbox/run.sh --net shell: interactive shell with network, to write a new probe
```

Results land in `target/sandbox/<mode>[-<agent>]-<timestamp>/`: `build.log`, `exit-code`, `output.log` (not for
`shell`), `diff.txt` (files the run added, changed or deleted in the container) and the probe's own files. The
image is built for the host's architecture (x86_64 or aarch64).

A probe is a short script in `sandbox/probes/<agent>.sh` built on `sandbox/probes/common.sh`, and it runs six
steps **in this order**, because the order is what makes the result attributable:

```sh
. sandbox/probes/common.sh

probe_npm_install @example/agent          # 1. the only step that runs vendor code before a snapshot exists
probe_version example                     # 2. --version, verbatim, plus the token the registry will carry
probe_help example                        # 3. --help, where the mechanism token is read from
probe_strings example EXAMPLE_API_KEY     # 4. which credential variables are really in the binary
probe_behaviour example "$HOME/.example" env:EXAMPLE_HOME @none   # 5 and 6: baseline, then apply and diff
```

Step 5 is inside `probe_behaviour` and is not optional: it baselines **both** the profile target and the agent's
default location, after the install and after anything the probe itself created. Skip it and the install's own
files get attributed to the agent, which reads as isolation that did not happen. Repeat `probe_behaviour` once per
mechanism under test; each repetition re-baselines, because a second launch measured against the first launch's
baseline is not attributable.

A probe never supplies a credential, never waits for input, and never launches the agent through `agent-profile` —
the CLI refuses an agent word it does not know, so a probe written that way silently measures nothing.

`sandbox/tests/` holds the harness's own tests. They need no container and no agent, and `just check` runs them.

**No container engine? Use CI.** **Actions → Sandbox → Run workflow** runs the same script on a fresh GitHub
runner, whose virtual machine is discarded afterwards. Name one agent, several, or `all`; each gets its own job.
Two artifacts come back per agent: `sandbox-transcript-<id>`, holding the finished evidence transcript, and
`sandbox-logs-<id>`, holding the raw logs and container diff. A `test`-mode run uploads `sandbox-test-results`
instead. Probes run only when triggered by hand; a pull request that changes the harness runs it in `test` mode
only (the crate's test suite, no agent), so pull-request CI never installs or runs a real agent. **It uses no
secrets, and none may be added** — a probe executes third-party installers, which is precisely the job that must
never hold a credential.

**macOS:** the harness runs under Docker Desktop, OrbStack, Colima or Podman Desktop for behaviour that does not
depend on macOS, with the base image and build cache kept in that engine's store as described above; use Tart
disposable macOS virtual machines when the behaviour does depend on macOS (for example the Keychain).
`sandbox-exec` is deprecated, so it is not a substitute.

Record each measurement in the adapter's `AdapterEvidence` (`verified_at`, `upstream_version`, `source_url`,
`notes`), in the design document's decision table, and — for anything a probe measured — as a committed transcript
under `docs/evidence/`. See [docs/evidence/README.md](docs/evidence/README.md) for what a transcript must contain
and how CI binds it to the run that produced it.

## Adding an adapter

An adapter is a small file in `crates/agent-profile/src/adapter/`, but most of the work is the evidence behind it.
In order:

1. **Write the probe first.** `sandbox/probes/<id>.sh`, as above. Run it and read what comes back; the mechanism
   the documentation describes and the one the agent implements are not always the same, and it is cheaper to find
   that out now than after the adapter is written around the wrong one.
2. **Commit the transcript**, byte for byte, from the CI artifact. Never hand-edited.
3. **Write the adapter**, with one `CapabilityClaim` per capability. Every `basis` starts with `measured:`,
   `cited:` or `unmeasured:`, and `unmeasured:` is required exactly when the state is `Unknown` — the contract
   suite enforces the biconditional, so an `Unknown` with a confident-sounding basis will not build.
4. **Register it** in `REAL_ADAPTERS` and add its contract and end-to-end rows.
5. **Add its README row**, with the capability states beside the support level. A support level on its own
   overclaims: `proven` describes the mechanism, not the account, and the two are only safe to print together.

The gates in `src/adapter/gate.rs` are the mechanical part of this and they will tell you what is missing. What
they cannot tell you is whether a claim is honest, which is what steps 1 and 2 are for.

Claim the weakest state the evidence supports. `NotSupported` with a reason is a better adapter than `Supported`
with a hope, because a user can work around a stated limit and cannot work around a wrong promise.

## Commit messages

We follow [Conventional Commits](https://www.conventionalcommits.org/). Pull requests are squash-merged and
the repository's squash commit title is set to the PR title, so the **PR title** becomes the commit on `main`; a
required check (`Conventional PR title`) enforces its form, because release-plz chooses the next version and
writes the changelog from these commits:

```
type(scope): description

[optional body]

[optional footer(s)]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

Examples:
- `feat(resolve): prefer the agent-specific repository mapping`
- `fix(launch): return the child exit code on Windows`
- `docs: document the Claude Code credential caveat`

## Pull request process

1. CI must be green on all platforms
2. Update documentation if you changed behaviour
3. Add tests for new functionality
4. Reference the spec section and any related issues

## Reporting bugs

Open an issue with:
- Steps to reproduce
- Expected behaviour (cite the spec section if it defines one)
- Actual behaviour
- Environment: OS and version, the coding agent and its version, `agent-profile --version`
- The `--dry-run` output for the failing invocation, **with any secret values redacted**

For security issues, do **not** open a public issue; see [SECURITY.md](SECURITY.md).

## Releasing

Releases are automated with [release-plz](https://release-plz.dev) (`release-plz.toml`,
`.github/workflows/release-plz.yml`); nobody runs a release command locally.

1. Every push to `main` with a `feat`, `fix`, `perf` or `refactor` commit that changes files under
   `crates/agent-profile/` opens or updates a release PR titled `chore: release vX.Y.Z`, which bumps the
   workspace version and updates `crates/agent-profile/CHANGELOG.md`. Commits that touch only files outside
   the crate, including `Cargo.lock` and the root `Cargo.toml`, never open one; to ship such a change (for
   example a security bump of a dependency), follow it with a `fix:` PR that changes a file under
   `crates/agent-profile/`.
2. Review that PR like any other. Edits pushed to it survive only until the next push to `main`: release-plz
   then closes the PR and opens a fresh one without them. Merging it is the release.
3. The merge tags `vX.Y.Z`, creates a GitHub release, and builds and uploads the binaries for Linux (x86_64,
   aarch64), macOS (x86_64, aarch64) and Windows (x86_64).

Until V3 v0.1 is complete (SP5), versions stay `0.0.x` and every release is marked as a pre-release.

Recovery, all from the GitHub web UI:
- A binary is missing from a release: **Actions → Release binaries → Run workflow** with the release tag.
- The tag exists but the release does not (release creation failed after tagging; re-running the Release-plz
  run is a silent no-op then): create the pre-release for that tag under **Releases → Draft a new release**,
  publish it, then run **Release binaries** with the tag.

The release PR is opened with the `RELEASE_PLZ_TOKEN` repository secret: a fine-grained personal access token
for this repository with Contents and Pull requests read/write, so CI runs on the release PR. It expires; when
release PRs stop appearing, renew it.

## Licence

By contributing, you agree that your contributions will be licensed under the
[PolyForm Noncommercial License 1.0.0](LICENSE).
