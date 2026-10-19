//! End-to-end lifecycle tests through the real `agent-profile` binary (SP5a design §3).

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use support::Root;

/// A guarded temp directory and its canonical path (copied from `tests/resolution.rs`).
fn scratch() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    support::assert_outside_any_repository(dir.path());
    let path = agent_profile::repo::canonical(dir.path()).unwrap();
    (dir, path)
}

/// Makes `path` a repository root: a `.git` directory holding `HEAD` (copied from `tests/resolution.rs`).
fn git_dir(path: &Path) {
    fs::create_dir_all(path.join(".git")).unwrap();
    fs::write(path.join(".git").join("HEAD"), "ref: refs/heads/main\n").unwrap();
}

fn key(path: &Path) -> String {
    toml_edit::Key::new(path.to_str().unwrap()).display_repr().into_owned()
}

fn mapping(repository: &Path, body: &str) -> String {
    format!("\n[repositories.{}]\n{body}\n", key(repository))
}

/// Runs `<agent> create <profile>` through the binary and asserts exit 0.
fn created(root: &Root, agent: &str, profile: &str) {
    let out = root.agent_profile([agent, "create", profile]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
}

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
fn delete_with_yes_removes_the_profile_and_the_empty_parent() {
    let root = Root::empty();
    created(&root, "fake", "work");
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        format!("deleted profile 'work' for fake: {}\n", root.profile_dir("work").display())
    );
    assert!(!root.path().join("profiles").join("work").exists());
}

#[test]
fn delete_keeps_sibling_agents_and_the_parent() {
    let root = Root::empty();
    created(&root, "fake", "work");
    created(&root, "aider", "work");
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(!root.profile_dir("work").exists());
    assert!(root.path().join("profiles").join("work").join("aider").is_dir());
}

#[test]
fn delete_without_a_terminal_needs_yes() {
    let root = Root::empty();
    created(&root, "fake", "work");
    let out = root
        .agent_profile(["fake", "delete", "work"])
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("--yes is required when input is not a terminal"),
        "{}",
        stderr(&out)
    );
    assert!(root.profile_dir("work").is_dir());
}

#[test]
fn delete_of_a_missing_profile_exits_zero_and_notes_a_dangling_reference() {
    let root = Root::empty();
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "profile 'work' for fake does not exist; nothing to delete\n");
    assert_eq!(stderr(&out), "");
    root.write_config("default_profile = \"work\"\n");
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(stderr(&out).contains("still referenced by default_profile"), "{}", stderr(&out));
}

#[test]
fn delete_refuses_each_blocking_reference_and_allows_another_agents_mapping() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let repo = base.join("acme");
    git_dir(&repo);
    for (config, blocked) in [
        (mapping(&repo, "profile = \"work\""), true),
        (mapping(&repo, "agents = { fake = \"work\" }"), true),
        (mapping(&repo, "agents = { fake = \"WORK\" }"), true),
        ("default_profile = \"Work\"\n".to_owned(), true),
        (mapping(&repo, "agents = { aider = \"work\" }"), false),
    ] {
        created(&root, "fake", "work");
        root.write_config(&config);
        let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
        if blocked {
            assert_eq!(out.status.code(), Some(4), "{config}");
            assert!(
                stderr(&out).contains("cannot delete profile 'work' for fake: it is referenced"),
                "{}",
                stderr(&out)
            );
            assert!(root.profile_dir("work").is_dir(), "{config}");
            std::fs::remove_dir_all(root.path().join("profiles")).unwrap();
        } else {
            assert_eq!(out.status.code(), Some(0), "{config}: {}", stderr(&out));
            assert!(!root.profile_dir("work").exists());
        }
    }
}

#[test]
fn delete_refuses_an_unparsable_config_before_touching_anything() {
    let root = Root::empty();
    created(&root, "fake", "work");
    root.write_config("= = =\n");
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(root.profile_dir("work").is_dir());
}

#[test]
fn delete_refuses_a_case_twin() {
    let root = Root::empty();
    created(&root, "fake", "Work");
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(root.profile_dir("Work").is_dir());
}

#[cfg(unix)]
#[test]
fn delete_refuses_symlinks_inside_and_outside_the_root_and_touches_nothing() {
    use std::os::unix::fs::symlink;
    let root = Root::empty();
    created(&root, "fake", "real");
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("keep"), "x").unwrap();
    // target is a symlink out of the root
    std::fs::create_dir_all(root.path().join("profiles").join("out")).unwrap();
    symlink(outside.path(), root.profile_dir("out")).unwrap();
    // parent is a symlink back inside the root
    symlink(root.path().join("profiles").join("real"), root.path().join("profiles").join("alias"))
        .unwrap();
    for name in ["out", "alias"] {
        let out = root.agent_profile(["fake", "delete", name, "--yes"]).output().unwrap();
        assert_eq!(out.status.code(), Some(4), "{name}: {}", stderr(&out));
        assert!(stderr(&out).contains("symbolic link"), "{}", stderr(&out));
    }
    assert!(outside.path().join("keep").is_file());
    assert!(root.profile_dir("real").is_dir());
}

#[test]
fn a_mapping_added_after_the_advisory_check_still_blocks_the_locked_delete() {
    use agent_profile::config::{self, AppRoot};
    use agent_profile::lifecycle;
    use agent_profile::name::{AgentId, Platform, ProfileName};
    let root = Root::empty();
    created(&root, "fake", "work");
    let app = AppRoot::from_path(root.path().to_path_buf());
    let agent = AgentId::parse("fake").unwrap();
    let profile = ProfileName::parse("work", Platform::host()).unwrap();
    assert!(lifecycle::delete_target(&app, &agent, &profile).unwrap().is_some());
    assert!(
        lifecycle::blocking_references(&config::Config::load(&app).unwrap(), &agent, &profile)
            .is_empty()
    );
    let (_dir, base) = scratch();
    let repo = base.join("acme");
    git_dir(&repo);
    config::link(&app, &repo, None, &profile).unwrap();
    assert!(matches!(
        lifecycle::delete(&app, &agent, &profile),
        Err(agent_profile::error::Error::ProfileReferenced { .. })
    ));
    assert!(root.profile_dir("work").is_dir());
}

#[test]
fn a_profile_that_vanishes_before_the_locked_delete_is_not_an_error() {
    // Same setup as the locked-delete test above; the target is removed before `lifecycle::delete`
    // takes the lock, and `delete` must report `Ok(DeleteOutcome::Vanished)` rather than error.
    use agent_profile::config::AppRoot;
    use agent_profile::lifecycle;
    use agent_profile::name::{AgentId, Platform, ProfileName};
    let root = Root::empty();
    created(&root, "fake", "work");
    let app = AppRoot::from_path(root.path().to_path_buf());
    let agent = AgentId::parse("fake").unwrap();
    let profile = ProfileName::parse("work", Platform::host()).unwrap();
    std::fs::remove_dir(root.profile_dir("work")).unwrap();
    let outcome = lifecycle::delete(&app, &agent, &profile).unwrap();
    assert!(matches!(outcome, lifecycle::DeleteOutcome::Vanished { .. }), "{outcome:?}");
}

#[test]
fn delete_removes_a_half_materialized_profile() {
    let root = Root::empty();
    std::fs::create_dir_all(root.path().join("profiles").join("work").join("aider")).unwrap(); // no config file
    let out = root.agent_profile(["aider", "delete", "work", "--yes"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(!root.path().join("profiles").join("work").exists());
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
