//! Agent adapters: the adapter trait and registry (spec §2, §4, §38), capability and evidence metadata
//! (spec §3, §28), profile presence (spec §8), lazy initialization (spec §9) and argument conflicts
//! (spec §21). SP2 design §4-§6.
//!
//! Adapters never spawn processes: `plan()` returns a `LaunchPlan` and the launcher runs it (spec §4).

pub mod gate;
pub mod metadata;

mod aider;
mod claude;
mod codex;
#[cfg(debug_assertions)]
mod fake;

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::config::{AppRoot, Config, check_case_twins};
use crate::error::{Error, Result};
use crate::exe::{self, Origin};
use crate::launch::LaunchPlan;
use crate::name::ProfileName;

pub use aider::Aider;
pub use claude::Claude;
pub use codex::Codex;
#[cfg(debug_assertions)]
pub use fake::Fake;
pub use metadata::{
    AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState, ConflictOption,
    EnvOverride, Mechanism, ProfilePresence, SupportLevel,
};

/// One supported coding agent (SP2 design §4.2).
pub trait Adapter: Sync {
    fn metadata(&self) -> &'static AdapterMetadata;

    /// Pure: no filesystem access. Every path the adapter owns for `profile`, in creation order. The first
    /// entry is always the adapter's profile directory.
    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)>;

    /// Reads the filesystem at most; never writes. Discovers the executable, then refuses case-only twins.
    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch>;

    /// Reads the filesystem at most; never writes; never consults `PATH` or the executable (spec §8).
    fn presence(&self, root: &AppRoot, profile: &ProfileName) -> ProfilePresence {
        if check_case_twins(root, profile).is_err() {
            return ProfilePresence::Absent;
        }
        let paths = self.paths(root, profile);
        if paths.iter().all(|(path, kind)| has_kind(path, *kind)) {
            ProfilePresence::Materialized
        } else {
            ProfilePresence::Absent
        }
    }

    /// Ensures every path in `planned.paths`, in order; idempotent; never overwrites a file (spec §9, §9.1).
    fn initialize(&self, planned: &PlannedLaunch) -> Result<()> {
        ensure_paths(&planned.paths)
    }
}

/// Everything `plan()` needs. The agent id is always `metadata().id`.
#[derive(Debug, Clone, Copy)]
pub struct PlanContext<'a> {
    pub profile: &'a ProfileName,
    pub root: &'a AppRoot,
    pub config: &'a Config,
    pub args: &'a [OsString],
    pub path_var: Option<&'a OsStr>,
}

/// What a profile path must be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathKind {
    Dir,
    /// A file created with `contents` when missing; an existing file is never modified.
    File {
        contents: &'static [u8],
    },
}

/// A path an adapter owns, and whether it already had the declared kind at plan time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePath {
    pub path: PathBuf,
    pub kind: PathKind,
    pub existed: bool,
}

/// A launch plus what the report shows but the launcher does not need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedLaunch {
    pub plan: LaunchPlan,
    pub profile: ProfileName,
    /// The adapter's profile directory, `<root>/profiles/<profile>/<agent>`.
    pub profile_dir: PathBuf,
    pub paths: Vec<ProfilePath>,
    pub executable_origin: Origin,
    /// The per-launch report text, e.g. `environment variable CODEX_HOME`.
    pub mechanism: String,
    /// Override variables whose values must never be printed (spec §22).
    pub sensitive_env: Vec<OsString>,
    /// Non-sensitive facts shown in the report.
    pub notes: Vec<String>,
}

/// The real adapters, in every build (SP2 design §4.2).
pub const REAL_ADAPTERS: &[&dyn Adapter] = &[&Claude, &Codex, &Aider];

/// The adapters this build knows: the real ones, plus `fake` when debug assertions are on.
pub fn registry() -> Vec<&'static dyn Adapter> {
    #[cfg_attr(not(debug_assertions), allow(unused_mut))]
    let mut adapters = REAL_ADAPTERS.to_vec();
    #[cfg(debug_assertions)]
    adapters.push(&Fake);
    adapters
}

/// The agent ids this build knows, in registry order.
pub fn known_agents() -> Vec<&'static str> {
    registry().iter().map(|adapter| adapter.metadata().id).collect()
}

/// The adapter for `id`, if this build knows it.
pub fn lookup(id: &str) -> Option<&'static dyn Adapter> {
    registry().into_iter().find(|adapter| adapter.metadata().id == id)
}

/// Refuses an opaque argument that selects the adapter's own mechanism (spec §21, SP2 design §6). The scan
/// stops at the first `--`, after which arguments are positional for the agent.
///
/// The scanned set is `metadata.conflicts` **chained with the mechanism's own option**, so a flag adapter
/// refuses the flag it launches with whether or not it remembered to list it: `conflicts` carries only the
/// *additional* options proven to control the same mechanism.
pub fn check_conflicts(metadata: &AdapterMetadata, args: &[OsString]) -> Result<()> {
    for arg in args.iter().take_while(|arg| arg.as_os_str() != "--") {
        let options = metadata.conflicts.iter().chain(metadata.mechanism.conflict_option());
        if options.into_iter().any(|option| option.matches(arg)) {
            return Err(Error::ArgumentConflict {
                agent: metadata.id.to_owned(),
                option: arg.to_string_lossy().into_owned(),
                mechanism: metadata.mechanism.to_string(),
            });
        }
    }
    Ok(())
}

/// `<root>/profiles/<profile>/<id>`.
pub(crate) fn profile_dir(root: &AppRoot, profile: &ProfileName, id: &str) -> PathBuf {
    root.profiles_dir().join(profile.as_str()).join(id)
}

/// Plans an adapter whose mechanism is one environment variable naming its profile directory.
pub(crate) fn env_dir_plan(
    adapter: &dyn Adapter,
    ctx: &PlanContext<'_>,
    var: &str,
) -> Result<PlannedLaunch> {
    let metadata = adapter.metadata();
    let found = discover(metadata, ctx)?;
    check_case_twins(ctx.root, ctx.profile)?;
    let paths = profile_paths(adapter, ctx);
    let dir = paths[0].path.clone();
    let mechanism = metadata.mechanism.sentence_for(&dir);
    Ok(PlannedLaunch {
        plan: LaunchPlan {
            executable: found.path,
            args: ctx.args.to_vec(),
            env: vec![(var.into(), dir.clone().into_os_string())],
            cwd: None,
        },
        profile: ctx.profile.clone(),
        profile_dir: dir,
        paths,
        executable_origin: found.origin,
        mechanism,
        sensitive_env: sensitive_env(metadata),
        notes: Vec::new(),
    })
}

/// Plans an adapter whose mechanism is an argument naming its profile's configuration file.
pub(crate) fn config_file_arg_plan(
    adapter: &dyn Adapter,
    ctx: &PlanContext<'_>,
    flag: &str,
) -> Result<PlannedLaunch> {
    let metadata = adapter.metadata();
    let found = discover(metadata, ctx)?;
    check_case_twins(ctx.root, ctx.profile)?;
    let paths = profile_paths(adapter, ctx);
    let file = paths
        .iter()
        .find(|entry| matches!(entry.kind, PathKind::File { .. }))
        .expect("a configuration-file adapter owns a file")
        .path
        .clone();
    let mut args: Vec<OsString> = vec![flag.into(), file.clone().into_os_string()];
    args.extend(ctx.args.iter().cloned());
    Ok(PlannedLaunch {
        plan: LaunchPlan { executable: found.path, args, env: Vec::new(), cwd: None },
        profile: ctx.profile.clone(),
        profile_dir: paths[0].path.clone(),
        paths,
        executable_origin: found.origin,
        mechanism: metadata.mechanism.sentence_for(&file),
        sensitive_env: sensitive_env(metadata),
        notes: Vec::new(),
    })
}

fn discover(metadata: &AdapterMetadata, ctx: &PlanContext<'_>) -> Result<exe::Found> {
    exe::discover(
        metadata.id,
        metadata.executable,
        ctx.config.agent_executable(metadata.id),
        ctx.path_var,
        &ctx.root.config_path(),
    )
}

fn profile_paths(adapter: &dyn Adapter, ctx: &PlanContext<'_>) -> Vec<ProfilePath> {
    adapter
        .paths(ctx.root, ctx.profile)
        .into_iter()
        .map(|(path, kind)| ProfilePath { existed: has_kind(&path, kind), path, kind })
        .collect()
}

fn sensitive_env(metadata: &AdapterMetadata) -> Vec<OsString> {
    metadata.env.iter().filter(|entry| entry.sensitive).map(|entry| entry.name.into()).collect()
}

/// Whether `fs::metadata` (which follows symlinks) reports the declared kind.
fn has_kind(path: &Path, kind: PathKind) -> bool {
    fs::metadata(path).is_ok_and(|metadata| match kind {
        PathKind::Dir => metadata.is_dir(),
        PathKind::File { .. } => metadata.is_file(),
    })
}

/// Ensures every path, ignoring `existed`, so a path created or removed after planning is still handled.
pub fn ensure_paths(paths: &[ProfilePath]) -> Result<()> {
    for entry in paths {
        let error = |source: io::Error| Error::ProfileDir { path: entry.path.clone(), source };
        match entry.kind {
            PathKind::Dir => {
                fs::create_dir_all(&entry.path).map_err(error)?;
                if !fs::metadata(&entry.path).map_err(error)?.is_dir() {
                    return Err(error(io::Error::other("the path exists but is not a directory")));
                }
            }
            PathKind::File { contents } => {
                if !has_kind(&entry.path, entry.kind) {
                    write_new_file(&entry.path, contents)?;
                }
            }
        }
    }
    Ok(())
}

fn write_new_file(path: &Path, contents: &[u8]) -> Result<()> {
    write_new_file_with(path, contents, || Ok(()))
}

/// A lock-free, durable, no-clobber file create (SP2 design §4.4). There is no sweep of leftover temp
/// files: without a lock a sweep could delete another launch's live temp file.
fn write_new_file_with(
    path: &Path,
    contents: &[u8],
    before_persist: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let error = |source: io::Error| Error::ProfileDir { path: path.to_path_buf(), source };
    let directory = path.parent().expect("profile file paths have a parent");
    let name = path.file_name().expect("profile file paths have a name").to_string_lossy();
    let mut temp = tempfile::Builder::new()
        .prefix(&format!("{name}."))
        .suffix(".tmp")
        .tempfile_in(directory)
        .map_err(error)?;
    temp.write_all(contents).and_then(|()| temp.as_file().sync_all()).map_err(error)?;
    before_persist()?;
    if let Err(persist) = temp.persist_noclobber(path) {
        // A concurrent launch won; its file is complete because it, too, was renamed into place.
        if fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
            return Ok(());
        }
        return Err(error(persist.error));
    }
    #[cfg(unix)]
    fs::File::open(directory).and_then(|directory| directory.sync_all()).map_err(|sync| {
        error(io::Error::new(
            sync.kind(),
            format!(
                "file created, but the directory could not be synced; it may not survive a power loss: {sync}"
            ),
        ))
    })?;
    Ok(())
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use crate::name::Platform;

    fn setup() -> (tempfile::TempDir, AppRoot, Config, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = AppRoot::from_path(dir.path().join("root"));
        let exe = dir.path().join("fake-agent-bin");
        fs::write(&exe, b"x").unwrap();
        fs::create_dir_all(root.path()).unwrap();
        fs::write(
            root.config_path(),
            format!("[agents.fake]\nexecutable = {:?}\n", exe.to_str().unwrap()),
        )
        .unwrap();
        let config = Config::load(&root).unwrap();
        (dir, root, config, exe)
    }

    fn profile(name: &str) -> ProfileName {
        ProfileName::parse(name, Platform::host()).unwrap()
    }

    fn plan_fake(
        root: &AppRoot,
        config: &Config,
        name: &str,
        args: &[OsString],
    ) -> Result<PlannedLaunch> {
        let profile = profile(name);
        Fake.plan(&PlanContext { profile: &profile, root, config, args, path_var: None })
    }

    #[test]
    fn fake_plan_sets_home_override_and_passes_args_verbatim() {
        let (_dir, root, config, exe) = setup();
        let args: Vec<OsString> = vec!["--foo".into(), "a b".into()];
        let planned = plan_fake(&root, &config, "work", &args).unwrap();
        let dir = root.profiles_dir().join("work").join("fake");
        assert_eq!(
            planned.plan,
            LaunchPlan {
                executable: exe,
                args,
                env: vec![("FAKE_AGENT_HOME".into(), dir.clone().into_os_string())],
                cwd: None
            }
        );
        assert_eq!(planned.profile_dir, dir);
        assert_eq!(
            planned.paths,
            vec![ProfilePath { path: dir, kind: PathKind::Dir, existed: false }]
        );
        assert_eq!(planned.executable_origin, Origin::Configured);
    }

    #[test]
    fn case_only_twin_is_refused_for_any_entry_type() {
        let (_dir, root, config, _exe) = setup();
        fs::create_dir_all(root.profiles_dir()).unwrap();
        fs::write(root.profiles_dir().join("work"), b"a file, not a directory").unwrap();
        let error = plan_fake(&root, &config, "WORK", &[]).unwrap_err();
        assert!(
            matches!(error, Error::ProfileCaseConflict { ref existing, .. } if existing == "work"),
            "{error:?}"
        );
        assert!(plan_fake(&root, &config, "work", &[]).is_ok());
    }

    #[test]
    fn initialize_is_idempotent_and_rejects_a_file_at_a_directory_path() {
        let (_dir, root, config, _exe) = setup();
        let planned = plan_fake(&root, &config, "work", &[]).unwrap();
        Fake.initialize(&planned).unwrap();
        Fake.initialize(&planned).unwrap();
        assert!(planned.profile_dir.is_dir());

        let blocked = plan_fake(&root, &config, "blocked", &[]).unwrap();
        fs::create_dir_all(blocked.profile_dir.parent().unwrap()).unwrap();
        fs::write(&blocked.profile_dir, b"file").unwrap();
        assert!(matches!(Fake.initialize(&blocked), Err(Error::ProfileDir { .. })));
    }

    #[test]
    fn conflict_scan_stops_at_double_dash_and_names_the_option() {
        let metadata = Aider.metadata();
        let args = |items: &[&str]| items.iter().map(OsString::from).collect::<Vec<_>>();
        match check_conflicts(metadata, &args(&["x", "--conf=f"])).unwrap_err() {
            Error::ArgumentConflict { agent, option, mechanism } => {
                assert_eq!((agent.as_str(), option.as_str()), ("aider", "--conf=f"));
                assert_eq!(mechanism, "argument --config <file>");
            }
            other => panic!("{other:?}"),
        }
        assert!(check_conflicts(metadata, &args(&["x", "--", "--config", "f"])).is_ok());
        assert!(check_conflicts(Codex.metadata(), &args(&["-p", "personal"])).is_ok());
    }

    fn new_file_path() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".aider.conf.yml");
        (dir, path)
    }

    #[test]
    fn new_file_does_not_exist_under_its_final_name_before_persist() {
        let (_dir, path) = new_file_path();
        write_new_file_with(&path, b"{}\n", || {
            assert!(!path.exists(), "the final name must appear only at persist");
            Ok(())
        })
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"{}\n");
    }

    #[test]
    fn the_temp_file_lives_beside_the_target() {
        let (dir, path) = new_file_path();
        let names = || -> Vec<String> {
            fs::read_dir(dir.path())
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        };
        write_new_file_with(&path, b"{}\n", || {
            let names = names();
            assert_eq!(names.len(), 1, "{names:?}");
            assert!(
                names[0].starts_with(".aider.conf.yml.") && names[0].ends_with(".tmp"),
                "{names:?}"
            );
            Ok(())
        })
        .unwrap();
        assert_eq!(names(), [".aider.conf.yml"]);
    }

    #[test]
    fn a_file_created_before_persist_wins_and_is_never_overwritten() {
        let (_dir, path) = new_file_path();
        write_new_file_with(&path, b"{}\n", || {
            fs::write(&path, b"theirs").unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"theirs");
    }

    #[test]
    fn a_directory_created_before_persist_is_a_profile_error() {
        let (_dir, path) = new_file_path();
        let result = write_new_file_with(&path, b"{}\n", || {
            fs::create_dir(&path).unwrap();
            Ok(())
        });
        assert!(matches!(result, Err(Error::ProfileDir { .. })), "{result:?}");
    }

    #[test]
    fn concurrent_file_creation_never_exposes_a_partial_file() {
        let (dir, path) = new_file_path();
        fs::write(dir.path().join(".aider.conf.yml.leftover.tmp"), b"stale").unwrap();
        let entry = ProfilePath {
            path: path.clone(),
            kind: PathKind::File { contents: b"{}\n" },
            existed: false,
        };
        let done = std::sync::atomic::AtomicBool::new(false);
        std::thread::scope(|scope| {
            let reader = scope.spawn(|| {
                while !done.load(std::sync::atomic::Ordering::SeqCst) {
                    if let Ok(bytes) = fs::read(&path) {
                        assert_eq!(bytes, b"{}\n");
                    }
                }
            });
            let writers: Vec<_> = (0..8)
                .map(|_| scope.spawn(|| ensure_paths(std::slice::from_ref(&entry))))
                .collect();
            for writer in writers {
                writer.join().unwrap().unwrap();
            }
            done.store(true, std::sync::atomic::Ordering::SeqCst);
            reader.join().unwrap();
        });
        assert_eq!(fs::read(&path).unwrap(), b"{}\n");
    }

    struct Secretive;

    static SECRETIVE: AdapterMetadata = AdapterMetadata {
        id: "secretive",
        executable: "fake-agent",
        mechanism: Mechanism::Env("PROFILE_SESSION_HANDLE"),
        support: SupportLevel::Experimental,
        evidence: AdapterEvidence {
            mechanism_id: "secretive-v1",
            verified_at: "2026-09-15",
            upstream_version: "0.0.0",
            source_url: "measured",
            notes: "test-only adapter",
        },
        capabilities: &[],
        env: &[EnvOverride { name: "PROFILE_SESSION_HANDLE", sensitive: true }],
        conflicts: &[],
    };

    impl Adapter for Secretive {
        fn metadata(&self) -> &'static AdapterMetadata {
            &SECRETIVE
        }

        fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
            vec![(profile_dir(root, profile, SECRETIVE.id), PathKind::Dir)]
        }

        fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
            env_dir_plan(self, ctx, "PROFILE_SESSION_HANDLE")
        }
    }

    #[test]
    fn sensitive_declarations_fill_sensitive_env_and_are_redacted() {
        let (dir, root, _config, exe) = setup();
        fs::write(
            root.config_path(),
            format!("[agents.secretive]\nexecutable = {:?}\n", exe.to_str().unwrap()),
        )
        .unwrap();
        let config = Config::load(&root).unwrap();
        let profile = profile("work");
        let ctx = PlanContext {
            profile: &profile,
            root: &root,
            config: &config,
            args: &[],
            path_var: None,
        };
        let planned = Secretive.plan(&ctx).unwrap();
        assert_eq!(planned.sensitive_env, vec![OsString::from("PROFILE_SESSION_HANDLE")]);
        let resolution = crate::resolve::resolve(
            crate::name::AgentId::parse("secretive").unwrap(),
            Some(profile.clone()),
            &config,
            &crate::repo::Discovery::NotInRepository,
        );
        let text = crate::output::report_lines(
            &planned,
            &resolution,
            crate::output::ReportMode::DryRun,
            &SECRETIVE,
        )
        .join("\n");
        assert!(text.contains("PROFILE_SESSION_HANDLE=<redacted>"), "{text}");
        drop(dir);
    }
}
