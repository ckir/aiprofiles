//! The common adapter contract suite (spec §34 "LaunchPlan", SP2 design §8.1): every registered adapter is
//! checked by the same assertions. Adding an adapter without a row here fails `plan_contract`.

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use agent_profile::adapter::gate::GateFailure;
use agent_profile::adapter::{
    self, Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    PathKind, PlanContext, PlannedLaunch, ProfilePath, ProfilePresence, SupportLevel, gate,
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

    // Gates A-shape, B and C over every real adapter. `fake` is excluded from A and C by design: it has no
    // upstream product, so demanding evidence of one would force a fabricated transcript (SP4 design D4).
    for adapter in adapter::REAL_ADAPTERS {
        let metadata = adapter.metadata();
        assert_eq!(gate::gates_before_transcripts(metadata), Ok(()), "{}", metadata.id);
    }
    // Gate B applies to every adapter including the fixture, which has real claims and must not model bad
    // ones.
    for adapter in &registry {
        let metadata = adapter.metadata();
        assert_eq!(gate::gate_b(metadata), Ok(()), "{}", metadata.id);
    }

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
        for (path, _) in &declared {
            assert!(
                path.starts_with(&planned.profile_dir),
                "{}: {} declares a path outside its profile dir {}",
                adapter.metadata().id,
                path.display(),
                planned.profile_dir.display()
            );
        }
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
        assert_eq!(adapter.presence(&fixture.root, &work), ProfilePresence::Materialized, "{id}");
        fs::remove_file(&fixture.exe).unwrap();
        fs::remove_file(fixture.root.config_path()).unwrap();
        assert_eq!(adapter.presence(&fixture.root, &work), ProfilePresence::Materialized, "{id}");
        // Materialize the twin's own paths too, so only the case-twin rule can make it Absent on a
        // case-sensitive filesystem (on a case-insensitive one they are the same entries).
        for (path, kind) in adapter.paths(&fixture.root, &profile("WORK")) {
            match kind {
                PathKind::Dir => fs::create_dir_all(&path).unwrap(),
                PathKind::File { contents } => fs::write(&path, contents).unwrap(),
            }
        }
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

#[test]
fn case_twin_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        fs::create_dir_all(fixture.root.profiles_dir().join("work")).unwrap();
        let error = fixture.plan(adapter, "WORK", &[]).unwrap_err();
        assert!(
            matches!(error, Error::ProfileCaseConflict { ref existing, .. } if existing == "work"),
            "{id}: {error:?}"
        );
        let entries: Vec<OsString> = fs::read_dir(fixture.root.profiles_dir())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(entries, [OsString::from("work")], "{id}");
    }
}

#[test]
fn not_installed_wins_over_a_case_twin_contract() {
    for adapter in adapter::registry() {
        let dir = tempfile::tempdir().unwrap();
        let root = AppRoot::from_path(dir.path().join("root"));
        fs::create_dir_all(root.profiles_dir().join("work")).unwrap();
        let empty = dir.path().join("empty");
        fs::create_dir(&empty).unwrap();
        let config = Config::load(&root).unwrap();
        let twin = profile("WORK");
        let id = adapter.metadata().id;
        let error = adapter
            .plan(&PlanContext {
                profile: &twin,
                root: &root,
                config: &config,
                args: &[],
                path_var: Some(empty.as_os_str()),
            })
            .unwrap_err();
        assert!(matches!(error, Error::AgentNotInstalled { .. }), "{id}: {error:?}");
    }
}

#[test]
fn existed_follows_the_declared_kind_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        let planned = fixture.plan(adapter, "work", &[]).unwrap();
        adapter.initialize(&planned).unwrap();
        let replanned = fixture.plan(adapter, "work", &[]).unwrap();
        assert!(replanned.paths.iter().all(|entry| entry.existed), "{id}: {:?}", replanned.paths);
    }
    let fixture = fixture();
    let aider = adapter::lookup("aider").unwrap();
    let planned = fixture.plan(aider, "work", &[]).unwrap();
    fs::create_dir_all(&planned.paths[1].path).unwrap();
    let existed: Vec<bool> =
        fixture.plan(aider, "work", &[]).unwrap().paths.iter().map(|entry| entry.existed).collect();
    assert_eq!(existed, [true, false], "a directory at the Aider file is not the file");
}

#[test]
fn initialization_ignores_a_stale_existed_contract() {
    for adapter in adapter::registry() {
        let fixture = fixture();
        let id = adapter.metadata().id;
        let first = fixture.plan(adapter, "work", &[]).unwrap();
        adapter.initialize(&first).unwrap();
        let stale = fixture.plan(adapter, "work", &[]).unwrap();
        fs::remove_dir_all(&stale.profile_dir).unwrap();
        adapter.initialize(&stale).unwrap();
        assert_eq!(
            adapter.presence(&fixture.root, &profile("work")),
            ProfilePresence::Materialized,
            "{id}"
        );
    }
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

// --- Gate negative fixtures (SP4 design §10) ---------------------------------------------------------
//
// The gates are library functions rather than inline assertions precisely so these can exist: a rule
// expressed only over `static METADATA` items is unreachable by `cargo mutants`, so a weaker-than-intended
// gate would pass because no shipped adapter exhibits the excluded combination.

/// A metadata value that passes every gate, for a test to break in exactly one way.
fn sound_metadata() -> AdapterMetadata {
    AdapterMetadata {
        id: "fixture",
        executable: "fixture",
        mechanism_summary: "environment variable FIXTURE_HOME",
        support: SupportLevel::Proven,
        evidence: AdapterEvidence {
            mechanism_id: "fixture-home-v1",
            verified_at: "2026-09-16",
            upstream_version: "1.0.0",
            source_url: "measured",
            notes: "measured in a sandbox",
        },
        capabilities: &[
            CapabilityClaim {
                capability: Capability::ConfigIsolation,
                state: CapabilityState::Supported,
                basis: "measured: config moved with the variable",
            },
            CapabilityClaim {
                capability: Capability::CredentialIsolation,
                state: CapabilityState::Unknown,
                basis: "unmeasured: requires an authenticated session",
            },
            CapabilityClaim {
                capability: Capability::StateIsolation,
                state: CapabilityState::NotSupported,
                basis: "measured: sessions stay in the default location",
            },
        ],
        env: &[],
        conflicts: &[],
    }
}

#[test]
fn the_sound_fixture_passes_every_gate() {
    assert_eq!(gate::gates_before_transcripts(&sound_metadata()), Ok(()));
}

#[test]
fn gate_b_rejects_an_unmeasured_prefix_on_a_non_unknown_state() {
    // The D5 attack verbatim: without the biconditional this reaches `Proven` with nothing measured.
    let mut metadata = sound_metadata();
    metadata.capabilities = &[CapabilityClaim {
        capability: Capability::ConfigIsolation,
        state: CapabilityState::NotGuaranteed,
        basis: "unmeasured: no vendor session available",
    }];
    assert_eq!(
        gate::gate_b(&metadata),
        Err(GateFailure::UnmeasuredMismatch {
            id: "fixture",
            capability: Capability::ConfigIsolation,
            state: CapabilityState::NotGuaranteed,
        })
    );
}

#[test]
fn gate_b_rejects_an_unknown_state_without_the_unmeasured_prefix() {
    let mut metadata = sound_metadata();
    metadata.capabilities = &[CapabilityClaim {
        capability: Capability::CredentialIsolation,
        state: CapabilityState::Unknown,
        basis: "measured: this claim was never measured, whatever it says",
    }];
    assert_eq!(
        gate::gate_b(&metadata),
        Err(GateFailure::UnmeasuredMismatch {
            id: "fixture",
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
        })
    );
}

#[test]
fn gate_b_rejects_a_basis_without_a_provenance_prefix() {
    let mut metadata = sound_metadata();
    metadata.capabilities = &[CapabilityClaim {
        capability: Capability::ConfigIsolation,
        state: CapabilityState::Supported,
        basis: "config moved with the variable",
    }];
    assert!(matches!(gate::gate_b(&metadata), Err(GateFailure::BasisPrefix { .. })));
}

#[test]
fn gate_b_rejects_a_basis_containing_a_newline() {
    // One `Vec` entry is one report line; an embedded newline would silently render as two.
    let mut metadata = sound_metadata();
    metadata.capabilities = &[CapabilityClaim {
        capability: Capability::ConfigIsolation,
        state: CapabilityState::Supported,
        basis: "measured: first line\nsecond line",
    }];
    assert_eq!(
        gate::gate_b(&metadata),
        Err(GateFailure::BasisNewline { id: "fixture", capability: Capability::ConfigIsolation })
    );
}

#[test]
fn gate_c_rejects_proven_with_two_unknown_claims() {
    let mut metadata = sound_metadata();
    metadata.capabilities = &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: the probe never ran",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: requires an authenticated session",
        },
    ];
    assert_eq!(
        gate::gate_c(&metadata),
        Err(GateFailure::ProvenWithTooManyUnknowns { id: "fixture", unknowns: 2 })
    );
}

#[test]
fn gate_c_rejects_proven_with_empty_notes() {
    let mut metadata = sound_metadata();
    metadata.evidence.notes = "";
    assert_eq!(gate::gate_c(&metadata), Err(GateFailure::ProvenWithoutNotes { id: "fixture" }));
}

#[test]
fn gate_c_rejects_an_unknown_version_that_did_not_degrade_to_experimental() {
    // Without this clause, Gate A's token exemption is a hole: a failed probe could still ship `Proven`
    // beside a `Supported` config claim, with no mechanism token observed anywhere.
    let mut metadata = sound_metadata();
    metadata.evidence.upstream_version = "unknown";
    assert_eq!(
        gate::gate_c(&metadata),
        Err(GateFailure::UnknownVersionNotExperimental { id: "fixture" })
    );

    metadata.support = SupportLevel::Experimental;
    assert_eq!(
        gate::gate_c(&metadata),
        Err(GateFailure::UnknownVersionNotExperimental { id: "fixture" }),
        "experimental alone is not enough; the config claim must be Unknown too"
    );

    metadata.capabilities = &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: the probe never ran",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: requires an authenticated session",
        },
    ];
    assert_eq!(gate::gate_c(&metadata), Ok(()));
}

/// The existing three-arm test above never isolates the support-level clause: with `unknowns > 1` its
/// third arm returns `Ok` for a different reason before that clause is reached, so a version reading the
/// support-level check away entirely still leaves all three arms agreeing with the weakened predicate.
/// This fixture holds `support == Proven` and exactly one `Unknown` claim (`ConfigIsolation`), so the
/// support-level clause is the sole thing standing between it and `Ok`.
///
/// TRAP: a second `Unknown` claim here would make `gate_c` fail on `unknowns > 1`
/// (`ProvenWithTooManyUnknowns`) before ever reaching the support-level clause, pinning the wrong rule.
#[test]
fn gate_c_rejects_a_proven_adapter_whose_probe_failed_even_with_an_unknown_config_claim() {
    let mut metadata = sound_metadata();
    metadata.evidence.upstream_version = "unknown";
    metadata.capabilities = &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: the probe never ran",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::NotSupported,
            basis: "measured: requires an authenticated session",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::NotSupported,
            basis: "measured: sessions stay in the default location",
        },
    ];
    assert_eq!(
        gate::gate_c(&metadata),
        Err(GateFailure::UnknownVersionNotExperimental { id: "fixture" })
    );
}

#[test]
fn gate_a_rejects_a_version_outside_the_permitted_charset() {
    // Gate A builds `docs/evidence/<id>-<version>.md` from this field, so it must name one file.
    let mut metadata = sound_metadata();
    metadata.evidence.upstream_version = "1.0.0 (build 7)";
    assert_eq!(
        gate::gate_a_shape(&metadata),
        Err(GateFailure::VersionCharset { id: "fixture", version: "1.0.0 (build 7)" })
    );
}

/// `"".bytes().all(...)` is vacuously true, so the charset clause alone never rejects an empty version;
/// only the `is_empty()` disjunct does.
#[test]
fn gate_a_rejects_an_empty_upstream_version() {
    let mut metadata = sound_metadata();
    metadata.evidence.upstream_version = "";
    assert_eq!(
        gate::gate_a_shape(&metadata),
        Err(GateFailure::VersionCharset { id: "fixture", version: "" })
    );
}

#[test]
fn gate_a_rejects_a_source_url_that_is_neither_a_url_nor_measured() {
    let mut metadata = sound_metadata();
    metadata.evidence.source_url = "the vendor told me";
    assert!(matches!(gate::gate_a_shape(&metadata), Err(GateFailure::SourceUrlShape { .. })));
    metadata.evidence.source_url = "https://example.com/docs";
    assert_eq!(gate::gate_a_shape(&metadata), Ok(()));
}

#[test]
fn gate_a_rejects_measured_without_notes() {
    let mut metadata = sound_metadata();
    metadata.evidence.notes = "";
    assert_eq!(
        gate::gate_a_shape(&metadata),
        Err(GateFailure::MeasuredWithoutNotes { id: "fixture" })
    );
}
