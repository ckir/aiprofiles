# Justfile for Agent Profile

# Default: run the local gate
default:
    just check

# Build the workspace
build:
    cargo build --workspace

# Run all tests
test:
    cargo nextest run --workspace --no-tests=pass
    cargo test --doc --workspace

# Run tests without stopping at the first failure
test-verbose:
    cargo nextest run --workspace --no-tests=pass --no-fail-fast

# Clippy, warnings are errors
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Format check
fmt-check:
    cargo fmt --check

# Format all code
fmt:
    cargo fmt --all

# Spell check
typos:
    typos

# Dependency advisories, licences, bans and sources
deny:
    cargo deny check

# Every shell file in the repository. SP4a took this from three files to seventeen, and the harness is
# now load-bearing: it decides what an evidence transcript says. `-s sh` because these run under the
# container's /bin/sh, not bash, and `find` rather than a glob because an unmatched glob passes through
# literally and would hand shellcheck a filename that does not exist.
shellcheck:
    find sandbox -name '*.sh' -exec shellcheck -s sh {} +

# The sandbox harness's own tests: probe steps, transcript assembly, matrix resolution.
# No container and no agent needed, so these run in the ordinary gate rather than in the Sandbox workflow.
probe-tests:
    sh sandbox/tests/probe-harness.sh

# The local gate: fmt + clippy + typos + shellcheck + test + the probe harness
check: fmt-check clippy typos shellcheck test probe-tests

# Background watcher
watch:
    bacon

# Run the test suite in a disposable container (Podman or Docker; see CONTRIBUTING.md)
sandbox-test:
    sh sandbox/run.sh test

# Install and probe one real agent in a disposable container: just probe claude
probe AGENT:
    sh sandbox/run.sh probe {{AGENT}}

# Interactive shell in a disposable container, for writing a probe
sandbox-shell:
    sh sandbox/run.sh --net shell

# Build the docs
doc:
    cargo doc --workspace --no-deps --open

# Install the git hooks (once per clone)
hooks:
    lefthook install

# Mutation testing over the library
mutants:
    cargo mutants --package agent-profile

# Clean build artifacts
clean:
    cargo clean
