//! `--json` documents end to end through the `agent-profile` binary (SP5b design §3.2 to §3.4, §5). Repositories
//! are synthetic `.git` layouts in guarded temp directories, so no test needs `git` and none discovers this checkout.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use agent_profile::{adapter, repo};
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

/// Parses stdout as one JSON document; panics with the text when it is not.
fn document(out: &Output) -> Value {
    let text = stdout(out);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("not JSON ({error}): {text:?}"))
}

/// The sorted member names of an object.
fn keys(value: &Value) -> Vec<String> {
    let mut names: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
    names.sort();
    names
}

#[test]
fn resolve_json_matches_the_text_form_and_keeps_exit_4_without_a_profile() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let acme = base.join("acme");
    git_dir(&acme);
    configure(&root, None, &mapping(&acme, "agents = { fake = \"mine\" }"));
    let output = run(&root, &acme, &["fake", "resolve", "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let tool_version = env!("CARGO_PKG_VERSION");
    assert_eq!(
        document(&output),
        json!({
            "schema": "agent-profile/resolve/1",
            "tool_version": tool_version,
            "agent": "fake",
            "profile": "mine",
            "source": "repository_agent_mapping",
            "repository": acme.display().to_string(),
        })
    );
    assert_eq!(stderr(&output), "");

    configure(&root, None, "");
    let output = run(&root, &acme, &["fake", "resolve", "--json"]);
    let text = run(&root, &acme, &["fake", "resolve"]);
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    let value = document(&output);
    assert_eq!(value["profile"], Value::Null);
    assert_eq!(value["source"], "none");
    assert_ne!(stderr(&output), "");
    assert_eq!(stderr(&output), stderr(&text));
}

#[test]
fn status_json_bare_and_per_agent_agree_with_the_text_report() {
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

    let bare = run(&root, &inner, &["status", "--json"]);
    assert_eq!(bare.status.code(), Some(0), "{}", stderr(&bare));
    let value = document(&bare);
    assert_eq!(value["schema"], "agent-profile/status/1");
    assert_eq!(value["agent"], Value::Null);
    assert_eq!(value["repository"], inner.display().to_string());
    assert_eq!(value["mapping"], json!({"profile": "inner", "agents": {"fake": "mine"}}));
    assert_eq!(value["default_profile"], "work");
    let rows = value["agents"].as_array().unwrap();
    let known = adapter::known_agents();
    assert_eq!(rows.len(), known.len());
    for (row, id) in rows.iter().zip(&known) {
        assert_eq!(row["agent"], *id);
        assert_eq!(row["presence"], Value::Null, "{row}");
    }
    assert_eq!(value["inapplicable_mappings"], json!([outer.display().to_string()]));

    let one = run(&root, &inner, &["fake", "status", "--json"]);
    assert_eq!(one.status.code(), Some(0), "{}", stderr(&one));
    let value = document(&one);
    assert_eq!(value["agent"], "fake");
    assert_eq!(value["agents"].as_array().unwrap().len(), 1);
    assert_eq!(value["agents"][0]["profile"], "mine");
    assert_eq!(value["agents"][0]["presence"], json!({"state": "absent", "existing": null}));

    let launch = run(&root, &inner, &["fake"]);
    assert_eq!(launch.status.code(), Some(0), "{}", stderr(&launch));
    let value = document(&run(&root, &inner, &["fake", "status", "--json"]));
    assert_eq!(value["agents"][0]["presence"]["state"], "materialized");
}

#[test]
fn status_json_outside_a_repository_with_no_config_is_all_null() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let output = run(&root, &base, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let value = document(&output);
    assert_eq!(
        keys(&value),
        [
            "agent",
            "agents",
            "default_profile",
            "inapplicable_mappings",
            "mapping",
            "repository",
            "schema",
            "tool_version"
        ]
    );
    assert_eq!(value["repository"], Value::Null);
    assert_eq!(value["mapping"], Value::Null);
    assert_eq!(value["default_profile"], Value::Null);
    assert_eq!(value["inapplicable_mappings"], json!([]));
    let rows = value["agents"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert_eq!(row["profile"], Value::Null, "{row}");
        assert_eq!(row["source"], "none", "{row}");
        assert_eq!(row["presence"], Value::Null, "{row}");
    }
}

#[test]
fn agent_status_json_reports_a_case_twin_as_a_conflict() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let repository = base.join("acme");
    git_dir(&repository);
    configure(&root, None, &mapping(&repository, "profile = \"work\""));
    fs::create_dir_all(root.path().join("profiles").join("WORK")).unwrap();
    let output = run(&root, &repository, &["fake", "status", "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        document(&output)["agents"][0]["presence"],
        json!({"state": "conflict", "existing": "WORK"})
    );
}

#[test]
fn json_leaves_stderr_and_exit_codes_as_the_text_form() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let acme = base.join("acme");
    git_dir(&acme);
    for resolvable in [true, false] {
        if resolvable {
            configure(&root, None, &mapping(&acme, "profile = \"work\""));
        } else {
            configure(&root, None, "");
        }
        for args in [&["status"][..], &["fake", "status"], &["fake", "resolve"]] {
            let plain = run(&root, &acme, args);
            let mut with_json = args.to_vec();
            with_json.push("--json");
            let json = run(&root, &acme, &with_json);
            assert_eq!(json.status.code(), plain.status.code(), "{args:?} resolvable={resolvable}");
            assert_eq!(stderr(&json), stderr(&plain), "{args:?} resolvable={resolvable}");
        }
    }
}

#[test]
fn json_members_are_in_order_and_documents_end_with_one_newline() {
    let root = Root::empty();
    let (_dir, base) = scratch();
    let output = run(&root, &base, &["status", "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.starts_with("{\n  \"schema\": \"agent-profile/status/1\",\n  \"tool_version\": \""),
        "{text}"
    );
    assert!(text.ends_with("}\n"), "{text:?}");
    assert!(!text.ends_with("}\n\n"), "{text:?}");
}
