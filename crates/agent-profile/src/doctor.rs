//! `doctor`: the check model and the host and repository checks (SP5c design §4, §5.1, §5.2, §6).
//!
//! Every check is read-only. A check that cannot run because one it depends on failed emits exactly one
//! `skip` record naming that check (design §4).

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::adapter::{
    self, Adapter, AdapterMetadata, Capability, CapabilityState, Mechanism, PathKind,
};
use crate::config::{AppRoot, Config};
use crate::error::Error;
use crate::exe::{self, Origin};
use crate::json;
use crate::lifecycle::{self, Presence};
use crate::name::Platform;
use crate::name::{AgentId, ProfileName};
use crate::output::{self, ExecutableStatus};
use crate::repo::{self, Discovery};
use crate::repositories::{self, State};
use crate::resolve;

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
    let discovery = repository_checks(env, &host, &mut report.checks);
    report.repository = match &discovery {
        Some(Discovery::Repository(root)) => Some(root.clone()),
        _ => None,
    };
    match target {
        Target::Host => {
            for adapter in adapter::registry() {
                let metadata = adapter.metadata();
                executable_checks(metadata, env, &host, false, &mut report.checks);
                report.checks.push(support_check(metadata));
            }
        }
        Target::Agent { agent, profile } => {
            let adapter = adapter::lookup(agent.as_str()).expect("the grammar admits known agents");
            let metadata = adapter.metadata();
            executable_checks(metadata, env, &host, true, &mut report.checks);
            report.checks.push(support_check(metadata));
            let resolved = resolution_check(
                agent,
                profile.as_deref(),
                &host,
                discovery.as_ref(),
                &mut report.checks,
            );
            report.profile = resolved.clone();
            let blocked = if host.root.is_none() { CheckId::AppRoot } else { CheckId::Resolution };
            profile_checks(adapter, resolved.as_ref(), env, &host, blocked, &mut report.checks);
            option_checks(metadata, &mut report.checks);
        }
    }
    report
}

/// What the host checks established, for the checks that depend on them.
struct Host {
    root: Option<AppRoot>,
    config: Option<Config>,
    /// `profiles_dir` failed on the profiles directory itself.
    profiles_failed: bool,
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
    check(
        id,
        None,
        Status::Skip,
        format!("{} skipped: {} failed", id.token(), dependency.token()),
        Value::Null,
    )
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

/// Why creating a missing path would fail below an existing ancestor.
#[derive(Clone, Copy)]
enum Blocker {
    DanglingLink,
    NotDirectory,
}

impl Blocker {
    fn describe(self) -> &'static str {
        match self {
            Blocker::DanglingLink => "a dangling symbolic link",
            Blocker::NotDirectory => "not a directory",
        }
    }
}

/// The entry that stops a launch creating the missing `path`: a dangling symbolic link between `path` and
/// its nearest existing ancestor, or that ancestor when it is not a directory. Nothing else blocks here.
fn blocked_ancestor(path: &Path) -> Option<(PathBuf, Blocker)> {
    let ancestor = nearest_existing(path)?;
    for candidate in path.ancestors().take_while(|candidate| *candidate != ancestor) {
        if fs::symlink_metadata(candidate).is_ok() {
            return Some((candidate.to_owned(), Blocker::DanglingLink));
        }
    }
    match fs::metadata(&ancestor) {
        Ok(metadata) if !metadata.is_dir() => Some((ancestor, Blocker::NotDirectory)),
        _ => None,
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
    let mut host = Host { root: None, config: None, profiles_failed: false };
    let root = match &env.root {
        Ok(root) => root,
        Err(message) => {
            checks.push(check(
                CheckId::AppRoot,
                None,
                Status::Fail,
                format!("cannot resolve the application root: {message}"),
                json!({ "path": Value::Null, "exists": Value::Null }),
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
        ProfilesDir::Failed => {
            host.profiles_failed = true;
            checks.push(skip(CheckId::CaseTwins, CheckId::ProfilesDir));
        }
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
        Err(error)
            if matches!(error.kind(), io::ErrorKind::NotFound | io::ErrorKind::NotADirectory) =>
        {
            if fs::symlink_metadata(path).is_ok() {
                (
                    Status::Fail,
                    format!(
                        "application root {} is a dangling symbolic link; remove it or set AGENT_PROFILE_HOME",
                        path.display()
                    ),
                    false,
                )
            } else if let Some((blocker, reason)) = blocked_ancestor(path) {
                (
                    Status::Fail,
                    format!(
                        "application root {} cannot be created: {} is {}; remove it or set AGENT_PROFILE_HOME",
                        path.display(),
                        blocker.display(),
                        reason.describe()
                    ),
                    false,
                )
            } else {
                (
                    Status::Info,
                    format!(
                        "application root {} does not exist; it is created on first use",
                        path.display()
                    ),
                    false,
                )
            }
        }
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
            let dangling = !exists && fs::symlink_metadata(&dir).is_ok();
            let (status, message) = if dangling {
                (
                    Status::Fail,
                    format!(
                        "profiles directory {} is a dangling symbolic link; remove it",
                        dir.display()
                    ),
                )
            } else if exists {
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
            if dangling {
                ProfilesDir::Failed
            } else if exists {
                ProfilesDir::Present
            } else {
                ProfilesDir::Absent
            }
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

/// Pushes the repository checks and returns the discovery, or `None` when it failed.
fn repository_checks(
    env: &Environment<'_>,
    host: &Host,
    checks: &mut Vec<Check>,
) -> Option<Discovery> {
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
            Some(Discovery::Repository(root))
        }
        Ok(Discovery::NotInRepository) => {
            checks.push(check(
                CheckId::Repository,
                Some(shown(start)),
                Status::Info,
                format!("{} is not in a repository", start.display()),
                Value::Null,
            ));
            Some(Discovery::NotInRepository)
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

/// A per-agent record.
fn agent_check(
    agent: &'static str,
    id: CheckId,
    subject: Option<String>,
    status: Status,
    message: String,
    data: Value,
) -> Check {
    Check { agent: Some(agent), ..check(id, subject, status, message, data) }
}

fn agent_skip(agent: &'static str, id: CheckId, dependency: CheckId) -> Check {
    Check { agent: Some(agent), ..skip(id, dependency) }
}

/// Pushes `executable`, then `repository_local_executable` when found (design §5.3).
fn executable_checks(
    metadata: &AdapterMetadata,
    env: &Environment<'_>,
    host: &Host,
    targeted: bool,
    checks: &mut Vec<Check>,
) {
    let id = metadata.id;
    if let (Some(root), Some(config)) = (&host.root, &host.config) {
        let explicit = config.agent_executable(id);
        checks.push(
            match ExecutableStatus::discover(metadata, explicit, env.path_var, &root.config_path())
            {
                Ok(status) => executable_record(metadata, &status, targeted),
                Err(error) => agent_check(
                    id,
                    CheckId::Executable,
                    None,
                    Status::Fail,
                    format!("cannot discover the executable of {id}: {error}"),
                    Value::Null,
                ),
            },
        );
    } else {
        checks.push(agent_skip(id, CheckId::Executable, mapping_dependency(host)));
    }
    if let Some(path) = exe::repository_local(metadata.executable, env.path_var, env.cwd) {
        checks.push(agent_check(
            id,
            CheckId::RepositoryLocalExecutable,
            Some(shown(&path)),
            Status::Warn,
            format!(
                "{} is found but never used: v0.1 searches only the configured override and absolute PATH entries",
                path.display()
            ),
            Value::Null,
        ));
    }
}

fn executable_record(
    metadata: &AdapterMetadata,
    status: &ExecutableStatus,
    targeted: bool,
) -> Check {
    let id = metadata.id;
    let (subject, level, message) = match status {
        ExecutableStatus::Found(found) if cfg!(unix) && !exe::is_executable(&found.path) => (
            Some(shown(&found.path)),
            Status::Fail,
            format!(
                "the configured executable {path} has no execute permission bit; chmod +x {path} or fix [agents.{id}] executable in config.toml",
                path = found.path.display()
            ),
        ),
        ExecutableStatus::Found(found) => (
            Some(shown(&found.path)),
            Status::Pass,
            format!(
                "found {} ({})",
                found.path.display(),
                match found.origin {
                    Origin::Configured => "configured override",
                    Origin::Path => "on PATH",
                }
            ),
        ),
        ExecutableStatus::NotOnPath => (
            None,
            if targeted { Status::Fail } else { Status::Info },
            format!(
                "{} was not found in any absolute PATH entry; install it or set [agents.{id}] executable in config.toml",
                metadata.executable
            ),
        ),
        ExecutableStatus::ConfiguredMissing(path) => (
            Some(shown(path)),
            Status::Fail,
            format!(
                "the configured executable {} does not exist or is not a file; fix [agents.{id}] executable in config.toml",
                path.display()
            ),
        ),
        ExecutableStatus::Unsupported(path) => (
            Some(shown(path)),
            Status::Fail,
            format!(
                "{} is a shell script or shim, which a launch refuses (exit 6); point [agents.{id}] executable at a native executable",
                path.display()
            ),
        ),
    };
    let data = serde_json::to_value(json::executable_view(status)).unwrap_or(Value::Null);
    agent_check(id, CheckId::Executable, subject, level, message, data)
}

/// The `support` record: the evidence, never a probe of the installed agent.
fn support_check(metadata: &AdapterMetadata) -> Check {
    let view = serde_json::to_value(json::agent_view(metadata, &ExecutableStatus::NotOnPath))
        .unwrap_or(Value::Null);
    let data = json!({
        "mechanism": view["mechanism"],
        "support": view["support"],
        "evidence": view["evidence"],
        "capabilities": view["capabilities"],
        "installed_version": Value::Null,
        "version_probe": "not_run",
    });
    agent_check(
        metadata.id,
        CheckId::Support,
        None,
        Status::Info,
        format!(
            "{} is {} against upstream version {} verified {}; the installed version is not probed",
            metadata.id,
            json::support_token(metadata.support),
            metadata.evidence.upstream_version,
            metadata.evidence.verified_at
        ),
        data,
    )
}

/// Pushes `resolution` and returns the profile it settled on.
fn resolution_check(
    agent: &AgentId,
    argument: Option<&str>,
    host: &Host,
    discovery: Option<&Discovery>,
    checks: &mut Vec<Check>,
) -> Option<ProfileName> {
    let id = adapter::lookup(agent.as_str()).expect("known agent").metadata().id;
    let Some(config) = &host.config else {
        checks.push(agent_skip(id, CheckId::Resolution, mapping_dependency(host)));
        return None;
    };
    let explicit = match argument.map(|raw| (raw, ProfileName::parse(raw, Platform::host()))) {
        Some((raw, Err(reason))) => {
            let error = Error::InvalidProfileName { name: raw.to_owned(), reason };
            checks.push(agent_check(
                id,
                CheckId::Resolution,
                Some(raw.to_owned()),
                Status::Fail,
                format!("{error}; give a valid profile name"),
                Value::Null,
            ));
            return None;
        }
        Some((_, Ok(name))) => Some(name),
        None => None,
    };
    let found = match (&explicit, discovery) {
        (Some(_), _) => Discovery::NotInRepository,
        (None, Some(found)) => found.clone(),
        (None, None) => {
            checks.push(agent_skip(id, CheckId::Resolution, CheckId::Repository));
            return None;
        }
    };
    let resolution = resolve::resolve(agent.clone(), explicit, config, &found);
    match resolution.profile {
        Some(name) => {
            checks.push(agent_check(
                id,
                CheckId::Resolution,
                Some(name.to_string()),
                Status::Info,
                format!("profile `{name}` resolves for {id} ({})", resolution.source.label()),
                Value::Null,
            ));
            Some(name)
        }
        None => {
            checks.push(agent_check(
                id,
                CheckId::Resolution,
                None,
                Status::Warn,
                format!(
                    "no profile resolves for {id} here; a launch here needs an explicit profile"
                ),
                Value::Null,
            ));
            None
        }
    }
}

/// Pushes `profile`, `profile_path` and `launch_environment`, or one `skip` each.
fn profile_checks(
    adapter: &dyn Adapter,
    profile: Option<&ProfileName>,
    env: &Environment<'_>,
    host: &Host,
    blocked: CheckId,
    checks: &mut Vec<Check>,
) {
    let id = adapter.metadata().id;
    let (Some(profile), Some(root)) = (profile, &host.root) else {
        for check in [CheckId::Profile, CheckId::ProfilePath, CheckId::LaunchEnvironment] {
            checks.push(agent_skip(id, check, blocked));
        }
        return;
    };
    let agent = AgentId::parse(id).expect("registry ids are valid");
    let paths = adapter.paths(root, profile);
    if host.profiles_failed {
        checks.push(agent_skip(id, CheckId::Profile, CheckId::ProfilesDir));
        checks.push(agent_skip(id, CheckId::ProfilePath, CheckId::ProfilesDir));
    } else {
        checks.push(profile_record(id, root, &agent, profile));
        for (path, kind) in &paths {
            checks.push(profile_path_record(id, path, *kind));
        }
    }
    checks.push(launch_environment(adapter.metadata(), &paths, env));
}

fn profile_record(
    id: &'static str,
    root: &AppRoot,
    agent: &AgentId,
    profile: &ProfileName,
) -> Check {
    let presence = lifecycle::presence(root, agent, profile);
    let existing = match &presence {
        Presence::Conflict { existing } => json!(existing),
        _ => Value::Null,
    };
    let (status, message) = match &presence {
        Presence::Conflict { existing } => (
            Status::Fail,
            format!(
                "profile `{profile}` conflicts with `{existing}`, which differs only in case; rename or remove one"
            ),
        ),
        Presence::Absent => (
            Status::Info,
            format!("profile `{profile}` does not exist for {id}; it is created on first launch"),
        ),
        Presence::Materialized | Presence::Known => {
            (Status::Pass, format!("profile `{profile}` exists for {id}"))
        }
    };
    agent_check(
        id,
        CheckId::Profile,
        Some(profile.to_string()),
        status,
        message,
        json!({ "state": presence.token(), "existing": existing }),
    )
}

/// The octal permission bits on Unix, `null` elsewhere.
#[cfg(unix)]
fn file_mode(metadata: &fs::Metadata) -> Value {
    use std::os::unix::fs::PermissionsExt;
    json!(format!("{:o}", metadata.permissions().mode() & 0o7777))
}

#[cfg(not(unix))]
fn file_mode(_metadata: &fs::Metadata) -> Value {
    Value::Null
}

fn profile_path_record(id: &'static str, path: &Path, kind: PathKind) -> Check {
    let (kind_token, noun) = match kind {
        PathKind::Dir => ("dir", "directory"),
        PathKind::File { .. } => ("file", "file"),
    };
    let mut exists = true;
    let mut mode = Value::Null;
    let (status, message) = match fs::metadata(path) {
        Ok(metadata) => {
            mode = file_mode(&metadata);
            let right = match kind {
                PathKind::Dir => metadata.is_dir(),
                PathKind::File { .. } => metadata.is_file(),
            };
            if !right {
                (
                    Status::Fail,
                    format!(
                        "{} exists but is not a {noun}; a launch refuses it; remove or rename it",
                        path.display()
                    ),
                )
            } else if cfg!(unix) && metadata.permissions().readonly() {
                (
                    Status::Warn,
                    format!(
                        "{} has no write permission bit, so the agent cannot write its state there",
                        path.display()
                    ),
                )
            } else {
                (Status::Pass, format!("{} is a {noun}", path.display()))
            }
        }
        Err(error)
            if matches!(error.kind(), io::ErrorKind::NotFound | io::ErrorKind::NotADirectory) =>
        {
            exists = false;
            missing_path(path)
        }
        Err(error) => (Status::Fail, format!("cannot examine {}: {error}", path.display())),
    };
    agent_check(
        id,
        CheckId::ProfilePath,
        Some(shown(path)),
        status,
        message,
        json!({ "path": shown(path), "exists": exists, "kind": kind_token, "mode": mode }),
    )
}

/// What a launch would meet creating `path`, which does not exist.
fn missing_path(path: &Path) -> (Status, String) {
    let created =
        (Status::Info, format!("{} does not exist; it is created on first launch", path.display()));
    if let Some((blocker, reason)) = blocked_ancestor(path) {
        return match reason {
            Blocker::DanglingLink => (
                Status::Fail,
                format!(
                    "{} is a dangling symbolic link, so a launch cannot create {}; remove it",
                    blocker.display(),
                    path.display()
                ),
            ),
            Blocker::NotDirectory => (
                Status::Fail,
                format!(
                    "{} is not a directory, so a launch cannot create {}; remove or rename it",
                    blocker.display(),
                    path.display()
                ),
            ),
        };
    }
    let Some(ancestor) = nearest_existing(path) else {
        return created;
    };
    match fs::metadata(&ancestor) {
        Ok(metadata) if cfg!(unix) && metadata.permissions().readonly() => (
            Status::Fail,
            format!(
                "{} has no write permission bit, so a launch cannot create {}; change its permissions",
                ancestor.display(),
                path.display()
            ),
        ),
        Ok(_) => created,
        Err(error) => (Status::Fail, format!("cannot examine {}: {error}", ancestor.display())),
    }
}

/// The single `launch_environment` record: how the profile is passed, never a value read from the
/// environment.
fn launch_environment(
    metadata: &AdapterMetadata,
    paths: &[(PathBuf, PathKind)],
    env: &Environment<'_>,
) -> Check {
    let (kind, name, wants_file) = match metadata.mechanism {
        Mechanism::Env(name) => ("env", name, false),
        Mechanism::EnvFile(name) => ("env_file", name, true),
        Mechanism::FlagDir(option) => ("flag_dir", option.spelling(), false),
        Mechanism::FlagFile(option) => ("flag_file", option.spelling(), true),
    };
    let target = paths
        .iter()
        .find(|(_, kind)| matches!(kind, PathKind::File { .. }) == wants_file)
        .map(|(path, _)| path.as_path())
        .expect("an adapter owns the path its mechanism points at");
    let variable = matches!(metadata.mechanism, Mechanism::Env(_) | Mechanism::EnvFile(_));
    let already_set = variable.then(|| (env.is_set)(name));
    let message = if variable {
        let mut message = format!("sets {name} to {}", target.display());
        if already_set == Some(true) {
            message += "; already set in this environment, the launch replaces it";
        }
        message
    } else {
        metadata.mechanism.sentence_for(target)
    };
    agent_check(
        metadata.id,
        CheckId::LaunchEnvironment,
        Some(name.to_owned()),
        Status::Info,
        message,
        json!({
            "kind": kind,
            "name": name,
            "value": shown(target),
            "already_set": already_set,
        }),
    )
}

/// Pushes `conflicting_option` and `capability` records.
fn option_checks(metadata: &AdapterMetadata, checks: &mut Vec<Check>) {
    let id = metadata.id;
    for option in metadata.conflicts.iter().chain(metadata.mechanism.conflict_option()) {
        checks.push(agent_check(
            id,
            CheckId::ConflictingOption,
            Some(option.spelling().to_owned()),
            Status::Info,
            format!("a launch refuses {} because the profile already sets it", option.spelling()),
            Value::Null,
        ));
    }
    for capability in Capability::ALL {
        let claim = metadata
            .capabilities
            .iter()
            .find(|claim| claim.capability == capability)
            .expect("registry adapters declare every capability");
        if claim.state != CapabilityState::Supported {
            checks.push(agent_check(
                id,
                CheckId::Capability,
                Some(json::capability_token(capability).to_owned()),
                Status::Info,
                format!(
                    "{} isolation is {}: {}",
                    output::capability_label(capability),
                    output::state_label(claim.state),
                    claim.basis
                ),
                Value::Null,
            ));
        }
    }
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
        assert_eq!(only(&report, CheckId::Config).message, "config skipped: app_root failed");
        assert_eq!(
            only(&report, CheckId::AppRoot).data,
            json!({ "path": Value::Null, "exists": Value::Null })
        );
    }

    #[cfg(unix)]
    #[test]
    fn dangling_root_fails_and_skips_its_dependents() {
        let f = fixture();
        std::os::unix::fs::symlink(f.base.join("missing"), &f.root).unwrap();
        let report = run_host(&f);
        let found = only(&report, CheckId::AppRoot);
        assert_eq!(found.status, Status::Fail);
        assert!(found.message.contains("dangling symbolic link"), "{}", found.message);
        for id in [CheckId::AppRootWritable, CheckId::Config, CheckId::ProfilesDir] {
            skipped(&report, id);
        }
    }

    #[cfg(unix)]
    #[test]
    fn dangling_profiles_dir_fails_and_skips_its_dependents() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        std::os::unix::fs::symlink(f.base.join("missing"), f.root.join("profiles")).unwrap();
        let report = run_host(&f);
        let found = only(&report, CheckId::ProfilesDir);
        assert_eq!(found.status, Status::Fail);
        assert!(found.message.contains("dangling symbolic link"), "{}", found.message);
        skipped(&report, CheckId::CaseTwins);
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
        assert_eq!(only(&report, CheckId::Mapping).message, "mapping skipped: config failed");
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
        let moved = found.iter().find(|check| check.subject.as_deref() == sub.to_str()).unwrap();
        assert!(
            moved.message.starts_with(&format!(
                "mapping {} is below the repository rooted at {}, so it no longer applies",
                sub.display(),
                acme.display()
            )),
            "{}",
            moved.message
        );
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

    // ---- Task 3: the per-agent checks -------------------------------------------------------------

    fn fake_id() -> AgentId {
        AgentId::parse("fake").unwrap()
    }

    /// A directory under the fixture holding an "installed" fake agent (or nothing).
    fn bin_dir(f: &Fixture, installed: bool) -> PathBuf {
        let dir = f.base.join(if installed { "bin-installed" } else { "bin-empty" });
        fs::create_dir_all(&dir).unwrap();
        if installed {
            let file = dir.join(if cfg!(windows) { "fake-agent.exe" } else { "fake-agent" });
            fs::write(&file, "x").unwrap();
            #[cfg(unix)]
            set_mode(&file, 0o755);
        }
        dir
    }

    fn run_with(
        f: &Fixture,
        target: &Target,
        path_var: Option<&OsStr>,
        is_set: &dyn Fn(&str) -> bool,
    ) -> Report {
        let environment = Environment {
            root: Ok(AppRoot::from_path(f.root.clone())),
            path_var,
            cwd: &f.base,
            start: &f.base,
            is_set,
        };
        run(target, &environment)
    }

    fn agent_target(profile: Option<&str>) -> Target {
        Target::Agent { agent: fake_id(), profile: profile.map(str::to_owned) }
    }

    /// `fake <profile> doctor` with the agent installed.
    fn run_fake(f: &Fixture, profile: Option<&str>) -> Report {
        let bin = bin_dir(f, true);
        run_with(f, &agent_target(profile), Some(bin.as_os_str()), &|_| false)
    }

    fn profile_dir_of(f: &Fixture, profile: &str) -> PathBuf {
        f.root.join("profiles").join(profile).join("fake")
    }

    fn skips_for_profile_checks(report: &Report) {
        for id in [CheckId::Profile, CheckId::ProfilePath, CheckId::LaunchEnvironment] {
            skipped(report, id);
        }
    }

    #[test]
    fn bare_doctor_has_three_records_per_agent() {
        let f = fixture();
        let report = run_host(&f);
        let seen: Vec<(&str, CheckId)> = report
            .checks
            .iter()
            .filter(|check| {
                matches!(
                    check.id,
                    CheckId::Executable | CheckId::Support | CheckId::RepositoryLocalExecutable
                )
            })
            .map(|check| (check.agent.unwrap(), check.id))
            .collect();
        let expected: Vec<(&str, CheckId)> = adapter::known_agents()
            .into_iter()
            .flat_map(|id| [(id, CheckId::Executable), (id, CheckId::Support)])
            .collect();
        assert_eq!(seen, expected);
    }

    #[test]
    fn not_installed_is_info_bare_and_fail_targeted() {
        let f = fixture();
        let bin = bin_dir(&f, false);
        let bare = run_with(&f, &Target::Host, Some(bin.as_os_str()), &|_| false);
        let fake = bare
            .checks
            .iter()
            .find(|check| check.id == CheckId::Executable && check.agent == Some("fake"))
            .unwrap();
        assert_eq!(fake.status, Status::Info);
        let targeted = run_with(&f, &agent_target(None), Some(bin.as_os_str()), &|_| false);
        assert_eq!(only(&targeted, CheckId::Executable).status, Status::Fail);
        assert!(targeted.failed());
    }

    fn run_at_root(f: &Fixture, root: PathBuf) -> Report {
        let environment = env(Ok(AppRoot::from_path(root)), &f.base);
        run(&Target::Host, &environment)
    }

    #[cfg(unix)]
    fn assert_root_blocked(report: &Report, root: &Path, ancestor: &Path, reason: &str) {
        let record = only(report, CheckId::AppRoot);
        assert_eq!(record.status, Status::Fail, "{record:?}");
        assert_eq!(record.subject, Some(shown(root)));
        assert_eq!(record.data, json!({ "path": shown(root), "exists": false }));
        assert!(record.message.contains(&ancestor.display().to_string()), "{}", record.message);
        assert!(record.message.contains(reason), "{}", record.message);
        for id in [CheckId::AppRootWritable, CheckId::Config, CheckId::ProfilesDir] {
            skipped(report, id);
        }
        assert!(report.failed());
    }

    #[cfg(unix)]
    #[test]
    fn dangling_ancestor_of_root_fails() {
        let f = fixture();
        let broken = f.base.join("broken");
        std::os::unix::fs::symlink(f.base.join("gone"), &broken).unwrap();
        let root = broken.join("app");
        let report = run_at_root(&f, root.clone());
        assert_root_blocked(&report, &root, &broken, "dangling symbolic link");
    }

    #[cfg(unix)]
    #[test]
    fn file_ancestor_of_root_fails() {
        let f = fixture();
        let file = f.base.join("plain");
        fs::write(&file, "x").unwrap();
        let root = file.join("app");
        let report = run_at_root(&f, root.clone());
        assert_root_blocked(&report, &root, &file, "not a directory");
    }

    #[cfg(unix)]
    #[test]
    fn configured_executable_without_execute_bit_fails() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        let bin = bin_dir(&f, true);
        let plain = f.base.join("plain-agent");
        fs::write(&plain, "x").unwrap();
        set_mode(&plain, 0o644);
        fs::write(
            f.root.join("config.toml"),
            format!("[agents.fake]\nexecutable = '{}'\n", plain.display()),
        )
        .unwrap();
        let bare = run_with(&f, &Target::Host, Some(bin.as_os_str()), &|_| false);
        let fake = bare
            .checks
            .iter()
            .find(|check| check.id == CheckId::Executable && check.agent == Some("fake"))
            .unwrap();
        let targeted = run_with(&f, &agent_target(None), Some(bin.as_os_str()), &|_| false);
        for record in [fake, only(&targeted, CheckId::Executable)] {
            assert_eq!(record.status, Status::Fail, "{record:?}");
            assert_eq!(record.data["state"], json!("found"));
            assert!(record.message.contains("no execute permission bit"), "{}", record.message);
            assert!(record.message.contains(&plain.display().to_string()));
        }
        assert!(targeted.failed());
    }

    #[test]
    fn configured_missing_and_shell_script_fail() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        let bin = bin_dir(&f, true);
        let nope = f.base.join("nope");
        fs::write(
            f.root.join("config.toml"),
            format!("[agents.fake]\nexecutable = '{}'\n", nope.display()),
        )
        .unwrap();
        let report = run_with(&f, &agent_target(None), Some(bin.as_os_str()), &|_| false);
        let found = only(&report, CheckId::Executable);
        assert_eq!(found.status, Status::Fail);
        assert_eq!(found.data["state"], json!("configured_missing"));

        let script = f.base.join("fake.cmd");
        fs::write(&script, "@echo off").unwrap();
        fs::write(
            f.root.join("config.toml"),
            format!("[agents.fake]\nexecutable = '{}'\n", script.display()),
        )
        .unwrap();
        let report = run_with(&f, &agent_target(None), Some(bin.as_os_str()), &|_| false);
        let found = only(&report, CheckId::Executable);
        assert_eq!(found.status, Status::Fail);
        assert_eq!(found.data["state"], json!("unsupported"));
    }

    #[cfg(unix)]
    #[test]
    fn repository_local_executable_warns() {
        let f = fixture();
        let local = f.base.join("bin");
        fs::create_dir_all(&local).unwrap();
        let exe = local.join("fake-agent");
        fs::write(&exe, "x").unwrap();
        set_mode(&exe, 0o755);
        let report = run_with(&f, &agent_target(None), Some(OsStr::new("bin")), &|_| false);
        let found = only(&report, CheckId::RepositoryLocalExecutable);
        assert_eq!(found.status, Status::Warn);
        assert_eq!(found.subject.as_deref(), Some(exe.to_str().unwrap()));
        let without = run_with(&f, &agent_target(None), None, &|_| false);
        assert!(find(&without, CheckId::RepositoryLocalExecutable).is_empty());
    }

    #[test]
    fn support_never_probes() {
        let f = fixture();
        let report = run_fake(&f, None);
        let found = only(&report, CheckId::Support);
        assert_eq!(found.status, Status::Info);
        assert_eq!(found.data["installed_version"], Value::Null);
        assert_eq!(found.data["version_probe"], json!("not_run"));
        assert_eq!(found.data["capabilities"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn invalid_profile_argument_fails_resolution() {
        let f = fixture();
        let report = run_fake(&f, Some("bad/name"));
        assert_eq!(only(&report, CheckId::Resolution).status, Status::Fail);
        skips_for_profile_checks(&report);
        assert_eq!(report.profile, None);
    }

    #[test]
    fn nothing_resolves_warns_and_skips() {
        let f = fixture();
        let report = run_fake(&f, None);
        assert_eq!(only(&report, CheckId::Resolution).status, Status::Warn);
        skips_for_profile_checks(&report);
    }

    #[test]
    fn discovery_failure_skips_resolution_only_without_profile() {
        let f = fixture();
        let missing = f.base.join("start");
        let bin = bin_dir(&f, true);
        let run_at = |profile: Option<&str>| {
            let environment = Environment {
                root: Ok(AppRoot::from_path(f.root.clone())),
                path_var: Some(bin.as_os_str()),
                cwd: &f.base,
                start: &missing,
                is_set: &|_| false,
            };
            run(&agent_target(profile), &environment)
        };
        let without = run_at(None);
        skipped(&without, CheckId::Resolution);
        let with = run_at(Some("work"));
        assert_eq!(only(&with, CheckId::Resolution).status, Status::Info);
    }

    #[test]
    fn absent_profile_is_lazy_info() {
        let f = fixture();
        fs::create_dir_all(f.root.join("profiles")).unwrap();
        let report = run_fake(&f, Some("work"));
        assert_eq!(only(&report, CheckId::Profile).status, Status::Info);
        assert_eq!(only(&report, CheckId::ProfilePath).status, Status::Info);
        assert!(!report.failed());
        assert_eq!(report.profile.as_ref().map(ProfileName::as_str), Some("work"));
    }

    #[test]
    fn fresh_root_profile_path_is_info() {
        let f = fixture();
        assert!(!f.root.exists());
        let report = run_fake(&f, Some("work"));
        assert_eq!(only(&report, CheckId::ProfilePath).status, Status::Info);
        assert!(!report.failed());
    }

    #[cfg(unix)]
    #[test]
    fn materialized_profile_passes_with_mode() {
        let f = fixture();
        let dir = profile_dir_of(&f, "work");
        fs::create_dir_all(&dir).unwrap();
        set_mode(&dir, 0o700);
        let report = run_fake(&f, Some("work"));
        assert_eq!(only(&report, CheckId::Profile).status, Status::Pass);
        let path = only(&report, CheckId::ProfilePath);
        assert_eq!(path.status, Status::Pass);
        assert_eq!(path.data["mode"], json!("700"));
    }

    #[test]
    fn file_where_the_profile_dir_goes_fails() {
        let f = fixture();
        fs::create_dir_all(f.root.join("profiles")).unwrap();
        let work = f.root.join("profiles").join("work");
        fs::write(&work, "x").unwrap();
        let report = run_fake(&f, Some("work"));
        let path = only(&report, CheckId::ProfilePath);
        assert_eq!(path.status, Status::Fail);
        assert!(path.message.contains(work.to_str().unwrap()), "{}", path.message);
    }

    #[test]
    fn wrong_kind_at_the_leaf_fails() {
        let f = fixture();
        let leaf = profile_dir_of(&f, "work");
        fs::create_dir_all(leaf.parent().unwrap()).unwrap();
        fs::write(&leaf, "x").unwrap();
        let report = run_fake(&f, Some("work"));
        assert_eq!(only(&report, CheckId::ProfilePath).status, Status::Fail);
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_profile_dir_is_not_a_fail() {
        let f = fixture();
        fs::create_dir_all(f.root.join("profiles")).unwrap();
        let real = f.base.join("real");
        fs::create_dir_all(&real).unwrap();
        std::os::unix::fs::symlink(&real, f.root.join("profiles").join("work")).unwrap();
        let report = run_fake(&f, Some("work"));
        let status = only(&report, CheckId::ProfilePath).status;
        assert!(matches!(status, Status::Info | Status::Pass), "{status:?}");
    }

    #[cfg(unix)]
    #[test]
    fn dangling_symlink_fails() {
        let f = fixture();
        fs::create_dir_all(f.root.join("profiles")).unwrap();
        std::os::unix::fs::symlink(f.base.join("nowhere"), f.root.join("profiles").join("work"))
            .unwrap();
        let report = run_fake(&f, Some("work"));
        let path = only(&report, CheckId::ProfilePath);
        assert_eq!(path.status, Status::Fail);
        assert!(path.message.contains("dangling"), "{}", path.message);
    }

    #[cfg(unix)]
    #[test]
    fn read_only_existing_leaf_warns() {
        let f = fixture();
        let dir = profile_dir_of(&f, "work");
        fs::create_dir_all(&dir).unwrap();
        set_mode(&dir, 0o555);
        let report = run_fake(&f, Some("work"));
        set_mode(&dir, 0o755);
        assert_eq!(only(&report, CheckId::ProfilePath).status, Status::Warn);
    }

    #[cfg(unix)]
    #[test]
    fn read_only_ancestor_fails() {
        let f = fixture();
        let profiles = f.root.join("profiles");
        fs::create_dir_all(&profiles).unwrap();
        set_mode(&profiles, 0o555);
        let report = run_fake(&f, Some("work"));
        set_mode(&profiles, 0o755);
        assert_eq!(only(&report, CheckId::ProfilePath).status, Status::Fail);
    }

    #[test]
    fn launch_environment_reports_without_values() {
        let f = fixture();
        let bin = bin_dir(&f, true);
        let report = run_with(&f, &agent_target(Some("work")), Some(bin.as_os_str()), &|name| {
            name == "FAKE_AGENT_HOME"
        });
        let found = only(&report, CheckId::LaunchEnvironment);
        assert_eq!(found.status, Status::Info);
        assert_eq!(found.data["kind"], json!("env"));
        assert_eq!(found.data["name"], json!("FAKE_AGENT_HOME"));
        assert_eq!(found.data["already_set"], json!(true));
        assert_eq!(found.data["value"], json!(profile_dir_of(&f, "work").display().to_string()));
        assert!(found.message.contains("already set"), "{}", found.message);
    }

    #[test]
    fn launch_environment_for_every_adapter() {
        for id in adapter::known_agents() {
            let f = fixture();
            let target = Target::Agent {
                agent: AgentId::parse(id).unwrap(),
                profile: Some("work".to_owned()),
            };
            let report = run_with(&f, &target, None, &|_| false);
            let found = only(&report, CheckId::LaunchEnvironment);
            let kind = found.data["kind"].as_str().unwrap_or_else(|| panic!("{id}: no kind"));
            assert!(["env", "env_file", "flag_dir", "flag_file"].contains(&kind), "{id}: {kind}");
            let flag = kind == "flag_dir" || kind == "flag_file";
            assert_eq!(found.data["already_set"].is_null(), flag, "{id}: {kind}");
            if !flag {
                assert!(found.data["already_set"].is_boolean(), "{id}");
            }
        }
    }

    #[test]
    fn config_failure_skips_executable_and_resolution() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        fs::write(f.root.join("config.toml"), "x =").unwrap();
        let report = run_fake(&f, None);
        skipped(&report, CheckId::Executable);
        skipped(&report, CheckId::Resolution);
        let bare = run_host(&f);
        let executables: Vec<&Check> = find(&bare, CheckId::Executable);
        assert_eq!(executables.len(), adapter::known_agents().len());
        assert!(executables.iter().all(|check| check.status == Status::Skip));
    }

    #[test]
    fn conflicting_option_and_capabilities() {
        let f = fixture();
        let report = run_fake(&f, Some("work"));
        let options = find(&report, CheckId::ConflictingOption);
        assert_eq!(options.len(), 1);
        assert_eq!(options[0].subject.as_deref(), Some("--fake-profile"));
        let capabilities = find(&report, CheckId::Capability);
        assert_eq!(capabilities.len(), 3);
        assert!(capabilities.iter().all(|check| check.status == Status::Info));
        for check in &capabilities {
            let label = Capability::ALL
                .into_iter()
                .find(|capability| {
                    Some(json::capability_token(*capability)) == check.subject.as_deref()
                })
                .map(output::capability_label)
                .expect("the subject is a capability token");
            assert!(
                check.message.starts_with(&format!("{label} isolation is unknown: ")),
                "{}",
                check.message
            );
        }
    }

    #[test]
    fn flag_adapter_lists_its_own_flag() {
        let f = fixture();
        let target = Target::Agent { agent: AgentId::parse("aider").unwrap(), profile: None };
        let report = run_with(&f, &target, None, &|_| false);
        let subjects: Vec<&str> = find(&report, CheckId::ConflictingOption)
            .iter()
            .map(|check| check.subject.as_deref().unwrap())
            .collect();
        assert!(subjects.contains(&"--config"), "{subjects:?}");
    }

    #[test]
    fn case_twin_profile_fails() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        if !case_sensitive(&f.root) {
            println!("skipped: the filesystem is case-insensitive");
            return;
        }
        fs::create_dir_all(f.root.join("profiles").join("Work")).unwrap();
        let report = run_fake(&f, Some("work"));
        let found = only(&report, CheckId::Profile);
        assert_eq!(found.status, Status::Fail);
        assert!(found.message.contains("Work"), "{}", found.message);
    }

    #[test]
    fn targeted_doctor_skips_profile_checks_when_profiles_dir_fails() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        fs::write(f.root.join("profiles"), "x").unwrap();
        let report = run_fake(&f, Some("work"));
        assert_eq!(only(&report, CheckId::ProfilesDir).status, Status::Fail);
        for id in [CheckId::Profile, CheckId::ProfilePath] {
            let found = only(&report, id);
            assert_eq!(found.status, Status::Skip, "{id:?}");
            assert_eq!(found.message, format!("{} skipped: profiles_dir failed", id.token()));
        }
        assert_ne!(only(&report, CheckId::LaunchEnvironment).status, Status::Skip);
    }

    #[test]
    fn configured_directory_executable_fails() {
        let f = fixture();
        fs::create_dir_all(&f.root).unwrap();
        let bin = bin_dir(&f, true);
        let dir = f.base.join("a-directory");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            f.root.join("config.toml"),
            format!("[agents.fake]\nexecutable = '{}'\n", dir.display()),
        )
        .unwrap();
        let report = run_with(&f, &agent_target(None), Some(bin.as_os_str()), &|_| false);
        let found = only(&report, CheckId::Executable);
        assert_eq!(found.status, Status::Fail);
        assert_eq!(found.data["state"], json!("configured_missing"));
    }
}
