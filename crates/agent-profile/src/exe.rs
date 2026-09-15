//! Executable discovery (spec §20): an explicit configured override, then the absolute entries of `PATH`.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, NotInstalledReason, Result};

/// Where a discovered executable came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Configured,
    Path,
}

/// A discovered executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub path: PathBuf,
    pub origin: Origin,
}

/// Extensions std would run through a shell or that need an interpreter: refused on every platform
/// (spec §23.2, SP2 design §7.2).
const SHELL_EXTENSIONS: [&str; 3] = ["bat", "cmd", "ps1"];

/// Windows `PATH` forms checked in each directory; the first directory holding any of them decides.
#[cfg(windows)]
const WINDOWS_FORMS: [&str; 5] = ["exe", "com", "cmd", "bat", "ps1"];

/// Finds `name` for `agent`: the `explicit` override when given, otherwise the first absolute `PATH`
/// directory that holds it. A shell script or a Windows shim is refused with a hint naming `config_file`.
pub fn discover(
    agent: &str,
    name: &str,
    explicit: Option<&Path>,
    path_var: Option<&OsStr>,
    config_file: &Path,
) -> Result<Found> {
    let found = match explicit {
        Some(path) => {
            if !fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
                return Err(not_installed(
                    agent,
                    NotInstalledReason::ExplicitMissing(path.to_path_buf()),
                ));
            }
            Found { path: path.to_path_buf(), origin: Origin::Configured }
        }
        None => search_path(agent, name, path_var, config_file)?,
    };
    let refused =
        found.path.extension().and_then(OsStr::to_str).is_some_and(|ext| {
            SHELL_EXTENSIONS.iter().any(|shell| ext.eq_ignore_ascii_case(shell))
        });
    if refused {
        return Err(unsupported(agent, found.path, config_file));
    }
    Ok(found)
}

fn search_path(
    agent: &str,
    name: &str,
    path_var: Option<&OsStr>,
    config_file: &Path,
) -> Result<Found> {
    // Only a Windows shim on `PATH` needs the hint.
    #[cfg(not(windows))]
    let _ = config_file;
    let mut ignored_relative = 0;
    for dir in path_var.into_iter().flat_map(std::env::split_paths) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        // A relative entry resolves against the working directory: repository-local discovery (spec §20, §36).
        if !dir.is_absolute() {
            ignored_relative += 1;
            continue;
        }
        match search_dir(&dir, name) {
            Some(Hit::Native(path)) => return Ok(Found { path, origin: Origin::Path }),
            #[cfg(windows)]
            Some(Hit::Unsupported(path)) => return Err(unsupported(agent, path, config_file)),
            None => {}
        }
    }
    Err(not_installed(agent, NotInstalledReason::NotOnPath { ignored_relative }))
}

/// What a `PATH` directory holds for an agent.
enum Hit {
    Native(PathBuf),
    /// A Windows form that needs a shell or interpreter (`.com`, `.cmd`, `.bat`, `.ps1`) and no `.exe`.
    #[cfg(windows)]
    Unsupported(PathBuf),
}

#[cfg(windows)]
fn search_dir(dir: &Path, name: &str) -> Option<Hit> {
    let mut forms = WINDOWS_FORMS
        .iter()
        .map(|ext| dir.join(format!("{name}.{ext}")))
        .filter(|candidate| is_executable(candidate));
    let first = forms.next()?;
    if first.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("exe")) {
        Some(Hit::Native(first))
    } else {
        Some(Hit::Unsupported(first))
    }
}

#[cfg(unix)]
fn search_dir(dir: &Path, name: &str) -> Option<Hit> {
    let candidate = dir.join(name);
    is_executable(&candidate).then_some(Hit::Native(candidate))
}

fn not_installed(agent: &str, reason: NotInstalledReason) -> Error {
    Error::AgentNotInstalled { agent: agent.to_owned(), reason, unknown_configured: Vec::new() }
}

fn unsupported(agent: &str, path: PathBuf, config_file: &Path) -> Error {
    Error::UnsupportedExecutable {
        agent: agent.to_owned(),
        path,
        config_file: config_file.to_path_buf(),
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = "/root/config.toml";

    fn make_file(dir: &Path, name: &str, executable: bool) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, b"x").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = if executable { 0o755 } else { 0o644 };
            fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        }
        #[cfg(windows)]
        let _ = executable;
        path
    }

    fn native(name: &str) -> String {
        if cfg!(windows) { format!("{name}.exe") } else { name.to_owned() }
    }

    fn find(name: &str, explicit: Option<&Path>, path_var: Option<&OsStr>) -> Result<Found> {
        discover("fake", name, explicit, path_var, Path::new(CONFIG))
    }

    #[test]
    fn explicit_override_wins_and_must_be_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let exe = make_file(dir.path(), "agent-bin", true);
        let found = find("fake-agent", Some(&exe), None).unwrap();
        assert_eq!(found, Found { path: exe, origin: Origin::Configured });

        let missing = dir.path().join("missing");
        let error = find("fake-agent", Some(&missing), None).unwrap_err();
        assert!(matches!(
            error,
            Error::AgentNotInstalled { reason: NotInstalledReason::ExplicitMissing(ref p), .. } if *p == missing
        ));
        let error = find("fake-agent", Some(dir.path()), None).unwrap_err();
        assert!(
            matches!(error, Error::AgentNotInstalled { .. }),
            "a directory is not an executable"
        );
    }

    #[test]
    fn path_search_takes_the_first_match_in_order() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let empty = tempfile::tempdir().unwrap();
        make_file(first.path(), &native("tool"), true);
        let expected = make_file(second.path(), &native("tool"), true);
        let path_var = std::env::join_paths([empty.path(), second.path(), first.path()]).unwrap();
        let found = find("tool", None, Some(&path_var)).unwrap();
        assert_eq!(found, Found { path: expected, origin: Origin::Path });
    }

    #[test]
    fn missing_from_path_is_not_installed() {
        let empty = tempfile::tempdir().unwrap();
        let path_var = std::env::join_paths([empty.path()]).unwrap();
        for path_var in [Some(path_var.as_os_str()), Some(OsStr::new("")), None] {
            let error = find("tool", None, path_var).unwrap_err();
            assert!(matches!(
                error,
                Error::AgentNotInstalled {
                    reason: NotInstalledReason::NotOnPath { ignored_relative: 0 },
                    ..
                }
            ));
        }
    }

    #[test]
    fn relative_path_entries_are_ignored_and_counted() {
        let mut entries =
            vec![PathBuf::from("."), PathBuf::from("bin"), PathBuf::from("..").join("tools")];
        if cfg!(windows) {
            entries.push(PathBuf::from(r"\tools"));
            entries.push(PathBuf::from("C:tools"));
        }
        let expected = entries.len();
        let path_var = std::env::join_paths(&entries).unwrap();
        let error = find("tool", None, Some(&path_var)).unwrap_err();
        match error {
            Error::AgentNotInstalled {
                reason: NotInstalledReason::NotOnPath { ignored_relative },
                ..
            } => assert_eq!(ignored_relative, expected),
            other => panic!("{other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn path_search_skips_files_without_an_execute_bit() {
        let dir = tempfile::tempdir().unwrap();
        make_file(dir.path(), "tool", false);
        let path_var = std::env::join_paths([dir.path()]).unwrap();
        assert!(find("tool", None, Some(&path_var)).is_err());
    }

    #[test]
    fn shell_scripts_are_refused_when_configured() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["agent.cmd", "agent.BAT", "agent.Cmd", "agent.ps1", "agent.PS1"] {
            let path = make_file(dir.path(), name, true);
            match find("fake-agent", Some(&path), None).unwrap_err() {
                Error::UnsupportedExecutable { agent, path: refused, config_file } => {
                    assert_eq!((agent.as_str(), refused), ("fake", path));
                    assert_eq!(config_file, PathBuf::from(CONFIG));
                }
                other => panic!("{name}: {other:?}"),
            }
        }
    }

    #[cfg(windows)]
    fn refused_on_path(dirs: &[&Path]) -> PathBuf {
        let path_var = std::env::join_paths(dirs).unwrap();
        match find("codex", None, Some(&path_var)).unwrap_err() {
            Error::UnsupportedExecutable { agent, path, config_file } => {
                assert_eq!(agent, "fake");
                assert_eq!(config_file, PathBuf::from(CONFIG));
                path
            }
            other => panic!("{other:?}"),
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_shim_only_directories_are_refused() {
        for form in ["codex.cmd", "codex.bat", "codex.ps1", "codex.com"] {
            let dir = tempfile::tempdir().unwrap();
            let shim = make_file(dir.path(), form, true);
            assert_eq!(refused_on_path(&[dir.path()]), shim, "{form}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_first_directory_decides_and_names_the_first_form() {
        let both = tempfile::tempdir().unwrap();
        make_file(both.path(), "codex.ps1", true);
        let cmd = make_file(both.path(), "codex.cmd", true);
        assert_eq!(refused_on_path(&[both.path()]), cmd);

        let later = tempfile::tempdir().unwrap();
        make_file(later.path(), "codex.exe", true);
        assert_eq!(refused_on_path(&[both.path(), later.path()]), cmd);

        let beside = tempfile::tempdir().unwrap();
        make_file(beside.path(), "codex.cmd", true);
        let exe = make_file(beside.path(), "codex.exe", true);
        let path_var = std::env::join_paths([beside.path()]).unwrap();
        assert_eq!(
            find("codex", None, Some(&path_var)).unwrap(),
            Found { path: exe, origin: Origin::Path }
        );
    }
}
