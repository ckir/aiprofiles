//! The `repositories` report: every stored repository mapping, classified by whether it still applies (SP5b
//! design §4). Classification never fails; a mapping that cannot be examined is reported, not raised.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use crate::adapter;
use crate::config::{AppRoot, Config, is_host_form_key};
use crate::error::Error;
use crate::name::{AgentId, ProfileName};
use crate::repo::{self, Discovery};

/// What became of a stored mapping key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Active,
    Moved,
    NotARepository,
    Missing,
    Unreadable,
    Foreign,
}

impl State {
    pub fn token(self) -> &'static str {
        match self {
            State::Active => "active",
            State::Moved => "moved",
            State::NotARepository => "not_a_repository",
            State::Missing => "missing",
            State::Unreadable => "unreadable",
            State::Foreign => "foreign",
        }
    }
}

/// A profile a mapping names, and whether its directory exists.
pub struct ProfileRef {
    pub name: ProfileName,
    pub exists: bool,
}

/// A per-agent profile a mapping names.
pub struct AgentRef {
    pub profile: ProfileName,
    pub exists: bool,
    pub known: bool,
}

/// One stored mapping and its classification.
pub struct Entry {
    pub key: String,
    pub state: State,
    pub detail: Option<String>,
    pub profile: Option<ProfileRef>,
    pub agents: BTreeMap<AgentId, AgentRef>,
}

/// Classifies a stored key, in the design's order of decision (SP5b design §4).
pub fn classify(key: &str) -> (State, Option<String>) {
    if !is_host_form_key(key) {
        return (State::Foreign, None);
    }
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        let allowed = matches!(
            Path::new(key).components().next(),
            Some(Component::Prefix(prefix)) if matches!(
                prefix.kind(),
                Prefix::Disk(_) | Prefix::VerbatimDisk(_) | Prefix::UNC(..) | Prefix::VerbatimUNC(..)
            )
        );
        if !allowed {
            return (State::Unreadable, Some("device path refused".to_owned()));
        }
    }
    match fs::metadata(key) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => return (State::Unreadable, Some("not a directory".to_owned())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return (State::Missing, None),
        Err(error) => return (State::Unreadable, Some(error.to_string())),
    }
    match repo::discover(Path::new(key)) {
        Err(Error::Repository { reason, .. }) => (State::Unreadable, Some(reason)),
        Err(error) => (State::Unreadable, Some(error.to_string())),
        Ok(Discovery::NotInRepository) => (State::NotARepository, None),
        Ok(Discovery::Repository(root)) if root == Path::new(key) => (State::Active, None),
        Ok(Discovery::Repository(root)) => (State::Moved, Some(root.display().to_string())),
    }
}

/// One entry per stored mapping, in stored-key order. Never fails.
pub fn report(root: &AppRoot, config: &Config) -> Vec<Entry> {
    let profiles = root.profiles_dir();
    let exists = |name: &ProfileName| profiles.join(name.as_str()).is_dir();
    config
        .mappings()
        .map(|(path, mapping)| {
            let key = path.to_string_lossy().into_owned();
            let (state, detail) = classify(&key);
            Entry {
                key,
                state,
                detail,
                profile: mapping
                    .profile
                    .as_ref()
                    .map(|name| ProfileRef { name: name.clone(), exists: exists(name) }),
                agents: mapping
                    .agents
                    .iter()
                    .map(|(id, profile)| {
                        (
                            id.clone(),
                            AgentRef {
                                profile: profile.clone(),
                                exists: exists(profile),
                                known: adapter::lookup(id.as_str()).is_some(),
                            },
                        )
                    })
                    .collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = repo::canonical(dir.path()).unwrap();
        (dir, path)
    }

    fn git_dir(path: &Path) {
        fs::create_dir_all(path.join(".git")).unwrap();
        fs::write(path.join(".git").join("HEAD"), "ref: refs/heads/main\n").unwrap();
    }

    fn s(path: &Path) -> &str {
        path.to_str().unwrap()
    }

    #[test]
    fn classify_orders_foreign_missing_unreadable_not_a_repository_active_and_moved() {
        let (_dir, base) = scratch();
        let acme = base.join("acme");
        git_dir(&acme);
        let sub = acme.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::create_dir_all(base.join("plain")).unwrap();
        fs::write(base.join("file.txt"), "x").unwrap();
        assert_eq!(classify(s(&acme)), (State::Active, None));
        assert_eq!(classify(s(&sub)), (State::Moved, Some(acme.display().to_string())));
        assert_eq!(classify(s(&base.join("plain"))), (State::NotARepository, None));
        assert_eq!(classify(s(&base.join("none"))), (State::Missing, None));
        assert_eq!(
            classify(s(&base.join("file.txt"))),
            (State::Unreadable, Some("not a directory".to_owned()))
        );
        let other = if cfg!(windows) { "/srv/acme" } else { r"C:\src\acme" };
        assert_eq!(classify(other), (State::Foreign, None));
    }

    #[cfg(unix)]
    #[test]
    fn classify_treats_a_dangling_symlink_as_missing_and_a_broken_git_as_unreadable() {
        let (_dir, base) = scratch();
        std::os::unix::fs::symlink(base.join("gone"), base.join("dangling")).unwrap();
        assert_eq!(classify(s(&base.join("dangling"))), (State::Missing, None));
        let broken = base.join("broken");
        fs::create_dir_all(&broken).unwrap();
        fs::write(broken.join(".git"), "gitdir: /nonexistent/x\n").unwrap();
        let (state, detail) = classify(s(&broken));
        assert_eq!(state, State::Unreadable);
        assert!(detail.is_some_and(|reason| !reason.is_empty()));
    }

    #[cfg(windows)]
    #[test]
    fn a_verbatim_disk_key_is_classified_like_a_plain_disk_key() {
        assert_eq!(classify(r"\\?\C:\agent-profile-nonexistent\x"), (State::Missing, None));
    }

    #[cfg(windows)]
    #[test]
    fn classify_refuses_a_device_namespace_key_before_touching_it() {
        assert_eq!(
            classify(r"\\.\pipe\agent-profile-test"),
            (State::Unreadable, Some("device path refused".into()))
        );
    }

    #[test]
    fn report_marks_missing_profile_directories_unknown_agents_and_keeps_stored_order() {
        let (_dir, base) = scratch();
        let acme = base.join("acme");
        git_dir(&acme);
        let zzz = base.join("zzz-missing");
        let home = tempfile::tempdir().unwrap();
        let root = AppRoot::from_path(home.path().to_path_buf());
        fs::write(
            root.config_path(),
            format!(
                "[repositories.'{}']\nprofile = \"work\"\nagents = {{ fake = \"mine\", nosuch = \"work\" }}\n\
                 [repositories.'{}']\nprofile = \"gone\"\n",
                acme.display(),
                zzz.display()
            ),
        )
        .unwrap();
        fs::create_dir_all(root.profiles_dir().join("work").join("fake")).unwrap();
        let config = Config::load(&root).unwrap();
        let entries = report(&root, &config);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, acme.display().to_string());
        assert_eq!(entries[0].state, State::Active);
        let profile = entries[0].profile.as_ref().unwrap();
        assert_eq!((profile.name.as_str(), profile.exists), ("work", true));
        let fake = &entries[0].agents[&AgentId::parse("fake").unwrap()];
        assert_eq!((fake.profile.as_str(), fake.exists, fake.known), ("mine", false, true));
        let nosuch = &entries[0].agents[&AgentId::parse("nosuch").unwrap()];
        assert_eq!((nosuch.profile.as_str(), nosuch.exists, nosuch.known), ("work", true, false));
        assert_eq!(entries[1].state, State::Missing);
        let gone = entries[1].profile.as_ref().unwrap();
        assert_eq!((gone.name.as_str(), gone.exists), ("gone", false));
    }
}
