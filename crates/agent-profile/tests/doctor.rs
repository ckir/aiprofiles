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
    let support = records(&doc, "support");
    assert_eq!(support.len(), 1);
    for member in ["mechanism", "support", "evidence"] {
        assert!(!support[0]["data"][member].is_null(), "{member}: {}", support[0]);
    }
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
    assert_eq!(code(&output), 5, "{}{}", stdout(&output), stderr(&output));
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

        fs::remove_file(&work).unwrap();
        std::os::unix::fs::symlink(root.path().join("missing"), &work).unwrap();
        let output = run(&root, &["fake", "doctor", "work", "--json"]);
        assert_eq!(code(&output), 5, "{}", stdout(&output));
        let doc = json_of(&output);
        let path_records = records(&doc, "profile_path");
        assert_eq!(path_records.len(), 1, "{path_records:?}");
        assert_eq!(path_records[0]["status"], "fail");
        assert_eq!(code(&run(&root, &["fake", "work", "--", "--version"])), 4);
    }
}

#[test]
fn agent_form_record_order() {
    let root = Root::new();
    fs::create_dir_all(root.profile_dir("work")).unwrap();
    let empty_path = tempfile::tempdir().unwrap();
    let output = command_with_path(&root, &["fake", "doctor", "work", "--json"], empty_path.path())
        .output()
        .unwrap();
    let doc = json_of(&output);
    let mut ids: Vec<&str> = Vec::new();
    for check in doc["checks"].as_array().unwrap() {
        if check["agent"] == "fake" {
            let id = check["id"].as_str().unwrap();
            if ids.last() != Some(&id) {
                ids.push(id);
            }
        }
    }
    assert_eq!(
        ids,
        [
            "executable",
            "support",
            "resolution",
            "profile",
            "profile_path",
            "launch_environment",
            "conflicting_option",
            "capability",
        ],
        "{}",
        stdout(&output)
    );
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
    for (args, expected) in [
        (&["doctor"][..], 0),
        (&["fake", "doctor", "work"], 0),
        (&["fake", "doctor"], 0),
        (&["fake", "doctor", "missing"], 0),
        (&["doctor", "--json"], 0),
    ] {
        let mut command = root.agent_profile(args);
        command.current_dir(&repo_dir);
        let output = command.output().unwrap();
        assert_eq!(code(&output), expected, "{args:?}: {}{}", stdout(&output), stderr(&output));
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
        assert_eq!(twins[0]["subject"], "Work", "{}", twins[0]);
        assert_eq!(twins[0]["data"]["names"], serde_json::json!(["Work", "work"]));
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

#[cfg(unix)]
#[test]
fn root_ancestor_verdict_matches_create() {
    let (_dir, base) = scratch();
    let broken = base.join("broken");
    std::os::unix::fs::symlink(base.join("gone"), &broken).unwrap();
    let home = broken.join("app");
    let root = Root::empty();
    let mut doctor = root.agent_profile(["doctor"]);
    doctor.env("AGENT_PROFILE_HOME", &home);
    let doctor = doctor.output().unwrap();
    assert_eq!(code(&doctor), 5, "{}{}", stdout(&doctor), stderr(&doctor));
    let mut create = root.agent_profile(["fake", "create", "work"]);
    create.env("AGENT_PROFILE_HOME", &home);
    let create = create.output().unwrap();
    assert_eq!(code(&create), 4, "{}", stderr(&create));
}

#[cfg(unix)]
#[test]
fn non_executable_override_verdict_matches_the_launch() {
    use std::os::unix::fs::PermissionsExt;
    let root = Root::empty();
    let plain = root.path().join("plain-agent");
    fs::write(&plain, "x").unwrap();
    fs::set_permissions(&plain, fs::Permissions::from_mode(0o644)).unwrap();
    root.write_config(&format!("[agents.fake]\nexecutable = {:?}\n", plain));
    fs::create_dir_all(root.profile_dir("work")).unwrap();
    let doctor = run(&root, &["fake", "doctor", "work"]);
    assert_eq!(code(&doctor), 5, "{}{}", stdout(&doctor), stderr(&doctor));
    let launch = run(&root, &["fake", "work", "--", "--version"]);
    assert_eq!(code(&launch), 6, "{}", stderr(&launch));
}

/// Every regular file and directory below `path`, with file contents, for the no-write checks.
fn tree(path: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    let mut found = Vec::new();
    let mut pending = vec![path.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                pending.push(entry_path.clone());
                found.push((entry_path, None));
            } else {
                found.push((entry_path.clone(), fs::read(&entry_path).ok()));
            }
        }
    }
    found.sort();
    found
}

#[test]
fn doctor_writes_nothing_on_failure_branches() {
    let invalid_config = Root::empty();
    invalid_config.write_config("not = [valid");
    let profile_is_a_file = Root::new();
    fs::create_dir_all(profile_is_a_file.path().join("profiles")).unwrap();
    fs::write(profile_is_a_file.path().join("profiles").join("work"), "x").unwrap();
    let profiles_is_a_file = Root::new();
    fs::write(profiles_is_a_file.path().join("profiles"), "x").unwrap();
    for (name, root, bare_code) in [
        ("invalid config", &invalid_config, 5),
        ("profile is a file", &profile_is_a_file, 0),
        ("profiles is a file", &profiles_is_a_file, 5),
    ] {
        let before = tree(root.path());
        for (args, expected) in [
            (&["doctor"][..], bare_code),
            (&["fake", "doctor", "work"], 5),
            (&["fake", "doctor", "work", "--json"], 5),
        ] {
            let output = run(root, args);
            let shown = format!("{name} {args:?}: {}{}", stdout(&output), stderr(&output));
            assert_eq!(code(&output), expected, "{shown}");
        }
        assert!(tree(root.path()) == before, "doctor changed the root: {name}");
    }

    let (_dir, base) = scratch();
    let home = base.join("absent").join("app");
    let root = Root::empty();
    let empty_path = tempfile::tempdir().unwrap();
    // With no root there is no config, so the targeted form fails on the uninstalled fake agent. The empty `PATH`
    // keeps it uninstalled: on Windows the test runner puts the fixture's directory on `PATH`.
    for (args, expected) in [(&["doctor"][..], 0), (&["fake", "doctor", "work"], 5)] {
        let mut command = command_with_path(&root, args, empty_path.path());
        command.env("AGENT_PROFILE_HOME", &home);
        let output = command.output().unwrap();
        assert_eq!(code(&output), expected, "{args:?}: {}{}", stdout(&output), stderr(&output));
    }
    assert!(!base.join("absent").exists(), "doctor created the missing root");
}

#[test]
fn bare_and_targeted_doctor_share_host_records() {
    let (_dir, base) = scratch();
    let repo_dir = base.join("acme");
    git_dir(&repo_dir);
    let root = Root::empty();
    configure(&root, &table(repo_dir.to_str().unwrap(), "profile = \"work\""));
    fs::create_dir_all(root.profile_dir("work")).unwrap();
    let mut documents = Vec::new();
    for args in [&["doctor", "--json"][..], &["fake", "doctor", "work", "--json"]] {
        let mut command = root.agent_profile(args);
        command.current_dir(&repo_dir);
        documents.push(json_of(&command.output().unwrap()));
    }
    let agentless = |doc: &Value| -> Vec<Value> {
        doc["checks"].as_array().unwrap().iter().filter(|c| c["agent"].is_null()).cloned().collect()
    };
    let bare = agentless(&documents[0]);
    assert!(bare.len() >= 8, "{bare:?}");
    assert!(bare.iter().any(|c| c["id"] == "default_profile"), "{bare:?}");
    assert!(bare.iter().any(|c| c["id"] == "mapping"), "{bare:?}");
    assert_eq!(bare, agentless(&documents[1]));
}

#[test]
fn targeted_case_twin_profile_end_to_end() {
    let root = Root::new();
    let profiles = root.path().join("profiles");
    fs::create_dir_all(&profiles).unwrap();
    fs::write(root.path().join("a"), "").unwrap();
    if root.path().join("A").exists() {
        println!("skipped: the filesystem is case-insensitive");
        return;
    }
    fs::create_dir_all(profiles.join("Work").join("fake")).unwrap();
    let output = run(&root, &["fake", "doctor", "work", "--json"]);
    assert_eq!(code(&output), 5, "{}{}", stdout(&output), stderr(&output));
    let doc = json_of(&output);
    let profile = records(&doc, "profile");
    assert_eq!(profile.len(), 1, "{profile:?}");
    assert_eq!(profile[0]["status"], "fail");
    assert!(profile[0]["message"].as_str().unwrap().contains("Work"), "{}", profile[0]);
    let create = run(&root, &["fake", "create", "work"]);
    assert_eq!(code(&create), 4, "{}", stderr(&create));
}

#[cfg(unix)]
#[test]
fn deleted_working_directory_falls_back_to_dot() {
    let root = Root::new();
    let gone = root.path().join("gone");
    fs::create_dir(&gone).unwrap();
    let template = root.agent_profile(["doctor", "--json"]);
    let mut command = Command::new("sh");
    for (name, value) in template.get_envs() {
        match value {
            Some(value) => command.env(name, value),
            None => command.env_remove(name),
        };
    }
    command.current_dir(&gone).args([
        "-c",
        r#"rmdir "$PWD" && exec "$0" doctor --json"#,
        env!("CARGO_BIN_EXE_agent-profile"),
    ]);
    let output = command.output().unwrap();
    assert!(!gone.exists());
    assert_eq!(stderr(&output), "");
    let doc = json_of(&output);
    let expected = if doc["result"] == "fail" { 5 } else { 0 };
    assert_eq!(code(&output), expected, "{}", stdout(&output));
    assert_eq!(records(&doc, "repository").len(), 1, "{}", stdout(&output));
}

#[test]
fn a_real_health_failure_exits_5_with_a_failing_summary_in_both_forms() {
    let root = Root::empty();
    let empty_path = tempfile::tempdir().unwrap();
    let text =
        command_with_path(&root, &["fake", "doctor", "work"], empty_path.path()).output().unwrap();
    assert_eq!(code(&text), 5, "{}{}", stdout(&text), stderr(&text));
    assert_eq!(stderr(&text), "");
    let shown = stdout(&text);
    assert!(shown.lines().any(|line| line.starts_with("FAIL ")), "{shown}");
    assert!(shown.trim_end().ends_with("doctor: 1 failed, 0 warnings"), "{shown}");

    let json = command_with_path(&root, &["fake", "doctor", "work", "--json"], empty_path.path())
        .output()
        .unwrap();
    assert_eq!(code(&json), 5, "{}{}", stdout(&json), stderr(&json));
    assert_eq!(stderr(&json), "");
    let doc = json_of(&json);
    assert_eq!(doc["result"], "fail");
    assert_eq!(doc["counts"]["fail"], 1, "{doc}");
    let failing: Vec<&str> = doc["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|check| check["status"] == "fail")
        .map(|check| check["id"].as_str().unwrap())
        .collect();
    assert_eq!(failing, ["executable"]);
}

#[test]
fn claude_doctor_prints_the_caveats_for_its_non_supported_capabilities() {
    let root = Root::empty();
    root.write_config(&format!(
        "[agents.claude]\nexecutable = {:?}\n",
        env!("CARGO_BIN_EXE_fake-agent")
    ));
    let text = run(&root, &["claude", "doctor", "work"]);
    assert_eq!(code(&text), 0, "{}{}", stdout(&text), stderr(&text));
    let shown = stdout(&text);
    assert!(
        shown
            .lines()
            .any(|line| line.starts_with("INFO  [claude] credentials isolation is conditional: ")),
        "{shown}"
    );
    assert!(
        shown
            .lines()
            .any(|line| line.starts_with("INFO  [claude] state isolation is not guaranteed: ")),
        "{shown}"
    );

    let json = run(&root, &["claude", "doctor", "work", "--json"]);
    let doc = json_of(&json);
    let capabilities = records(&doc, "capability");
    let message = |subject: &str| -> &str {
        capabilities
            .iter()
            .find(|check| check["subject"] == subject)
            .unwrap_or_else(|| panic!("no {subject} capability record: {capabilities:?}"))["message"]
            .as_str()
            .unwrap()
    };
    assert!(message("credentials").starts_with("credentials isolation is conditional: "));
    assert!(message("state").starts_with("state isolation is not guaranteed: "));
    // `Supported` capabilities are not caveats.
    assert!(capabilities.iter().all(|check| check["subject"] != "config"), "{capabilities:?}");
}

/// The human `doctor` output names the adapter's mechanism id, as `--json` already does, so a mechanism
/// change between releases can be seen without `--json` (spec section 28).
#[test]
fn doctor_text_names_the_mechanism_id_that_json_carries() {
    let root = Root::empty();
    root.write_config(&format!(
        "[agents.claude]\nexecutable = {:?}\n",
        env!("CARGO_BIN_EXE_fake-agent")
    ));
    let shown = stdout(&run(&root, &["claude", "doctor", "work"]));
    let support = shown
        .lines()
        .find(|line| line.contains("[claude] claude is "))
        .unwrap_or_else(|| panic!("no support line: {shown}"));
    assert!(support.contains("; mechanism claude-config-dir-v1; "), "{support}");

    let doc = json_of(&run(&root, &["claude", "doctor", "work", "--json"]));
    assert_eq!(records(&doc, "support")[0]["data"]["mechanism"]["id"], "claude-config-dir-v1");
}
