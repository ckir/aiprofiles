//! Repository discovery (spec §13) and canonical repository identity (spec §14). SP3 design §5.
//!
//! Discovery never runs Git and never reads Git configuration: it reads the `.git` entry, the existence of
//! `HEAD`, and the capped contents of a `.git` file and its `commondir` (design D1).

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use crate::error::{Error, Result};

/// The largest `.git` file or `commondir` file discovery reads.
const MAX_METADATA_FILE: u64 = 64 * 1024;

/// The outcome of discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Discovery {
    /// The canonical working-tree root.
    Repository(PathBuf),
    NotInRepository,
}

/// `fs::canonicalize`, then `strip_verbatim` (design §5.4).
pub fn canonical(path: &Path) -> io::Result<PathBuf> {
    fs::canonicalize(path).map(|path| strip_verbatim(&path))
}

/// `\\?\C:\x` becomes `C:\x` and `\\?\UNC\server\share\x` becomes `\\server\share\x`; any other path is
/// returned unchanged.
#[cfg(windows)]
pub fn strip_verbatim(path: &Path) -> PathBuf {
    use std::path::Prefix;
    let mut components = path.components();
    let Some(Component::Prefix(prefix)) = components.next() else {
        return path.to_path_buf();
    };
    let mut stripped = match prefix.kind() {
        Prefix::VerbatimDisk(letter) => PathBuf::from(format!("{}:", char::from(letter))),
        Prefix::VerbatimUNC(server, share) => {
            let mut unc = std::ffi::OsString::from(r"\\");
            unc.push(server);
            unc.push(r"\");
            unc.push(share);
            PathBuf::from(unc)
        }
        _ => return path.to_path_buf(),
    };
    stripped.push(components.as_path());
    stripped
}

/// Identity: Unix paths have no verbatim form.
#[cfg(not(windows))]
pub fn strip_verbatim(path: &Path) -> PathBuf {
    path.to_path_buf()
}

/// Whether discovery may touch `target`, a `gitdir` or `commondir` target joined to its base (design §5.3).
/// An allow-list: on Windows only a local drive, or the network share `repository_dir` itself lives on.
pub fn target_allowed(target: &Path, repository_dir: &Path) -> bool {
    if target.as_os_str().as_encoded_bytes().contains(&0) {
        return false;
    }
    prefix_allowed(target, repository_dir)
}

#[cfg(windows)]
fn prefix_allowed(target: &Path, repository_dir: &Path) -> bool {
    use std::path::Prefix;
    fn share(path: &Path) -> Option<(&OsStr, &OsStr)> {
        match path.components().next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => {
                    Some((server, share))
                }
                _ => None,
            },
            _ => None,
        }
    }
    match target.components().next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(_) | Prefix::VerbatimDisk(_) => true,
            Prefix::UNC(..) | Prefix::VerbatimUNC(..) => {
                match (share(target), share(repository_dir)) {
                    (Some((server, name)), Some((repo_server, repo_name))) => {
                        server.eq_ignore_ascii_case(repo_server)
                            && name.eq_ignore_ascii_case(repo_name)
                    }
                    _ => false,
                }
            }
            _ => false,
        },
        _ => false,
    }
}

#[cfg(not(windows))]
fn prefix_allowed(_target: &Path, _repository_dir: &Path) -> bool {
    true
}

/// Finds the repository containing `start` (design §5.1-§5.2).
pub fn discover(start: &Path) -> Result<Discovery> {
    let canonical_start = canonical(start)
        .map_err(|error| repository(start, format!("cannot resolve the directory: {error}")))?;
    match fs::metadata(&canonical_start) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => return Err(repository(start, "not a directory")),
        Err(error) => {
            return Err(repository(start, format!("cannot resolve the directory: {error}")));
        }
    }
    for dir in canonical_start.ancestors() {
        if examine(dir)? {
            return Ok(Discovery::Repository(dir.to_path_buf()));
        }
    }
    Ok(Discovery::NotInRepository)
}

/// Whether `dir` is a repository root; `false` when `dir/.git` does not exist (design §5.2).
fn examine(dir: &Path) -> Result<bool> {
    let dot_git = dir.join(".git");
    let metadata = match fs::metadata(&dot_git) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return match fs::symlink_metadata(&dot_git) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
                Ok(_) => Err(repository(&dot_git, ".git is a broken symbolic link")),
                Err(error) => Err(repository(&dot_git, format!("cannot read .git: {error}"))),
            };
        }
        Err(error) => return Err(repository(&dot_git, format!("cannot read .git: {error}"))),
    };
    if metadata.is_dir() {
        return if is_file(&dot_git.join("HEAD")) {
            Ok(true)
        } else {
            Err(repository(&dot_git, "invalid .git directory: no HEAD file"))
        };
    }
    if !metadata.is_file() {
        return Err(repository(&dot_git, ".git is neither a directory nor a file"));
    }
    let bytes = read_capped(&dot_git)
        .map_err(|error| repository(&dot_git, format!("cannot read .git: {error}")))?
        .ok_or_else(|| repository(&dot_git, "invalid .git file: too large"))?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| repository(&dot_git, "invalid .git file: not UTF-8"))?;
    let value = first_line(text)
        .strip_prefix("gitdir: ")
        .ok_or_else(|| repository(&dot_git, "invalid .git file: no gitdir line"))?;

    let target = dir.join(value);
    if !target_allowed(&target, dir) {
        return Err(repository(
            &dot_git,
            format!("gitdir points to a network or device path: {}", target.display()),
        ));
    }
    let missing = || {
        repository(
            &dot_git,
            format!("gitdir {} is missing or is not a Git directory", target.display()),
        )
    };
    let gitdir = canonical(&target).map_err(|_| missing())?;
    if !fs::metadata(&gitdir).is_ok_and(|metadata| metadata.is_dir())
        || !is_file(&gitdir.join("HEAD"))
    {
        return Err(missing());
    }
    check_commondir(dir, &gitdir)?;
    Ok(true)
}

/// A `commondir` entry, when present, must name an existing directory (design §5.2).
fn check_commondir(dir: &Path, gitdir: &Path) -> Result<()> {
    let file = gitdir.join("commondir");
    match fs::symlink_metadata(&file) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        _ => {}
    }
    let invalid = |detail: String| repository(&file, format!("invalid commondir file: {detail}"));
    match fs::metadata(&file) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return Err(invalid("not a regular file".to_owned())),
        Err(error) => return Err(invalid(format!("cannot read: {error}"))),
    }
    let bytes = read_capped(&file)
        .map_err(|error| invalid(format!("cannot read: {error}")))?
        .ok_or_else(|| invalid("too large".to_owned()))?;
    let text = std::str::from_utf8(&bytes).map_err(|_| invalid("not UTF-8".to_owned()))?;
    let value = first_line(text);
    if value.is_empty() {
        return Err(invalid("empty".to_owned()));
    }
    let target = gitdir.join(value);
    if !target_allowed(&target, dir) {
        return Err(repository(
            &file,
            format!("commondir points to a network or device path: {}", target.display()),
        ));
    }
    if !fs::metadata(&target).is_ok_and(|metadata| metadata.is_dir()) {
        return Err(repository(
            &file,
            format!("commondir {} is missing or is not a directory", target.display()),
        ));
    }
    Ok(())
}

/// The candidate mapping keys for `unlink --repo <repo>`, in order, duplicates dropped (design §6.2).
pub fn unlink_keys(cwd: &Path, repo: &OsStr) -> Vec<PathBuf> {
    let joined = cwd.join(repo);
    let mut keys = Vec::new();
    let mut push = |key: PathBuf| {
        if !keys.contains(&key) {
            keys.push(key);
        }
    };
    if let Ok(key) = canonical(&joined) {
        push(key);
    }
    if let Some(key) = resolved(&joined) {
        push(key);
    }
    push(strip_verbatim(&joined));
    keys
}

/// The canonical deepest existing ancestor of `path`, followed by the remaining components with `.` dropped
/// and `..` applied lexically.
fn resolved(path: &Path) -> Option<PathBuf> {
    let components: Vec<Component<'_>> = path.components().collect();
    for split in (1..=components.len()).rev() {
        let base: PathBuf = components[..split].iter().collect();
        let Ok(mut key) = canonical(&base) else { continue };
        for component in &components[split..] {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    key.pop();
                }
                Component::Normal(name) => key.push(name),
                Component::Prefix(_) | Component::RootDir => return None,
            }
        }
        return Some(key);
    }
    None
}

fn repository(path: &Path, reason: impl Into<String>) -> Error {
    Error::Repository { path: path.to_path_buf(), reason: reason.into() }
}

fn is_file(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

/// The file's bytes, or `None` when it is larger than `MAX_METADATA_FILE`.
fn read_capped(path: &Path) -> io::Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(MAX_METADATA_FILE + 1).read_to_end(&mut bytes)?;
    Ok((bytes.len() as u64 <= MAX_METADATA_FILE).then_some(bytes))
}

/// The first line without its line ending.
fn first_line(text: &str) -> &str {
    text.split('\n').next().unwrap_or_default().trim_end_matches('\r')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// A temp directory that no enclosing `.git` can turn into a repository (design §8.5).
    fn guarded() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for ancestor in dir.path().ancestors() {
            assert!(
                fs::symlink_metadata(ancestor.join(".git")).is_err(),
                "{} has a .git entry, so this test cannot build a layout outside a repository",
                ancestor.display()
            );
        }
        dir
    }

    fn root_of(dir: &tempfile::TempDir) -> PathBuf {
        canonical(dir.path()).unwrap()
    }

    fn git_dir(path: &Path) {
        fs::create_dir_all(path.join(".git")).unwrap();
        fs::write(path.join(".git").join("HEAD"), "ref: refs/heads/main\n").unwrap();
    }

    fn write(path: &Path, contents: impl AsRef<[u8]>) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn found(start: &Path) -> PathBuf {
        match discover(start) {
            Ok(Discovery::Repository(root)) => root,
            other => panic!("{}: {other:?}", start.display()),
        }
    }

    /// Asserts `Error::Repository` with `path` and a reason starting with `reason`.
    fn refused(start: &Path, path: &Path, reason: &str) {
        match discover(start) {
            Err(Error::Repository { path: actual, reason: actual_reason }) => {
                assert_eq!(actual, path, "{actual_reason}");
                assert!(actual_reason.starts_with(reason), "{actual_reason:?} !~ {reason:?}");
            }
            other => panic!("{}: {other:?}", start.display()),
        }
    }

    #[test]
    fn a_git_directory_with_head_is_the_root_from_the_root_and_a_deep_subdirectory() {
        let dir = guarded();
        let root = root_of(&dir);
        git_dir(&root);
        fs::create_dir_all(root.join("a").join("b").join("c")).unwrap();
        assert_eq!(found(&root), root);
        assert_eq!(found(&root.join("a").join("b").join("c")), root);
    }

    #[test]
    fn a_worktree_git_file_with_gitdir_and_commondir_is_a_root() {
        let dir = guarded();
        let root = root_of(&dir);
        let main = root.join("main");
        git_dir(&main);
        let admin = main.join(".git").join("worktrees").join("wt");
        write(&admin.join("HEAD"), "ref: refs/heads/wt\n");
        write(&admin.join("commondir"), "../..\n");
        let worktree = root.join("wt");
        write(&worktree.join(".git"), format!("gitdir: {}\n", admin.to_str().unwrap()));
        fs::create_dir_all(worktree.join("src")).unwrap();
        assert_eq!(found(&worktree.join("src")), worktree);
        assert_eq!(found(&main), main);
    }

    #[test]
    fn a_submodule_git_file_into_modules_is_a_root() {
        let dir = guarded();
        let root = root_of(&dir);
        git_dir(&root);
        write(&root.join(".git").join("modules").join("x").join("HEAD"), "0123\n");
        let submodule = root.join("x");
        write(&submodule.join(".git"), "gitdir: ../.git/modules/x\n");
        assert_eq!(found(&submodule), submodule);
    }

    #[test]
    fn a_nested_repository_is_its_own_root() {
        let dir = guarded();
        let root = root_of(&dir);
        git_dir(&root);
        let nested = root.join("vendor").join("lib");
        git_dir(&nested);
        assert_eq!(found(&nested), nested);
        assert_eq!(found(&root.join("vendor")), root);
    }

    #[test]
    fn a_start_inside_the_git_directory_reaches_the_repository() {
        let dir = guarded();
        let root = root_of(&dir);
        git_dir(&root);
        let objects = root.join(".git").join("objects");
        fs::create_dir_all(&objects).unwrap();
        assert_eq!(found(&objects), root);
    }

    #[test]
    fn no_git_anywhere_is_not_in_a_repository() {
        let dir = guarded();
        let deep = root_of(&dir).join("a").join("b");
        fs::create_dir_all(&deep).unwrap();
        assert_eq!(discover(&deep).unwrap(), Discovery::NotInRepository);
    }

    #[test]
    fn a_missing_start_and_a_file_start_are_repository_errors() {
        let dir = guarded();
        let missing = dir.path().join("missing");
        refused(&missing, &missing, "cannot resolve the directory: ");
        let file = dir.path().join("file");
        fs::write(&file, b"x").unwrap();
        refused(&file, &file, "not a directory");
    }

    #[test]
    fn broken_git_entries_are_errors_never_the_parent_root() {
        let big = "gitdir: x\n".to_owned() + &"#".repeat(MAX_METADATA_FILE as usize);
        type Layout = Box<dyn Fn(&Path, &Path)>;
        // Each layout builds a nested `inner` inside the repository `outer`; `(file, reason)` is expected.
        let cases: Vec<(&str, Layout, &str, &str)> = vec![
            (
                "empty .git directory",
                Box::new(|_, inner| fs::create_dir_all(inner.join(".git")).unwrap()),
                ".git",
                "invalid .git directory: no HEAD file",
            ),
            (
                "HEAD is a directory",
                Box::new(|_, inner| fs::create_dir_all(inner.join(".git").join("HEAD")).unwrap()),
                ".git",
                "invalid .git directory: no HEAD file",
            ),
            (
                "garbage .git file",
                Box::new(|_, inner| write(&inner.join(".git"), "garbage\n")),
                ".git",
                "invalid .git file: no gitdir line",
            ),
            (
                "gitdir that does not exist",
                Box::new(|_, inner| write(&inner.join(".git"), "gitdir: ../nowhere\n")),
                ".git",
                "gitdir ",
            ),
            (
                "gitdir without HEAD",
                Box::new(|outer, inner| {
                    fs::create_dir_all(outer.join("admin")).unwrap();
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                ".git",
                "gitdir ",
            ),
            (
                "commondir target missing",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    write(&outer.join("admin").join("commondir"), "../gone\n");
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "commondir ",
            ),
            (
                ".git file over 64 KiB",
                Box::new(move |_, inner| write(&inner.join(".git"), &big)),
                ".git",
                "invalid .git file: too large",
            ),
            (
                "commondir over 64 KiB",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    write(
                        &outer.join("admin").join("commondir"),
                        "#".repeat(MAX_METADATA_FILE as usize + 1),
                    );
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "invalid commondir file: too large",
            ),
            (
                "commondir is a directory",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    fs::create_dir_all(outer.join("admin").join("commondir")).unwrap();
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "invalid commondir file: not a regular file",
            ),
            (
                "empty commondir",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    write(&outer.join("admin").join("commondir"), "\n");
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "invalid commondir file: empty",
            ),
            (
                "gitdir without the space",
                Box::new(|outer, inner| {
                    // The gitdir target is valid, so only the missing space can refuse this layout.
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    write(&inner.join(".git"), "gitdir:../admin\n");
                }),
                ".git",
                "invalid .git file: no gitdir line",
            ),
            (
                "non-UTF-8 .git file",
                Box::new(|_, inner| write(&inner.join(".git"), b"gitdir: \xff\n")),
                ".git",
                "invalid .git file: not UTF-8",
            ),
            (
                "non-UTF-8 commondir",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    write(&outer.join("admin").join("commondir"), b"\xff\n");
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "invalid commondir file: not UTF-8",
            ),
            (
                "gitdir with a NUL",
                Box::new(|_, inner| write(&inner.join(".git"), "gitdir: ../a\0b\n")),
                ".git",
                "gitdir points to a network or device path: ",
            ),
            (
                "commondir with a NUL",
                Box::new(|outer, inner| {
                    write(&outer.join("admin").join("HEAD"), "x\n");
                    write(&outer.join("admin").join("commondir"), "../a\0b\n");
                    write(&inner.join(".git"), "gitdir: ../admin\n");
                }),
                "admin/commondir",
                "commondir points to a network or device path: ",
            ),
        ];
        for (name, layout, file, reason) in cases {
            let dir = guarded();
            let outer = root_of(&dir);
            git_dir(&outer);
            let inner = outer.join("inner");
            fs::create_dir_all(&inner).unwrap();
            layout(&outer, &inner);
            let expected = if file == ".git" {
                inner.join(".git")
            } else {
                outer.join("admin").join("commondir")
            };
            match discover(&inner) {
                Err(Error::Repository { path, reason: actual }) => {
                    assert_eq!(path, expected, "{name}: {actual}");
                    assert!(actual.starts_with(reason), "{name}: {actual:?} !~ {reason:?}");
                }
                other => panic!("{name}: {other:?}"),
            }
        }
    }

    /// The cap is "at most 64 KiB" (design §5.2), so a file of exactly that size is still read. The refusals one
    /// byte over are in the table above.
    #[test]
    fn a_git_file_and_a_commondir_of_exactly_64_kib_are_accepted() {
        let dir = guarded();
        let root = root_of(&dir);
        let padded = |first: &str| {
            let mut text = first.to_owned();
            text.push_str(&"#".repeat(MAX_METADATA_FILE as usize - text.len()));
            assert_eq!(text.len() as u64, MAX_METADATA_FILE);
            text
        };
        write(&root.join("admin").join("HEAD"), "x\n");
        write(&root.join("admin").join("commondir"), padded(".\n"));
        let worktree = root.join("wt");
        write(&worktree.join(".git"), padded("gitdir: ../admin\n"));
        assert_eq!(found(&worktree), worktree);
    }

    #[test]
    fn a_git_file_with_crlf_line_endings_is_accepted() {
        let dir = guarded();
        let root = root_of(&dir);
        write(&root.join("admin").join("HEAD"), "x\r\n");
        write(&root.join("admin").join("commondir"), ".\r\n");
        let worktree = root.join("wt");
        write(&worktree.join(".git"), "gitdir: ../admin\r\nsecond line\r\n");
        assert_eq!(found(&worktree), worktree);
    }

    #[test]
    fn a_gitdir_or_commondir_error_names_the_joined_target() {
        let dir = guarded();
        let root = root_of(&dir);
        let worktree = root.join("wt");
        write(&worktree.join(".git"), "gitdir: ../nowhere\n");
        let target = worktree.join("../nowhere");
        refused(
            &worktree,
            &worktree.join(".git"),
            &format!("gitdir {} is missing or is not a Git directory", target.display()),
        );
        write(&root.join("admin").join("HEAD"), "x\n");
        write(&root.join("admin").join("commondir"), "../gone\n");
        write(&worktree.join(".git"), "gitdir: ../admin\n");
        let target = root.join("admin").join("../gone");
        refused(
            &worktree,
            &root.join("admin").join("commondir"),
            &format!("commondir {} is missing or is not a directory", target.display()),
        );
    }

    #[test]
    fn a_nul_target_is_refused_on_every_platform() {
        assert!(!target_allowed(Path::new("a\0b"), Path::new("repo")));
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_start_resolves_to_the_real_root() {
        let dir = guarded();
        let root = root_of(&dir);
        let real = root.join("real");
        git_dir(&real);
        std::os::unix::fs::symlink(&real, root.join("link")).unwrap();
        assert_eq!(found(&root.join("link")), real);
    }

    #[cfg(unix)]
    #[test]
    fn a_dangling_git_symlink_is_an_error() {
        let dir = guarded();
        let root = root_of(&dir);
        std::os::unix::fs::symlink(root.join("nowhere"), root.join(".git")).unwrap();
        refused(&root, &root.join(".git"), ".git is a broken symbolic link");
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_commondir_is_an_error() {
        use std::os::unix::fs::PermissionsExt;
        let dir = guarded();
        let root = root_of(&dir);
        let commondir = root.join("admin").join("commondir");
        write(&root.join("admin").join("HEAD"), "x\n");
        write(&commondir, "..\n");
        write(&root.join("wt").join(".git"), "gitdir: ../admin\n");
        fs::set_permissions(&commondir, fs::Permissions::from_mode(0o000)).unwrap();
        if fs::read(&commondir).is_ok() {
            eprintln!("skipped: running as a user that ignores file permissions");
            return;
        }
        refused(&root.join("wt"), &commondir, "invalid commondir file: cannot read: ");
    }

    #[cfg(unix)]
    #[test]
    fn a_fifo_commondir_is_refused_without_blocking() {
        let dir = guarded();
        let root = root_of(&dir);
        let commondir = root.join("admin").join("commondir");
        write(&root.join("admin").join("HEAD"), "x\n");
        let status = Command::new("mkfifo").arg(&commondir).status().unwrap();
        assert!(status.success());
        write(&root.join("wt").join(".git"), "gitdir: ../admin\n");
        let start = root.join("wt");
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || sender.send(discover(&start)).unwrap());
        let result = receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("discovery blocked on a FIFO");
        match result {
            Err(Error::Repository { path, reason }) => {
                assert_eq!(path, commondir);
                assert_eq!(reason, "invalid commondir file: not a regular file");
            }
            other => panic!("{other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_git_entry_that_is_neither_a_directory_nor_a_file_is_an_error() {
        let dir = guarded();
        let root = root_of(&dir);
        let status = Command::new("mkfifo").arg(root.join(".git")).status().unwrap();
        assert!(status.success());
        fs::create_dir_all(root.join("sub")).unwrap();
        refused(&root.join("sub"), &root.join(".git"), ".git is neither a directory nor a file");
    }

    #[cfg(windows)]
    #[test]
    fn a_junction_start_resolves_to_the_real_root() {
        let dir = guarded();
        let root = root_of(&dir);
        let real = root.join("real");
        git_dir(&real);
        let junction = root.join("junction");
        let status = Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(&junction)
            .arg(&real)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(found(&junction.join(".")), real);
    }

    #[cfg(windows)]
    #[test]
    fn strip_verbatim_handles_drive_unc_and_other_verbatim_forms() {
        for (input, expected) in [
            (r"\\?\C:\x\y", r"C:\x\y"),
            (r"\\?\C:\", r"C:\"),
            (r"\\?\UNC\server\share\x", r"\\server\share\x"),
            (r"\\?\Volume{0f0e}\x", r"\\?\Volume{0f0e}\x"),
            (r"\\?\GLOBALROOT\Device\x", r"\\?\GLOBALROOT\Device\x"),
            (r"C:\plain", r"C:\plain"),
            (r"\\server\share\x", r"\\server\share\x"),
        ] {
            assert_eq!(strip_verbatim(Path::new(input)), Path::new(expected), "{input}");
            assert_eq!(
                strip_verbatim(Path::new(input)).as_os_str(),
                OsStr::new(expected),
                "{input}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn target_allowed_accepts_local_drives_and_the_repository_share_only() {
        let local = Path::new(r"C:\repo");
        let shared = Path::new(r"\\server\share\repo");
        for (target, repository_dir, allowed) in [
            (r"\\server\share\x", local, false),
            (r"//server/share/x", local, false),
            (r"\/server/share/x", local, false),
            (r"//./pipe/x", local, false),
            (r"\\.\pipe\x", local, false),
            (r"\\?\UNC\s\x", local, false),
            (r"\\?\Volume{0f0e}\x", local, false),
            (r"\\?\GLOBALROOT\x", local, false),
            (r"\\evil\\share\x", local, false),
            (r"C:\??\UNC\s\x", local, true),
            (r"C:\x", local, true),
            (r"\\?\C:\x", local, true),
            (r"\\SERVER\Share\other", shared, true),
            (r"\\?\UNC\server\share\x", shared, true),
            (r"\\server\elsewhere\x", shared, false),
            (r"C:\a\0b", local, true),
            ("C:\\a\0b", local, false),
        ] {
            assert_eq!(
                target_allowed(Path::new(target), repository_dir),
                allowed,
                "{target:?} from {}",
                repository_dir.display()
            );
        }
        // Rust joins a rooted value onto the drive of the base, so the value stays local.
        #[allow(clippy::join_absolute_paths)]
        let joined = local.join(r"\??\UNC\s\x");
        assert_eq!(joined, Path::new(r"C:\??\UNC\s\x"));
    }

    #[cfg(windows)]
    #[test]
    fn network_and_device_gitdir_targets_are_refused_before_any_filesystem_call() {
        for value in
            [r"\\server\share\x", r"\\.\pipe\x", r"\\?\Volume{0f0e}\x", r"//server/share/x"]
        {
            let dir = guarded();
            let root = root_of(&dir);
            write(&root.join(".git"), format!("gitdir: {value}\n"));
            refused(
                &root,
                &root.join(".git"),
                &format!(
                    "gitdir points to a network or device path: {}",
                    root.join(value).display()
                ),
            );
            write(&root.join("admin").join("HEAD"), "x\n");
            write(&root.join("admin").join("commondir"), format!("{value}\n"));
            write(&root.join(".git"), "gitdir: admin\n");
            refused(
                &root,
                &root.join("admin").join("commondir"),
                "commondir points to a network or device path: ",
            );
        }
    }

    /// `git` isolated from the developer's and the machine's configuration (design §8.2).
    fn git(cwd: &Path, args: &[&str]) -> String {
        let empty = cwd.parent().unwrap().join("empty-gitconfig");
        fs::write(&empty, b"").unwrap();
        let mut command = Command::new("git");
        for (name, _) in std::env::vars_os() {
            if name.to_string_lossy().starts_with("GIT_") {
                command.env_remove(name);
            }
        }
        let output = command
            .current_dir(cwd)
            .env("GIT_CONFIG_GLOBAL", &empty)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"])
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    fn git_toplevel(cwd: &Path) -> PathBuf {
        canonical(Path::new(git(cwd, &["rev-parse", "--show-toplevel"]).trim_end())).unwrap()
    }

    fn git_repository(path: &Path) {
        fs::create_dir_all(path).unwrap();
        git(path, &["init", "-q"]);
        git(path, &["commit", "-q", "--allow-empty", "-m", "init"]);
    }

    #[test]
    fn discovery_matches_git_for_init_worktree_and_submodule() {
        let dir = guarded();
        let root = root_of(&dir);
        let main = root.join("main");
        git_repository(&main);
        let library = root.join("library");
        git_repository(&library);
        git(&main, &["worktree", "add", "-q", "../wt"]);
        git(
            &main,
            &[
                "-c",
                "protocol.file.allow=always",
                "submodule",
                "add",
                "-q",
                library.to_str().unwrap(),
                "sub",
            ],
        );
        git(&main, &["commit", "-q", "-m", "submodule"]);
        let deep = main.join("sub").join("nested");
        fs::create_dir_all(&deep).unwrap();
        for start in [main.clone(), root.join("wt"), main.join("sub"), deep] {
            assert_eq!(found(&start), git_toplevel(&start), "{}", start.display());
        }
    }

    #[test]
    fn broken_nested_git_entries_inside_a_real_repository_are_errors() {
        for (name, layout) in [
            ("gitdir to nowhere", "gitdir: ../nowhere\n"),
            ("garbage", "garbage\n"),
            ("empty directory", ""),
        ] {
            let dir = guarded();
            let main = root_of(&dir).join("main");
            git_repository(&main);
            let nested = main.join("nested");
            fs::create_dir_all(&nested).unwrap();
            if layout.is_empty() {
                fs::create_dir(nested.join(".git")).unwrap();
            } else {
                fs::write(nested.join(".git"), layout).unwrap();
            }
            assert!(matches!(discover(&nested), Err(Error::Repository { .. })), "{name}");
        }
    }

    #[test]
    fn unlink_keys_prefer_the_canonical_path_then_the_resolved_then_the_literal() {
        let dir = guarded();
        let root = root_of(&dir);
        let repo = root.join("repo");
        fs::create_dir_all(&repo).unwrap();
        assert_eq!(unlink_keys(&root, OsStr::new("repo")), [repo]);

        let cwd = root.join("cwd");
        fs::create_dir_all(&cwd).unwrap();
        assert_eq!(
            unlink_keys(&cwd, OsStr::new("../gone")),
            [root.join("gone"), cwd.join("../gone")]
        );
        assert_eq!(
            unlink_keys(&cwd, OsStr::new("./missing/../gone")),
            [cwd.join("gone"), cwd.join("./missing/../gone")]
        );
    }

    #[test]
    fn unlink_keys_resolve_a_deleted_directory_under_a_non_canonical_ancestor() {
        let dir = guarded();
        let spelled = dir.path().join("sub").join("..").join("gone");
        let keys = unlink_keys(dir.path(), OsStr::new("sub/../gone"));
        assert_eq!(keys[0], root_of(&dir).join("gone"), "{keys:?}");
        assert_eq!(keys.last().unwrap(), &strip_verbatim(&spelled), "{keys:?}");
    }
}
