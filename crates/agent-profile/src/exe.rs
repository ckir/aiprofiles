//! Executable discovery (spec §20): an explicit configured override, then `PATH`.

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

/// Finds `name` for `agent`: the `explicit` override when given, otherwise the first match on `path_var`.
/// A `.bat` or `.cmd` result is refused, because std would run it through `cmd.exe` (spec §23.2).
pub fn discover(
    agent: &str,
    name: &str,
    explicit: Option<&Path>,
    path_var: Option<&OsStr>,
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
        None => {
            let path = path_var
                .into_iter()
                .flat_map(std::env::split_paths)
                .filter(|dir| !dir.as_os_str().is_empty())
                .map(|dir| dir.join(file_name(name)))
                .find(|candidate| is_executable(candidate))
                .ok_or_else(|| not_installed(agent, NotInstalledReason::NotOnPath))?;
            Found { path, origin: Origin::Path }
        }
    };
    let is_batch = found
        .path
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|ext| ext.eq_ignore_ascii_case("bat") || ext.eq_ignore_ascii_case("cmd"));
    if is_batch {
        return Err(Error::UnsupportedExecutable { path: found.path });
    }
    Ok(found)
}

fn not_installed(agent: &str, reason: NotInstalledReason) -> Error {
    Error::AgentNotInstalled { agent: agent.to_owned(), reason, unknown_configured: Vec::new() }
}

fn file_name(name: &str) -> String {
    if cfg!(windows) { format!("{name}.exe") } else { name.to_owned() }
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

    #[test]
    fn explicit_override_wins_and_must_be_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let exe = make_file(dir.path(), "agent-bin", true);
        let found = discover("fake", "fake-agent", Some(&exe), None).unwrap();
        assert_eq!(found, Found { path: exe, origin: Origin::Configured });

        let missing = dir.path().join("missing");
        let error = discover("fake", "fake-agent", Some(&missing), None).unwrap_err();
        assert!(matches!(
            error,
            Error::AgentNotInstalled { reason: NotInstalledReason::ExplicitMissing(ref p), .. } if *p == missing
        ));
        let error = discover("fake", "fake-agent", Some(dir.path()), None).unwrap_err();
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
        let name = file_name("tool");
        make_file(first.path(), &name, true);
        let expected = make_file(second.path(), &name, true);
        let path_var = std::env::join_paths([empty.path(), second.path(), first.path()]).unwrap();
        let found = discover("fake", "tool", None, Some(&path_var)).unwrap();
        assert_eq!(found, Found { path: expected, origin: Origin::Path });
    }

    #[test]
    fn missing_from_path_is_not_installed() {
        let empty = tempfile::tempdir().unwrap();
        let path_var = std::env::join_paths([empty.path()]).unwrap();
        for path_var in [Some(path_var.as_os_str()), Some(OsStr::new("")), None] {
            let error = discover("fake", "tool", None, path_var).unwrap_err();
            assert!(matches!(
                error,
                Error::AgentNotInstalled { reason: NotInstalledReason::NotOnPath, .. }
            ));
        }
    }

    #[cfg(unix)]
    #[test]
    fn path_search_skips_files_without_an_execute_bit() {
        let dir = tempfile::tempdir().unwrap();
        make_file(dir.path(), "tool", false);
        let path_var = std::env::join_paths([dir.path()]).unwrap();
        assert!(discover("fake", "tool", None, Some(&path_var)).is_err());
    }

    #[test]
    fn batch_files_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["agent.cmd", "agent.BAT", "agent.Cmd"] {
            let path = make_file(dir.path(), name, true);
            let error = discover("fake", "fake-agent", Some(&path), None).unwrap_err();
            assert!(matches!(error, Error::UnsupportedExecutable { .. }), "{name}");
        }
    }
}
