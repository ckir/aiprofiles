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

# The local gate: fmt + clippy + typos + test
check: fmt-check clippy typos test

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

# Generate the changelog
changelog:
    git-cliff --output CHANGELOG.md

# Mutation testing over the library
mutants:
    cargo mutants --package agent-profile

# Clean build artifacts
clean:
    cargo clean

# Release: bump every crate in lockstep, tag, commit
# Usage: just release <patch|minor|major>
release VERSION_BUMP:
    cargo release {{VERSION_BUMP}} --workspace --execute
