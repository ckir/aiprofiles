//! End-to-end lifecycle tests through the real `agent-profile` binary (SP5a design §3).

mod support;

use std::process::Output;

use support::Root;

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn create_materializes_once_and_then_reports_already_exists() {
    let root = Root::empty();
    let out = root.agent_profile(["fake", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        format!("created profile 'work' for fake: {}\n", root.profile_dir("work").display())
    );
    assert!(root.profile_dir("work").is_dir());
    let out = root.agent_profile(["fake", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        stdout(&out),
        format!("profile 'work' for fake already exists: {}\n", root.profile_dir("work").display())
    );
}

#[test]
fn create_on_a_fresh_root_creates_the_root() {
    let dir = tempfile::tempdir().unwrap();
    support::assert_outside_any_repository(dir.path());
    let home = dir.path().join("brand-new");
    let out = Root::empty()
        .agent_profile(["fake", "create", "work"])
        .env("AGENT_PROFILE_HOME", &home)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(home.join("profiles").join("work").join("fake").is_dir());
    assert!(!home.join("config.toml").exists());
}

#[test]
fn create_for_a_file_adapter_writes_the_initial_file_once_and_never_overwrites() {
    // aider owns a configuration file (SP2 design; adapter/aider.rs). create needs no executable.
    let root = Root::empty();
    let out = root.agent_profile(["aider", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let dir = root.path().join("profiles").join("work").join("aider");
    let file = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.is_file())
        .expect("aider's config file");
    std::fs::write(&file, "# edited by the user\n").unwrap();
    let out = root.agent_profile(["aider", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("already exists"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "# edited by the user\n");
}

#[test]
fn create_completes_a_half_materialized_profile_and_reports_created() {
    let root = Root::empty();
    std::fs::create_dir_all(root.path().join("profiles").join("work")).unwrap(); // no fake/ below it
    let out = root.agent_profile(["fake", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).starts_with("created profile 'work' for fake:"));
}

#[test]
fn create_refuses_a_case_twin_a_reserved_name_and_an_invalid_config() {
    let root = Root::empty();
    std::fs::create_dir_all(root.profile_dir("Work")).unwrap();
    let out = root.agent_profile(["fake", "create", "work"]).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(stderr(&out).contains("differs only in letter case"), "{}", stderr(&out));
    let out = root.agent_profile(["fake", "create", "status"]).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(stderr(&out).contains("invalid profile name"), "{}", stderr(&out));
    root.write_config("this is not toml = = =\n");
    let out = root.agent_profile(["fake", "create", "other"]).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(!root.profile_dir("other").exists());
}
