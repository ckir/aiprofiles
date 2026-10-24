// Copyright (c) 2026 Costas Kirgoussios
//
// This software is licensed under the PolyForm Noncommercial License 1.0.0.
// You may obtain a copy of the License at:
// https://polyformproject.org/licenses/noncommercial/1.0.0/

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
use crate::name::{AgentId, Platform, ProfileName};

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

/// What `list` and `profiles` show (SP5a design §3): one row per valid profile directory, plus the count
/// of entries that were skipped as unrecognized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    /// One row per profile directory, sorted by profile name: the profile and the ids of the agents
    /// materialized for it, in the given `adapters` order. A row may list no agent.
    pub rows: Vec<(ProfileName, Vec<String>)>,
    /// Entries of the profiles directory and child directories under a listed profile that were not part
    /// of any row: non-directories, invalid names and unknown agent directories.
    pub ignored: usize,
}

/// Reads what exists on disk, human-readable: each valid profile directory under `<root>/profiles`
/// becomes a row naming every `adapter` that is materialized for it, in `adapters` order. Purely
/// read-only: nothing is created, and a missing profiles directory is simply an empty listing.
///
/// Entries whose name starts with `.` are skipped and not counted. Anything else that is not a
/// directory, not valid UTF-8 or not a valid profile name for the host platform is an unrecognized
/// entry, as is any directory below a listed profile that is not a known agent id.
pub fn list_profiles(root: &AppRoot, adapters: &[&dyn Adapter]) -> Result<Listing> {
    let io_context = "could not read the profiles directory".to_owned();
    let entries = match fs::read_dir(root.profiles_dir()) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(Listing { rows: Vec::new(), ignored: 0 });
        }
        Err(source) => return Err(Error::Io { context: io_context, source }),
    };
    let mut rows: Vec<(ProfileName, Vec<String>)> = Vec::new();
    let mut ignored = 0usize;
    for entry in entries {
        let entry = entry.map_err(|source| Error::Io { context: io_context.clone(), source })?;
        let name = entry.file_name();
        // A name starting with `.` is an editor or OS dropping (*.swp, .DS_Store): it is not part of any
        // row, and the user did not put it in a profile either, so it is not counted at all.
        if name.as_encoded_bytes().starts_with(b".") {
            continue;
        }
        // `file_type` does not follow symlinks, so a symlinked profile entry counts as not a directory.
        if !entry
            .file_type()
            .map_err(|source| Error::Io { context: io_context.clone(), source })?
            .is_dir()
        {
            ignored += 1;
            continue;
        }
        let profile = match name
            .to_str()
            .map(|name| ProfileName::parse(name, Platform::host()))
            .transpose()
        {
            Ok(Some(profile)) => profile,
            // Not valid UTF-8 (`Ok(None)`) or a reserved or otherwise invalid name, e.g. the command
            // words: the user did not create it through this tool, so count and move on.
            _ => {
                ignored += 1;
                continue;
            }
        };
        // `presence` reads the filesystem only and never writes (SP5a design §3); a bare directory with
        // no config file is not materialized for a file adapter.
        let agents: Vec<String> = adapters
            .iter()
            .filter(|adapter| adapter.presence(root, &profile) == ProfilePresence::Materialized)
            .map(|adapter| adapter.metadata().id.to_owned())
            .collect();
        let directory = root.profiles_dir().join(profile.as_str());
        // An OS-refused read one level down names the profile directory that failed, not the profiles
        // directory above it, so a user with many profiles knows which one to look at.
        let child_context = format!("could not read the profile directory {}", directory.display());
        ignored += unrecognized_children(&directory, adapters, &child_context)?;
        rows.push((profile, agents));
    }
    // `str`'s `Ord` is byte order, so rows sort by profile name bytes.
    rows.sort_by(|(a, _), (b, _)| a.as_str().as_bytes().cmp(b.as_str().as_bytes()));
    Ok(Listing { rows, ignored })
}

/// The number of child directories of a listed profile directory that are not known agent ids: each one
/// is an unrecognized entry and the caller's `ignored` grows by one per child. A child file is never
/// counted — only child directories can be unrecognized — and a child entry whose name starts with `.`
/// is skipped without counting, mirroring the profiles-directory rule: the same OS litter (*.swp,
/// .DS_Store) must behave the same one level deeper.
fn unrecognized_children(
    directory: &Path,
    adapters: &[&dyn Adapter],
    io_context: &str,
) -> Result<usize> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        // Gone between listing the profile and reading it: nothing unrecognized under a vanished dir.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(source) => return Err(Error::Io { context: io_context.to_owned(), source }),
    };
    let mut count = 0usize;
    for entry in entries {
        let entry = entry.map_err(|source| Error::Io { context: io_context.to_owned(), source })?;
        if entry.file_name().as_encoded_bytes().starts_with(b".") {
            continue;
        }
        // lstat-based, as at the profiles-directory level: a symlink child is not a directory, so it is
        // simply not counted — no refusal.
        if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        if !adapters.iter().any(|adapter| entry.file_name().to_str() == Some(adapter.metadata().id))
        {
            count += 1;
        }
    }
    Ok(count)
}

/// What `<agent> status` and the `status` document report for a profile (SP3 design §7.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Presence {
    Materialized,
    Known,
    Absent,
    /// A profile whose name differs only by case exists; `existing` is its spelling.
    Conflict {
        existing: String,
    },
}

impl Presence {
    /// The human label printed by `status`.
    pub fn label(&self) -> String {
        match self {
            Presence::Materialized => "materialized".to_owned(),
            Presence::Known => "known".to_owned(),
            Presence::Absent => "absent".to_owned(),
            Presence::Conflict { existing } => format!("conflicts with {existing}"),
        }
    }

    /// The closed token used in JSON documents.
    pub fn token(&self) -> &'static str {
        match self {
            Presence::Materialized => "materialized",
            Presence::Known => "known",
            Presence::Absent => "absent",
            Presence::Conflict { .. } => "conflict",
        }
    }
}

/// The presence of `profile` for `agent`; a case-only twin is reported before the adapter is asked.
pub fn presence(root: &AppRoot, agent: &AgentId, profile: &ProfileName) -> Presence {
    if let Err(Error::ProfileCaseConflict { existing, .. }) = check_case_twins(root, profile) {
        return Presence::Conflict { existing };
    }
    let adapter = adapter::lookup(agent.as_str()).expect("known agents have an adapter");
    match adapter.presence(root, profile) {
        ProfilePresence::Materialized => Presence::Materialized,
        ProfilePresence::Known => Presence::Known,
        ProfilePresence::Absent => Presence::Absent,
    }
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use crate::adapter::Fake;
    use crate::name::Platform;

    fn profile(name: &str) -> ProfileName {
        ProfileName::parse(name, Platform::host()).unwrap()
    }

    #[test]
    fn presence_labels_and_tokens_cover_every_variant() {
        let cases = [
            (Presence::Materialized, "materialized", "materialized"),
            (Presence::Known, "known", "known"),
            (Presence::Absent, "absent", "absent"),
            (Presence::Conflict { existing: "Mine".into() }, "conflicts with Mine", "conflict"),
        ];
        for (presence, label, token) in cases {
            assert_eq!(presence.label(), label);
            assert_eq!(presence.token(), token);
        }
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
