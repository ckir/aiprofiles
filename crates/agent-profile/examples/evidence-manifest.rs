//! Prints one `<id> <upstream_version>` line per real adapter, in registry order.
//!
//! This exists for §5.3 step 1, which requires the verification job to take the agent id and the version
//! **from the registry, never by parsing the filename**. The filename-to-id map is not invertible:
//! `upstream_version` permits `-` (§5), so `cursor-1.0.0-beta.1.md` splits two ways, and the wrong split
//! silently verifies the wrong transcript.
//!
//! It is an example rather than a binary because it is a build-time tool, not a shipped surface: examples
//! are covered by `cargo clippy --all-targets` but never land in a release archive.
//!
//! `REAL_ADAPTERS` rather than `registry()`: the `fake` adapter exists only under debug assertions
//! (`adapter/mod.rs:119-121`) and has no evidence to verify.

use agent_profile::adapter::REAL_ADAPTERS;

fn main() {
    for adapter in REAL_ADAPTERS {
        let metadata = adapter.metadata();
        println!("{} {}", metadata.id, metadata.evidence.upstream_version);
    }
}
