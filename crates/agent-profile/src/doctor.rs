//! `doctor`: the check model and the host and repository checks (SP5c design §4, §5.1, §5.2, §6).
//!
//! Every check is read-only. A check that cannot run because one it depends on failed emits exactly one
//! `skip` record naming that check (design §4).

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::adapter;
use crate::config::{AppRoot, Config};
use crate::error::Error;
use crate::json;
use crate::lifecycle;
use crate::name::Platform;
use crate::name::{AgentId, ProfileName};
use crate::repo::{self, Discovery};
use crate::repositories::{self, State};

/// The outcome of one check (design §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Info,
    Warn,
    Fail,
    Skip,
}

impl Status {
    pub fn token(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Info => "info",
            Status::Warn => "warn",
            Status::Fail => "fail",
            Status::Skip => "skip",
        }
    }
}

/// The closed set of check identifiers (design §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckId {
    Platform,
    AppRoot,
    AppRootWritable,
    Config,
    ConfigAgent,
    ProfilesDir,
    CaseTwins,
    DefaultProfile,
    Repository,
    Mapping,
    MappingAgent,
    Executable,
    RepositoryLocalExecutable,
    Support,
    Resolution,
    Profile,
    ProfilePath,
    LaunchEnvironment,
    ConflictingOption,
    Capability,
}

impl CheckId {
    pub fn token(self) -> &'static str {
        match self {
            CheckId::Platform => "platform",
            CheckId::AppRoot => "app_root",
            CheckId::AppRootWritable => "app_root_writable",
            CheckId::Config => "config",
            CheckId::ConfigAgent => "config_agent",
            CheckId::ProfilesDir => "profiles_dir",
            CheckId::CaseTwins => "case_twins",
            CheckId::DefaultProfile => "default_profile",
            CheckId::Repository => "repository",
            CheckId::Mapping => "mapping",
            CheckId::MappingAgent => "mapping_agent",
            CheckId::Executable => "executable",
            CheckId::RepositoryLocalExecutable => "repository_local_executable",
            CheckId::Support => "support",
            CheckId::Resolution => "resolution",
            CheckId::Profile => "profile",
            CheckId::ProfilePath => "profile_path",
            CheckId::LaunchEnvironment => "launch_environment",
            CheckId::ConflictingOption => "conflicting_option",
            CheckId::Capability => "capability",
        }
    }
}

/// One check record.
#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub id: CheckId,
    pub agent: Option<&'static str>,
    pub subject: Option<String>,
    pub status: Status,
    pub message: String,
    /// `Value::Null` when the id has no data.
    pub data: Value,
}

/// What a run reads from the process.
pub struct Environment<'a> {
    /// `AppRoot::resolve()`, its error rendered with `Display`.
    pub root: std::result::Result<AppRoot, String>,
    pub path_var: Option<&'a OsStr>,
    /// The process's current directory.
    pub cwd: &'a Path,
    /// The current directory, or `cwd.join(--repo)`.
    pub start: &'a Path,
    /// Whether an environment variable is set.
    pub is_set: &'a dyn Fn(&str) -> bool,
}

/// What to examine.
pub enum Target {
    Host,
    Agent { agent: AgentId, profile: Option<String> },
}

/// The ordered records of a run, and the facts the JSON envelope needs.
pub struct Report {
    pub agent: Option<AgentId>,
    /// The profile the agent form examined.
    pub profile: Option<ProfileName>,
    pub repository: Option<PathBuf>,
    pub checks: Vec<Check>,
}

impl Report {
    pub fn failed(&self) -> bool {
        self.count(Status::Fail) > 0
    }

    pub fn count(&self, status: Status) -> usize {
        self.checks.iter().filter(|check| check.status == status).count()
    }
}

/// Runs every check in scope, in the contract's order: host checks, then repository checks.
pub fn run(target: &Target, env: &Environment<'_>) -> Report {
    let mut report = Report {
        agent: match target {
            Target::Host => None,
            Target::Agent { agent, .. } => Some(agent.clone()),
        },
        profile: None,
        repository: None,
        checks: Vec::new(),
    };
    let host = host_checks(env, &mut report.checks);
    report.repository = repository_checks(env, &host, &mut report.checks);
    report
}

/// What the host checks established, for the checks that depend on them.
struct Host {
    root: Option<AppRoot>,
    config: Option<Config>,
}

fn check(
    id: CheckId,
    subject: Option<String>,
    status: Status,
    message: String,
    data: Value,
) -> Check {
    Check { id, agent: None, subject, status, message, data }
}

fn skip(id: CheckId, dependency: CheckId) -> Check {
    check(id, None, Status::Skip, format!("skipped: {} failed", dependency.token()), Value::Null)
}

fn shown(path: &Path) -> String {
    path.display().to_string()
}

/// The nearest ancestor of `path` (or `path` itself) that exists, following symbolic links. A dangling
/// link, a missing entry and an entry below a non-directory all count as not existing; any other error
/// stops the walk at the path that raised it.
fn nearest_existing(path: &Path) -> Option<PathBuf> {
    let mut current = path;
    loop {
        match fs::metadata(current) {
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) =>
            {
                current = current.parent()?;
            }
            _ => return Some(current.to_owned()),
        }
    }
}

fn host_checks(env: &Environment<'_>, checks: &mut Vec<Check>) -> Host {
    checks.push(check(
        CheckId::Platform,
        None,
        Status::Info,
        format!(
            "{} {}, agent-profile {}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            json::TOOL_VERSION
        ),
        json!({ "os": std::env::consts::OS, "arch": std::env::consts::ARCH }),
    ));
    let mut host = Host { root: None, config: None };
    let root = match &env.root {
        Ok(root) => root,
        Err(message) => {
            checks.push(check(
                CheckId::AppRoot,
                None,
                Status::Fail,
                format!("cannot resolve the application root: {message}"),
                Value::Null,
            ));
            for id in [
                CheckId::AppRootWritable,
                CheckId::Config,
                CheckId::ConfigAgent,
                CheckId::ProfilesDir,
                CheckId::CaseTwins,
                CheckId::DefaultProfile,
            ] {
                checks.push(skip(id, CheckId::AppRoot));
            }
            return host;
        }
    };
    let root_ok = app_root(root, checks);
    if !root_ok {
        for id in [
            CheckId::AppRootWritable,
            CheckId::Config,
            CheckId::ConfigAgent,
            CheckId::ProfilesDir,
            CheckId::CaseTwins,
            CheckId::DefaultProfile,
        ] {
            checks.push(skip(id, CheckId::AppRoot));
        }
        return host;
    }
    host.root = Some(root.clone());
    app_root_writable(root, checks);
    host.config = config_checks(root, checks);
    match profiles_dir(root, checks) {
        ProfilesDir::Present => case_twins(root, checks),
        ProfilesDir::Absent => {}
        ProfilesDir::Failed => checks.push(skip(CheckId::CaseTwins, CheckId::ProfilesDir)),
    }
    match &host.config {
        Some(config) => checks.push(default_profile(config)),
        None => checks.push(skip(CheckId::DefaultProfile, CheckId::Config)),
    }
    host
}

/// Pushes the `app_root` record and says whether the root can be used.
fn app_root(root: &AppRoot, checks: &mut Vec<Check>) -> bool {
    let path = root.path();
    let subject = Some(shown(path));
    let (status, message, exists) = match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            (Status::Pass, format!("application root {} is a directory", path.display()), true)
        }
        Ok(_) => (
            Status::Fail,
            format!(
                "application root {} exists and is not a directory; remove it or set AGENT_PROFILE_HOME",
                path.display()
            ),
            true,
        ),
        Err(error) if error.kind() == io::ErrorKind::NotFound => (
            Status::Info,
            format!(
                "application root {} does not exist; it is created on first use",
                path.display()
            ),
            false,
        ),
        Err(error) => (
            Status::Fail,
            format!("cannot examine the application root {}: {error}", path.display()),
            true,
        ),
    };
    let usable = status != Status::Fail;
    checks.push(check(
        CheckId::AppRoot,
        subject,
        status,
        message,
        json!({ "path": shown(path), "exists": exists }),
    ));
    usable
}

fn app_root_writable(root: &AppRoot, checks: &mut Vec<Check>) {
    let path = root.path();
    let subject = Some(shown(path));
    let exists = fs::metadata(path).is_ok();
    let data = json!({ "path": shown(path), "exists": exists });
    #[cfg(windows)]
    {
        checks.push(check(
            CheckId::AppRootWritable,
            subject,
            Status::Info,
            format!("writability of {} is not checked on this platform", path.display()),
            data,
        ));
    }
    #[cfg(not(windows))]
    {
        let probe = nearest_existing(path).unwrap_or_else(|| path.to_owned());
        let (status, message) = match fs::metadata(&probe) {
            Ok(metadata) if metadata.permissions().readonly() => (
                Status::Fail,
                format!(
                    "{} has no write permission bit, so lazy initialization and create cannot create anything under {}; \
                     change its permissions (only permission bits were read, never ACLs or ownership)",
                    probe.display(),
                    path.display()
                ),
            ),
            Ok(_) => (
                Status::Pass,
                format!(
                    "{} has a write permission bit (only permission bits were read, never ACLs or ownership)",
                    probe.display()
                ),
            ),
            Err(error) => (
                Status::Fail,
                format!(
                    "cannot examine {} to decide whether {} is writable: {error}",
                    probe.display(),
                    path.display()
                ),
            ),
        };
        checks.push(check(CheckId::AppRootWritable, subject, status, message, data));
    }
}

/// Pushes `config`, `config_agent` and returns the configuration when it loaded.
fn config_checks(root: &AppRoot, checks: &mut Vec<Check>) -> Option<Config> {
    let path = root.config_path();
    let subject = Some(shown(&path));
    let exists = fs::metadata(&path).is_ok();
    let data = json!({ "path": shown(&path), "exists": exists });
    match Config::load(root) {
        Ok(config) => {
            let (status, message) = if exists {
                (Status::Pass, format!("configuration {} is valid", path.display()))
            } else {
                (Status::Info, format!("no configuration at {}; defaults apply", path.display()))
            };
            checks.push(check(CheckId::Config, subject, status, message, data));
            let known = adapter::known_agents();
            for id in config.configured_agents() {
                if !known.contains(&id) {
                    checks.push(check(
                        CheckId::ConfigAgent,
                        Some(id.to_owned()),
                        Status::Warn,
                        format!(
                            "`{id}` in {} is not a known agent; this table is ignored",
                            path.display()
                        ),
                        Value::Null,
                    ));
                }
            }
            Some(config)
        }
        Err(error) => {
            checks.push(check(CheckId::Config, subject, Status::Fail, error.to_string(), data));
            checks.push(skip(CheckId::ConfigAgent, CheckId::Config));
            None
        }
    }
}

fn default_profile(config: &Config) -> Check {
    match config.default_profile() {
        Some(name) => check(
            CheckId::DefaultProfile,
            Some(name.to_string()),
            Status::Info,
            format!("the global default profile is `{name}`"),
            Value::Null,
        ),
        None => check(
            CheckId::DefaultProfile,
            None,
            Status::Info,
            "no global default profile is set".to_owned(),
            Value::Null,
        ),
    }
}

/// What the `profiles_dir` check established for the checks that depend on it.
enum ProfilesDir {
    Present,
    Absent,
    /// The failure names the profiles directory itself (design §4).
    Failed,
}

/// Pushes the `profiles_dir` record.
fn profiles_dir(root: &AppRoot, checks: &mut Vec<Check>) -> ProfilesDir {
    let dir = root.profiles_dir();
    let adapters = adapter::registry();
    match lifecycle::list_profiles(root, &adapters) {
        Ok(listing) => {
            let exists = fs::metadata(&dir).is_ok();
            let (status, message) = if exists {
                (
                    Status::Pass,
                    format!(
                        "profiles directory {} lists {} profile(s), {} entr(ies) ignored",
                        dir.display(),
                        listing.rows.len(),
                        listing.ignored
                    ),
                )
            } else {
                (
                    Status::Info,
                    format!(
                        "profiles directory {} does not exist; it is created on first use",
                        dir.display()
                    ),
                )
            };
            checks.push(check(
                CheckId::ProfilesDir,
                Some(shown(&dir)),
                status,
                message,
                json!({
                    "path": shown(&dir),
                    "exists": exists,
                    "profiles": listing.rows.len(),
                    "ignored": listing.ignored,
                }),
            ));
            if exists { ProfilesDir::Present } else { ProfilesDir::Absent }
        }
        Err(error) => {
            let (subject, itself) = match fs::read_dir(&dir) {
                Err(_) => (dir.clone(), true),
                Ok(_) => match unreadable_profile(&dir) {
                    Some(child) => (child, false),
                    None => (dir.clone(), true),
                },
            };
            checks.push(check(
                CheckId::ProfilesDir,
                Some(shown(&subject)),
                Status::Fail,
                format!("{error}; fix the permissions of {} or remove it", subject.display()),
                json!({
                    "path": shown(&subject),
                    "exists": true,
                    "profiles": Value::Null,
                    "ignored": Value::Null,
                }),
            ));
            if itself { ProfilesDir::Failed } else { ProfilesDir::Present }
        }
    }
}

/// The first profile directory, in byte order of its name, that `list_profiles` would descend into and
/// whose `read_dir` fails.
fn unreadable_profile(dir: &Path) -> Option<PathBuf> {
    let mut children: Vec<(String, PathBuf)> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_owned();
            (!name.starts_with('.') && ProfileName::parse(&name, Platform::host()).is_ok())
                .then(|| (name, entry.path()))
        })
        .collect();
    children.sort_by(|(a, _), (b, _)| a.as_bytes().cmp(b.as_bytes()));
    children.into_iter().map(|(_, path)| path).find(
        |path| matches!(fs::read_dir(path), Err(error) if error.kind() != io::ErrorKind::NotFound),
    )
}

/// One `fail` per pair of `profiles/` entries whose UTF-8 names differ only in ASCII case.
fn case_twins(root: &AppRoot, checks: &mut Vec<Check>) {
    let Ok(entries) = fs::read_dir(root.profiles_dir()) else {
        return;
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .collect();
    names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    for (index, a) in names.iter().enumerate() {
        for b in &names[index + 1..] {
            if a.eq_ignore_ascii_case(b) {
                checks.push(check(
                    CheckId::CaseTwins,
                    Some(a.clone()),
                    Status::Fail,
                    format!(
                        "profiles `{a}` and `{b}` differ only in case; every command on either exits 4; rename or remove one"
                    ),
                    json!({ "names": [a, b] }),
                ));
            }
        }
    }
}

/// Pushes the repository checks and returns the discovered root.
fn repository_checks(
    env: &Environment<'_>,
    host: &Host,
    checks: &mut Vec<Check>,
) -> Option<PathBuf> {
    let start = env.start;
    let found = match repo::discover(start) {
        Ok(Discovery::Repository(root)) => {
            checks.push(check(
                CheckId::Repository,
                Some(shown(start)),
                Status::Info,
                format!("{} is in the repository rooted at {}", start.display(), root.display()),
                Value::Null,
            ));
            Some(root)
        }
        Ok(Discovery::NotInRepository) => {
            checks.push(check(
                CheckId::Repository,
                Some(shown(start)),
                Status::Info,
                format!("{} is not in a repository", start.display()),
                Value::Null,
            ));
            None
        }
        Err(error) => {
            let reason = match &error {
                Error::Repository { reason, .. } => reason.clone(),
                other => other.to_string(),
            };
            checks.push(check(
                CheckId::Repository,
                Some(shown(start)),
                Status::Fail,
                format!("cannot discover the repository from {}: {reason}", start.display()),
                Value::Null,
            ));
            None
        }
    };
    mapping_checks(host, checks);
    found
}

fn mapping_checks(host: &Host, checks: &mut Vec<Check>) {
    let (Some(root), Some(config)) = (&host.root, &host.config) else {
        checks.push(skip(CheckId::Mapping, mapping_dependency(host)));
        checks.push(skip(CheckId::MappingAgent, mapping_dependency(host)));
        return;
    };
    let entries = repositories::report(root, config);
    for entry in &entries {
        let key = &entry.key;
        let detail = entry.detail.as_deref().unwrap_or("");
        let hint = "run `agent-profile repositories` and edit config.toml";
        let (status, message) = match entry.state {
            State::Active => (Status::Pass, format!("mapping {key} is active")),
            State::Moved => (
                Status::Warn,
                format!(
                    "mapping {key} is below the repository rooted at {detail}, so it no longer applies; {hint}"
                ),
            ),
            State::Missing => (
                Status::Warn,
                format!("mapping {key} names a directory that does not exist; {hint}"),
            ),
            State::NotARepository => (
                Status::Warn,
                format!("mapping {key} is a directory that is not a repository; {hint}"),
            ),
            State::Unreadable => {
                (Status::Warn, format!("mapping {key} cannot be examined ({detail}); {hint}"))
            }
            State::Foreign => (
                Status::Info,
                format!(
                    "mapping {key} is a path of another platform; it is kept but never applies here"
                ),
            ),
        };
        let data = serde_json::to_value(json::repository_view(entry)).unwrap_or(Value::Null);
        checks.push(check(CheckId::Mapping, Some(key.clone()), status, message, data));
    }
    for entry in &entries {
        for (agent, reference) in &entry.agents {
            if !reference.known {
                checks.push(check(
                    CheckId::MappingAgent,
                    Some(format!("{} {agent}", entry.key)),
                    Status::Warn,
                    format!(
                        "mapping {} names `{agent}`, which is not a known agent; edit config.toml",
                        entry.key
                    ),
                    Value::Null,
                ));
            }
        }
    }
}

/// The failed check that makes `mapping` and `mapping_agent` skip.
fn mapping_dependency(host: &Host) -> CheckId {
    if host.root.is_none() { CheckId::AppRoot } else { CheckId::Config }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(root: std::result::Result<AppRoot, String>, dir: &'a Path) -> Environment<'a> {
        Environment { root, path_var: None, cwd: dir, start: dir, is_set: &|_| false }
    }

    fn find(report: &Report, id: CheckId) -> Vec<&Check> {
        report.checks.iter().filter(|check| check.id == id).collect()
    }

    fn only(report: &Report, id: CheckId) -> &Check {
        let found = find(report, id);
        assert_eq!(found.len(), 1, "{:?}: {:?}", id, found);
        found[0]
    }

    fn skipped(report: &Report, id: CheckId) {
        let found = find(report, id);
        assert_eq!(found.len(), 1, "{id:?}: {found:?}");
        assert_eq!(found[0].status, Status::Skip, "{id:?}");
        assert_eq!(found[0].subject, None);
    }

    /// A tempdir holding an application root directory `root` (not yet created) and a start directory.
    struct Fixture {
        _dir: tempfile::TempDir,
        base: PathBuf,
        root: PathBuf,
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let base = repo::canonical(dir.path()).unwrap();
        let root = base.join("root");
        Fixture { _dir: dir, base, root }
    }

    fn run_host(fixture: &Fixture) -> Report {
        let environment = env(Ok(AppRoot::from_path(fixture.root.clone())), &fixture.base);
        run(&Target::Host, &environment)
    }

    #[cfg(unix)]
    fn set_mode(path: &Path, mode: u32) {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }

    fn git_dir(path: &Path) {
        fs::create_dir_all(path.join(".git")).unwrap();
        fs::write(path.join(".git").join("HEAD"), "ref: refs/heads/main\n").unwrap();
    }

    #[test]
    fn fresh_root_is_info_not_fail() {
        let f = fixture();
        let report = run_host(&f);
        assert_eq!(only(&report, CheckId::AppRoot).status, Status::Info);
        assert_eq!(only(&report, CheckId::Config).status, Status::Info);
        assert_eq!(only(&report, CheckId::ProfilesDir).status, Status::Info);
        assert!(find(&report, CheckId::CaseTwins).is_empty());
        assert!(!report.failed());
    }

    #[test]
    fn unresolvable_root_skips_its_dependents() {
        let f = fixture();
        let environment = env(Err("no home".to_owned()), &f.base);
        let report = run(&Target::Host, &environment);
        assert_eq!(only(&report, CheckId::AppRoot).status, Status::Fail);
        for id in [
            CheckId::AppRootWritable,
            CheckId::Config,
            CheckId::ConfigAgent,
            CheckId::DefaultProfile,
            CheckId::ProfilesDir,
            CheckId::CaseTwins,
            CheckId::Mapping,
            CheckId::MappingAgent,
        ] {
            skipped(&report, id);
        }
        assert_eq!(only(&report, CheckId::Config).message, "skipped: app_root failed");
    }

    #[test]
    fn root_that_is_a_file_fails() {
        let f = fixture();
        fs::write(&f.root, "x").unwrap();
        let report = run_host(&f);
        assert_eq!(only(&report, CheckId::AppRoot).status, Status::Fail);
    }

    #[cfg(unix)]
    #[test]
    fn read_only_root_fails_on_unix() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        set_mode(&f.root, 0o555);
        if fs::write(f.root.join("probe"), "x").is_ok() {
            set_mode(&f.root, 0o755);
            println!("skipped: the read-only mode does not bite (running as root)");
            return;
        }
        let report = run_host(&f);
        set_mode(&f.root, 0o755);
        assert_eq!(only(&report, CheckId::AppRootWritable).status, Status::Fail);
    }

    #[cfg(windows)]
    #[test]
    fn windows_writability_is_info() {
        let f = fixture();
        let report = run_host(&f);
        assert_eq!(only(&report, CheckId::AppRootWritable).status, Status::Info);
    }

    #[test]
    fn invalid_config_fails_and_skips_dependents() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        fs::write(f.root.join("config.toml"), "x =").unwrap();
        let report = run_host(&f);
        assert_eq!(only(&report, CheckId::Config).status, Status::Fail);
        for id in
            [CheckId::ConfigAgent, CheckId::DefaultProfile, CheckId::Mapping, CheckId::MappingAgent]
        {
            skipped(&report, id);
        }
        assert_eq!(only(&report, CheckId::Mapping).message, "skipped: config failed");
    }

    #[test]
    fn unknown_configured_agent_warns() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        fs::write(f.root.join("config.toml"), "[agents.claud]\nexecutable = \"/x\"\n").unwrap();
        let report = run_host(&f);
        let found = find(&report, CheckId::ConfigAgent);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].status, Status::Warn);
        assert_eq!(found[0].subject.as_deref(), Some("claud"));
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_profiles_dir_fails_and_skips_twins() {
        let f = fixture();
        let profiles = f.root.join("profiles");
        fs::create_dir_all(&profiles).unwrap();
        set_mode(&profiles, 0o000);
        if fs::read_dir(&profiles).is_ok() {
            set_mode(&profiles, 0o755);
            println!("skipped: the mode does not bite (running as root)");
            return;
        }
        let report = run_host(&f);
        set_mode(&profiles, 0o755);
        let found = only(&report, CheckId::ProfilesDir);
        assert_eq!(found.status, Status::Fail);
        assert_eq!(found.subject.as_deref(), Some(profiles.to_str().unwrap()));
        skipped(&report, CheckId::CaseTwins);
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_profile_child_names_it() {
        let f = fixture();
        let work = f.root.join("profiles").join("work");
        fs::create_dir_all(&work).unwrap();
        set_mode(&work, 0o000);
        if fs::read_dir(&work).is_ok() {
            set_mode(&work, 0o755);
            println!("skipped: the mode does not bite (running as root)");
            return;
        }
        let report = run_host(&f);
        set_mode(&work, 0o755);
        let found = only(&report, CheckId::ProfilesDir);
        assert_eq!(found.status, Status::Fail);
        assert_eq!(found.subject.as_deref(), Some(work.to_str().unwrap()));
        assert!(find(&report, CheckId::CaseTwins).iter().all(|check| check.status != Status::Skip));
    }

    /// Whether the filesystem under `dir` tells `a` and `A` apart.
    fn case_sensitive(dir: &Path) -> bool {
        let probe = dir.join("probe-a");
        fs::write(&probe, "x").unwrap();
        let sensitive = !dir.join("PROBE-A").exists();
        fs::remove_file(&probe).unwrap();
        sensitive
    }

    #[test]
    fn case_twins_fail_per_pair() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        if !case_sensitive(&f.root) {
            println!("skipped: the filesystem is case-insensitive");
            return;
        }
        fs::create_dir_all(f.root.join("profiles").join("work")).unwrap();
        fs::create_dir_all(f.root.join("profiles").join("Work")).unwrap();
        let report = run_host(&f);
        let found = find(&report, CheckId::CaseTwins);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].status, Status::Fail);
        assert_eq!(found[0].data["names"], json!(["Work", "work"]));
        assert!(found[0].message.contains("Work") && found[0].message.contains("work"));
    }

    #[cfg(unix)]
    #[test]
    fn case_twins_ignores_non_utf8_and_keeps_dot_entries() {
        use std::os::unix::ffi::OsStrExt;
        let f = fixture();
        let profiles = f.root.join("profiles");
        fs::create_dir_all(&profiles).unwrap();
        if !case_sensitive(&f.root) {
            println!("skipped: the filesystem is case-insensitive");
            return;
        }
        fs::write(profiles.join(".x"), "").unwrap();
        fs::write(profiles.join(".X"), "").unwrap();
        let odd = profiles.join(OsStr::from_bytes(b"bad\xff"));
        if fs::write(&odd, "").is_err() {
            println!("skipped: the filesystem refuses non-UTF-8 names");
            return;
        }
        let report = run_host(&f);
        let found = find(&report, CheckId::CaseTwins);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].status, Status::Fail);
        assert_eq!(found[0].data["names"], json!([".X", ".x"]));
    }

    #[test]
    fn default_profile_is_info() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        fs::write(f.root.join("config.toml"), "default_profile = \"work\"\n").unwrap();
        let report = run_host(&f);
        let found = only(&report, CheckId::DefaultProfile);
        assert_eq!(found.status, Status::Info);
        assert_eq!(found.subject.as_deref(), Some("work"));
    }

    fn key(path: &Path) -> String {
        toml_edit::Key::new(path.to_str().unwrap()).display_repr().into_owned()
    }

    #[test]
    fn mapping_states() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        let acme = f.base.join("acme");
        git_dir(&acme);
        let sub = acme.join("sub");
        fs::create_dir_all(&sub).unwrap();
        let plain = f.base.join("plain");
        fs::create_dir_all(&plain).unwrap();
        let gone = f.base.join("gone");
        let file = f.base.join("file.txt");
        fs::write(&file, "x").unwrap();
        let foreign =
            if cfg!(windows) { "/srv/acme".to_owned() } else { r"C:\src\acme".to_owned() };
        let mut config = String::new();
        for (path, body) in [
            (acme.to_str().unwrap().to_owned(), "profile = \"work\""),
            (sub.to_str().unwrap().to_owned(), "profile = \"work\""),
            (plain.to_str().unwrap().to_owned(), "profile = \"work\""),
            (gone.to_str().unwrap().to_owned(), "profile = \"work\""),
            (file.to_str().unwrap().to_owned(), "profile = \"work\""),
            (foreign, "profile = \"work\""),
        ] {
            config += &format!("\n[repositories.{}]\n{body}\n", key(Path::new(&path)));
        }
        fs::write(f.root.join("config.toml"), &config).unwrap();
        let report = run_host(&f);
        let app_root = AppRoot::from_path(f.root.clone());
        let loaded = Config::load(&app_root).unwrap();
        let entries = repositories::report(&app_root, &loaded);
        let found = find(&report, CheckId::Mapping);
        assert_eq!(found.len(), entries.len());
        assert_eq!(entries.len(), 6);
        let by_key: Vec<(Status, &str)> =
            found.iter().map(|check| (check.status, check.subject.as_deref().unwrap())).collect();
        let order: Vec<&str> = entries.iter().map(|entry| entry.key.as_str()).collect();
        assert_eq!(by_key.iter().map(|(_, key)| *key).collect::<Vec<_>>(), order);
        let status_of =
            |path: &Path| by_key.iter().find(|(_, key)| *key == path.to_str().unwrap()).unwrap().0;
        assert_eq!(status_of(&acme), Status::Pass);
        assert_eq!(status_of(&sub), Status::Warn);
        assert_eq!(status_of(&gone), Status::Warn);
        assert_eq!(status_of(&plain), Status::Warn);
        assert_eq!(status_of(&file), Status::Warn);
        assert_eq!(
            by_key
                .iter()
                .find(|(_, key)| key.contains("acme") && !key.starts_with(f.base.to_str().unwrap()))
                .unwrap()
                .0,
            Status::Info
        );
        for (check, entry) in found.iter().zip(&entries) {
            assert_eq!(check.data, serde_json::to_value(json::repository_view(entry)).unwrap());
        }
    }

    #[test]
    fn unknown_mapping_agent_warns() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        let repository = f.base.join("acme");
        git_dir(&repository);
        fs::write(
            f.root.join("config.toml"),
            format!("[repositories.{}]\nagents = {{ nosuch = \"work\" }}\n", key(&repository)),
        )
        .unwrap();
        let report = run_host(&f);
        let found = find(&report, CheckId::MappingAgent);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].status, Status::Warn);
        assert_eq!(found[0].subject, Some(format!("{} nosuch", repository.display())));
    }

    #[test]
    fn repository_discovery_error_fails() {
        let f = fixture();
        let missing = f.base.join("missing");
        let environment = Environment {
            root: Ok(AppRoot::from_path(f.root.clone())),
            path_var: None,
            cwd: &f.base,
            start: &missing,
            is_set: &|_| false,
        };
        let report = run(&Target::Host, &environment);
        assert_eq!(only(&report, CheckId::Repository).status, Status::Fail);
        assert_eq!(report.repository, None);
    }
}
