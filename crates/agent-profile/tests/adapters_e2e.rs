//! One end-to-end launch per real adapter through the `agent-profile` binary (SP2 design §8.2). The
//! `fake-agent` fixture is copied under each agent's executable name onto a `PATH` holding only that
//! directory, so no real agent ever runs.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};

use support::Root;

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A directory holding the fixture under `name` (with `.exe` on Windows).
fn bin_with(name: &str) -> tempfile::TempDir {
    let bin = tempfile::tempdir().unwrap();
    let file = if cfg!(windows) { format!("{name}.exe") } else { name.to_owned() };
    fs::copy(env!("CARGO_BIN_EXE_fake-agent"), bin.path().join(file)).unwrap();
    bin
}

fn profile_dir(root: &Root, agent: &str) -> PathBuf {
    root.path().join("profiles").join("work").join(agent)
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap()
}

#[test]
fn env_dir_agents_receive_their_home_argv_cwd_and_exit_code() {
    for (agent, var) in [("claude", "CLAUDE_CONFIG_DIR"), ("codex", "CODEX_HOME")] {
        let root = Root::empty();
        let bin = bin_with(agent);
        let cwd = tempfile::tempdir().unwrap();
        let output = root
            .agent_profile([agent, "work", "--", "--flag", "a b"])
            .env("PATH", bin.path())
            .env(var, "/wrong")
            .env("FAKE_AGENT_ECHO_ENV", var)
            .env("FAKE_AGENT_EXIT", "5")
            .current_dir(cwd.path())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(5), "{agent}: {}", stderr(&output));
        let report = support::report(&output.stdout);
        assert_eq!(report["argv"], serde_json::json!(["--flag", "a b"]), "{agent}");
        let dir = profile_dir(&root, agent);
        assert_eq!(report["env"][var], serde_json::json!(dir.to_str().unwrap()), "{agent}");
        assert_eq!(
            canonical(Path::new(report["cwd"].as_str().unwrap())),
            canonical(cwd.path()),
            "{agent}"
        );
        assert!(dir.is_dir(), "{agent}");
    }
}

#[test]
fn aider_receives_its_config_file_first_and_the_file_is_created() {
    let root = Root::empty();
    let bin = bin_with("aider");
    let cwd = tempfile::tempdir().unwrap();
    let output = root
        .agent_profile(["aider", "work", "--", "--model", "x"])
        .env("PATH", bin.path())
        .env("FAKE_AGENT_EXIT", "5")
        .current_dir(cwd.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(5), "{}", stderr(&output));
    let file = profile_dir(&root, "aider").join(".aider.conf.yml");
    let report = support::report(&output.stdout);
    assert_eq!(
        report["argv"],
        serde_json::json!(["--config", file.to_str().unwrap(), "--model", "x"])
    );
    assert_eq!(canonical(Path::new(report["cwd"].as_str().unwrap())), canonical(cwd.path()));
    assert_eq!(fs::read(&file).unwrap(), b"{}\n");
}

#[test]
fn aider_concurrent_first_launches_all_succeed() {
    let root = Root::empty();
    let bin = bin_with("aider");
    let children: Vec<_> = (0..8)
        .map(|_| {
            root.agent_profile(["aider", "work"])
                .env("PATH", bin.path())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    for child in children {
        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    }
    assert_eq!(fs::read(profile_dir(&root, "aider").join(".aider.conf.yml")).unwrap(), b"{}\n");
}

#[test]
fn aider_conflict_is_refused_before_anything_launches_or_is_created() {
    let root = Root::empty();
    let bin = bin_with("aider");
    let output = root
        .agent_profile(["aider", "work", "--", "--conf", "f"])
        .env("PATH", bin.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(output.stdout.is_empty(), "the agent must not run");
    assert!(
        stderr(&output)
            .contains("`--conf` conflicts with how agent-profile selects the aider profile"),
        "{}",
        stderr(&output)
    );
    assert!(!profile_dir(&root, "aider").exists());
}

#[test]
fn aider_conflict_is_reported_even_when_aider_is_not_installed() {
    let root = Root::empty();
    let output = root
        .agent_profile(["aider", "work", "--", "--conf", "f"])
        .env("PATH", "")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
}

#[test]
fn codex_dry_run_notes_a_new_profile_and_lists_what_would_be_created() {
    let root = Root::empty();
    let bin = bin_with("codex");
    let output = root
        .agent_profile(["codex", "work", "--dry-run"])
        .env("PATH", bin.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let dir = profile_dir(&root, "codex");
    assert!(
        text.contains(&format!("creates:      {} (would be created)", dir.display())),
        "{text}"
    );
    assert!(
        text.contains(
            "note:         new profile starts logged out; run codex login with this profile"
        ),
        "{text}"
    );
    assert!(!dir.exists());
}

#[test]
fn a_relative_path_entry_holding_the_agent_is_ignored() {
    let root = Root::empty();
    let bin = bin_with("codex");
    let output = root
        .agent_profile(["codex", "work"])
        .env("PATH", ".")
        .current_dir(bin.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("(1 relative PATH entries were ignored)"),
        "{}",
        stderr(&output)
    );
    assert!(output.stdout.is_empty());
}

#[cfg(windows)]
#[test]
fn a_cmd_shim_on_path_is_refused_with_the_config_hint() {
    let root = Root::empty();
    let bin = tempfile::tempdir().unwrap();
    fs::write(bin.path().join("codex.cmd"), b"@echo off\r\n").unwrap();
    let output = root.agent_profile(["codex", "work"]).env("PATH", bin.path()).output().unwrap();
    assert_eq!(output.status.code(), Some(6), "{}", stderr(&output));
    let text = stderr(&output);
    assert!(
        text.contains("codex.cmd, which agent-profile cannot launch without a shell"),
        "{text}"
    );
    assert!(text.contains(&root.path().join("config.toml").display().to_string()), "{text}");
}
