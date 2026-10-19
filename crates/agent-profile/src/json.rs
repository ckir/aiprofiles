//! The serialisable view layer behind `--json` (SP5b design §3.1 to §3.4).
//!
//! Every document is a struct whose field order is its JSON member order; an absent value is `null`,
//! never a missing member, so a consumer sees the same members on every run.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::config::Config;
use crate::lifecycle::Presence;
use crate::name::AgentId;
use crate::output::is_proper_ancestor;
use crate::repo::Discovery;
use crate::resolve::{Resolution, ResolutionSource};

/// The `tool_version` member of every document.
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Two-space pretty print plus a trailing newline.
pub fn render<T: Serialize>(document: &T) -> String {
    let mut text = serde_json::to_string_pretty(document).expect("view types serialise");
    text.push('\n');
    text
}

/// The closed JSON token for a resolution source (the text label is for humans).
pub fn source_token(source: ResolutionSource) -> &'static str {
    match source {
        ResolutionSource::Explicit => "explicit",
        ResolutionSource::RepositoryAgent => "repository_agent_mapping",
        ResolutionSource::RepositoryDefault => "repository_mapping",
        ResolutionSource::GlobalDefault => "global_default",
        ResolutionSource::None => "none",
    }
}

/// The `resolve` document.
#[derive(Debug, Serialize)]
pub struct ResolveDocument {
    schema: &'static str,
    tool_version: &'static str,
    agent: String,
    profile: Option<String>,
    source: &'static str,
    repository: Option<String>,
}

pub fn resolve(resolution: &Resolution) -> ResolveDocument {
    ResolveDocument {
        schema: "agent-profile/resolve/1",
        tool_version: TOOL_VERSION,
        agent: resolution.agent.to_string(),
        profile: resolution.profile.as_ref().map(ToString::to_string),
        source: source_token(resolution.source),
        repository: resolution.repository.as_ref().map(|path| path.display().to_string()),
    }
}

/// A profile's presence: the token, plus the twin's spelling when the state is `conflict`.
#[derive(Debug, Serialize)]
pub struct PresenceView {
    state: &'static str,
    existing: Option<String>,
}

/// One agent's resolution within the `status` document.
#[derive(Debug, Serialize)]
pub struct AgentResolutionView {
    agent: String,
    profile: Option<String>,
    source: &'static str,
    presence: Option<PresenceView>,
}

/// The mapping entry that applies to the current repository.
#[derive(Debug, Serialize)]
pub struct MappingView {
    profile: Option<String>,
    agents: BTreeMap<String, String>,
}

/// The `status` document.
#[derive(Debug, Serialize)]
pub struct StatusDocument {
    schema: &'static str,
    tool_version: &'static str,
    agent: Option<String>,
    repository: Option<String>,
    mapping: Option<MappingView>,
    default_profile: Option<String>,
    agents: Vec<AgentResolutionView>,
    inapplicable_mappings: Vec<String>,
}

pub fn status(
    agent: Option<&AgentId>,
    discovery: &Discovery,
    config: &Config,
    agents: &[(Resolution, Option<Presence>)],
) -> StatusDocument {
    let (repository, mapping, inapplicable_mappings) = match discovery {
        Discovery::Repository(root) => (
            Some(root.display().to_string()),
            config.mapping(root).map(|mapping| MappingView {
                profile: mapping.profile.as_ref().map(ToString::to_string),
                agents: mapping
                    .agents
                    .iter()
                    .map(|(id, profile)| (id.to_string(), profile.to_string()))
                    .collect(),
            }),
            config
                .mappings()
                .filter(|(key, _)| is_proper_ancestor(key, root))
                .map(|(key, _)| key.display().to_string())
                .collect(),
        ),
        Discovery::NotInRepository => (None, None, Vec::new()),
    };
    StatusDocument {
        schema: "agent-profile/status/1",
        tool_version: TOOL_VERSION,
        agent: agent.map(ToString::to_string),
        repository,
        mapping,
        default_profile: config.default_profile().map(ToString::to_string),
        agents: agents
            .iter()
            .map(|(resolution, presence)| AgentResolutionView {
                agent: resolution.agent.to_string(),
                profile: resolution.profile.as_ref().map(ToString::to_string),
                source: source_token(resolution.source),
                presence: presence.as_ref().map(|presence| PresenceView {
                    state: presence.token(),
                    existing: match presence {
                        Presence::Conflict { existing } => Some(existing.clone()),
                        _ => None,
                    },
                }),
            })
            .collect(),
        inapplicable_mappings,
    }
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use crate::config::AppRoot;
    use crate::name::{Platform, ProfileName};
    use serde_json::Value;

    fn fake() -> AgentId {
        AgentId::parse("fake").unwrap()
    }

    fn profile(name: &str) -> ProfileName {
        ProfileName::parse(name, Platform::host()).unwrap()
    }

    fn parsed(document: &impl Serialize) -> Value {
        serde_json::from_str(&render(document)).unwrap()
    }

    fn keys(value: &Value) -> Vec<&str> {
        value.as_object().unwrap().keys().map(String::as_str).collect()
    }

    #[test]
    fn resolve_document_carries_every_member_with_null_for_absent() {
        let resolution = Resolution {
            agent: fake(),
            profile: None,
            source: ResolutionSource::None,
            repository: None,
        };
        let text = render(&resolve(&resolution));
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["schema"], "agent-profile/resolve/1");
        assert_eq!(value["tool_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(value["agent"], "fake");
        assert_eq!(value["profile"], Value::Null);
        assert_eq!(value["source"], "none");
        assert_eq!(value["repository"], Value::Null);
        assert_eq!(value.as_object().unwrap().len(), 6);
        assert!(
            text.starts_with("{\n  \"schema\": \"agent-profile/resolve/1\",\n  \"tool_version\": ")
        );
        assert!(text.ends_with("}\n"));
    }

    #[test]
    fn source_tokens_are_snake_case_and_closed() {
        assert_eq!(source_token(ResolutionSource::Explicit), "explicit");
        assert_eq!(source_token(ResolutionSource::RepositoryAgent), "repository_agent_mapping");
        assert_eq!(source_token(ResolutionSource::RepositoryDefault), "repository_mapping");
        assert_eq!(source_token(ResolutionSource::GlobalDefault), "global_default");
        assert_eq!(source_token(ResolutionSource::None), "none");
    }

    #[test]
    fn status_document_outside_a_repository_is_all_null_and_empty() {
        let rows = [(
            Resolution {
                agent: fake(),
                profile: None,
                source: ResolutionSource::None,
                repository: None,
            },
            None,
        )];
        let value = parsed(&status(None, &Discovery::NotInRepository, &Config::default(), &rows));
        assert_eq!(value["agent"], Value::Null);
        assert_eq!(value["repository"], Value::Null);
        assert_eq!(value["mapping"], Value::Null);
        assert_eq!(value["default_profile"], Value::Null);
        assert_eq!(value["agents"][0]["presence"], Value::Null);
        assert_eq!(value["inapplicable_mappings"], serde_json::json!([]));
        assert_eq!(
            keys(&value),
            [
                "agent",
                "agents",
                "default_profile",
                "inapplicable_mappings",
                "mapping",
                "repository",
                "schema",
                "tool_version"
            ]
        );
    }

    #[test]
    fn status_document_inside_a_repository_carries_the_mapping_presence_and_ancestors() {
        let temp = tempfile::tempdir().unwrap();
        let outer = temp.path().join("outer");
        let inner = outer.join("inner");
        let root = AppRoot::from_path(temp.path().join("home"));
        std::fs::create_dir_all(root.config_path().parent().unwrap()).unwrap();
        let text = format!(
            "default_profile = \"work\"\n\
             [repositories.'{outer}']\nprofile = \"outer\"\n\
             [repositories.'{inner}']\nprofile = \"inner\"\nagents = {{ fake = \"mine\" }}\n",
            outer = outer.display(),
            inner = inner.display()
        );
        std::fs::write(root.config_path(), text).unwrap();
        let config = Config::load(&root).unwrap();
        let rows = [(
            Resolution {
                agent: fake(),
                profile: Some(profile("mine")),
                source: ResolutionSource::RepositoryAgent,
                repository: Some(inner.clone()),
            },
            Some(Presence::Conflict { existing: "Mine".into() }),
        )];
        let agent = fake();
        let value =
            parsed(&status(Some(&agent), &Discovery::Repository(inner.clone()), &config, &rows));
        assert_eq!(value["agent"], "fake");
        assert_eq!(value["repository"], inner.display().to_string());
        assert_eq!(value["mapping"]["profile"], "inner");
        assert_eq!(value["mapping"]["agents"], serde_json::json!({"fake": "mine"}));
        assert_eq!(value["default_profile"], "work");
        assert_eq!(value["agents"][0]["source"], "repository_agent_mapping");
        assert_eq!(
            value["agents"][0]["presence"],
            serde_json::json!({"state": "conflict", "existing": "Mine"})
        );
        assert_eq!(
            value["inapplicable_mappings"],
            serde_json::json!([outer.display().to_string()])
        );
    }
}
