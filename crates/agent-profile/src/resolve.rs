//! The single profile resolver shared by every command (spec §12, SP3 design §4.3).

use std::path::PathBuf;

use crate::config::Config;
use crate::name::{AgentId, ProfileName};
use crate::repo::Discovery;

/// Where a resolved profile came from (spec §12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionSource {
    Explicit,
    RepositoryAgent,
    RepositoryDefault,
    GlobalDefault,
    None,
}

impl ResolutionSource {
    /// The lower-case label used in human output.
    pub fn label(self) -> &'static str {
        match self {
            ResolutionSource::Explicit => "explicit",
            ResolutionSource::RepositoryAgent => "repository agent mapping",
            ResolutionSource::RepositoryDefault => "repository mapping",
            ResolutionSource::GlobalDefault => "global default",
            ResolutionSource::None => "none",
        }
    }
}

/// The canonical resolution object (spec §12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub agent: AgentId,
    pub profile: Option<ProfileName>,
    pub source: ResolutionSource,
    pub repository: Option<PathBuf>,
}

/// Pure: callers run discovery first. A mapping applies only when its key is component-equal to the
/// discovered root, so a nested repository never inherits its parent's mapping (design D5).
pub fn resolve(
    agent: AgentId,
    explicit: Option<ProfileName>,
    config: &Config,
    discovery: &Discovery,
) -> Resolution {
    let repository = match discovery {
        Discovery::Repository(root) => Some(root.clone()),
        Discovery::NotInRepository => None,
    };
    let mapping = repository.as_deref().and_then(|root| config.mapping(root));
    let (profile, source) = if let Some(profile) = explicit {
        (Some(profile), ResolutionSource::Explicit)
    } else if let Some(profile) = mapping.and_then(|mapping| mapping.agents.get(&agent)) {
        (Some(profile.clone()), ResolutionSource::RepositoryAgent)
    } else if let Some(profile) = mapping.and_then(|mapping| mapping.profile.as_ref()) {
        (Some(profile.clone()), ResolutionSource::RepositoryDefault)
    } else if let Some(profile) = config.default_profile() {
        (Some(profile.clone()), ResolutionSource::GlobalDefault)
    } else {
        (None, ResolutionSource::None)
    };
    Resolution { agent, profile, source, repository }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppRoot;
    use crate::name::Platform;
    use std::path::Path;

    fn profile(name: &str) -> ProfileName {
        ProfileName::parse(name, Platform::host()).unwrap()
    }

    fn agent(id: &str) -> AgentId {
        AgentId::parse(id).unwrap()
    }

    /// `/r` on Unix and `C:\r` on Windows, so the key is absolute on the host.
    fn root(name: &str) -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(format!(r"C:\{name}"))
        } else {
            PathBuf::from(format!("/{name}"))
        }
    }

    fn config(text: &str) -> Config {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.toml"), text).unwrap();
        Config::load(&AppRoot::from_path(dir.path().to_path_buf())).unwrap()
    }

    fn key(path: &Path) -> String {
        toml_edit::Key::new(path.to_str().unwrap()).display_repr().into_owned()
    }

    fn resolved(
        config: &Config,
        explicit: Option<&str>,
        discovery: &Discovery,
        id: &str,
    ) -> (Option<String>, ResolutionSource) {
        let resolution = resolve(agent(id), explicit.map(profile), config, discovery);
        (resolution.profile.map(|profile| profile.to_string()), resolution.source)
    }

    #[test]
    fn each_precedence_step_wins_over_the_ones_below_it() {
        let repo = root("acme");
        let full = config(&format!(
            "default_profile = \"global\"\n[repositories.{}]\nprofile = \"repo\"\nagents = {{ claude = \"agent\" }}\n",
            key(&repo)
        ));
        let inside = Discovery::Repository(repo.clone());
        assert_eq!(
            resolved(&full, Some("explicit"), &inside, "claude"),
            (Some("explicit".to_owned()), ResolutionSource::Explicit)
        );
        assert_eq!(
            resolved(&full, None, &inside, "claude"),
            (Some("agent".to_owned()), ResolutionSource::RepositoryAgent)
        );
        assert_eq!(
            resolved(&full, None, &inside, "codex"),
            (Some("repo".to_owned()), ResolutionSource::RepositoryDefault)
        );
        let agents_only = config(&format!(
            "default_profile = \"global\"\n[repositories.{}]\nagents = {{ claude = \"agent\" }}\n",
            key(&repo)
        ));
        assert_eq!(
            resolved(&agents_only, None, &inside, "codex"),
            (Some("global".to_owned()), ResolutionSource::GlobalDefault)
        );
        assert_eq!(resolved(&config(""), None, &inside, "codex"), (None, ResolutionSource::None));
    }

    #[test]
    fn a_mapping_applies_only_to_its_exact_root() {
        let parent = root("acme");
        let text = format!("[repositories.{}]\nprofile = \"work\"\n", key(&parent));
        let config = config(&text);
        let nested = Discovery::Repository(parent.join("vendor").join("lib"));
        assert_eq!(resolved(&config, None, &nested, "claude"), (None, ResolutionSource::None));
        let trailing = Discovery::Repository(PathBuf::from(format!(
            "{}{}",
            parent.display(),
            std::path::MAIN_SEPARATOR
        )));
        assert_eq!(
            resolved(&config, None, &trailing, "claude"),
            (Some("work".to_owned()), ResolutionSource::RepositoryDefault)
        );
    }

    #[test]
    fn outside_a_repository_only_the_global_default_applies() {
        let text = format!(
            "default_profile = \"global\"\n[repositories.{}]\nprofile = \"work\"\n",
            key(&root("acme"))
        );
        let outside = Discovery::NotInRepository;
        assert_eq!(
            resolved(&config(&text), None, &outside, "claude"),
            (Some("global".to_owned()), ResolutionSource::GlobalDefault)
        );
        let text = format!("[repositories.{}]\nprofile = \"work\"\n", key(&root("acme")));
        assert_eq!(
            resolved(&config(&text), None, &outside, "claude"),
            (None, ResolutionSource::None)
        );
    }

    #[test]
    fn the_repository_is_reported_for_every_source() {
        let repo = root("acme");
        let text = format!(
            "default_profile = \"global\"\n[repositories.{}]\nprofile = \"repo\"\nagents = {{ claude = \"agent\" }}\n",
            key(&repo)
        );
        let full = config(&text);
        let inside = Discovery::Repository(repo.clone());
        let other = Discovery::Repository(root("other"));
        for (explicit, discovery, id, source) in [
            (Some("x"), &inside, "claude", ResolutionSource::Explicit),
            (None, &inside, "claude", ResolutionSource::RepositoryAgent),
            (None, &inside, "codex", ResolutionSource::RepositoryDefault),
            (None, &other, "codex", ResolutionSource::GlobalDefault),
        ] {
            let resolution = resolve(agent(id), explicit.map(profile), &full, discovery);
            assert_eq!(resolution.source, source);
            let expected = match discovery {
                Discovery::Repository(root) => Some(root.clone()),
                Discovery::NotInRepository => None,
            };
            assert_eq!(resolution.repository, expected, "{source:?}");
        }
        let none = resolve(agent("codex"), None, &config(""), &other);
        assert_eq!((none.source, none.repository), (ResolutionSource::None, Some(root("other"))));
        let outside =
            resolve(agent("codex"), Some(profile("x")), &full, &Discovery::NotInRepository);
        assert_eq!(outside.repository, None);
    }
}
