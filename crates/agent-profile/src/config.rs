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
use crate::name::AgentId;

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

/// Applies the strict SP1 schema (design §6.2) to a parsed document.
fn validate(path: &Path, table: &toml::Table) -> Result<Config> {
    let mut config = Config::default();
    for (key, value) in table {
        if key != "agents" {
            return Err(invalid(path, Some(key.clone()), "unknown key".to_owned()));
        }
        let agents = value
            .as_table()
            .ok_or_else(|| invalid(path, Some(key.clone()), "must be a table".to_owned()))?;
        for (id, agent) in agents {
            let agent_key = format!("agents.{id}");
            if AgentId::parse(id).is_none() {
                return Err(invalid(
                    path,
                    Some(agent_key),
                    "agent ids must match [a-z][a-z0-9-]*".to_owned(),
                ));
            }
            let agent = agent.as_table().ok_or_else(|| {
                invalid(path, Some(agent_key.clone()), "must be a table".to_owned())
            })?;
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
                    return Err(invalid(
                        path,
                        Some(field_key),
                        "must be an absolute path".to_owned(),
                    ));
                }
                executable = Some(candidate);
            }
            config.agents.insert(id.clone(), executable);
        }
    }
    Ok(config)
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

    // 5. The edit, re-validated.
    edit(&mut document)?;
    let updated = document.to_string();
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

    fn temp_files(root: &AppRoot) -> Vec<String> {
        fs::read_dir(root.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(TEMP_PREFIX) && name.ends_with(TEMP_SUFFIX))
            .collect()
    }
}
