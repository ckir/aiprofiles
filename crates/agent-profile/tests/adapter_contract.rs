//! The common adapter contract suite (spec §34 "LaunchPlan", SP2 design §8.1): every registered adapter is
//! checked by the same assertions. Adding an adapter without a row here fails `plan_contract`.

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use agent_profile::adapter::{
    self, Adapter, Capability, PathKind, PlanContext, PlannedLaunch, ProfilePath, ProfilePresence,
    SupportLevel,
};
use agent_profile::config::{AppRoot, Config};
use agent_profile::error::{Error, Result};
use agent_profile::launch::LaunchPlan;
use agent_profile::name::{AgentId, Platform, ProfileName};

struct Fixture {
    _dir: tempfile::TempDir,
    root: AppRoot,
    exe: PathBuf,
}

/// An application root whose `config.toml` points every registered agent at one dummy executable.
fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = AppRoot::from_path(dir.path().join("root"));
    fs::create_dir_all(root.path()).unwrap();
    let exe = dir.path().join("agent-bin");
    fs::write(&exe, b"x").unwrap();
    let config: String = adapter::registry()
        .iter()
        .map(|adapter| {
            format!(
                "[agents.{}]\nexecutable = {:?}\n",
                adapter.metadata().id,
                exe.to_str().unwrap()
            )
        })
        .collect();
    fs::write(root.config_path(), config).unwrap();
    Fixture { _dir: dir, root, exe }
}

fn profile(name: &str) -> ProfileName {
    ProfileName::parse(name, Platform::host()).unwrap()
}

fn args(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

impl Fixture {
    fn plan(
        &self,
        adapter: &dyn Adapter,
        name: &str,
        opaque: &[OsString],
    ) -> Result<PlannedLaunch> {
        let config = Config::load(&self.root).unwrap();
        let profile = profile(name);
        adapter.plan(&PlanContext {
            profile: &profile,
            root: &self.root,
            config: &config,
            args: opaque,
            path_var: None,
        })
    }

    fn dir(&self, id: &str) -> PathBuf {
        self.root.profiles_dir().join("work").join(id)
    }
}

/// The exact plan design §5 specifies for profile `work` and opaque args `["x", "a b"]`.
fn expected(fixture: &Fixture, id: &str) -> PlannedLaunch {
    let dir = fixture.dir(id);
    let opaque = args(&["x", "a b"]);
    let env_dir = |var: &str, notes: Vec<String>| PlannedLaunch {
        plan: LaunchPlan {
            executable: fixture.exe.clone(),
            args: opaque.clone(),
            env: vec![(var.into(), dir.clone().into_os_string())],
            cwd: None,
        },
        profile: profile("work"),
        profile_dir: dir.clone(),
        paths: vec![ProfilePath { path: dir.clone(), kind: PathKind::Dir, existed: false }],
        executable_origin: agent_profile::exe::Origin::Configured,
        mechanism: format!("environment variable {var}"),
        sensitive_env: Vec::new(),
        notes,
    };
    match id {
        "claude" => env_dir("CLAUDE_CONFIG_DIR", Vec::new()),
        "codex" => env_dir(
            "CODEX_HOME",
            vec!["new profile starts logged out; run codex login with this profile".to_owned()],
        ),
        "fake" => env_dir("FAKE_AGENT_HOME", Vec::new()),
        "aider" => {
            let file = dir.join(".aider.conf.yml");
            let mut planned_args = vec![OsString::from("--config"), file.clone().into_os_string()];
            planned_args.extend(opaque.clone());
            PlannedLaunch {
                plan: LaunchPlan {
                    executable: fixture.exe.clone(),
                    args: planned_args,
                    env: Vec::new(),
                    cwd: None,
                },
                profile: profile("work"),
                profile_dir: dir.clone(),
                paths: vec![
                    ProfilePath { path: dir.clone(), kind: PathKind::Dir, existed: false },
                    ProfilePath {
                        path: file.clone(),
                        kind: PathKind::File { contents: b"{}\n" },
                        existed: false,
                    },
                ],
                executable_origin: agent_profile::exe::Origin::Configured,
                mechanism: format!("argument --config {}", file.display()),
                sensitive_env: Vec::new(),
                notes: vec![
                    "--config is layered over .aider.conf.yml in the working directory, git root and home"
                        .to_owned(),
                ],
            }
        }
        other => panic!("no contract row for adapter `{other}`; add one to this suite"),
    }
}

#[test]
fn plan_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        let planned = fixture.plan(adapter, "work", &args(&["x", "a b"])).unwrap();
        assert_eq!(planned, expected(&fixture, id), "{id}");
    }
}

#[test]
fn codex_new_profile_note_disappears_once_the_home_exists() {
    let fixture = fixture();
    let codex = adapter::lookup("codex").unwrap();
    let planned = fixture.plan(codex, "work", &[]).unwrap();
    codex.initialize(&planned).unwrap();
    assert_eq!(fixture.plan(codex, "work", &[]).unwrap().notes, Vec::<String>::new());
}

#[test]
fn declared_environment_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let metadata = adapter.metadata();
        let planned = fixture.plan(adapter, "work", &[]).unwrap();
        for (name, _) in &planned.plan.env {
            assert!(
                metadata.env.iter().any(|declared| name == declared.name),
                "{}: {name:?} is set but not declared",
                metadata.id
            );
        }
        let sensitive: Vec<OsString> = metadata
            .env
            .iter()
            .filter(|declared| declared.sensitive)
            .map(|declared| declared.name.into())
            .collect();
        assert_eq!(planned.sensitive_env, sensitive, "{}", metadata.id);
    }
}

#[test]
fn conflict_contract() {
    for adapter in adapter::registry() {
        let metadata = adapter.metadata();
        let (refused, accepted): (&[&[&str]], &[&[&str]]) = match metadata.id {
            "aider" => (
                &[
                    &["--config", "f"],
                    &["--config=f"],
                    &["--confi", "f"],
                    &["--conf", "f"],
                    &["--con=f"],
                    &["-c", "f"],
                    &["-cf"],
                ],
                &[&["--co"], &["--code-theme", "x"], &["--", "--config", "f"]],
            ),
            "codex" => (&[], &[&["-p", "personal"], &["--profile", "personal"]]),
            "claude" => (&[], &[&["--settings", "s"]]),
            "fake" => (&[&["--fake-profile", "x"]], &[&["--", "--fake-profile", "x"]]),
            other => panic!("no conflict row for adapter `{other}`"),
        };
        for items in refused {
            assert!(
                matches!(
                    adapter::check_conflicts(metadata, &args(items)),
                    Err(Error::ArgumentConflict { .. })
                ),
                "{}: {items:?} must be refused",
                metadata.id
            );
        }
        for items in accepted {
            assert!(
                adapter::check_conflicts(metadata, &args(items)).is_ok(),
                "{}: {items:?} must be accepted",
                metadata.id
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn a_non_utf8_config_value_is_still_a_conflict() {
    use std::os::unix::ffi::OsStringExt;
    let arg = OsString::from_vec(b"--config=\xff".to_vec());
    let metadata = adapter::lookup("aider").unwrap().metadata();
    assert!(matches!(
        adapter::check_conflicts(metadata, &[arg]),
        Err(Error::ArgumentConflict { .. })
    ));
}

fn is_iso_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        })
}

#[test]
fn metadata_invariants() {
    let registry = adapter::registry();
    let mut ids: Vec<&str> = registry.iter().map(|adapter| adapter.metadata().id).collect();
    for adapter in &registry {
        let metadata = adapter.metadata();
        let id = metadata.id;
        assert_eq!(
            AgentId::parse(id).map(|parsed| parsed.as_str().to_owned()),
            Some(id.to_owned())
        );
        assert_eq!(adapter::lookup(id).map(|found| found.metadata().id), Some(id));
        for capability in Capability::ALL {
            let claims: Vec<_> = metadata
                .capabilities
                .iter()
                .filter(|claim| claim.capability == capability)
                .collect();
            assert_eq!(claims.len(), 1, "{id}: {capability:?}");
            assert!(!claims[0].basis.is_empty(), "{id}: {capability:?}");
        }
        assert_eq!(metadata.capabilities.len(), Capability::ALL.len(), "{id}");
        let evidence = metadata.evidence;
        assert!(is_iso_date(evidence.verified_at), "{id}: {}", evidence.verified_at);
        for field in [evidence.mechanism_id, evidence.upstream_version, evidence.source_url] {
            assert!(!field.is_empty(), "{id}");
        }
        for option in metadata.conflicts {
            assert!(option.long.iter().all(|spelling| spelling.starts_with("--")), "{id}");
        }
    }
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), registry.len(), "adapter ids must be unique");

    let real: Vec<(&str, SupportLevel)> = adapter::REAL_ADAPTERS
        .iter()
        .map(|adapter| (adapter.metadata().id, adapter.metadata().support))
        .collect();
    assert_eq!(
        real,
        [
            ("claude", SupportLevel::Proven),
            ("codex", SupportLevel::Proven),
            ("aider", SupportLevel::Proven)
        ]
    );
}

#[test]
fn paths_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let planned = fixture.plan(adapter, "work", &[]).unwrap();
        let declared: Vec<(PathBuf, PathKind)> =
            planned.paths.iter().map(|entry| (entry.path.clone(), entry.kind)).collect();
        assert_eq!(
            adapter.paths(&fixture.root, &profile("work")),
            declared,
            "{}",
            adapter.metadata().id
        );
        assert_eq!(planned.profile_dir, declared[0].0, "{}", adapter.metadata().id);
        assert_eq!(declared[0].1, PathKind::Dir, "{}", adapter.metadata().id);
    }
}

#[test]
fn presence_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        let work = profile("work");
        assert_eq!(adapter.presence(&fixture.root, &work), ProfilePresence::Absent, "{id}");
        let planned = fixture.plan(adapter, "work", &[]).unwrap();
        adapter.initialize(&planned).unwrap();
        fs::remove_file(&fixture.exe).unwrap();
        fs::remove_file(fixture.root.config_path()).unwrap();
        assert_eq!(adapter.presence(&fixture.root, &work), ProfilePresence::Materialized, "{id}");
        assert_eq!(
            adapter.presence(&fixture.root, &profile("WORK")),
            ProfilePresence::Absent,
            "{id}"
        );
    }
    let fixture = fixture();
    let aider = adapter::lookup("aider").unwrap();
    let planned = fixture.plan(aider, "work", &[]).unwrap();
    aider.initialize(&planned).unwrap();
    fs::remove_file(&planned.paths[1].path).unwrap();
    assert!(planned.profile_dir.is_dir());
    assert_eq!(aider.presence(&fixture.root, &profile("work")), ProfilePresence::Absent);
}

fn assert_profile_dir_error(result: Result<()>, context: &str) {
    assert!(matches!(result, Err(Error::ProfileDir { .. })), "{context}: {result:?}");
}

#[test]
fn initialization_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        let planned = fixture.plan(adapter, "work", &[]).unwrap();
        adapter.initialize(&planned).unwrap();
        adapter.initialize(&planned).unwrap();
        for entry in &planned.paths {
            match entry.kind {
                PathKind::Dir => assert!(entry.path.is_dir(), "{id}"),
                PathKind::File { contents } => {
                    assert_eq!(fs::read(&entry.path).unwrap(), contents, "{id}")
                }
            }
        }

        let blocked = fixture.plan(adapter, "blocked", &[]).unwrap();
        fs::create_dir_all(blocked.profile_dir.parent().unwrap()).unwrap();
        fs::write(&blocked.profile_dir, b"a file where a directory belongs").unwrap();
        assert_profile_dir_error(adapter.initialize(&blocked), id);
    }

    let fixture = fixture();
    let aider = adapter::lookup("aider").unwrap();
    let planned = fixture.plan(aider, "work", &[]).unwrap();
    assert_eq!(planned.paths[1].kind, PathKind::File { contents: b"{}\n" });
    fs::create_dir_all(&planned.profile_dir).unwrap();
    fs::write(&planned.paths[1].path, b"model: mine\n").unwrap();
    aider.initialize(&planned).unwrap();
    assert_eq!(fs::read(&planned.paths[1].path).unwrap(), b"model: mine\n");

    let directory = fixture.plan(aider, "directory", &[]).unwrap();
    fs::create_dir_all(&directory.paths[1].path).unwrap();
    assert_profile_dir_error(aider.initialize(&directory), "directory at the Aider file");

    #[cfg(unix)]
    {
        let dangling = fixture.plan(aider, "dangling", &[]).unwrap();
        fs::create_dir_all(&dangling.profile_dir).unwrap();
        std::os::unix::fs::symlink(dangling.profile_dir.join("nowhere"), &dangling.paths[1].path)
            .unwrap();
        assert_profile_dir_error(aider.initialize(&dangling), "dangling symlink at the Aider file");
    }
}
