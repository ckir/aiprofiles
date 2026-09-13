//! SP0 smoke tests: the stub binary's contract and the fake-agent fixture's contract.

use std::process::{Command, Output};

fn agent_profile(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-profile")).args(args).output().unwrap()
}

#[test]
fn version_exits_zero() {
    let output = agent_profile(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "{stdout}");
}

#[test]
fn unimplemented_invocation_is_usage_error() {
    for args in [&["doctor"][..], &["claude", "work"], &["zzz-unknown"]] {
        let output = agent_profile(args);
        assert_eq!(output.status.code(), Some(2), "args {args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("not yet implemented"), "args {args:?}: {stderr}");
    }
}
