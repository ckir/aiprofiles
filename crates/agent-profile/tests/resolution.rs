//! Repository resolution end to end through the `agent-profile` binary (SP3 design §8.4; spec §34
//! "Resolution"). Repositories are synthetic `.git` layouts in guarded temp directories, so no test needs `git`
//! and none discovers this checkout.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use agent_profile::repo;
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

fn key(path: &Path) -> String {
    toml_edit::Key::new(path.to_str().unwrap()).display_repr().into_owned()
}

/// `config.toml` pointing `fake` at the fixture, with an optional default and mapping tables.
fn configure(root: &Root, default: Option<&str>, tables: &str) {
    let default = default.map(|name| format!("default_profile = {name:?}\n")).unwrap_or_default();
    root.write_config(&format!(
        "{default}[agents.fake]\nexecutable = {:?}\n{tables}",
        env!("CARGO_BIN_EXE_fake-agent")
    ));
}

fn mapping(repository: &Path, body: &str) -> String {
    format!("\n[repositories.{}]\n{body}\n", key(repository))
}

fn run(root: &Root, cwd: &Path, args: &[&str]) -> Output {
    root.agent_profile(args).current_dir(cwd).output().unwrap()
}

/// Launches `fake` with no profile word from `cwd` and returns the profile directory it received.
fn launched_home(root: &Root, cwd: &Path) -> String {
    let output = root
        .agent_profile(["fake"])
        .current_dir(cwd)
        .env("FAKE_AGENT_ECHO_ENV", "FAKE_AGENT_HOME")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    support::report(&output.stdout)["env"]["FAKE_AGENT_HOME"].as_str().unwrap().to_owned()
}

fn home(root: &Root, profile: &str) -> String {
    root.profile_dir(profile).to_str().unwrap().to_owned()
}

#[test]
fn a_resolved_launch_uses_each_source_in_precedence_order() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let repository = base.join("acme");
    git_dir(&repository);
    let deep = repository.join("src").join("deep");
    fs::create_dir_all(&deep).unwrap();

    configure(
        &root,
        Some("global"),
        &mapping(&repository, "profile = \"repo\"\nagents = { fake = \"agent\" }"),
    );
    assert_eq!(launched_home(&root, &deep), home(&root, "agent"));
    configure(&root, Some("global"), &mapping(&repository, "profile = \"repo\""));
    assert_eq!(launched_home(&root, &deep), home(&root, "repo"));
    configure(&root, Some("global"), "");
    assert_eq!(launched_home(&root, &deep), home(&root, "global"));
    assert_eq!(launched_home(&root, &base), home(&root, "global"));

    configure(&root, None, "");
    let output = run(&root, &deep, &["fake"]);
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("link this repository (agent-profile link <profile>)"),
        "{}",
        stderr(&output)
    );
    let output = run(&root, &base, &["fake"]);
    assert_eq!(output.status.code(), Some(4));
    assert!(!stderr(&output).contains("link this repository"), "{}", stderr(&output));
    assert!(
        stderr(&output).contains(&format!(
            "or set default_profile in {}",
            root.path().join("config.toml").display()
        )),
        "{}",
        stderr(&output)
    );
}

#[test]
fn current_resolve_and_status_agree_for_every_source() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let repository = base.join("acme");
    git_dir(&repository);
    let explicit_launch = || {
        let output = run(&root, &repository, &["fake", "explicit", "--dry-run"]);
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
        assert!(
            stdout(&output).contains("profile:      explicit (explicit)"),
            "{}",
            stdout(&output)
        );
        assert!(
            stdout(&output).contains(&format!("repository:   {}", repository.display())),
            "{}",
            stdout(&output)
        );
    };
    configure(&root, None, "");
    explicit_launch();

    for (default, tables, profile, source) in [
        (
            Some("global"),
            mapping(&repository, "profile = \"repo\"\nagents = { fake = \"agent\" }"),
            "agent",
            "repository agent mapping",
        ),
        (Some("global"), mapping(&repository, "profile = \"repo\""), "repo", "repository mapping"),
        (Some("global"), String::new(), "global", "global default"),
    ] {
        configure(&root, default, &tables);
        let current = run(&root, &repository, &["fake", "current"]);
        assert_eq!(current.status.code(), Some(0), "{}", stderr(&current));
        assert_eq!(stdout(&current), format!("{profile}\n"));

        let resolve = run(&root, &repository, &["fake", "resolve"]);
        assert_eq!(resolve.status.code(), Some(0), "{}", stderr(&resolve));
        assert_eq!(
            stdout(&resolve),
            format!(
                "agent:        fake\nprofile:      {profile}\nsource:       {source}\nrepository:   {}\n",
                repository.display()
            )
        );

        let status = run(&root, &repository, &["status"]);
        assert_eq!(status.status.code(), Some(0), "{}", stderr(&status));
        assert!(
            stdout(&status).contains(&format!("\nfake:         {profile} ({source})\n")),
            "{}",
            stdout(&status)
        );
        explicit_launch();
    }

    configure(&root, None, "");
    let current = run(&root, &repository, &["fake", "current"]);
    assert_eq!(current.status.code(), Some(4));
    assert!(current.stdout.is_empty());
    assert!(stderr(&current).contains("no profile selected for `fake`"), "{}", stderr(&current));
    let resolve = run(&root, &repository, &["fake", "resolve"]);
    assert_eq!(resolve.status.code(), Some(4));
    assert_eq!(
        stdout(&resolve),
        format!(
            "agent:        fake\nprofile:      none\nsource:       none\nrepository:   {}\n",
            repository.display()
        )
    );
    assert!(stderr(&resolve).contains("no profile selected for `fake`"), "{}", stderr(&resolve));
    let status = run(&root, &repository, &["status"]);
    assert_eq!(status.status.code(), Some(0));
    assert!(stdout(&status).contains("\nfake:         none\n"), "{}", stdout(&status));
    explicit_launch();
}

#[test]
fn status_reports_the_repository_mappings_and_an_ancestor_note() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let outer = base.join("outer");
    git_dir(&outer);
    let inner = outer.join("vendor").join("inner");
    git_dir(&inner);
    configure(
        &root,
        Some("work"),
        &(mapping(&outer, "profile = \"outer\"")
            + &mapping(&inner, "profile = \"inner\"\nagents = { fake = \"mine\" }")),
    );
    let output = run(&root, &inner.join("."), &["status"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], format!("repository:   {}", inner.display()));
    assert_eq!(
        &lines[1..4],
        ["mapping:      inner", "agents:       fake=mine", "default:      work"]
    );
    assert_eq!(lines[4], "claude:       inner (repository mapping)");
    assert!(lines.contains(&"fake:         mine (repository agent mapping)"), "{text}");
    assert_eq!(
        *lines.last().unwrap(),
        format!(
            "note:         {} has a mapping that does not apply to this repository",
            outer.display()
        )
    );

    let outside = run(&root, &base, &["status"]);
    assert_eq!(outside.status.code(), Some(0));
    assert!(
        stdout(&outside).starts_with("repository:   not in a repository\nmapping:      none\n")
    );
}

#[test]
fn agent_status_shows_one_agent_and_its_presence() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let repository = base.join("acme");
    git_dir(&repository);
    configure(&root, None, &mapping(&repository, "profile = \"work\""));
    let presence = || {
        let output = run(&root, &repository, &["fake", "status"]);
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
        let text = stdout(&output);
        assert!(!text.contains("claude:"), "{text}");
        assert!(text.contains("fake:         work (repository mapping)\n"), "{text}");
        text.lines().find(|line| line.starts_with("presence:")).map(str::to_owned)
    };
    assert_eq!(presence().as_deref(), Some("presence:     absent"));
    let launch = run(&root, &repository, &["fake"]);
    assert_eq!(launch.status.code(), Some(0), "{}", stderr(&launch));
    assert_eq!(presence().as_deref(), Some("presence:     materialized"));
    configure(&root, None, "");
    let output = run(&root, &repository, &["fake", "status"]);
    assert!(!stdout(&output).contains("presence:"), "{}", stdout(&output));
}

#[test]
fn a_case_twin_profile_directory_is_reported_as_a_conflict() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let repository = base.join("acme");
    git_dir(&repository);
    configure(&root, None, &mapping(&repository, "profile = \"work\""));
    fs::create_dir_all(root.path().join("profiles").join("WORK")).unwrap();
    let output = run(&root, &repository, &["fake", "status"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(stdout(&output).contains("presence:     conflicts with WORK\n"), "{}", stdout(&output));
}

#[test]
fn link_and_unlink_with_and_without_repo() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let repository = base.join("acme");
    git_dir(&repository);
    let sub = repository.join("sub");
    fs::create_dir_all(&sub).unwrap();
    let shown = repository.display();

    for (cwd, args, expected) in [
        (&sub, &["link", "work"][..], format!("linked {shown} -> work\n")),
        (&sub, &["link", "work"], format!("already linked {shown} -> work\n")),
        (
            &base,
            &["link", "other", "--repo", "acme/sub"],
            format!("changed {shown}: work -> other\n"),
        ),
        (
            &base,
            &["fake", "link", "personal", "--repo=acme"],
            format!(
                "linked fake: {shown} -> personal\nnote:         profile personal has not been launched with fake yet\n"
            ),
        ),
        (&sub, &["fake", "current"], "personal\n".to_owned()),
        (&sub, &["fake", "unlink"], format!("unlinked fake: {shown} (was personal)\n")),
        (&sub, &["fake", "current"], "other\n".to_owned()),
        (&sub, &["fake", "unlink"], format!("no mapping to remove for fake: {shown}\n")),
        (&base, &["unlink", "--repo", "acme"], format!("unlinked {shown} (was other)\n")),
        (&base, &["unlink", "--repo", "acme"], format!("no mapping to remove for {shown}\n")),
    ] {
        let output = run(&root, cwd, args);
        assert_eq!(output.status.code(), Some(0), "{args:?}: {}", stderr(&output));
        assert_eq!(stdout(&output), expected, "{args:?}");
    }
    let launch = run(&root, &repository, &["fake", "work"]);
    assert_eq!(launch.status.code(), Some(0), "{}", stderr(&launch));
    let output = run(&root, &repository, &["fake", "link", "work"]);
    assert_eq!(stdout(&output), format!("linked fake: {shown} -> work\n"));
}

#[test]
fn link_and_unlink_outside_a_repository_exit_4() {
    let root = Root::new();
    let (_dir, base) = scratch();
    for args in
        [&["link", "work"][..], &["fake", "unlink"], &["unlink"], &["link", "work", "--repo", "."]]
    {
        let output = run(&root, &base, args);
        assert_eq!(output.status.code(), Some(4), "{args:?}: {}", stderr(&output));
        assert_eq!(
            stderr(&output),
            format!(
                "agent-profile: error: repository {}: not inside a Git repository\n",
                base.display()
            ),
            "{args:?}"
        );
        assert!(output.stdout.is_empty());
    }
    assert!(!root.path().join("config.toml.lock").exists());
}

#[test]
fn unlink_repo_removes_orphan_mappings_and_never_an_enclosing_one() {
    let root = Root::new();
    let (_dir, base) = scratch();

    let deleted = base.join("deleted");
    git_dir(&deleted);
    let enclosing = base.join("enclosing");
    git_dir(&enclosing);
    let recreated = enclosing.join("nested");
    git_dir(&recreated);
    let main = base.join("main");
    git_dir(&main);
    let worktree = base.join("wt");
    fs::create_dir_all(main.join(".git").join("worktrees").join("wt")).unwrap();
    fs::write(main.join(".git").join("worktrees").join("wt").join("HEAD"), "x\n").unwrap();
    fs::create_dir_all(&worktree).unwrap();
    fs::write(worktree.join(".git"), "gitdir: ../main/.git/worktrees/wt\n").unwrap();

    for (cwd, profile) in [(&deleted, "d"), (&enclosing, "e"), (&recreated, "r"), (&worktree, "w")]
    {
        let output = run(&root, cwd, &["link", profile]);
        assert_eq!(output.status.code(), Some(0), "{}: {}", cwd.display(), stderr(&output));
    }

    fs::remove_dir_all(&deleted).unwrap();
    fs::remove_dir_all(recreated.join(".git")).unwrap();
    fs::remove_dir_all(main.join(".git").join("worktrees")).unwrap();
    let broken = run(&root, &worktree, &["fake", "current"]);
    assert_eq!(broken.status.code(), Some(4), "the stale worktree is a discovery error");

    let sub = base.join("sub");
    fs::create_dir_all(&sub).unwrap();
    for (repo, expected) in [
        ("../deleted", format!("unlinked {} (was d)\n", deleted.display())),
        ("../enclosing/nested", format!("unlinked {} (was r)\n", recreated.display())),
        ("../wt", format!("unlinked {} (was w)\n", worktree.display())),
    ] {
        let output = run(&root, &sub, &["unlink", "--repo", repo]);
        assert_eq!(output.status.code(), Some(0), "{repo}: {}", stderr(&output));
        assert_eq!(stdout(&output), expected, "{repo}");
    }
    let text = fs::read_to_string(root.path().join("config.toml")).unwrap();
    assert_eq!(text.matches("[repositories.").count(), 1, "{text}");
    assert_eq!(run(&root, &enclosing, &["fake", "current"]).stdout, b"e\n");

    let output = run(&root, &recreated, &["fake", "unlink"]);
    assert_eq!(
        stdout(&output),
        format!("no mapping to remove for fake: {}\n", enclosing.display())
    );
}

#[test]
fn unlink_notes_an_ancestor_mapping_and_remaining_agent_mappings() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let outer = base.join("outer");
    git_dir(&outer);
    let inner = outer.join("inner");
    git_dir(&inner);
    configure(
        &root,
        None,
        &(mapping(&outer, "profile = \"o\"") + &mapping(&inner, "agents = { fake = \"f\" }")),
    );
    let output = run(&root, &inner, &["unlink"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        format!(
            "no mapping to remove for {inner}\nnote:         {outer} has a mapping; remove it with agent-profile unlink --repo {outer}\nnote:         agent mappings remain: fake=f; remove them with agent-profile <agent> unlink\n",
            inner = inner.display(),
            outer = outer.display()
        )
    );
}

#[test]
fn a_worktree_and_a_submodule_each_resolve_their_own_mapping() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let main = base.join("main");
    git_dir(&main);
    let admin = main.join(".git").join("worktrees").join("wt");
    fs::create_dir_all(&admin).unwrap();
    fs::write(admin.join("HEAD"), "x\n").unwrap();
    fs::write(admin.join("commondir"), "../..\n").unwrap();
    let worktree = base.join("wt");
    fs::create_dir_all(&worktree).unwrap();
    fs::write(worktree.join(".git"), format!("gitdir: {}\n", admin.to_str().unwrap())).unwrap();
    let modules = main.join(".git").join("modules").join("sub");
    fs::create_dir_all(&modules).unwrap();
    fs::write(modules.join("HEAD"), "x\n").unwrap();
    let submodule = main.join("sub");
    fs::create_dir_all(&submodule).unwrap();
    fs::write(submodule.join(".git"), "gitdir: ../.git/modules/sub\n").unwrap();

    configure(
        &root,
        None,
        &(mapping(&main, "profile = \"main\"")
            + &mapping(&worktree, "profile = \"tree\"")
            + &mapping(&submodule, "profile = \"module\"")),
    );
    assert_eq!(launched_home(&root, &main), home(&root, "main"));
    assert_eq!(launched_home(&root, &worktree), home(&root, "tree"));
    assert_eq!(launched_home(&root, &submodule), home(&root, "module"));
}

#[test]
fn an_explicit_launch_ignores_a_broken_repository_and_a_resolved_launch_does_not() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let broken = base.join("broken");
    fs::create_dir_all(broken.join(".git")).unwrap();

    let output = run(&root, &broken, &["fake", "work"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(support::report(&output.stdout).get("pid").is_some());
    let dry = run(&root, &broken, &["fake", "work", "--dry-run"]);
    assert_eq!(dry.status.code(), Some(0), "{}", stderr(&dry));
    assert!(stdout(&dry).contains("repository:   none\n"), "{}", stdout(&dry));

    let output = run(&root, &broken, &["fake"]);
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert_eq!(
        stderr(&output),
        format!(
            "agent-profile: error: repository {}: invalid .git directory: no HEAD file\n",
            broken.join(".git").display()
        )
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn every_command_usage_error_through_the_binary() {
    let root = Root::new();
    let (_dir, base) = scratch();
    for (args, code, message) in [
        (&["claude", "link", "work", "--repo"][..], 2, "`--repo` needs a path"),
        (&["link", "work", "--repo", "a", "--repo", "b"], 2, "`--repo` may be given only once"),
        (&["status", "--repo="], 2, "`--repo` needs a non-empty path"),
        (&["fake", "Link", "work"], 2, "command words are lower case: `link`"),
        (&["status", "--"], 2, "`status` takes no agent arguments; remove `--`"),
        (&["unlink", "--", "--repo", "x"], 2, "`unlink` takes no agent arguments; remove `--`"),
        (&["fake", "unlink", "--verbose"], 2, "unknown option \"--verbose\" for `unlink`"),
        (&["fake", "resolve", "--json"], 2, "`--json` is not yet implemented"),
        (&["link"], 2, "`link` needs a profile: agent-profile [<agent>] link <profile>"),
        (&["fake", "link", "a", "b"], 2, "`link` takes one profile"),
        (&["fake", "current", "x"], 2, "`current` takes no arguments"),
        (&["resolve"], 2, "`resolve` needs an agent: agent-profile <agent> resolve"),
        (&["current"], 2, "`current` needs an agent: agent-profile <agent> current"),
        (
            &["fake", "--repo", "x"],
            2,
            "unknown option \"--repo\"; agent arguments must follow `--`",
        ),
        (&["LINK", "work"], 2, "unknown agent `LINK`"),
        (&["link", "status"], 4, "\"status\" is a reserved command word"),
        (&["link", "Create"], 4, "\"create\" is a reserved command word"),
    ] {
        let output = run(&root, &base, args);
        assert_eq!(output.status.code(), Some(code), "{args:?}: {}", stderr(&output));
        assert_eq!(stderr(&output).lines().count(), 1, "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).contains(message), "{args:?}: {}", stderr(&output));
        assert!(output.stdout.is_empty(), "{args:?}");
    }
}

#[test]
fn command_help_and_the_top_level_help() {
    let root = Root::new();
    let (_dir, base) = scratch();
    for (args, first) in [
        (&["link", "-h"][..], "Usage: agent-profile [<agent>] link <profile> [--repo <path>]"),
        (
            &["link", "status", "-h"],
            "Usage: agent-profile [<agent>] link <profile> [--repo <path>]",
        ),
        (&["resolve", "-h"], "Usage: agent-profile <agent> resolve [--repo <path>]"),
        (&["fake", "unlink", "--help"], "Usage: agent-profile [<agent>] unlink [--repo <path>]"),
        (&["fake", "create", "-h"], "Usage: agent-profile <agent> <profile>"),
    ] {
        let output = run(&root, &base, args);
        assert_eq!(output.status.code(), Some(0), "{args:?}");
        assert!(stdout(&output).starts_with(first), "{args:?}: {}", stdout(&output));
    }
    let output = run(&root, &base, &["--help"]);
    assert_eq!(output.status.code(), Some(0));
    for word in ["current", "resolve", "status", "link", "unlink"] {
        assert!(
            stdout(&output).lines().any(|line| line.trim_start().starts_with("agent-profile")
                && line.split_whitespace().any(|part| part == word)),
            "{word}: {}",
            stdout(&output)
        );
    }
}

#[test]
fn missing_repo_paths_exit_4() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let missing = base.join("missing");
    for args in
        [&["link", "work", "--repo", "missing"][..], &["fake", "resolve", "--repo", "missing"]]
    {
        let output = run(&root, &base, args);
        assert_eq!(output.status.code(), Some(4), "{args:?}: {}", stderr(&output));
        assert!(
            stderr(&output).starts_with(&format!(
                "agent-profile: error: repository {}: cannot resolve the directory: ",
                missing.display()
            )),
            "{args:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn link_through_a_link_to_the_repository_stores_the_canonical_root() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let real = base.join("real");
    git_dir(&real);
    let alias = base.join("alias");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, &alias).unwrap();
    #[cfg(windows)]
    {
        let status = std::process::Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(&alias)
            .arg(&real)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());
    }
    let output = run(&root, &base, &["link", "work", "--repo", "alias"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), format!("linked {} -> work\n", real.display()));
    assert_eq!(launched_home(&root, &real), home(&root, "work"));
    assert_eq!(launched_home(&root, &alias), home(&root, "work"));
}

#[cfg(unix)]
#[test]
fn a_non_utf8_repo_value_is_a_path() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let output = root
        .agent_profile(["fake", "resolve", "--repo"])
        .arg(support::non_utf8())
        .current_dir(&base)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert!(stderr(&output).contains("cannot resolve the directory"), "{}", stderr(&output));
}
