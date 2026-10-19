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

#[test]
fn agents_json_lists_every_registered_agent_with_executable_states() {
    let root = Root::new();
    let mut command = root.agent_profile(["agents", "--json"]);
    command.env_remove("PATH");
    let out = command.output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let value = document(&out);
    assert_eq!(value["schema"], "agent-profile/agents/1");
    let list = value["agents"].as_array().unwrap();
    let known = adapter::known_agents();
    assert_eq!(list.len(), known.len());
    for (element, id) in list.iter().zip(known) {
        assert_eq!(element["agent"], id);
        let executable = &element["executable"];
        if id == "fake" {
            assert_eq!(executable["state"], "found");
            assert_eq!(executable["origin"], "configured");
            assert_eq!(executable["path"], env!("CARGO_BIN_EXE_fake-agent"));
        } else {
            assert_eq!(executable["state"], "not_on_path", "{id}");
            assert_eq!(executable["path"], Value::Null);
            assert_eq!(executable["origin"], Value::Null);
        }
    }

    let root = Root::empty();
    let shim = root.path().join("fake.cmd");
    fs::write(&shim, "@echo off\n").unwrap();
    root.write_config(&format!("[agents.fake]\nexecutable = {:?}\n", shim.to_str().unwrap()));
    let out = run(&root, root.path(), &["agents", "--json"]);
    let value = document(&out);
    let fake = value["agents"].as_array().unwrap().iter().find(|a| a["agent"] == "fake").unwrap();
    assert_eq!(fake["executable"]["state"], "unsupported");
    assert_eq!(fake["executable"]["path"], shim.to_str().unwrap());

    let root = Root::empty();
    let missing =
        if cfg!(windows) { r"C:\nonexistent\fake-agent" } else { "/nonexistent/fake-agent" };
    root.write_config(&format!("[agents.fake]\nexecutable = {missing:?}\n"));
    let out = run(&root, root.path(), &["agents", "--json"]);
    let value = document(&out);
    let fake = value["agents"].as_array().unwrap().iter().find(|a| a["agent"] == "fake").unwrap();
    assert_eq!(fake["executable"]["state"], "configured_missing");
    assert_eq!(fake["executable"]["path"], missing);
    assert_eq!(fake["executable"]["origin"], Value::Null);
}

#[test]
fn profiles_and_list_json_share_the_schema_and_keep_the_stderr_notes() {
    let root = Root::empty();
    for (agent, profile) in [("fake", "work"), ("aider", "work"), ("fake", "personal")] {
        let out = run(&root, root.path(), &[agent, "create", profile]);
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    }
    fs::write(root.path().join("profiles").join("notes.txt"), "x").unwrap();

    let profiles = run(&root, root.path(), &["profiles", "--json"]);
    assert_eq!(profiles.status.code(), Some(0), "{}", stderr(&profiles));
    let value = document(&profiles);
    assert_eq!(value["schema"], "agent-profile/profiles/1");
    assert_eq!(value["agent"], Value::Null);
    assert_eq!(
        value["profiles"],
        json!([{"name": "personal", "agents": ["fake"]}, {"name": "work", "agents": ["aider", "fake"]}])
    );
    assert_eq!(value["ignored_entries"], 1);
    assert_eq!(stderr(&profiles), "agent-profile: ignored 1 unrecognized entries\n");

    let list = run(&root, root.path(), &["list", "--json"]);
    assert_eq!(stdout(&list), stdout(&profiles));

    let aider = run(&root, root.path(), &["aider", "list", "--json"]);
    let value = document(&aider);
    assert_eq!(value["agent"], "aider");
    assert_eq!(value["profiles"], json!([{"name": "work", "agents": ["aider", "fake"]}]));

    let codex = run(&root, root.path(), &["codex", "list", "--json"]);
    assert_eq!(codex.status.code(), Some(0), "{}", stderr(&codex));
    assert_eq!(document(&codex)["profiles"], json!([]));
    assert_eq!(
        stderr(&codex),
        "agent-profile: ignored 1 unrecognized entries\nagent-profile: no profiles for codex\n"
    );

    let empty = Root::empty();
    let out = run(&empty, empty.path(), &["profiles", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let value = document(&out);
    assert_eq!(value["profiles"], json!([]));
    assert_eq!(value["ignored_entries"], 0);
}

#[cfg(unix)]
#[test]
fn profiles_json_on_an_unreadable_profiles_directory_leaves_stdout_empty() {
    use std::os::unix::fs::PermissionsExt;
    let root = Root::empty();
    let out = run(&root, root.path(), &["fake", "create", "work"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let profiles = root.path().join("profiles");
    let mode =
        |bits: u32| fs::set_permissions(&profiles, fs::Permissions::from_mode(bits)).unwrap();
    mode(0o000);
    if fs::read_dir(&profiles).is_ok() {
        mode(0o755); // a privileged user is not refused: nothing to pin here
        return;
    }
    let out = run(&root, root.path(), &["profiles", "--json"]);
    mode(0o755);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(stdout(&out), "");
    assert!(stderr(&out).contains("could not read the profiles directory"), "{}", stderr(&out));
}

/// Asserts `value` is an object whose sorted member names are exactly `expected`, and that each of `nullable` is
/// present as an explicit JSON `null` (an absent key is a failure, not a pass).
fn pin(value: &Value, expected: &[&str], nullable: &[&str]) {
    assert_eq!(keys(value), expected, "{value}");
    for name in nullable {
        assert_eq!(
            value.get(*name),
            Some(&Value::Null),
            "`{name}` must be present and null in {value}"
        );
    }
}

/// The raw stdout opens with `schema` then `tool_version` and ends with exactly one newline after the closing brace.
fn assert_leads_with_schema(out: &Output, command: &str) {
    let text = stdout(out);
    let prefix =
        format!("{{\n  \"schema\": \"agent-profile/{command}/1\",\n  \"tool_version\": \"");
    assert!(text.starts_with(&prefix), "{command}: {text}");
    assert!(text.ends_with("}\n"), "{command}: {text:?}");
    assert!(!text.ends_with("}\n\n"), "{command}: {text:?}");
}

#[test]
fn every_schema_pins_its_member_lists_on_a_null_bearing_fixture() {
    // resolve with no profile, inside a repository: `profile` null; outside one: `repository` null too.
    let root = Root::empty();
    let (_dir, base) = scratch();
    let acme = base.join("acme");
    git_dir(&acme);
    let resolve = ["agent", "profile", "repository", "schema", "source", "tool_version"];
    let out = run(&root, &acme, &["fake", "resolve", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{}", stderr(&out));
    assert_leads_with_schema(&out, "resolve");
    pin(&document(&out), &resolve, &["profile"]);
    let out = run(&root, &base, &["fake", "resolve", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{}", stderr(&out));
    pin(&document(&out), &resolve, &["profile", "repository"]);

    // status outside any repository with no config: every nullable top-level member and the row members.
    let status = [
        "agent",
        "agents",
        "default_profile",
        "inapplicable_mappings",
        "mapping",
        "repository",
        "schema",
        "tool_version",
    ];
    let out = run(&root, &base, &["status", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let value = document(&out);
    pin(&value, &status, &["agent", "default_profile", "mapping", "repository"]);
    let rows = value["agents"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        pin(row, &["agent", "presence", "profile", "source"], &["presence", "profile"]);
    }
    let out = run(&root, &base, &["fake", "status", "--json"]);
    let value = document(&out);
    pin(&value, &status, &["default_profile", "mapping", "repository"]);
    assert_eq!(value["agent"], "fake");
    pin(&value["agents"][0], &["agent", "presence", "profile", "source"], &["presence", "profile"]);

    // status in a repository: `mapping` with a null `profile`, and `presence` with a null `existing`.
    configure(&root, None, &mapping(&acme, "agents = { fake = \"mine\" }"));
    let out = run(&root, &acme, &["fake", "status", "--json"]);
    let value = document(&out);
    pin(&value, &status, &["default_profile"]);
    pin(&value["mapping"], &["agents", "profile"], &["profile"]);
    pin(&value["agents"][0]["presence"], &["existing", "state"], &["existing"]);

    // presence on a conflict: `existing` is a string, the member list is unchanged.
    configure(&root, None, &mapping(&acme, "profile = \"work\""));
    fs::create_dir_all(root.path().join("profiles").join("WORK")).unwrap();
    let out = run(&root, &acme, &["fake", "status", "--json"]);
    let value = document(&out);
    pin(&value["agents"][0]["presence"], &["existing", "state"], &[]);
    assert_eq!(value["agents"][0]["presence"]["existing"], "WORK");

    // agents with PATH removed: unfound executables carry null path and origin.
    let agents_root = Root::new();
    let mut command = agents_root.agent_profile(["agents", "--json"]);
    command.env_remove("PATH");
    let out = command.output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_leads_with_schema(&out, "agents");
    let value = document(&out);
    pin(&value, &["agents", "schema", "tool_version"], &[]);
    let list = value["agents"].as_array().unwrap();
    assert!(!list.is_empty());
    let mut unfound = 0;
    for element in list {
        pin(
            element,
            &["agent", "capabilities", "evidence", "executable", "mechanism", "support"],
            &[],
        );
        let executable = &element["executable"];
        assert_eq!(keys(executable), ["origin", "path", "state"], "{element}");
        if executable["state"] == "not_on_path" {
            unfound += 1;
            pin(executable, &["origin", "path", "state"], &["origin", "path"]);
        }
        pin(&element["mechanism"], &["description", "id"], &[]);
        pin(&element["evidence"], &["notes", "source_url", "upstream_version", "verified_at"], &[]);
        let capabilities = element["capabilities"].as_array().unwrap();
        assert!(!capabilities.is_empty(), "{element}");
        for capability in capabilities {
            pin(capability, &["basis", "capability", "state"], &[]);
        }
    }
    assert!(unfound > 0, "no agent exercised the null executable path");

    // profiles on an empty root.
    let empty = Root::empty();
    let out = run(&empty, empty.path(), &["profiles", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_leads_with_schema(&out, "profiles");
    let value = document(&out);
    pin(&value, &["agent", "ignored_entries", "profiles", "schema", "tool_version"], &["agent"]);
    assert_eq!(value["profiles"], json!([]));
    let work = Root::empty();
    let out = run(&work, work.path(), &["fake", "create", "work"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let value = document(&run(&work, work.path(), &["profiles", "--json"]));
    pin(&value["profiles"][0], &["agents", "name"], &[]);

    // repositories: a missing-key entry with an empty table has a null `detail` and a null `profile`.
    let table_root = Root::empty();
    let gone = base.join("gone");
    let present = base.join("present");
    git_dir(&present);
    configure(
        &table_root,
        None,
        &(mapping(&gone, "") + &mapping(&present, "profile = \"p\"\nagents = { fake = \"q\" }")),
    );
    let out = run(&table_root, table_root.path(), &["repositories", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_leads_with_schema(&out, "repositories");
    let value = document(&out);
    pin(&value, &["repositories", "schema", "tool_version"], &[]);
    let entries = value["repositories"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    let entry_keys = ["agents", "detail", "profile", "repository", "state"];
    let missing = entries.iter().find(|e| e["state"] == "missing").unwrap();
    pin(missing, &entry_keys, &["detail", "profile"]);
    let active = entries.iter().find(|e| e["state"] == "active").unwrap();
    pin(active, &entry_keys, &["detail"]);
    pin(&active["profile"], &["exists", "name"], &[]);
    pin(&active["agents"]["fake"], &["exists", "known_agent", "profile"], &[]);
}

/// Visits every object member of `value`: its name must not contain `=`, and a string value must not look like a
/// secret unless the member is one of the free-text members.
fn walk_for_secrets(value: &Value, member: &str, found: &mut usize) {
    match value {
        Value::Object(map) => {
            for (name, inner) in map {
                assert!(!name.contains('='), "member name {name:?} contains '='");
                walk_for_secrets(inner, name, found);
            }
        }
        Value::Array(items) => {
            for item in items {
                walk_for_secrets(item, member, found);
            }
        }
        Value::String(text) => {
            *found += 1;
            if !["basis", "notes", "description"].contains(&member) {
                for word in ["TOKEN", "SECRET", "PASSWORD"] {
                    assert!(
                        !text.contains(word),
                        "member {member:?} value {text:?} contains {word}"
                    );
                }
            }
        }
        _ => {}
    }
}

#[test]
fn documents_carry_no_environment_or_secret_shaped_member() {
    let root = Root::new();
    let (_dir, base) = scratch();
    let acme = base.join("acme");
    git_dir(&acme);
    configure(&root, Some("work"), &mapping(&acme, "profile = \"work\""));
    let mut documents = Vec::new();
    for (args, cwd) in [
        (&["agents", "--json"][..], root.path()),
        (&["status", "--json"], acme.as_path()),
        (&["fake", "status", "--json"], acme.as_path()),
        (&["status", "--json"], base.as_path()),
        (&["repositories", "--json"], base.as_path()),
    ] {
        let mut command = root.agent_profile(args);
        command
            .current_dir(cwd)
            .env("API_TOKEN", "TOKEN=leak")
            .env("MY_SECRET", "SECRET=leak")
            .env("DB_PASSWORD", "PASSWORD=leak");
        let out = command.output().unwrap();
        assert_eq!(out.status.code(), Some(0), "{args:?}: {}", stderr(&out));
        documents.push(document(&out));
    }
    let mut found = 0;
    for value in &documents {
        walk_for_secrets(value, "", &mut found);
    }
    assert!(found > 20, "the walk visited only {found} strings");
}
