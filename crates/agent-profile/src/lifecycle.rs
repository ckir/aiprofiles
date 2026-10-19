//! The profile lifecycle: materializing and deleting what an adapter owns for a profile (SP5a design §3).
//!
//! Like `config::link` and `config::unlink`, the logic is a pure function over the application root that
//! returns an outcome value; the CLI layer renders it and decides the exit code.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::adapter::{self, Adapter, ProfilePath, ProfilePresence};
use crate::config::{self, AppRoot, Config, check_case_twins};
use crate::error::{Error, Result};
use crate::name::{AgentId, ProfileName};

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

/// Text for one blocking reference: the stored key for a repository-wide `profile` field, the stored key
/// with the agent table for an `agents.<id>` field, and the literal `default_profile` for the global
/// default (SP5a design §3).
fn blocking_label(agent: Option<&AgentId>, root: &Path) -> String {
    match agent {
        None => root.display().to_string(),
        Some(agent) => format!("{} (agents.{agent})", root.display()),
    }
}

/// Every mapping field and the global default that names `profile` for `agent`, ignoring ASCII case, in
/// the order the mapping fields are read and with `default_profile` last. An empty result means nothing
/// stands in the way of deleting the profile's directory for `agent`.
pub fn blocking_references(config: &Config, agent: &AgentId, profile: &ProfileName) -> Vec<String> {
    let mut references: Vec<String> = config
        .mappings_referencing(profile)
        .into_iter()
        // A repository-wide mapping blocks every agent; an agent field blocks only that agent, so
        // another agent's mapping never blocks this one.
        .filter(|reference| reference.agent.is_none() || reference.agent.as_ref() == Some(agent))
        .map(|reference| blocking_label(reference.agent.as_ref(), &reference.root))
        .collect();
    if config
        .default_profile()
        .is_some_and(|name| name.as_str().eq_ignore_ascii_case(profile.as_str()))
    {
        references.push("default_profile".to_owned());
    }
    references
}

/// The profile directory `delete` would remove: `<root>/profiles/<P>/<agent>`, shown to the user
/// un-canonicalized (SP5a design §3). `None` when nothing is there. `Error::ProfileReferenced` is not
/// consulted here; `blocking_references` is the caller's advisory or locked check.
///
/// Safety is structural (SP5a design §3): the path is always inside `<root>/profiles` by construction, and
/// every component below the root is checked so a symlink or a containment escape is refused before any
/// removal, whatever the caller plans to `remove_dir_all`.
pub fn delete_target(
    root: &AppRoot,
    agent: &AgentId,
    profile: &ProfileName,
) -> Result<Option<PathBuf>> {
    check_case_twins(root, profile)?;
    let parent = root.profiles_dir().join(profile.as_str());
    let target = parent.join(agent.as_str());
    // Existence decides first (SP5a design §3.2): a missing entry branches straight to the idempotent
    // no-directory outcome (`Ok(None)`) and the containment refusals below never run, so the no-entry
    // result stays exit 0 by design even when a parent above the target is a symlink.
    let Ok(metadata) = fs::symlink_metadata(&target) else {
        return Ok(None); // not there, so nothing to delete
    };
    // `symlink_metadata` does not follow the final path component, so the `is_symlink` test below sees
    // the entry itself and not its target: the rustdoc of `FileType::is_symlink` ties it to
    // `symlink_metadata` for exactly this reason, because `fs::metadata` follows links and would always
    // answer false. A Windows junction carries the reparse-point attribute, so `is_symlink` is true for
    // a junction too and this one test refuses both spellings.
    if metadata.is_symlink() {
        return Err(symlink_refusal(target));
    }
    // The parent one level above the target is refused after the target's own entry: `symlink_metadata`
    // lstats the final component only, so a symlinked parent never changes the final component's
    // identity and the no-entry branch above stays reachable.
    if symlinked(&parent)? {
        return Err(symlink_refusal(parent));
    }
    if !metadata.is_dir() {
        return Err(Error::ProfileDir {
            path: target,
            source: io::Error::other("is not a directory"),
        });
    }
    let resolved_profiles = fs::canonicalize(root.profiles_dir())
        .map_err(|source| Error::ProfileDir { path: root.profiles_dir(), source })?;
    let resolved = fs::canonicalize(&target)
        .map_err(|source| Error::ProfileDir { path: target.clone(), source })?;
    if !resolved.starts_with(&resolved_profiles) {
        return Err(Error::ProfileDir {
            path: target,
            source: io::Error::other("resolves outside the profiles directory"),
        });
    }
    Ok(Some(target))
}

/// Whether the entry at `path` is itself a symlink (or, on Windows, a junction). `Ok(false)` when the
/// entry is missing, so the caller's symlink check never fires on `NotFound`.
fn symlinked(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.is_symlink()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(Error::ProfileDir { path: path.to_path_buf(), source }),
    }
}

/// The refusal for a symlinked entry, with the fixed message the tests anchor on.
fn symlink_refusal(path: PathBuf) -> Error {
    Error::ProfileDir {
        path,
        source: io::Error::other("is a symbolic link or junction; refusing to delete through it"),
    }
}

/// What `delete` did (SP5a design §3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteOutcome {
    /// The profile directory was removed; `path` is the un-canonicalized directory, `parent_note` a
    /// non-fatal problem with removing the now-empty profile directory above it.
    Deleted { path: PathBuf, parent_note: Option<String> },
    /// Nothing was there by the time the locked delete ran: the profile vanished between the advisory
    /// check and the lock, which is the state the user asked for, not an error.
    Vanished { path: PathBuf },
}

/// Removes the profile directory for `agent` under the same exclusive lock `link` and `update` take
/// (spec §18), re-checking `delete_target` and `blocking_references` inside it: the advisory checks outside
/// the lock can be seconds stale, and a deletion that never leaves a mapping or the global default
/// pointing at nothing is the invariant this function exists to hold (SP5a design §3).
pub fn delete(root: &AppRoot, agent: &AgentId, profile: &ProfileName) -> Result<DeleteOutcome> {
    config::locked(root, |config| {
        let target = root.profiles_dir().join(profile.as_str()).join(agent.as_str());
        let Some(target) = delete_target(root, agent, profile)? else {
            return Ok(DeleteOutcome::Vanished { path: target });
        };
        let references = blocking_references(config, agent, profile);
        if !references.is_empty() {
            return Err(Error::ProfileReferenced {
                agent: agent.to_string(),
                profile: profile.to_string(),
                references,
            });
        }
        fs::remove_dir_all(&target).map_err(|error| {
            Error::ProfileDir {
                path: target.clone(),
                // A failed removal may leave any suffix of the tree removed, so the raw error is
                // re-worded to send the user back here instead of leaving the root half-deleted and quiet.
                source: io::Error::other(format!(
                    "{error}; the profile may be partially removed, retry the deletion"
                )),
            }
        })?;
        // The profile directories above the target turn empty as agents' profiles are deleted one by one;
        // an empty one is swept. A failure to sweep is a note, not an error: the profile itself is gone.
        let parent = target.parent().expect("the target is a profile directory below its parent");
        let mut parent_note = None;
        if fs::read_dir(parent).is_ok_and(|mut entries| entries.next().is_none())
            && let Err(error) = fs::remove_dir(parent)
        {
            parent_note =
                Some(format!("could not remove empty directory {}: {error}", parent.display()));
        }
        Ok(DeleteOutcome::Deleted { path: target, parent_note })
    })
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
