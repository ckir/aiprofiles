//! `doctor` end to end through the `agent-profile` binary, text and JSON forms (SP5c design §9).

mod support;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use agent_profile::repo;
use serde_json::Value;
use support::Root;

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A guarded temp directory and its canonical path.
fn scratch() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    support::assert_outside_any_repository(dir.path());
    let path = repo::canonical(dir.path()).unwrap();
    (dir, path)
}

/// Makes `path` a repository root: a `.git` directory holding `HEAD`.
fn git_dir(path: &Path) {
    fs::create_dir_all(path.join(".git")).unwrap();
    fs::write(path.join(".git").join("HEAD"), "ref: refs/heads/main\n").unwrap();
}

fn key(path: &str) -> String {
    toml_edit::Key::new(path).display_repr().into_owned()
}

fn table(path: &str, body: &str) -> String {
    format!("\n[repositories.{}]\n{body}\n", key(path))
}

fn configure(root: &Root, tables: &str) {
    root.write_config(&format!(
        "[agents.fake]\nexecutable = {:?}\n{tables}",
        env!("CARGO_BIN_EXE_fake-agent")
    ));
}

fn run(root: &Root, args: &[&str]) -> Output {
    root.agent_profile(args).output().unwrap()
}

/// `agent-profile` with an explicit `PATH`, so the executable verdicts do not depend on the machine.
fn command_with_path(root: &Root, args: &[&str], path: &Path) -> Command {
    let mut command = root.agent_profile(args);
    command.env("PATH", path);
    command
}

fn json_of(output: &Output) -> Value {
    serde_json::from_str::<Value>(&stdout(output))
        .unwrap_or_else(|e| panic!("not JSON ({e}): {}", stdout(output)))
}

fn records<'a>(doc: &'a Value, id: &str) -> Vec<&'a Value> {
    doc["checks"].as_array().unwrap().iter().filter(|check| check["id"] == id).collect()
}

fn code(output: &Output) -> i32 {
    output.status.code().unwrap()
}

fn keys(value: &Value) -> BTreeSet<String> {
    value.as_object().unwrap().keys().cloned().collect()
}

#[test]
fn fresh_host_is_green_in_both_forms() {
    let root = Root::empty();
    let fresh = root.path().join("fresh");
    let empty_path = tempfile::tempdir().unwrap();

    let mut host = command_with_path(&root, &["doctor"], empty_path.path());
    host.env("AGENT_PROFILE_HOME", &fresh);
    let output = host.output().unwrap();
    assert_eq!(code(&output), 0, "{}{}", stdout(&output), stderr(&output));

    let fixture_dir = Path::new(env!("CARGO_BIN_EXE_fake-agent")).parent().unwrap();
    let mut agent = command_with_path(&root, &["fake", "doctor", "work"], fixture_dir);
    agent.env("AGENT_PROFILE_HOME", &fresh);
    let output = agent.output().unwrap();
    assert_eq!(code(&output), 0, "{}{}", stdout(&output), stderr(&output));
    assert!(!fresh.exists(), "doctor must not create the application root");
}

#[test]
fn uninstalled_agent_fails_only_when_targeted() {
    let root = Root::empty();
    let empty_path = tempfile::tempdir().unwrap();
    let output = command_with_path(&root, &["doctor"], empty_path.path()).output().unwrap();
    assert_eq!(code(&output), 0, "{}", stdout(&output));

    let output = command_with_path(&root, &["fake", "doctor", "work", "--json"], empty_path.path())
        .output()
        .unwrap();
    assert_eq!(code(&output), 5, "{}", stdout(&output));
    let doc = json_of(&output);
    let executable = records(&doc, "executable");
    assert_eq!(executable.len(), 1);
    assert_eq!(executable[0]["status"], "fail");
}

#[test]
fn json_document_is_stable() {
    let installed = Root::new();
    let bare = Root::empty();
    let empty_path = tempfile::tempdir().unwrap();
    let mut documents = Vec::new();
    for root in [&installed, &bare] {
        let output =
            command_with_path(root, &["doctor", "--json"], empty_path.path()).output().unwrap();
        let doc = json_of(&output);
        let expected = if doc["result"] == "fail" { 5 } else { 0 };
        assert_eq!(code(&output), expected);
        assert!(doc["result"] == "ok" || doc["result"] == "fail");
        documents.push(doc);
    }
    assert_eq!(keys(&documents[0]), keys(&documents[1]));
    assert_eq!(keys(&documents[0]["counts"]), keys(&documents[1]["counts"]));
    let check_keys: BTreeSet<BTreeSet<String>> = documents
        .iter()
        .flat_map(|doc| doc["checks"].as_array().unwrap().iter().map(keys))
        .collect();
    assert_eq!(check_keys.len(), 1, "every check has the same keys: {check_keys:?}");
}

#[test]
fn text_and_json_carry_the_same_records() {
    let root = Root::new();
    let text = run(&root, &["fake", "doctor", "work"]);
    let json = run(&root, &["fake", "doctor", "work", "--json"]);
    assert_eq!(code(&text), code(&json));
    let doc = json_of(&json);
    let checks = doc["checks"].as_array().unwrap();
    let text = stdout(&text);
    let status_lines: Vec<&str> = text
        .lines()
        .filter(|line| {
            ["PASS", "INFO", "WARN", "FAIL", "SKIP"]
                .iter()
                .any(|token| line.starts_with(&format!("{token} ")))
        })
        .collect();
    assert_eq!(status_lines.len(), checks.len(), "{text}");
    for (line, check) in status_lines.iter().zip(checks) {
        assert!(line.ends_with(check["message"].as_str().unwrap()), "{line} / {check}");
    }
}

#[test]
fn invalid_config_still_prints_one_document() {
    let root = Root::empty();
    root.write_config("x =");
    let output = run(&root, &["doctor", "--json"]);
    assert_eq!(code(&output), 5);
    assert_eq!(stderr(&output), "");
    let doc = json_of(&output);
    let config = records(&doc, "config");
    assert_eq!(config.len(), 1);
    assert_eq!(config[0]["status"], "fail");
    let mapping = records(&doc, "mapping");
    assert_eq!(mapping.len(), 1);
    assert_eq!(mapping[0]["status"], "skip");
    assert!(mapping[0]["subject"].is_null());
}

#[test]
fn missing_repo_is_a_fail_record_not_an_error() {
    let root = Root::new();
    let output = run(&root, &["doctor", "--repo", "nope", "--json"]);
    assert_eq!(code(&output), 5, "{}{}", stdout(&output), stderr(&output));
    let doc = json_of(&output);
    assert_eq!(records(&doc, "repository")[0]["status"], "fail");

    let output = run(&root, &["fake", "doctor", "--repo", "nope", "--json"]);
    assert_ne!(code(&output), 4);
    let doc = json_of(&output);
    assert_eq!(records(&doc, "repository")[0]["status"], "fail");
    assert_eq!(records(&doc, "resolution")[0]["status"], "skip");
}

#[test]
fn invalid_profile_argument_is_exit_5() {
    let root = Root::new();
    let output = run(&root, &["fake", "doctor", "bad/name", "--json"]);
    assert_eq!(code(&output), 5);
    assert_eq!(stderr(&output), "");
    let doc = json_of(&output);
    assert_eq!(records(&doc, "resolution")[0]["status"], "fail");
}

#[test]
fn usage_errors_exit_2_with_empty_stdout() {
    let root = Root::new();
    for args in [&["doctor", "work"][..], &["fake", "doctor", "a", "b"], &["doctor", "--bogus"]] {
        let output = run(&root, args);
        assert_eq!(code(&output), 2, "{args:?}: {}", stderr(&output));
        assert_eq!(stdout(&output), "", "{args:?}");
    }
}

#[test]
fn unresolvable_root_is_a_fail_record() {
    let root = Root::empty();
    let mut command = root.agent_profile(["doctor", "--json"]);
    command.env("AGENT_PROFILE_HOME", "relative");
    let output = command.output().unwrap();
    assert_eq!(code(&output), 5, "{}{}", stdout(&output), stderr(&output));
    let doc = json_of(&output);
    assert_eq!(records(&doc, "app_root")[0]["status"], "fail");
}

#[test]
fn profile_path_verdict_matches_the_launch() {
    let root = Root::new();
    let profiles = root.path().join("profiles");
    fs::create_dir_all(&profiles).unwrap();
    let work = profiles.join("work");
    fs::write(&work, "not a directory").unwrap();

    let output = run(&root, &["fake", "doctor", "work", "--json"]);
    assert_eq!(code(&output), 5, "{}", stdout(&output));
    assert_eq!(records(&json_of(&output), "profile_path")[0]["status"], "fail");
    assert_eq!(code(&run(&root, &["fake", "work", "--", "--version"])), 4);

    #[cfg(unix)]
    {
        fs::remove_file(&work).unwrap();
        let real = root.path().join("real");
        fs::create_dir_all(&real).unwrap();
        std::os::unix::fs::symlink(&real, &work).unwrap();
        let output = run(&root, &["fake", "doctor", "work", "--json"]);
        let doc = json_of(&output);
        let failures: Vec<&Value> = doc["checks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|check| check["status"] == "fail")
            .collect();
        assert!(failures.is_empty(), "{failures:?}");
        assert_eq!(code(&output), 0);
        assert_eq!(code(&run(&root, &["fake", "work", "--", "--version"])), 0);
    }
}

/// Relative path, kind and file bytes of everything below `base`; no timestamps, which OS indexers can touch.
fn snapshot(base: &Path) -> Vec<(PathBuf, &'static str, Vec<u8>)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(PathBuf, &'static str, Vec<u8>)>) {
        let mut entries: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
        entries.sort();
        for path in entries {
            let meta = fs::symlink_metadata(&path).unwrap();
            let relative = path.strip_prefix(base).unwrap().to_owned();
            if meta.is_dir() {
                out.push((relative, "dir", Vec::new()));
                walk(base, &path, out);
            } else if meta.is_file() {
                out.push((relative, "file", fs::read(&path).unwrap()));
            } else {
                out.push((relative, "other", Vec::new()));
            }
        }
    }
    let mut out = Vec::new();
    walk(base, base, &mut out);
    out
}

#[test]
fn doctor_writes_nothing() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let repo_dir = base.join("acme");
    git_dir(&repo_dir);
    configure(&root, &table(repo_dir.to_str().unwrap(), "profile = \"work\""));
    fs::create_dir_all(root.profile_dir("work")).unwrap();

    let before = (snapshot(root.path()), snapshot(&repo_dir));
    for args in [
        &["doctor"][..],
        &["fake", "doctor", "work"],
        &["fake", "doctor"],
        &["fake", "doctor", "missing"],
        &["doctor", "--json"],
    ] {
        let mut command = root.agent_profile(args);
        command.current_dir(&repo_dir);
        let output = command.output().unwrap();
        assert_ne!(code(&output), 2, "{args:?}: {}", stderr(&output));
    }
    let after = (snapshot(root.path()), snapshot(&repo_dir));
    assert!(before == after, "doctor changed the root or the repository");
}

#[test]
fn no_environment_value_leaks() {
    const SENTINEL: &str = "SENTINEL-7f3a";
    let root = Root::new();
    for extra in [&[][..], &["--json"]] {
        let mut args = vec!["fake", "doctor", "work"];
        args.extend_from_slice(extra);
        let mut command = root.agent_profile(&args);
        command.env("FAKE_AGENT_HOME", SENTINEL);
        let output = command.output().unwrap();
        assert!(!stdout(&output).contains(SENTINEL));
        assert!(!stderr(&output).contains(SENTINEL));
        if !extra.is_empty() {
            let doc = json_of(&output);
            assert_eq!(records(&doc, "launch_environment")[0]["data"]["already_set"], true);
        }
    }
}

#[test]
fn case_twins_end_to_end() {
    let root = Root::new();
    let profiles = root.path().join("profiles");
    fs::create_dir_all(&profiles).unwrap();
    fs::write(root.path().join("a"), "").unwrap();
    let case_sensitive = !root.path().join("A").exists();
    fs::create_dir(profiles.join("work")).unwrap();
    if case_sensitive {
        fs::create_dir(profiles.join("Work")).unwrap();
    }
    let output = run(&root, &["doctor", "--json"]);
    let doc = json_of(&output);
    let twins = records(&doc, "case_twins");
    if case_sensitive {
        assert_eq!(code(&output), 5);
        assert_eq!(twins.len(), 1, "{twins:?}");
        assert_eq!(twins[0]["status"], "fail");
        let shown = twins[0].to_string();
        assert!(shown.contains("work") && shown.contains("Work"), "{shown}");
    } else {
        assert!(twins.is_empty(), "{twins:?}");
    }
}

#[test]
fn repository_local_executable_end_to_end() {
    let root = Root::empty();
    let name = if cfg!(windows) { "fake-agent.exe" } else { "fake-agent" };
    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let file = bin.join(name);
    fs::write(&file, "").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let output =
        command_with_path(&root, &["doctor", "--json"], Path::new("bin")).output().unwrap();
    let doc = json_of(&output);
    let local = records(&doc, "repository_local_executable");
    assert_eq!(local.len(), 1, "{}", stdout(&output));
    assert_eq!(local[0]["status"], "warn");
}
