//! `repositories` end to end through the `agent-profile` binary, text form (SP5b design §4). The JSON tests join
//! this file with the document.

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

#[cfg(unix)]
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
    let foreign = r"C:\src\acme";
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
        (&["repositories", "--json"], "`--json` is not yet implemented"),
    ] {
        let output = run(&root, args);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).contains(message), "{args:?}: {}", stderr(&output));
        assert_eq!(stdout(&output), "", "{args:?}");
    }
}
