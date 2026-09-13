//! SP0 smoke tests: the stub binary's contract and the fake-agent fixture's contract.

mod support;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn agent_profile(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-profile")).args(args).output().unwrap()
}

fn stdout_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

/// Asserts the fake-agent fixture-error contract: exit 125, nothing on stdout, a message on stderr.
fn assert_fixture_error(output: &Output, context: &str) {
    assert_eq!(output.status.code(), Some(125), "{context}");
    assert!(output.stdout.is_empty(), "{context}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("fake-agent: "), "{context}: {stderr}");
}

#[test]
fn version_exits_zero() {
    let output = agent_profile(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "{stdout}");
}

#[test]
fn help_exits_zero_and_says_scaffold() {
    let output = agent_profile(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("SP0 scaffold"), "{stdout}");
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

#[test]
fn fake_agent_echoes_argv_exactly() {
    let args = ["--foo", "bar", "--", "a b"];
    let output = support::fake_agent().args(args).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["argv"], serde_json::json!(args));
}

#[test]
fn fake_agent_exit_code_is_controllable() {
    let output = support::fake_agent().env("FAKE_AGENT_EXIT", "7").output().unwrap();
    assert_eq!(output.status.code(), Some(7));
    // A requested non-zero exit still prints the full report.
    assert_eq!(stdout_json(&output)["argv"], serde_json::json!([]));
}

#[test]
fn fake_agent_echoes_cwd() {
    // Not the package directory, which is where the test runner already starts the child.
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let output = support::fake_agent().current_dir(&dir).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    let cwd = stdout_json(&output)["cwd"].as_str().map(PathBuf::from).unwrap();
    assert_eq!(cwd.canonicalize().unwrap(), dir.canonicalize().unwrap());
}

#[test]
fn fake_agent_echoes_only_listed_env() {
    let output = support::fake_agent()
        .env("FAKE_AGENT_ECHO_ENV", "AP_TEST_SET,,AP_TEST_UNSET")
        .env("AP_TEST_SET", "yes")
        .env("AP_TEST_NOT_LISTED", "no")
        .env_remove("AP_TEST_UNSET")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["env"], serde_json::json!({ "AP_TEST_SET": "yes" }));
}

#[test]
fn fake_agent_env_is_empty_without_echo_list() {
    let output = support::fake_agent().env_remove("FAKE_AGENT_ECHO_ENV").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["env"], serde_json::json!({}));
}

#[test]
fn fake_agent_invalid_exit_code_is_fixture_error() {
    for value in ["", "256", "-1", "seven"] {
        let output = support::fake_agent().env("FAKE_AGENT_EXIT", value).output().unwrap();
        assert_fixture_error(&output, &format!("FAKE_AGENT_EXIT={value:?}"));
    }
}

#[test]
fn fake_agent_non_utf8_argument_is_fixture_error() {
    let output = support::fake_agent().arg(non_utf8()).output().unwrap();
    assert_fixture_error(&output, "non-UTF-8 argument");
}

#[test]
fn fake_agent_non_utf8_env_value_is_fixture_error() {
    let output = support::fake_agent()
        .env("FAKE_AGENT_ECHO_ENV", "AP_TEST_NON_UTF8")
        .env("AP_TEST_NON_UTF8", non_utf8())
        .output()
        .unwrap();
    assert_fixture_error(&output, "non-UTF-8 echoed env value");
}

#[test]
fn fake_agent_helper_clears_fixture_env() {
    let command = support::fake_agent();
    let envs: BTreeMap<&OsStr, Option<&OsStr>> = command.get_envs().collect();
    for name in ["FAKE_AGENT_EXIT", "FAKE_AGENT_ECHO_ENV"] {
        assert_eq!(envs.get(OsStr::new(name)), Some(&None), "{name} must be removed");
    }
}

#[cfg(unix)]
fn non_utf8() -> std::ffi::OsString {
    use std::os::unix::ffi::OsStringExt;
    std::ffi::OsString::from_vec(vec![0x66, 0xff])
}

#[cfg(windows)]
fn non_utf8() -> std::ffi::OsString {
    use std::os::windows::ffi::OsStringExt;
    // A lone surrogate: valid in an OS string, not valid UTF-8.
    std::ffi::OsString::from_wide(&[0x0066, 0xD800])
}
