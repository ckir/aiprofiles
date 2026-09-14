//! The single profile resolver shared by every command (spec §12). SP1 resolves only an explicit
//! profile; SP3 replaces the body of `resolve`, not the types.

use std::path::PathBuf;

use crate::name::{AgentId, ProfileName};

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

/// SP1 stub: an explicit profile resolves as `Explicit`; anything else resolves to `None`.
pub fn resolve(agent: AgentId, explicit: Option<ProfileName>) -> Resolution {
    let source =
        if explicit.is_some() { ResolutionSource::Explicit } else { ResolutionSource::None };
    Resolution { agent, profile: explicit, source, repository: None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::name::Platform;

    #[test]
    fn explicit_profile_resolves_as_explicit() {
        let agent = AgentId::parse("fake").unwrap();
        let work = ProfileName::parse("work", Platform::host()).unwrap();
        let resolution = resolve(agent.clone(), Some(work.clone()));
        assert_eq!(
            resolution,
            Resolution {
                agent: agent.clone(),
                profile: Some(work),
                source: ResolutionSource::Explicit,
                repository: None
            }
        );
        let none = resolve(agent.clone(), None);
        assert_eq!(none.source, ResolutionSource::None);
        assert_eq!(none.profile, None);
    }
}
