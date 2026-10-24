// Copyright (c) 2026 Costas Kirgoussios
//
// This software is licensed under the PolyForm Noncommercial License 1.0.0.
// You may obtain a copy of the License at:
// https://polyformproject.org/licenses/noncommercial/1.0.0/

//! `repositories` end to end through the `agent-profile` binary, text and JSON forms (SP5b design §4).

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use agent_profile::repo;
use serde_json::{Value, json};
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

#[test]
fn repositories_prints_one_block_per_mapping_with_states_and_annotations() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let acme = base.join("acme");
    git_dir(&acme);
    let sub = acme.join("sub");
    fs::create_dir_all(&sub).unwrap();
    let plain = base.join("plain");
    fs::create_dir_all(&plain).unwrap();
    let gone = base.join("gone");
    let file = base.join("file.txt");
    fs::write(&file, "x").unwrap();
    let empty = base.join("empty");
    git_dir(&empty);
    let foreign = if cfg!(windows) { "/srv/acme" } else { r"C:\src\acme" };
    let p = |path: &Path| path.to_str().unwrap().to_owned();
    let mut tables = String::new();
    tables +=
        &table(&p(&acme), "profile = \"work\"\nagents = { fake = \"mine\", nosuch = \"work\" }");
    tables += &table(&p(&sub), "profile = \"work\"");
    tables += &table(&p(&plain), "profile = \"work\"");
    tables += &table(&p(&gone), "profile = \"work\"");
    tables += &table(&p(&file), "profile = \"work\"");
    tables += &table(&p(&empty), "");
    tables += &table(foreign, "profile = \"work\"");
    configure(&root, &tables);
    let created = run(&root, &["fake", "create", "work"]);
    assert_eq!(created.status.code(), Some(0), "{}", stderr(&created));

    let output = run(&root, &["repositories"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(stderr(&output), "");
    let text = stdout(&output);
    let blocks: Vec<&str> = text.trim_end_matches('\n').split("\n\n").collect();
    // Sorted by key string: `/` sorts before `C`, so the foreign key comes last.
    let mut keys =
        vec![p(&acme), p(&sub), p(&plain), p(&gone), p(&file), p(&empty), foreign.to_owned()];
    keys.sort();
    assert_eq!(blocks.len(), keys.len(), "{text}");
    for (block, key) in blocks.iter().zip(&keys) {
        assert_eq!(block.lines().next().unwrap(), format!("repository:   {key}"), "{text}");
    }
    let block = |path: &str| {
        blocks
            .iter()
            .find(|b| b.starts_with(&format!("repository:   {path}\n")))
            .unwrap()
            .to_owned()
    };
    assert_eq!(
        block(&p(&acme)),
        format!(
            "repository:   {}\nstate:        active\nprofile:      work\n\
             agents:       fake=mine (no profile directory), nosuch=work (unknown agent)",
            p(&acme)
        )
    );
    assert_eq!(
        block(&p(&sub)),
        format!(
            "repository:   {}\nstate:        moved (now {})\nprofile:      work\nagents:       none",
            p(&sub),
            p(&acme)
        )
    );
    assert!(block(&p(&plain)).contains("\nstate:        not_a_repository\n"), "{text}");
    assert!(block(&p(&gone)).contains("\nstate:        missing\n"), "{text}");
    assert!(block(&p(&file)).contains("\nstate:        unreadable (not a directory)\n"), "{text}");
    assert!(block(foreign).contains("\nstate:        foreign\n"), "{text}");
    assert_eq!(
        block(&p(&empty)),
        format!(
            "repository:   {}\nstate:        active\nprofile:      none\nagents:       none\n\
             note:         maps nothing; remove it by editing config.toml",
            p(&empty)
        )
    );
    assert_eq!(text.matches("note:").count(), 1, "{text}");
    // The pair that is both missing and unknown carries its annotations in the design's order.
    let both = Root::empty();
    let (_d2, b2) = scratch();
    let repo2 = b2.join("r");
    git_dir(&repo2);
    configure(&both, &table(&p(&repo2), "agents = { nosuch = \"nowhere\" }"));
    let output = run(&both, &["repositories"]);
    assert!(
        stdout(&output)
            .contains("agents:       nosuch=nowhere (no profile directory) (unknown agent)\n"),
        "{}",
        stdout(&output)
    );
}

/// Design §4 rule 6: a key that resolves through a symlink to a root with another spelling is `moved`, never
/// `active`, because nothing discovers the symlink spelling as a root.
#[cfg(unix)]
#[test]
fn repositories_reports_a_symlinked_key_as_moved_to_the_real_root() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let real = base.join("real");
    git_dir(&real);
    let alias = base.join("alias");
    std::os::unix::fs::symlink(&real, &alias).unwrap();
    configure(&root, &table(alias.to_str().unwrap(), "profile = \"work\""));

    let text = run(&root, &["repositories"]);
    assert_eq!(text.status.code(), Some(0), "{}", stderr(&text));
    assert_eq!(
        stdout(&text),
        format!(
            "repository:   {}\nstate:        moved (now {})\n\
             profile:      work (no profile directory)\nagents:       none\n",
            alias.display(),
            real.display()
        )
    );

    let json = run(&root, &["repositories", "--json"]);
    assert_eq!(json.status.code(), Some(0), "{}", stderr(&json));
    let entry = &document(&json)["repositories"][0];
    assert_eq!(entry["repository"], alias.display().to_string());
    assert_eq!(entry["state"], "moved");
    assert_eq!(entry["detail"], real.display().to_string());
}

#[test]
fn repositories_with_no_mappings_prints_the_empty_line_and_creates_nothing() {
    let root = Root::empty();
    let output = run(&root, &["repositories"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), "no repository mappings\n");
    assert_eq!(stderr(&output), "");
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn repositories_continues_past_an_unreadable_entry_sorted_first() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    // `aaa` is a regular file (unreadable) and sorts before `bbb`, a repository.
    let aaa = base.join("aaa");
    fs::write(&aaa, "x").unwrap();
    let bbb = base.join("bbb");
    git_dir(&bbb);
    let tables = table(aaa.to_str().unwrap(), "profile = \"p\"")
        + &table(bbb.to_str().unwrap(), "profile = \"p\"");
    configure(&root, &tables);
    let output = run(&root, &["repositories"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("state:        unreadable (not a directory)\n"), "{text}");
    assert!(
        text.contains(&format!("repository:   {}\nstate:        active\n", bbb.display())),
        "{text}"
    );
    assert!(text.find("aaa").unwrap() < text.find("bbb").unwrap(), "{text}");
}

#[test]
fn repositories_refuses_an_agent_word_arguments_and_repo() {
    let root = Root::new();
    for (args, message) in [
        (
            &["fake", "repositories"][..],
            "`repositories` takes no agent: agent-profile repositories",
        ),
        (&["repositories", "x"], "`repositories` takes no arguments"),
        (&["repositories", "--repo", "x"], "unknown option \"--repo\" for `repositories`"),
        (&["repositories", "--yes"], "unknown option \"--yes\" for `repositories`"),
    ] {
        let output = run(&root, args);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).contains(message), "{args:?}: {}", stderr(&output));
        assert_eq!(stdout(&output), "", "{args:?}");
    }
}

/// The sorted member names of an object.
fn keys(value: &Value) -> Vec<String> {
    let mut names: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
    names.sort();
    names
}

fn document(output: &Output) -> Value {
    let text = stdout(output);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("not JSON ({error}): {text:?}"))
}

#[test]
fn repositories_json_carries_every_state_with_detail_and_references() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let acme = base.join("acme");
    git_dir(&acme);
    let sub = acme.join("sub");
    fs::create_dir_all(&sub).unwrap();
    let plain = base.join("plain");
    fs::create_dir_all(&plain).unwrap();
    let gone = base.join("gone");
    let file = base.join("file.txt");
    fs::write(&file, "x").unwrap();
    let empty = base.join("empty");
    git_dir(&empty);
    let badgit = base.join("badgit");
    fs::create_dir_all(&badgit).unwrap();
    let missing_gitdir = if cfg!(windows) { r"C:\nonexistent\x" } else { "/nonexistent/x" };
    fs::write(badgit.join(".git"), format!("gitdir: {missing_gitdir}\n")).unwrap();
    let foreign = if cfg!(windows) { "/srv/acme" } else { r"C:\src\acme" };
    let p = |path: &Path| path.to_str().unwrap().to_owned();
    let mut tables = String::new();
    tables +=
        &table(&p(&acme), "profile = \"work\"\nagents = { fake = \"mine\", nosuch = \"work\" }");
    tables += &table(&p(&sub), "profile = \"work\"");
    tables += &table(&p(&plain), "profile = \"work\"");
    tables += &table(&p(&gone), "profile = \"work\"");
    tables += &table(&p(&file), "profile = \"work\"");
    tables += &table(&p(&empty), "");
    tables += &table(&p(&badgit), "profile = \"work\"");
    tables += &table(foreign, "profile = \"work\"");
    configure(&root, &tables);
    let created = run(&root, &["fake", "create", "work"]);
    assert_eq!(created.status.code(), Some(0), "{}", stderr(&created));

    let output = run(&root, &["repositories", "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(stderr(&output), "");
    let value = document(&output);
    let entries = value["repositories"].as_array().unwrap();
    let mut expected = vec![
        p(&acme),
        p(&sub),
        p(&plain),
        p(&gone),
        p(&file),
        p(&empty),
        p(&badgit),
        foreign.to_owned(),
    ];
    expected.sort();
    let listed: Vec<&str> = entries.iter().map(|e| e["repository"].as_str().unwrap()).collect();
    assert_eq!(listed, expected);
    let entry = |path: &str| {
        entries.iter().find(|e| e["repository"] == path).unwrap_or_else(|| panic!("{path}"))
    };
    let work = json!({ "name": "work", "exists": true });

    let acme_entry = entry(&p(&acme));
    assert_eq!(acme_entry["state"], "active");
    assert_eq!(acme_entry["detail"], Value::Null);
    assert_eq!(acme_entry["profile"], work);
    assert_eq!(
        acme_entry["agents"],
        json!({
            "fake": { "profile": "mine", "exists": false, "known_agent": true },
            "nosuch": { "profile": "work", "exists": true, "known_agent": false },
        })
    );
    let sub_entry = entry(&p(&sub));
    assert_eq!(sub_entry["state"], "moved");
    assert_eq!(sub_entry["detail"], json!(p(&acme)));
    assert_eq!(sub_entry["profile"], work);
    assert_eq!(sub_entry["agents"], json!({}));
    for (path, state) in
        [(p(&plain), "not_a_repository"), (p(&gone), "missing"), (foreign.to_owned(), "foreign")]
    {
        assert_eq!(entry(&path)["state"], state);
        assert_eq!(entry(&path)["detail"], Value::Null, "{path}");
        assert_eq!(entry(&path)["profile"], work);
    }
    let file_entry = entry(&p(&file));
    assert_eq!(file_entry["state"], "unreadable");
    assert_eq!(file_entry["detail"], "not a directory");
    let badgit_entry = entry(&p(&badgit));
    assert_eq!(badgit_entry["state"], "unreadable");
    assert!(!badgit_entry["detail"].as_str().unwrap().is_empty(), "{badgit_entry}");
    let empty_entry = entry(&p(&empty));
    assert_eq!(empty_entry["state"], "active");
    assert_eq!(empty_entry["profile"], Value::Null);
    assert_eq!(empty_entry["agents"], json!({}));

    let blank = Root::empty();
    let output = run(&blank, &["repositories", "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(stderr(&output), "");
    assert_eq!(document(&output)["repositories"], json!([]));
}

#[test]
fn repositories_json_members_are_pinned_on_a_null_bearing_entry() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let gone = base.join("gone");
    configure(&root, &table(gone.to_str().unwrap(), ""));
    let output = run(&root, &["repositories", "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let value = document(&output);
    assert_eq!(keys(&value), ["repositories", "schema", "tool_version"]);
    assert_eq!(value["schema"], "agent-profile/repositories/1");
    let entry = &value["repositories"][0];
    assert_eq!(keys(entry), ["agents", "detail", "profile", "repository", "state"]);
    assert_eq!(entry["state"], "missing");
    assert_eq!(entry.get("detail"), Some(&Value::Null));
    assert_eq!(entry.get("profile"), Some(&Value::Null));
    assert_eq!(entry["agents"], json!({}));
}

#[test]
fn an_orphaned_mapping_is_reported_missing_and_never_rewritten_by_reads() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let acme = base.join("acme");
    let other = base.join("other");
    git_dir(&acme);
    git_dir(&other);
    configure(&root, "");
    let mut link = root.agent_profile(["fake", "link", "work"]);
    link.current_dir(&acme);
    let link = link.output().unwrap();
    assert_eq!(link.status.code(), Some(0), "{}", stderr(&link));
    fs::remove_dir_all(&acme).unwrap();

    let config = root.path().join("config.toml");
    let before = fs::read(&config).unwrap();
    let header = format!("[repositories.{}]", key(acme.to_str().unwrap()));
    assert!(String::from_utf8_lossy(&before).contains(&header));
    // `current` and `resolve` find no profile for the other repository and exit 4; the point is the config bytes.
    for (cwd, args) in [
        (&base, &["repositories"][..]),
        (&base, &["repositories", "--json"]),
        (&base, &["status"]),
        (&base, &["fake", "status"]),
        (&base, &["fake", "resolve"]),
        (&base, &["fake", "current"]),
        (&other, &["repositories"]),
        (&other, &["status", "--json"]),
        (&other, &["fake", "resolve", "--json"]),
        (&other, &["fake", "current"]),
    ] {
        let mut command = root.agent_profile(args);
        command.current_dir(cwd);
        let output = command.output().unwrap();
        let shown = format!("{args:?}: {}{}", stdout(&output), stderr(&output));
        assert!(matches!(output.status.code(), Some(0 | 4)), "{shown}");
        assert!(fs::read(&config).unwrap() == before, "config.toml changed after {shown}");
        if args[0] == "repositories" {
            assert_eq!(output.status.code(), Some(0), "{shown}");
        }
    }

    let output = run(&root, &["repositories"]);
    assert!(
        stdout(&output)
            .contains(&format!("repository:   {}\nstate:        missing\n", acme.display())),
        "{}",
        stdout(&output)
    );
    let output = run(&root, &["repositories", "--json"]);
    assert!(stdout(&output).contains("\"missing\""), "{}", stdout(&output));
}
