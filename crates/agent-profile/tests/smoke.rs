//! Smoke tests: the binary's top-level contract and the fake-agent fixture's contract.

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
fn help_exits_zero_and_shows_launch_usage() {
    let output = agent_profile(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("agent-profile <agent> <profile>"), "{stdout}");
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
    let output = support::fake_agent().arg(support::non_utf8()).output().unwrap();
    assert_fixture_error(&output, "non-UTF-8 argument");
}

#[test]
fn fake_agent_non_utf8_env_value_is_fixture_error() {
    let output = support::fake_agent()
        .env("FAKE_AGENT_ECHO_ENV", "AP_TEST_NON_UTF8")
        .env("AP_TEST_NON_UTF8", support::non_utf8())
        .output()
        .unwrap();
    assert_fixture_error(&output, "non-UTF-8 echoed env value");
}

#[test]
fn fake_agent_helper_clears_fixture_env() {
    let command = support::fake_agent();
    let envs: BTreeMap<&OsStr, Option<&OsStr>> = command.get_envs().collect();
    for name in support::FIXTURE_VARS {
        assert_eq!(envs.get(OsStr::new(name)), Some(&None), "{name} must be removed");
    }
}

#[test]
fn fake_agent_reports_its_pid() {
    let child = support::fake_agent().stdout(std::process::Stdio::piped()).spawn().unwrap();
    let pid = child.id();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["pid"], serde_json::json!(pid));
}

#[test]
fn fake_agent_reads_stdin_when_asked() {
    use std::io::Write;
    let mut child = support::fake_agent()
        .env("FAKE_AGENT_STDIN", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"line one\nline two").unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_json(&output)["stdin"], serde_json::json!("line one\nline two"));
}

#[test]
fn fake_agent_writes_requested_stderr_after_the_report() {
    let output = support::fake_agent().env("FAKE_AGENT_STDERR", "to stderr").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"to stderr");
    assert!(stdout_json(&output).get("argv").is_some());
}

#[test]
fn fake_agent_sleeps_after_reporting() {
    let started = std::time::Instant::now();
    let output = support::fake_agent().env("FAKE_AGENT_SLEEP_MS", "300").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(started.elapsed() >= std::time::Duration::from_millis(300));
    assert!(stdout_json(&output).get("pid").is_some());
}

#[test]
fn fake_agent_reports_a_distinct_sleeper_pid() {
    let output = support::fake_agent().env("FAKE_AGENT_SPAWN_SLEEPER", "200").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    let report = stdout_json(&output);
    let sleeper = report["sleeper_pid"].as_u64().unwrap();
    assert_ne!(Some(sleeper), report["pid"].as_u64());
}

#[test]
fn fake_agent_invalid_control_values_are_fixture_errors() {
    for (name, value) in [
        ("FAKE_AGENT_STDIN", "yes"),
        ("FAKE_AGENT_SLEEP_MS", "-1"),
        ("FAKE_AGENT_SLEEP_MS", "soon"),
        ("FAKE_AGENT_SPAWN_SLEEPER", ""),
        ("FAKE_AGENT_CTRL_C_EXIT", "256"),
        ("FAKE_AGENT_BREAKAWAY", "yes"),
    ] {
        let output = support::fake_agent().env(name, value).output().unwrap();
        assert_fixture_error(&output, &format!("{name}={value:?}"));
    }
}

#[cfg(unix)]
#[test]
fn fake_agent_ctrl_c_exit_is_windows_only() {
    let output = support::fake_agent().env("FAKE_AGENT_CTRL_C_EXIT", "42").output().unwrap();
    assert_fixture_error(&output, "FAKE_AGENT_CTRL_C_EXIT on Unix");
}
