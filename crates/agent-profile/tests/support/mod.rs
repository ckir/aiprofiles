//! Shared helpers for integration tests (SP0 design §3.4).

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

/// Path to the `fake-agent` fixture binary, built on first use.
///
/// `CARGO_BIN_EXE_*` only covers binaries of this package, so the fixture — a separate package —
/// is built here with `cargo build -p fake-agent`. The cache is per test process; concurrent
/// builds from parallel test processes are serialized by cargo's build-directory lock.
pub fn fake_agent_path() -> PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(build_fake_agent).clone()
}

fn build_fake_agent() -> PathBuf {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(&cargo)
        .args(["build", "-p", "fake-agent", "--message-format=json"])
        .output()
        .unwrap_or_else(|e| panic!("failed to run {cargo:?}: {e}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "cargo build -p fake-agent failed\n{stdout}\n{stderr}");

    let mut executables = Vec::new();
    for line in stdout.lines() {
        let message: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("unparsable cargo message {line:?}: {e}\n{stderr}"));
        let is_bin = message["target"]["kind"]
            .as_array()
            .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"));
        let selected = message["reason"] == "compiler-artifact"
            && message["target"]["name"] == "fake-agent"
            && is_bin;
        // No let-chains here: they need Rust 1.88 and the workspace MSRV is 1.85.
        if let (true, Some(executable)) = (selected, message["executable"].as_str()) {
            executables.push(PathBuf::from(executable));
        }
    }
    assert_eq!(
        executables.len(),
        1,
        "expected exactly one fake-agent executable\n{stdout}\n{stderr}"
    );
    executables.remove(0)
}
