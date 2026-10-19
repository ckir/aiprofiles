//! The profile lifecycle: materializing what an adapter owns for a profile (SP5a design §3).
//!
//! Like `config::link` and `config::unlink`, the logic is a pure function over the application root that
//! returns an outcome value; the CLI layer renders it and decides the exit code.

use std::path::PathBuf;

use crate::adapter::{self, Adapter, ProfilePath, ProfilePresence};
use crate::config::{AppRoot, check_case_twins};
use crate::error::Result;
use crate::name::ProfileName;

/// What `create` did (SP5a design §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateOutcome {
    /// Every path the adapter owns was ensured; a missing one was created.
    Created,
    /// Every path already existed, so nothing was written.
    AlreadyExists,
}

/// Ensures the paths `adapter` owns for `profile` under `root` exactly once: an already materialized
/// profile is left untouched, so a user's hand-edited configuration file is never overwritten. The
/// returned path is the first of the adapter's paths, the profile directory; the function is the same
/// kind of pure, outcome-returning function as `config::link`.
///
/// The outcome is decided from `presence` *before* anything is written, so it reads the state the user
/// sees, not the state this call would leave behind.
pub fn create(
    root: &AppRoot,
    adapter: &dyn Adapter,
    profile: &ProfileName,
) -> Result<(CreateOutcome, PathBuf)> {
    check_case_twins(root, profile)?;
    let paths: Vec<ProfilePath> = adapter
        .paths(root, profile)
        .into_iter()
        .map(|(path, kind)| ProfilePath { path, kind, existed: false })
        .collect();
    let outcome = if adapter.presence(root, profile) == ProfilePresence::Materialized {
        CreateOutcome::AlreadyExists
    } else {
        adapter::ensure_paths(&paths)?;
        CreateOutcome::Created
    };
    Ok((outcome, paths[0].path.clone()))
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use crate::adapter::Fake;
    use crate::name::Platform;

    fn profile(name: &str) -> ProfileName {
        ProfileName::parse(name, Platform::host()).unwrap()
    }

    /// The outcome reads presence before any write: absent → `Created`, materialized → `AlreadyExists`,
    /// and a half-written profile — the parent directory present, a declared path missing — is not
    /// materialized, so it is completed and `Created`.
    #[test]
    fn create_outcome_reads_presence_before_anything_is_written() {
        let dir = tempfile::tempdir().unwrap();
        let root = AppRoot::from_path(dir.path().to_path_buf());
        let work = profile("work");
        let expected = root.profiles_dir().join("work").join("fake");

        let (outcome, path) = create(&root, &Fake, &work).unwrap();
        assert_eq!(outcome, CreateOutcome::Created);
        assert_eq!(path, expected);
        assert!(path.is_dir());

        let (outcome, path) = create(&root, &Fake, &work).unwrap();
        assert_eq!(outcome, CreateOutcome::AlreadyExists);
        assert_eq!(path, expected);

        let half = profile("half");
        std::fs::create_dir_all(root.profiles_dir().join("half")).unwrap();
        let (outcome, path) = create(&root, &Fake, &half).unwrap();
        assert_eq!(outcome, CreateOutcome::Created);
        assert_eq!(path, root.profiles_dir().join("half").join("fake"));
        assert!(path.is_dir());
    }
}
