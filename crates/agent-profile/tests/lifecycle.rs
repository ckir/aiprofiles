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
    // The non-empty parent is never swept, so no "could not remove" note reaches stderr.
    assert_eq!(stderr(&out), "");
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

/// `delete` runs under `config.toml.lock` (owner ruling: across the reference check AND the removal).
/// With the lock held by another handle it gives up with the lock message after the bounded wait and
/// removes nothing; a `delete` that skipped the lock would simply remove the profile here.
#[test]
fn delete_gives_up_while_another_handle_holds_the_config_lock() {
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use agent_profile::config::AppRoot;
    use agent_profile::error::Error;
    use agent_profile::lifecycle;
    use agent_profile::name::{AgentId, Platform, ProfileName};
    let root = Root::empty();
    created(&root, "fake", "work");
    let holder = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.path().join("config.toml.lock"))
        .unwrap();
    holder.lock().unwrap();
    let app = AppRoot::from_path(root.path().to_path_buf());

    // A watchdog, so a regression that waits forever fails instead of hanging the suite.
    let (done_tx, done_rx) = mpsc::channel();
    let started = Instant::now();
    std::thread::spawn(move || {
        let agent = AgentId::parse("fake").unwrap();
        let profile = ProfileName::parse("work", Platform::host()).unwrap();
        let _ = done_tx.send(lifecycle::delete(&app, &agent, &profile));
    });
    let result =
        done_rx.recv_timeout(Duration::from_secs(30)).expect("delete did not give up within 30 s");
    let elapsed = started.elapsed();

    match result {
        Err(Error::ConfigWrite { source, .. }) => assert!(
            source
                .to_string()
                .contains("another agent-profile process holds the configuration lock"),
            "{source}"
        ),
        other => panic!("{other:?}"),
    }
    assert!(elapsed >= Duration::from_secs(9), "gave up after {elapsed:?}, before the 10 s bound");
    assert!(root.profile_dir("work").is_dir());
    drop(holder);
}

/// An OS-refused removal is reported as the failure it is (spec §16.2; SP5a design §3.2 step 8): exit 4,
/// the tree still on disk, and a message that sends the user back to retry, never a `deleted` line.
#[cfg(unix)]
#[test]
fn delete_reports_an_os_refused_removal_and_never_claims_success() {
    use std::os::unix::fs::PermissionsExt;
    let root = Root::empty();
    created(&root, "fake", "work");
    let target = root.profile_dir("work");
    fs::write(target.join("settings.json"), "{}").unwrap();
    let mode = |bits: u32| fs::set_permissions(&target, fs::Permissions::from_mode(bits)).unwrap();
    mode(0o555); // the entries inside can no longer be unlinked
    if fs::write(target.join("probe"), "").is_ok() {
        mode(0o755); // a privileged user is not refused: nothing to pin here
        return;
    }
    let out = root.agent_profile(["fake", "delete", "work", "--yes"]).output().unwrap();
    mode(0o755);
    assert_eq!(out.status.code(), Some(4), "{}", stderr(&out));
    assert_eq!(stdout(&out), "");
    assert!(target.join("settings.json").is_file());
    let message = stderr(&out);
    assert!(
        message.contains("the profile may be partially removed, retry the deletion"),
        "{message}"
    );
    assert!(!message.contains("deleted profile"), "{message}");
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
fn profiles_lists_materialized_agents_per_profile_sorted() {
    let root = Root::empty();
    created(&root, "fake", "work");
    created(&root, "aider", "work");
    created(&root, "fake", "home");
    std::fs::create_dir_all(root.path().join("profiles").join("half").join("aider")).unwrap(); // no file: not materialized
    let out = root.agent_profile(["profiles"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), "half  (none materialized)\nhome  fake\nwork  aider, fake\n");
    assert_eq!(stderr(&out), "");
    let out = root.agent_profile(["list"]).output().unwrap();
    assert_eq!(stdout(&out), "half  (none materialized)\nhome  fake\nwork  aider, fake\n");
}

#[test]
fn profiles_on_an_empty_or_missing_root_prints_no_profiles_and_creates_nothing() {
    let root = Root::empty();
    let out = root.agent_profile(["profiles"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "no profiles\n");
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn profiles_counts_unrecognized_entries_on_stderr_and_ignores_dot_names() {
    let root = Root::empty();
    created(&root, "fake", "work");
    std::fs::write(root.path().join("profiles").join("notes.txt"), "x").unwrap();
    std::fs::create_dir_all(root.path().join("profiles").join("status")).unwrap(); // reserved word
    std::fs::create_dir_all(root.path().join("profiles").join("work").join("zzz")).unwrap(); // unknown agent
    std::fs::write(root.path().join("profiles").join(".DS_Store"), "x").unwrap();
    let out = root.agent_profile(["profiles"]).output().unwrap();
    assert_eq!(stdout(&out), "work  fake\n");
    assert_eq!(stderr(&out), "agent-profile: ignored 3 unrecognized entries\n");
}

/// An OS-refused read of one profile directory is a general error (spec §33: exit 1), not a skipped
/// entry, and the message names that directory rather than the profiles directory above it.
#[cfg(unix)]
#[test]
fn profiles_names_the_profile_directory_the_os_refused_to_read() {
    use std::os::unix::fs::PermissionsExt;
    let root = Root::empty();
    created(&root, "fake", "work");
    created(&root, "fake", "other");
    let work = root.path().join("profiles").join("work");
    let mode = |bits: u32| fs::set_permissions(&work, fs::Permissions::from_mode(bits)).unwrap();
    mode(0o000);
    if fs::read_dir(&work).is_ok() {
        mode(0o755); // a privileged user is not refused: nothing to pin here
        return;
    }
    let out = root.agent_profile(["profiles"]).output().unwrap();
    mode(0o755);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(stdout(&out), "");
    let message = stderr(&out);
    assert!(message.contains("could not read the profile directory "), "{message}");
    assert!(message.contains("profiles/work"), "{message}");
    assert!(!message.contains("could not read the profiles directory"), "{message}");
}

#[test]
fn agent_list_prints_names_only_and_says_so_when_empty() {
    let root = Root::empty();
    created(&root, "fake", "work");
    created(&root, "fake", "home");
    created(&root, "aider", "other");
    let out = root.agent_profile(["fake", "list"]).output().unwrap();
    assert_eq!(stdout(&out), "home\nwork\n");
    assert_eq!(stderr(&out), "");
    let out = root.agent_profile(["claude", "list"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
    assert_eq!(stderr(&out), "agent-profile: no profiles for claude\n");
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

#[test]
fn agents_lists_every_registered_agent_with_the_full_matrix() {
    let root = Root::new(); // config points fake at the fixture
    let out = root.agent_profile(["agents"]).env_remove("PATH").output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    let ids = agent_profile::adapter::known_agents();
    for id in &ids {
        assert!(text.contains(&format!("agent:{}{id}\n", " ".repeat(8))), "{id} missing");
    }
    assert_eq!(text.matches("mechanism:").count(), ids.len());
    assert_eq!(text.matches("support:").count(), ids.len());
    assert_eq!(text.matches("version:").count(), ids.len());
    assert_eq!(text.matches("config:").count(), ids.len());
    assert_eq!(text.matches("credentials:").count(), ids.len());
    assert_eq!(text.matches("state:").count(), ids.len());
    assert!(text.contains(&format!(
        "executable:{}{} (configured)",
        " ".repeat(3),
        env!("CARGO_BIN_EXE_fake-agent")
    )));
    assert_eq!(text.matches("not found on PATH").count(), ids.len() - 1);
    assert_eq!(text.matches("\n\n").count(), ids.len() - 1);
}

#[test]
fn agents_reports_a_configured_but_missing_executable() {
    let root = Root::empty();
    // Config::load requires an absolute path, and `/x` is not absolute on Windows.
    let missing =
        if cfg!(windows) { r"C:\nonexistent\fake-agent" } else { "/nonexistent/fake-agent" };
    root.write_config(&format!("[agents.fake]\nexecutable = {missing:?}\n"));
    let out = root.agent_profile(["agents"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(stdout(&out).contains(&format!("{missing} (configured, missing)")));
}

/// The fourth executable state of SP5a design §3.4: a configured shell script or shim is listed as a
/// fact, so `agents` stays a discovery command (exit 0) whatever the configuration points at.
#[test]
fn agents_reports_a_configured_shell_script_as_unsupported() {
    let root = Root::empty();
    let shim = root.path().join("fake.cmd");
    fs::write(&shim, "@echo off\n").unwrap();
    root.write_config(&format!("[agents.fake]\nexecutable = {:?}\n", shim.to_str().unwrap()));
    let out = root.agent_profile(["agents"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains(&format!("{} (unsupported: shell script or shim)", shim.display())),
        "{text}"
    );
}

/// Every path below `base` with file contents, sorted, for the untouched-outside-the-profiles-root checks.
fn listing(base: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    let mut found = Vec::new();
    let mut pending = vec![base.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                pending.push(path.clone());
                found.push((path, None));
            } else {
                found.push((path.clone(), fs::read(&path).ok()));
            }
        }
    }
    found.sort();
    found
}

#[test]
fn path_traversal_names_are_refused_by_create_and_delete_outside_the_profiles_root() {
    let root = Root::new();
    created(&root, "fake", "keep");
    // Siblings of `profiles/` inside the app root, laid out so each traversal name would hit real data:
    // `../x` and `a/b` resolve below the root, and `..` resolves to the root itself.
    fs::write(root.path().join("sentinel.txt"), "sentinel").unwrap();
    fs::create_dir_all(root.path().join("x").join("fake")).unwrap();
    fs::write(root.path().join("x").join("fake").join("keep"), "x").unwrap();
    fs::create_dir_all(root.path().join("profiles").join("a")).unwrap();
    let before = listing(root.path());
    for name in ["../x", "a/b", ".."] {
        for verb in [&["fake", "delete", name, "--yes"][..], &["fake", "create", name]] {
            let out = root.agent_profile(verb).output().unwrap();
            assert_eq!(out.status.code(), Some(4), "{verb:?}: {}{}", stdout(&out), stderr(&out));
            assert!(
                stderr(&out).contains(&format!("invalid profile name {name:?}")),
                "{verb:?}: {}",
                stderr(&out)
            );
            assert_eq!(stdout(&out), "", "{verb:?}");
        }
    }
    assert!(listing(root.path()) == before, "a traversal name changed the application root");
}
