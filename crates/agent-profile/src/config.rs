//! The application root (spec §7), `config.toml` strict reads (spec §17) and locked, atomic writes
//! (spec §18, §18.1).

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use tempfile::NamedTempFile;

use crate::error::{Error, Result};
use crate::name::{AgentId, Platform, ProfileName};

/// Environment variable that overrides the application root.
pub const HOME_ENV: &str = "AGENT_PROFILE_HOME";

const CONFIG_FILE: &str = "config.toml";
const LOCK_FILE: &str = "config.toml.lock";
const TEMP_PREFIX: &str = ".config.toml.";
const TEMP_SUFFIX: &str = ".tmp";
const LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const LOCK_RETRY: Duration = Duration::from_millis(50);

/// The directory holding `config.toml` and `profiles/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRoot(PathBuf);

impl AppRoot {
    /// A root at an explicit path. Tests use this instead of changing the process environment.
    pub fn from_path(path: PathBuf) -> AppRoot {
        AppRoot(path)
    }

    /// `AGENT_PROFILE_HOME` when set, otherwise `<home>/.agent-profile`. Never creates anything.
    pub fn resolve() -> Result<AppRoot> {
        AppRoot::resolve_from(std::env::var_os(HOME_ENV), std::env::home_dir())
    }

    fn resolve_from(home_override: Option<OsString>, home_dir: Option<PathBuf>) -> Result<AppRoot> {
        match home_override {
            Some(value) => {
                let path = PathBuf::from(value);
                if path.as_os_str().is_empty() || !path.is_absolute() {
                    return Err(Error::AppRoot {
                        message: format!(
                            "{HOME_ENV} must be an absolute path, got {:?}",
                            path.as_os_str()
                        ),
                    });
                }
                Ok(AppRoot(path))
            }
            None => match home_dir {
                Some(home) if home.is_absolute() => Ok(AppRoot(home.join(".agent-profile"))),
                _ => Err(Error::AppRoot {
                    message: format!(
                        "cannot determine an absolute home directory; set {HOME_ENV} to an absolute path"
                    ),
                }),
            },
        }
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn config_path(&self) -> PathBuf {
        self.0.join(CONFIG_FILE)
    }

    pub fn profiles_dir(&self) -> PathBuf {
        self.0.join("profiles")
    }

    fn lock_path(&self) -> PathBuf {
        self.0.join(LOCK_FILE)
    }
}

/// The validated contents of `config.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    agents: BTreeMap<String, Option<PathBuf>>,
    default_profile: Option<ProfileName>,
    /// Keyed by the stored key string, so iteration is ordered by it.
    repositories: BTreeMap<String, Mapping>,
}

/// One `[repositories.'<root>']` entry (SP3 design §6.1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mapping {
    pub profile: Option<ProfileName>,
    pub agents: BTreeMap<AgentId, ProfileName>,
}

/// A mapping field that names a profile (SP3 design §4.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// The stored key.
    pub root: PathBuf,
    /// `None` for the entry's `profile`, the agent for an `agents.<id>` field.
    pub agent: Option<AgentId>,
    /// The stored spelling.
    pub profile: ProfileName,
}

/// What `link` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkOutcome {
    Linked,
    Changed { old: ProfileName },
    AlreadyLinked,
}

/// What `unlink` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnlinkOutcome {
    /// `root` is the stored key of the entry the mapping was removed from.
    Unlinked { root: PathBuf, old: ProfileName },
    /// `shown` is the first candidate key.
    NothingToRemove { shown: PathBuf },
}

impl Config {
    /// Reads `<root>/config.toml`. A missing file is an empty configuration; anything invalid is an error.
    pub fn load(root: &AppRoot) -> Result<Config> {
        let path = root.config_path();
        match fs::read(&path) {
            Ok(bytes) => parse(&path, &bytes),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
            Err(error) => Err(invalid(&path, None, format!("cannot read the file: {error}"))),
        }
    }

    /// The explicit executable override for an agent, if configured.
    pub fn agent_executable(&self, id: &str) -> Option<&Path> {
        self.agents.get(id).and_then(|executable| executable.as_deref())
    }

    /// Every agent id that has an `agents.<id>` table, in sorted order.
    pub fn configured_agents(&self) -> impl Iterator<Item = &str> {
        self.agents.keys().map(String::as_str)
    }

    /// The hand-edited global default (SP3 design D3).
    pub fn default_profile(&self) -> Option<&ProfileName> {
        self.default_profile.as_ref()
    }

    /// The entry whose key is component-equal to `root`; validation guarantees there is at most one.
    pub fn mapping(&self, root: &Path) -> Option<&Mapping> {
        self.mappings().find(|(key, _)| *key == root).map(|(_, mapping)| mapping)
    }

    /// Every entry, ordered by the stored key string.
    pub fn mappings(&self) -> impl Iterator<Item = (&Path, &Mapping)> {
        self.repositories.iter().map(|(key, mapping)| (Path::new(key.as_str()), mapping))
    }

    /// Every mapping field naming `profile`, ignoring ASCII case, so a case-only twin is never missed.
    pub fn mappings_referencing(&self, profile: &ProfileName) -> Vec<Reference> {
        let matches = |name: &ProfileName| name.as_str().eq_ignore_ascii_case(profile.as_str());
        let mut references = Vec::new();
        for (root, mapping) in self.mappings() {
            if let Some(name) = mapping.profile.as_ref().filter(|name| matches(name)) {
                references.push(Reference {
                    root: root.to_path_buf(),
                    agent: None,
                    profile: name.clone(),
                });
            }
            for (agent, name) in mapping.agents.iter().filter(|(_, name)| matches(name)) {
                references.push(Reference {
                    root: root.to_path_buf(),
                    agent: Some(agent.clone()),
                    profile: name.clone(),
                });
            }
        }
        references
    }
}

fn invalid(path: &Path, key: Option<String>, detail: String) -> Error {
    Error::ConfigInvalid { path: path.to_path_buf(), key, detail }
}

fn parse(path: &Path, bytes: &[u8]) -> Result<Config> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| invalid(path, None, "the file is not valid UTF-8".to_owned()))?;
    let table: toml::Table = text.parse().map_err(|error: toml::de::Error| {
        let line = error.span().map(|span| text[..span.start].matches('\n').count() + 1);
        let detail = match line {
            Some(line) => format!("TOML syntax error at line {line}: {}", error.message()),
            None => format!("TOML syntax error: {}", error.message()),
        };
        invalid(path, None, detail)
    })?;
    validate(path, &table)
}

/// Applies the strict schema (SP1 design §6.2, SP3 design §6.1) to a parsed document.
fn validate(path: &Path, table: &toml::Table) -> Result<Config> {
    let mut config = Config::default();
    for (key, value) in table {
        match key.as_str() {
            "agents" => validate_agents(path, key, value, &mut config)?,
            "default_profile" => {
                let name = value.as_str().ok_or_else(|| {
                    invalid(path, Some(key.clone()), "must be a string".to_owned())
                })?;
                config.default_profile = Some(profile_name(path, key, name)?);
            }
            "repositories" => validate_repositories(path, key, value, &mut config)?,
            _ => return Err(invalid(path, Some(key.clone()), "unknown key".to_owned())),
        }
    }
    Ok(config)
}

fn profile_name(path: &Path, key: &str, name: &str) -> Result<ProfileName> {
    ProfileName::parse(name, Platform::host()).map_err(|reason| {
        invalid(path, Some(key.to_owned()), format!("invalid profile name {name:?}: {reason}"))
    })
}

/// `repositories.<key>` with the key quoted as TOML would write it.
fn repository_key(root: &str) -> String {
    format!("repositories.{}", toml_edit::Key::new(root).display_repr())
}

/// Absolute in Unix form (`/…`) or Windows form (`C:\…`, `C:/…`, `\\…`), whatever the host (SP3 design §6.1).
fn is_absolute_key(root: &str) -> bool {
    let bytes = root.as_bytes();
    let drive = bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/');
    root.starts_with('/') || root.starts_with(r"\\") || drive
}

fn validate_repositories(
    path: &Path,
    key: &str,
    value: &toml::Value,
    config: &mut Config,
) -> Result<()> {
    let repositories = value
        .as_table()
        .ok_or_else(|| invalid(path, Some(key.to_owned()), "must be a table".to_owned()))?;
    for (root, entry) in repositories {
        let entry_key = repository_key(root);
        if !is_absolute_key(root) {
            return Err(invalid(path, Some(entry_key), "must be an absolute path".to_owned()));
        }
        let entry = entry
            .as_table()
            .ok_or_else(|| invalid(path, Some(entry_key.clone()), "must be a table".to_owned()))?;
        let mut mapping = Mapping::default();
        for (field, value) in entry {
            let field_key = format!("{entry_key}.{field}");
            match field.as_str() {
                "profile" => {
                    let name = value.as_str().ok_or_else(|| {
                        invalid(path, Some(field_key.clone()), "must be a string".to_owned())
                    })?;
                    mapping.profile = Some(profile_name(path, &field_key, name)?);
                }
                "agents" => {
                    let agents = value.as_table().ok_or_else(|| {
                        invalid(path, Some(field_key.clone()), "must be a table".to_owned())
                    })?;
                    for (id, name) in agents {
                        let agent_key = format!("{field_key}.{id}");
                        let Some(agent) = AgentId::parse(id) else {
                            return Err(invalid(
                                path,
                                Some(agent_key),
                                "agent ids must match [a-z][a-z0-9-]*".to_owned(),
                            ));
                        };
                        let name = name.as_str().ok_or_else(|| {
                            invalid(path, Some(agent_key.clone()), "must be a string".to_owned())
                        })?;
                        mapping.agents.insert(agent, profile_name(path, &agent_key, name)?);
                    }
                }
                _ => return Err(invalid(path, Some(field_key), "unknown key".to_owned())),
            }
        }
        config.repositories.insert(root.clone(), mapping);
    }
    let roots: Vec<&String> = config.repositories.keys().collect();
    for (index, first) in roots.iter().enumerate() {
        if let Some(second) =
            roots[index + 1..].iter().find(|second| Path::new(second) == Path::new(first))
        {
            return Err(invalid(
                path,
                Some(repository_key(first)),
                format!("names the same directory as {}", repository_key(second)),
            ));
        }
    }
    Ok(())
}

fn validate_agents(path: &Path, key: &str, value: &toml::Value, config: &mut Config) -> Result<()> {
    let agents = value
        .as_table()
        .ok_or_else(|| invalid(path, Some(key.to_owned()), "must be a table".to_owned()))?;
    for (id, agent) in agents {
        let agent_key = format!("agents.{id}");
        if AgentId::parse(id).is_none() {
            return Err(invalid(
                path,
                Some(agent_key),
                "agent ids must match [a-z][a-z0-9-]*".to_owned(),
            ));
        }
        let agent = agent
            .as_table()
            .ok_or_else(|| invalid(path, Some(agent_key.clone()), "must be a table".to_owned()))?;
        let mut executable = None;
        for (field, value) in agent {
            let field_key = format!("{agent_key}.{field}");
            if field != "executable" {
                return Err(invalid(path, Some(field_key), "unknown key".to_owned()));
            }
            let text = value.as_str().ok_or_else(|| {
                invalid(path, Some(field_key.clone()), "must be a string".to_owned())
            })?;
            let candidate = PathBuf::from(text);
            if !candidate.is_absolute() {
                return Err(invalid(path, Some(field_key), "must be an absolute path".to_owned()));
            }
            executable = Some(candidate);
        }
        config.agents.insert(id.clone(), executable);
    }
    Ok(())
}

/// Locked, atomic read-modify-write of `config.toml` (spec §18, design §6.4).
pub fn update(
    root: &AppRoot,
    edit: impl FnOnce(&mut toml_edit::DocumentMut) -> Result<()>,
) -> Result<()> {
    update_with(
        root,
        edit,
        || Ok(()),
        |temp, destination| temp.persist(destination).map(drop).map_err(|error| error.error),
    )
}

pub(crate) fn update_with(
    root: &AppRoot,
    edit: impl FnOnce(&mut toml_edit::DocumentMut) -> Result<()>,
    before_persist: impl FnOnce() -> Result<()>,
    replace: impl FnOnce(NamedTempFile, &Path) -> io::Result<()>,
) -> Result<()> {
    let config_path = root.config_path();
    let write_error =
        |path: &Path, source: io::Error| Error::ConfigWrite { path: path.to_path_buf(), source };

    // 1. The root.
    fs::create_dir_all(root.path()).map_err(|source| write_error(root.path(), source))?;

    // 2. The exclusive lock, bounded.
    let lock_path = root.lock_path();
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|source| write_error(&lock_path, source))?;
    acquire(&lock, &lock_path)?;

    // 3. Best-effort sweep of temp files left by crashed writers.
    if let Ok(entries) = fs::read_dir(root.path()) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(TEMP_PREFIX) && name.ends_with(TEMP_SUFFIX) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    // 4. The latest configuration, validated.
    let text = match fs::read(&config_path) {
        Ok(bytes) => {
            parse(&config_path, &bytes)?;
            String::from_utf8(bytes).expect("parse checked UTF-8")
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(invalid(&config_path, None, format!("cannot read the file: {error}")));
        }
    };
    let mut document: toml_edit::DocumentMut = text
        .parse()
        .map_err(|error| invalid(&config_path, None, format!("TOML syntax error: {error}")))?;

    // 5. The edit, re-validated. An edit that changes nothing writes nothing.
    edit(&mut document)?;
    let updated = document.to_string();
    if updated == text {
        return Ok(());
    }
    parse(&config_path, updated.as_bytes())?;

    // 6. The temp file in the same directory, synced.
    let mut temp = tempfile::Builder::new()
        .prefix(TEMP_PREFIX)
        .suffix(TEMP_SUFFIX)
        .tempfile_in(root.path())
        .map_err(|source| write_error(&config_path, source))?;
    temp.write_all(updated.as_bytes()).map_err(|source| write_error(&config_path, source))?;
    temp.as_file().sync_all().map_err(|source| write_error(&config_path, source))?;
    before_persist()?;

    // 7. The atomic replace. On failure `replace` drops the temp file, which deletes it.
    replace(temp, &config_path).map_err(|source| write_error(&config_path, source))?;

    // 7a. Durability of the rename on Unix.
    #[cfg(unix)]
    File::open(root.path()).and_then(|directory| directory.sync_all()).map_err(|error| {
        write_error(
            &config_path,
            io::Error::new(
                error.kind(),
                format!(
                    "configuration replaced, but the directory could not be synced; the change may \
                     not survive a power loss: {error}"
                ),
            ),
        )
    })?;

    // 8. The lock is released when `lock` drops.
    drop(lock);
    Ok(())
}

/// Maps `repository` (or only `agent` in it) to `profile` (SP3 design §6.2). The caller has discovered
/// `repository`.
pub fn link(
    root: &AppRoot,
    repository: &Path,
    agent: Option<&AgentId>,
    profile: &ProfileName,
) -> Result<LinkOutcome> {
    let Some(key) = repository.to_str() else {
        return Err(Error::Repository {
            path: repository.to_path_buf(),
            reason: "a repository path that is not valid UTF-8 cannot be linked".to_owned(),
        });
    };
    check_case_twins(root, profile)?;
    let mut outcome = LinkOutcome::Linked;
    update(root, |document| {
        let stored = stored_keys(document)
            .into_iter()
            .find(|stored| Path::new(stored) == repository)
            .unwrap_or_else(|| key.to_owned());
        let repositories = child_table(document.as_table_mut(), "repositories", false);
        let entry = child_table(repositories, &stored, false);
        outcome = link_outcome(mapped(entry, agent), profile);
        if outcome == LinkOutcome::AlreadyLinked {
            return Ok(());
        }
        let value = toml_edit::value(profile.as_str());
        match agent {
            None => entry.insert("profile", value),
            Some(agent) => child_table(entry, "agents", true).insert(agent.as_str(), value),
        };
        Ok(())
    })?;
    Ok(outcome)
}

fn link_outcome(current: Option<String>, profile: &ProfileName) -> LinkOutcome {
    match current {
        None => LinkOutcome::Linked,
        Some(current) if current == profile.as_str() => LinkOutcome::AlreadyLinked,
        Some(current) => LinkOutcome::Changed {
            old: ProfileName::parse(&current, Platform::host()).expect("validated profile name"),
        },
    }
}

/// Removes the mapping at the first of `keys` that has one at that field, under one lock (SP3 design §6.2).
/// `keys` is never empty.
pub fn unlink(root: &AppRoot, keys: &[PathBuf], agent: Option<&AgentId>) -> Result<UnlinkOutcome> {
    let mut outcome = UnlinkOutcome::NothingToRemove { shown: keys[0].clone() };
    update(root, |document| {
        let Some(repositories) =
            document.get_mut("repositories").and_then(toml_edit::Item::as_table_like_mut)
        else {
            return Ok(());
        };
        let found = keys.iter().find_map(|key| {
            repositories.iter().find_map(|(stored, entry)| {
                let entry = entry.as_table_like()?;
                (Path::new(stored) == key)
                    .then(|| mapped(entry, agent))
                    .flatten()
                    .map(|old| (stored.to_owned(), old))
            })
        });
        let Some((stored, old)) = found else {
            return Ok(());
        };
        let entry = repositories
            .get_mut(&stored)
            .and_then(toml_edit::Item::as_table_like_mut)
            .expect("validated entry");
        match agent {
            None => {
                entry.remove("profile");
            }
            Some(agent) => {
                let agents = entry
                    .get_mut("agents")
                    .and_then(toml_edit::Item::as_table_like_mut)
                    .expect("validated agents table");
                agents.remove(agent.as_str());
                if agents.is_empty() {
                    entry.remove("agents");
                }
            }
        }
        if entry.is_empty() {
            repositories.remove(&stored);
        }
        outcome = UnlinkOutcome::Unlinked {
            root: PathBuf::from(stored),
            old: ProfileName::parse(&old, Platform::host()).expect("validated profile name"),
        };
        Ok(())
    })?;
    Ok(outcome)
}

/// The stored `repositories` keys of a validated document.
fn stored_keys(document: &toml_edit::DocumentMut) -> Vec<String> {
    document
        .get("repositories")
        .and_then(toml_edit::Item::as_table_like)
        .map(|repositories| repositories.iter().map(|(key, _)| key.to_owned()).collect())
        .unwrap_or_default()
}

/// The profile an entry maps at the field `agent` selects.
fn mapped(entry: &dyn toml_edit::TableLike, agent: Option<&AgentId>) -> Option<String> {
    let value = match agent {
        None => entry.get("profile"),
        Some(agent) => entry
            .get("agents")
            .and_then(toml_edit::Item::as_table_like)
            .and_then(|agents| agents.get(agent.as_str())),
    };
    value.and_then(toml_edit::Item::as_str).map(str::to_owned)
}

/// The table at `key` in `parent`, created when missing: an inline table when `inline`, otherwise an
/// implicit standard table.
fn child_table<'a>(
    parent: &'a mut dyn toml_edit::TableLike,
    key: &str,
    inline: bool,
) -> &'a mut dyn toml_edit::TableLike {
    let item = parent.entry(key).or_insert_with(|| {
        if inline {
            toml_edit::Item::Value(toml_edit::Value::InlineTable(toml_edit::InlineTable::new()))
        } else {
            let mut table = toml_edit::Table::new();
            table.set_implicit(true);
            toml_edit::Item::Table(table)
        }
    });
    item.as_table_like_mut().expect("validated as a table")
}

/// Refuses a profile whose name differs from an existing `profiles/` entry only in ASCII case (SP1 design §7.3).
pub(crate) fn check_case_twins(root: &AppRoot, profile: &ProfileName) -> Result<()> {
    let Ok(entries) = fs::read_dir(root.profiles_dir()) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name != profile.as_str() && name.eq_ignore_ascii_case(profile.as_str()) {
            return Err(Error::ProfileCaseConflict {
                requested: profile.to_string(),
                existing: name.to_owned(),
            });
        }
    }
    Ok(())
}

fn acquire(lock: &File, lock_path: &Path) -> Result<()> {
    let deadline = Instant::now() + LOCK_TIMEOUT;
    loop {
        match lock.try_lock() {
            Ok(()) => return Ok(()),
            Err(std::fs::TryLockError::WouldBlock) if Instant::now() < deadline => {
                thread::sleep(LOCK_RETRY);
            }
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(Error::ConfigWrite {
                    path: lock_path.to_path_buf(),
                    source: io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "another agent-profile process holds the configuration lock; retry, or check \
                         for a stuck process",
                    ),
                });
            }
            Err(std::fs::TryLockError::Error(source)) => {
                return Err(Error::ConfigWrite { path: lock_path.to_path_buf(), source });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    fn temp_root() -> (tempfile::TempDir, AppRoot) {
        let dir = tempfile::tempdir().unwrap();
        let root = AppRoot::from_path(dir.path().to_path_buf());
        (dir, root)
    }

    fn absolute(name: &str) -> String {
        std::env::temp_dir().join(name).to_str().unwrap().to_owned()
    }

    fn set_agent(doc: &mut toml_edit::DocumentMut, id: &str, executable: &str) {
        let agents = doc
            .entry("agents")
            .or_insert_with(|| {
                let mut agents = toml_edit::Table::new();
                agents.set_implicit(true);
                toml_edit::Item::Table(agents)
            })
            .as_table_mut()
            .expect("agents is a table");
        let mut agent = toml_edit::Table::new();
        agent.insert("executable", toml_edit::value(executable));
        agents.insert(id, toml_edit::Item::Table(agent));
    }

    #[test]
    fn app_root_override_must_be_absolute_and_non_empty() {
        let abs = std::env::temp_dir();
        assert_eq!(
            AppRoot::resolve_from(Some(abs.clone().into_os_string()), None).unwrap().path(),
            abs
        );
        for bad in ["", "relative/dir"] {
            let error = AppRoot::resolve_from(Some(bad.into()), Some(abs.clone())).unwrap_err();
            assert!(matches!(error, Error::AppRoot { .. }), "{bad:?}: {error:?}");
        }
    }

    #[test]
    fn non_utf8_home_override_is_an_app_root_error() {
        #[cfg(unix)]
        let value = {
            use std::os::unix::ffi::OsStringExt;
            OsString::from_vec(vec![0x66, 0xff])
        };
        #[cfg(windows)]
        let value = {
            use std::os::windows::ffi::OsStringExt;
            OsString::from_wide(&[0x66, 0xD800])
        };
        let error = AppRoot::resolve_from(Some(value), None).unwrap_err();
        assert!(matches!(error, Error::AppRoot { .. }), "{error:?}");
    }

    #[test]
    fn app_root_defaults_to_dot_agent_profile_in_home() {
        let home = std::env::temp_dir();
        let root = AppRoot::resolve_from(None, Some(home.clone())).unwrap();
        assert_eq!(root.path(), home.join(".agent-profile"));
        assert!(matches!(AppRoot::resolve_from(None, None), Err(Error::AppRoot { .. })));
        for bad in ["", "relative/home"] {
            let error = AppRoot::resolve_from(None, Some(PathBuf::from(bad))).unwrap_err();
            assert!(matches!(error, Error::AppRoot { .. }), "{bad:?}: {error:?}");
        }
    }

    #[test]
    fn schema_accepts_the_sp1_forms() {
        let exe = absolute("fake-agent");
        let text = format!("# comment\n[agents.fake]\nexecutable = {exe:?}\n\n[agents.claude]\n");
        let config = parse(Path::new("c"), text.as_bytes()).unwrap();
        assert_eq!(config.agent_executable("fake"), Some(Path::new(&exe)));
        assert_eq!(config.agent_executable("claude"), None);
        assert_eq!(config.agent_executable("codex"), None);
        assert_eq!(config.configured_agents().collect::<Vec<_>>(), ["claude", "fake"]);
        assert_eq!(parse(Path::new("c"), b"").unwrap(), Config::default());
    }

    #[test]
    fn schema_rejects_every_error_class_naming_the_key() {
        let exe = absolute("x");
        let cases = [
            (b"\xff".to_vec(), None),
            (b"[agents\n".to_vec(), None),
            (b"default = \"work\"\n".to_vec(), Some("default")),
            (b"agents = 3\n".to_vec(), Some("agents")),
            (b"[agents]\nFake = {}\n".to_vec(), Some("agents.Fake")),
            (b"[agents]\nfake = 3\n".to_vec(), Some("agents.fake")),
            (format!("[agents.fake]\npath = {exe:?}\n").into_bytes(), Some("agents.fake.path")),
            (b"[agents.fake]\nexecutable = 3\n".to_vec(), Some("agents.fake.executable")),
            (
                b"[agents.fake]\nexecutable = \"relative/x\"\n".to_vec(),
                Some("agents.fake.executable"),
            ),
        ];
        for (bytes, key) in cases {
            match parse(Path::new("c"), &bytes) {
                Err(Error::ConfigInvalid { key: actual, .. }) => {
                    assert_eq!(actual.as_deref(), key, "{:?}", String::from_utf8_lossy(&bytes))
                }
                other => panic!("{:?}: {other:?}", String::from_utf8_lossy(&bytes)),
            }
        }
    }

    #[test]
    fn failed_replacement_preserves_previous_config_and_leaves_no_temp_file() {
        let (_dir, root) = temp_root();
        let exe = absolute("old");
        let previous = format!("[agents.fake]\nexecutable = {exe:?}\n");
        fs::write(root.config_path(), &previous).unwrap();
        let error = update_with(
            &root,
            |doc| {
                set_agent(doc, "fake", &absolute("new"));
                Ok(())
            },
            || Ok(()),
            |temp, _destination| {
                drop(temp);
                Err(io::Error::other("injected replace failure"))
            },
        )
        .unwrap_err();
        assert!(matches!(error, Error::ConfigWrite { .. }), "{error:?}");
        assert_eq!(fs::read_to_string(root.config_path()).unwrap(), previous);
        assert_eq!(temp_files(&root), Vec::<String>::new());
    }

    #[test]
    fn reader_during_write_sees_previous_then_new_complete_content() {
        let (_dir, root) = temp_root();
        let previous = format!("[agents.fake]\nexecutable = {:?}\n", absolute("old"));
        fs::write(root.config_path(), &previous).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let reader = {
            let barrier = Arc::clone(&barrier);
            let root = root.clone();
            thread::spawn(move || {
                barrier.wait();
                let seen = fs::read_to_string(root.config_path()).unwrap();
                barrier.wait();
                seen
            })
        };
        update_with(
            &root,
            |doc| {
                set_agent(doc, "fake", &absolute("new"));
                Ok(())
            },
            || {
                barrier.wait();
                barrier.wait();
                Ok(())
            },
            |temp, destination| temp.persist(destination).map(drop).map_err(|error| error.error),
        )
        .unwrap();
        assert_eq!(reader.join().unwrap(), previous);
        let config = Config::load(&root).unwrap();
        assert_eq!(config.agent_executable("fake"), Some(Path::new(&absolute("new"))));
    }

    #[test]
    fn update_sweeps_stale_temp_files_and_never_reads_them() {
        let (_dir, root) = temp_root();
        fs::write(root.path().join(".config.toml.stale.tmp"), "garbage = [").unwrap();
        update(&root, |doc| {
            set_agent(doc, "fake", &absolute("x"));
            Ok(())
        })
        .unwrap();
        assert_eq!(temp_files(&root), Vec::<String>::new());
        assert!(Config::load(&root).unwrap().agent_executable("fake").is_some());
    }

    /// The Unix spelling on Unix, the Windows spelling on Windows.
    fn host<'a>(unix: &'a str, windows: &'a str) -> &'a str {
        if cfg!(windows) { windows } else { unix }
    }

    fn name(text: &str) -> ProfileName {
        ProfileName::parse(text, Platform::host()).unwrap()
    }

    fn agent(id: &str) -> AgentId {
        AgentId::parse(id).unwrap()
    }

    fn key(root: &str) -> String {
        toml_edit::Key::new(root).display_repr().into_owned()
    }

    #[test]
    fn schema_accepts_the_sp3_forms() {
        let acme = host("/src/acme", r"C:\src\acme");
        let text = format!(
            "default_profile = \"work\"\n\n[repositories.{}]\nprofile = \"work\"\nagents = {{ claude = \"personal\" }}\n\n[repositories.{}]\n\n[repositories.{}]\nprofile = \"other\"\n",
            key(acme),
            key(host("/empty", r"C:\empty")),
            key(host(r"C:\elsewhere", "/elsewhere")),
        );
        let config = parse(Path::new("c"), text.as_bytes()).unwrap();
        assert_eq!(config.default_profile(), Some(&name("work")));
        let expected = Mapping {
            profile: Some(name("work")),
            agents: BTreeMap::from([(agent("claude"), name("personal"))]),
        };
        assert_eq!(config.mapping(Path::new(acme)), Some(&expected));
        let trailing = format!("{acme}{}", std::path::MAIN_SEPARATOR);
        assert_eq!(config.mapping(Path::new(&trailing)), Some(&expected));
        assert_eq!(
            config.mapping(Path::new(host("/empty", r"C:\empty"))),
            Some(&Mapping::default())
        );
        assert_eq!(config.mappings().count(), 3);
        assert_eq!(config.mapping(Path::new(host("/src", r"C:\src"))), None);
        #[cfg(windows)]
        assert_eq!(config.mapping(Path::new("C:/src/acme")), Some(&expected));
    }

    #[test]
    fn schema_rejects_every_sp3_error_class_naming_the_key() {
        let acme = key(host("/acme", r"C:\acme"));
        let entry = |body: &str| format!("[repositories.{acme}]\n{body}\n");
        let entry_key = format!("repositories.{acme}");
        let cases = [
            ("default_profile = 3\n".to_owned(), "default_profile".to_owned(), "must be a string"),
            (
                "default_profile = \"link\"\n".to_owned(),
                "default_profile".to_owned(),
                "invalid profile name \"link\": \"link\" is a reserved command word",
            ),
            ("repositories = 3\n".to_owned(), "repositories".to_owned(), "must be a table"),
            (
                "[repositories.relative]\n".to_owned(),
                "repositories.relative".to_owned(),
                "must be an absolute path",
            ),
            (format!("[repositories]\n{acme} = 3\n"), entry_key.clone(), "must be a table"),
            (entry("profile = 3"), format!("{entry_key}.profile"), "must be a string"),
            (entry("profile = \".x\""), format!("{entry_key}.profile"), "invalid profile name"),
            (entry("agents = 3"), format!("{entry_key}.agents"), "must be a table"),
            (
                entry("agents = { Claude = \"x\" }"),
                format!("{entry_key}.agents.Claude"),
                "agent ids must match",
            ),
            (
                entry("agents = { claude = 3 }"),
                format!("{entry_key}.agents.claude"),
                "must be a string",
            ),
            (
                entry("agents = { claude = \"a b\" }"),
                format!("{entry_key}.agents.claude"),
                "invalid profile name",
            ),
            (entry("path = \"x\""), format!("{entry_key}.path"), "unknown key"),
        ];
        for (text, expected_key, detail) in cases {
            match parse(Path::new("c"), text.as_bytes()) {
                Err(Error::ConfigInvalid { key: Some(actual), detail: actual_detail, .. }) => {
                    assert_eq!(actual, expected_key, "{text:?}");
                    assert!(actual_detail.starts_with(detail), "{text:?}: {actual_detail}");
                }
                other => panic!("{text:?}: {other:?}"),
            }
        }
    }

    #[test]
    fn schema_accepts_keys_absolute_in_either_platform_form() {
        for root in ["/a", r"C:\a", "C:/a", r"\\server\share\a", "z:/"] {
            let text = format!("[repositories.{}]\n", key(root));
            assert!(parse(Path::new("c"), text.as_bytes()).is_ok(), "{root}");
        }
        for root in ["a", "C:", "C:a", r"\a", "~/a", ""] {
            let text = format!("[repositories.{}]\n", key(root));
            assert!(
                matches!(parse(Path::new("c"), text.as_bytes()), Err(Error::ConfigInvalid { .. })),
                "{root:?}"
            );
        }
    }

    #[test]
    fn component_equal_keys_are_rejected_naming_both() {
        let keys: &[&str] =
            if cfg!(windows) { &[r"C:\x", "C:/x", r"c:\x\"] } else { &["/a", "/a/"] };
        let text: String =
            keys.iter().map(|root| format!("[repositories.{}]\n", key(root))).collect();
        match parse(Path::new("c"), text.as_bytes()) {
            Err(Error::ConfigInvalid { key: Some(first), detail, .. }) => {
                let mut sorted = keys.to_vec();
                sorted.sort();
                assert_eq!(first, format!("repositories.{}", key(sorted[0])));
                assert_eq!(
                    detail,
                    format!("names the same directory as repositories.{}", key(sorted[1]))
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn link_reports_every_outcome_and_writes_only_on_change() {
        let (_dir, root) = temp_root();
        let repo = PathBuf::from(host("/src/acme", r"C:\src\acme"));
        let claude = agent("claude");
        assert_eq!(link(&root, &repo, None, &name("work")).unwrap(), LinkOutcome::Linked);
        assert_eq!(
            link(&root, &repo, Some(&claude), &name("personal")).unwrap(),
            LinkOutcome::Linked
        );
        let written = fs::read(root.config_path()).unwrap();
        assert_eq!(link(&root, &repo, None, &name("work")).unwrap(), LinkOutcome::AlreadyLinked);
        assert_eq!(
            link(&root, &repo, Some(&claude), &name("personal")).unwrap(),
            LinkOutcome::AlreadyLinked
        );
        assert_eq!(fs::read(root.config_path()).unwrap(), written, "no write when already linked");
        assert_eq!(
            link(&root, &repo, None, &name("other")).unwrap(),
            LinkOutcome::Changed { old: name("work") }
        );
        assert_eq!(
            link(&root, &repo, Some(&claude), &name("work")).unwrap(),
            LinkOutcome::Changed { old: name("personal") }
        );
        let config = Config::load(&root).unwrap();
        assert_eq!(
            config.mapping(&repo),
            Some(&Mapping {
                profile: Some(name("other")),
                agents: BTreeMap::from([(claude, name("work"))]),
            })
        );
    }

    #[test]
    fn link_and_unlink_preserve_comments_and_formatting() {
        let (_dir, root) = temp_root();
        let repo = host("/src/acme", r"C:\src\acme");
        let other = host("/src/other", r"C:\src\other");
        let text = format!(
            "# my settings\ndefault_profile   =   \"work\" # trailing\n\n[repositories.{}]\nprofile = \"work\" # keep\n",
            key(other)
        );
        fs::write(root.config_path(), &text).unwrap();
        link(&root, Path::new(repo), Some(&agent("claude")), &name("personal")).unwrap();
        let linked = fs::read_to_string(root.config_path()).unwrap();
        assert!(linked.starts_with(&text), "{linked}");
        assert_eq!(
            unlink(&root, &[PathBuf::from(repo)], Some(&agent("claude"))).unwrap(),
            UnlinkOutcome::Unlinked { root: PathBuf::from(repo), old: name("personal") }
        );
        assert_eq!(fs::read_to_string(root.config_path()).unwrap(), text);
    }

    #[test]
    fn unlink_removes_empty_agents_and_empty_entries() {
        let (_dir, root) = temp_root();
        let repo = PathBuf::from(host("/src/acme", r"C:\src\acme"));
        let keys = [repo.clone()];
        link(&root, &repo, None, &name("work")).unwrap();
        link(&root, &repo, Some(&agent("claude")), &name("personal")).unwrap();
        assert_eq!(
            unlink(&root, &keys, None).unwrap(),
            UnlinkOutcome::Unlinked { root: repo.clone(), old: name("work") }
        );
        assert_eq!(
            unlink(&root, &keys, None).unwrap(),
            UnlinkOutcome::NothingToRemove { shown: repo.clone() }
        );
        assert_eq!(
            unlink(&root, &keys, Some(&agent("codex"))).unwrap(),
            UnlinkOutcome::NothingToRemove { shown: repo.clone() }
        );
        assert!(Config::load(&root).unwrap().mapping(&repo).is_some());
        assert_eq!(
            unlink(&root, &keys, Some(&agent("claude"))).unwrap(),
            UnlinkOutcome::Unlinked { root: repo.clone(), old: name("personal") }
        );
        assert_eq!(Config::load(&root).unwrap().mapping(&repo), None);
        let text = fs::read_to_string(root.config_path()).unwrap();
        assert!(!text.contains("acme"), "{text}");
    }

    #[test]
    fn link_and_unlink_find_a_component_equal_stored_key() {
        let (_dir, root) = temp_root();
        let stored = host("/src/acme/", "C:/src/acme/");
        fs::write(
            root.config_path(),
            format!("[repositories.{}]\nprofile = \"work\"\n", key(stored)),
        )
        .unwrap();
        let repo = PathBuf::from(host("/src/acme", r"C:\src\acme"));
        assert_eq!(
            link(&root, &repo, Some(&agent("claude")), &name("personal")).unwrap(),
            LinkOutcome::Linked
        );
        assert_eq!(Config::load(&root).unwrap().mappings().count(), 1);
        assert_eq!(
            unlink(&root, std::slice::from_ref(&repo), None).unwrap(),
            UnlinkOutcome::Unlinked { root: PathBuf::from(stored), old: name("work") }
        );
    }

    #[test]
    fn unlink_takes_the_first_key_with_a_mapping_at_that_field() {
        let (_dir, root) = temp_root();
        let first = PathBuf::from(host("/a", r"C:\a"));
        let second = PathBuf::from(host("/b", r"C:\b"));
        let third = PathBuf::from(host("/c", r"C:\c"));
        link(&root, &first, Some(&agent("claude")), &name("x")).unwrap();
        link(&root, &second, None, &name("y")).unwrap();
        link(&root, &third, None, &name("z")).unwrap();
        let keys = [first.clone(), second.clone(), third.clone()];
        assert_eq!(
            unlink(&root, &keys, None).unwrap(),
            UnlinkOutcome::Unlinked { root: second, old: name("y") }
        );
        let missing = [PathBuf::from(host("/gone", r"C:\gone")), first.clone()];
        assert_eq!(
            unlink(&root, &missing, None).unwrap(),
            UnlinkOutcome::NothingToRemove { shown: missing[0].clone() }
        );
        assert_eq!(
            unlink(&root, &missing, Some(&agent("claude"))).unwrap(),
            UnlinkOutcome::Unlinked { root: first, old: name("x") }
        );
    }

    #[test]
    fn unlink_of_nothing_creates_no_configuration_file() {
        let (_dir, root) = temp_root();
        let keys = [PathBuf::from(host("/a", r"C:\a"))];
        assert!(matches!(unlink(&root, &keys, None), Ok(UnlinkOutcome::NothingToRemove { .. })));
        assert!(!root.config_path().exists());
    }

    #[test]
    fn link_refuses_a_case_twin_and_a_non_utf8_root() {
        let (_dir, root) = temp_root();
        fs::create_dir_all(root.profiles_dir().join("work")).unwrap();
        let repo = PathBuf::from(host("/a", r"C:\a"));
        let error = link(&root, &repo, None, &name("WORK")).unwrap_err();
        assert!(
            matches!(error, Error::ProfileCaseConflict { ref existing, .. } if existing == "work"),
            "{error:?}"
        );
        assert!(!root.config_path().exists());
        #[cfg(unix)]
        let non_utf8 = {
            use std::os::unix::ffi::OsStringExt;
            PathBuf::from(OsString::from_vec(b"/a\xff".to_vec()))
        };
        #[cfg(windows)]
        let non_utf8 = {
            use std::os::windows::ffi::OsStringExt;
            PathBuf::from(OsString::from_wide(&[0x43, 0x3A, 0x5C, 0xD800]))
        };
        match link(&root, &non_utf8, None, &name("work")).unwrap_err() {
            Error::Repository { path, reason } => {
                assert_eq!(path, non_utf8);
                assert_eq!(reason, "a repository path that is not valid UTF-8 cannot be linked");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn mappings_referencing_finds_case_twins_non_canonical_keys_and_agent_fields() {
        let text = format!(
            "[repositories.{}]\nprofile = \"Work\"\n\n[repositories.{}]\nagents = {{ claude = \"work\", codex = \"other\" }}\n\n[repositories.{}]\nprofile = \"other\"\n",
            key(host("/a", r"C:\a")),
            key(host("/b/../c/", "c:/b/../c/")),
            key(host("/d", r"C:\d")),
        );
        let config = parse(Path::new("c"), text.as_bytes()).unwrap();
        assert_eq!(
            config.mappings_referencing(&name("work")),
            [
                Reference {
                    root: PathBuf::from(host("/a", r"C:\a")),
                    agent: None,
                    profile: name("Work"),
                },
                Reference {
                    root: PathBuf::from(host("/b/../c/", "c:/b/../c/")),
                    agent: Some(agent("claude")),
                    profile: name("work"),
                },
            ]
        );
        assert_eq!(config.mappings_referencing(&name("none")), []);
    }

    #[test]
    fn an_edit_that_changes_nothing_writes_nothing() {
        let (_dir, root) = temp_root();
        let text = "default_profile='work'   # odd spacing\n";
        fs::write(root.config_path(), text).unwrap();
        update_with(
            &root,
            |_| Ok(()),
            || panic!("nothing to persist"),
            |_, _| panic!("no replace"),
        )
        .unwrap();
        assert_eq!(fs::read_to_string(root.config_path()).unwrap(), text);
    }

    fn temp_files(root: &AppRoot) -> Vec<String> {
        fs::read_dir(root.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(TEMP_PREFIX) && name.ends_with(TEMP_SUFFIX))
            .collect()
    }
}
