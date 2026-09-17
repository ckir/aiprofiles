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
    find sandbox scripts -name '*.sh' -exec shellcheck -s sh {} +

# Fails if a comment in sandbox/**/*.sh or crates/**/*.rs cites another file by line number, which rots
# the moment either file is edited. See scripts/check-line-citations.sh for the escape hatch.
line-citations:
    sh scripts/check-line-citations.sh

# The sandbox harness's own tests: probe steps, transcript assembly, matrix resolution.
# No container and no agent needed, so these run in the ordinary gate rather than in the Sandbox workflow.
probe-tests:
    sh sandbox/tests/probe-harness.sh
    sh sandbox/tests/transcript.sh
    sh sandbox/tests/resolve-agents.sh
    sh sandbox/tests/verify-transcripts.sh

# Regenerate the `## Tools` table in docs/dev-tooling.md from .claude/recommended-tools.json
tools-doc:
    sh scripts/gen-tool-table.sh

# Fails if docs/dev-tooling.md's `## Tools` table has drifted from .claude/recommended-tools.json
tools-doc-check:
    sh scripts/gen-tool-table.sh --check

# The local gate: fmt + clippy + typos + shellcheck + line-citations + test + the probe harness +
# tools-doc freshness
check: fmt-check clippy typos shellcheck line-citations test probe-tests tools-doc-check

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
