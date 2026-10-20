//! The serialisable view layer behind `--json` (SP5b design §3.1 to §3.7).
//!
//! Every document is a struct whose field order is its JSON member order; an absent value is `null`,
//! never a missing member, so a consumer sees the same members on every run.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::adapter::{AdapterMetadata, Capability, CapabilityState, SupportLevel};
use crate::config::Config;
use crate::exe::Origin;
use crate::lifecycle::{Listing, Presence};
use crate::name::AgentId;
use crate::output::{ExecutableStatus, is_proper_ancestor};
use crate::repo::Discovery;
use crate::repositories::Entry;
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

/// The closed JSON token for a capability state.
pub fn state_token(state: CapabilityState) -> &'static str {
    match state {
        CapabilityState::Supported => "supported",
        CapabilityState::NotSupported => "not_supported",
        CapabilityState::NotGuaranteed => "not_guaranteed",
        CapabilityState::Conditional => "conditional",
        CapabilityState::Unknown => "unknown",
    }
}

/// The closed JSON token for a capability.
pub fn capability_token(capability: Capability) -> &'static str {
    match capability {
        Capability::ConfigIsolation => "config",
        Capability::CredentialIsolation => "credentials",
        Capability::StateIsolation => "state",
    }
}

/// The closed JSON token for a support level.
pub fn support_token(support: SupportLevel) -> &'static str {
    match support {
        SupportLevel::Proven => "proven",
        SupportLevel::Experimental => "experimental",
    }
}

/// An agent's executable evidence.
#[derive(Debug, Serialize)]
pub struct ExecutableView {
    state: &'static str,
    path: Option<String>,
    origin: Option<&'static str>,
}

/// How the agent is pointed at the profile.
#[derive(Debug, Serialize)]
pub struct MechanismView {
    id: &'static str,
    description: String,
}

/// The evidence the adapter's claims rest on.
#[derive(Debug, Serialize)]
pub struct EvidenceView {
    upstream_version: &'static str,
    verified_at: &'static str,
    source_url: &'static str,
    notes: &'static str,
}

/// One capability claim.
#[derive(Debug, Serialize)]
pub struct CapabilityView {
    capability: &'static str,
    state: &'static str,
    basis: &'static str,
}

/// One registry entry of the `agents` document.
#[derive(Debug, Serialize)]
pub struct AgentView {
    agent: &'static str,
    executable: ExecutableView,
    mechanism: MechanismView,
    support: &'static str,
    evidence: EvidenceView,
    capabilities: Vec<CapabilityView>,
}

/// The `agents` document.
#[derive(Debug, Serialize)]
pub struct AgentsDocument {
    schema: &'static str,
    tool_version: &'static str,
    agents: Vec<AgentView>,
}

/// The JSON view of one executable status.
pub fn executable_view(status: &ExecutableStatus) -> ExecutableView {
    match status {
        ExecutableStatus::Found(found) => ExecutableView {
            state: "found",
            path: Some(found.path.display().to_string()),
            origin: Some(match found.origin {
                Origin::Configured => "configured",
                Origin::Path => "path",
            }),
        },
        ExecutableStatus::NotOnPath => {
            ExecutableView { state: "not_on_path", path: None, origin: None }
        }
        ExecutableStatus::ConfiguredMissing(path) => ExecutableView {
            state: "configured_missing",
            path: Some(path.display().to_string()),
            origin: None,
        },
        ExecutableStatus::Unsupported(path) => ExecutableView {
            state: "unsupported",
            path: Some(path.display().to_string()),
            origin: None,
        },
    }
}

/// The JSON view of one registry entry and its executable status.
pub fn agent_view(metadata: &AdapterMetadata, status: &ExecutableStatus) -> AgentView {
    let capabilities = Capability::ALL
        .into_iter()
        .map(|capability| {
            let claim = metadata
                .capabilities
                .iter()
                .find(|claim| claim.capability == capability)
                .expect("registry adapters declare every capability");
            CapabilityView {
                capability: capability_token(capability),
                state: state_token(claim.state),
                basis: claim.basis,
            }
        })
        .collect();
    AgentView {
        agent: metadata.id,
        executable: executable_view(status),
        mechanism: MechanismView {
            id: metadata.evidence.mechanism_id,
            description: metadata.mechanism.to_string(),
        },
        support: support_token(metadata.support),
        evidence: EvidenceView {
            upstream_version: metadata.evidence.upstream_version,
            verified_at: metadata.evidence.verified_at,
            source_url: metadata.evidence.source_url,
            notes: metadata.evidence.notes,
        },
        capabilities,
    }
}

/// Builds the `agents` document from registry metadata and each agent's executable status.
pub fn agents(entries: &[(&AdapterMetadata, ExecutableStatus)]) -> AgentsDocument {
    let agents = entries.iter().map(|(metadata, status)| agent_view(metadata, status)).collect();
    AgentsDocument { schema: "agent-profile/agents/1", tool_version: TOOL_VERSION, agents }
}

/// One profile and the agents materialized for it.
#[derive(Debug, Serialize)]
pub struct ProfileView {
    name: String,
    agents: Vec<String>,
}

/// The `profiles` (and `list`) document.
#[derive(Debug, Serialize)]
pub struct ProfilesDocument {
    schema: &'static str,
    tool_version: &'static str,
    agent: Option<String>,
    profiles: Vec<ProfileView>,
    ignored_entries: usize,
}

impl ProfilesDocument {
    /// Whether no profile row survived the agent filter.
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
}

/// Builds the `profiles` document; with an agent, only rows that materialize it, each row unchanged.
pub fn profiles(listing: &Listing, agent: Option<&AgentId>) -> ProfilesDocument {
    ProfilesDocument {
        schema: "agent-profile/profiles/1",
        tool_version: TOOL_VERSION,
        agent: agent.map(ToString::to_string),
        profiles: listing
            .rows
            .iter()
            .filter(|(_, ids)| agent.is_none_or(|agent| ids.iter().any(|id| id == agent.as_str())))
            .map(|(name, ids)| ProfileView { name: name.to_string(), agents: ids.clone() })
            .collect(),
        ignored_entries: listing.ignored,
    }
}

/// A profile a repository entry names, and whether its directory exists.
#[derive(Debug, Serialize)]
pub struct ProfileRefView {
    name: String,
    exists: bool,
}

/// An agent field of a repository entry.
#[derive(Debug, Serialize)]
pub struct AgentRefView {
    profile: String,
    exists: bool,
    known_agent: bool,
}

/// One stored repository mapping with its classification.
#[derive(Debug, Serialize)]
pub struct RepositoryView {
    repository: String,
    state: &'static str,
    detail: Option<String>,
    profile: Option<ProfileRefView>,
    agents: BTreeMap<String, AgentRefView>,
}

/// The `repositories` document.
#[derive(Debug, Serialize)]
pub struct RepositoriesDocument {
    schema: &'static str,
    tool_version: &'static str,
    repositories: Vec<RepositoryView>,
}

/// The JSON view of one classified repository entry.
pub fn repository_view(entry: &Entry) -> RepositoryView {
    RepositoryView {
        repository: entry.key.clone(),
        state: entry.state.token(),
        detail: entry.detail.clone(),
        profile: entry.profile.as_ref().map(|profile| ProfileRefView {
            name: profile.name.to_string(),
            exists: profile.exists,
        }),
        agents: entry
            .agents
            .iter()
            .map(|(id, agent)| {
                (
                    id.as_str().to_owned(),
                    AgentRefView {
                        profile: agent.profile.to_string(),
                        exists: agent.exists,
                        known_agent: agent.known,
                    },
                )
            })
            .collect(),
    }
}

/// Builds the `repositories` document; the entries arrive sorted and keep their order.
pub fn repositories(entries: &[Entry]) -> RepositoriesDocument {
    RepositoriesDocument {
        schema: "agent-profile/repositories/1",
        tool_version: TOOL_VERSION,
        repositories: entries.iter().map(repository_view).collect(),
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
    fn state_tokens_are_snake_case_and_closed() {
        assert_eq!(state_token(CapabilityState::Supported), "supported");
        assert_eq!(state_token(CapabilityState::NotSupported), "not_supported");
        assert_eq!(state_token(CapabilityState::NotGuaranteed), "not_guaranteed");
        assert_eq!(state_token(CapabilityState::Conditional), "conditional");
        assert_eq!(state_token(CapabilityState::Unknown), "unknown");
        assert_eq!(capability_token(Capability::ConfigIsolation), "config");
        assert_eq!(capability_token(Capability::CredentialIsolation), "credentials");
        assert_eq!(capability_token(Capability::StateIsolation), "state");
        assert_eq!(support_token(SupportLevel::Proven), "proven");
        assert_eq!(support_token(SupportLevel::Experimental), "experimental");
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

    #[test]
    fn agents_document_renders_each_executable_state_and_the_full_matrix() {
        use crate::adapter::{self, Capability};
        use crate::exe::{Found, Origin};
        use crate::output::ExecutableStatus;
        use std::path::PathBuf;

        let metadata = adapter::lookup("fake").unwrap().metadata();
        let entries = [
            (
                metadata,
                ExecutableStatus::Found(Found {
                    path: PathBuf::from("/bin/fake"),
                    origin: Origin::Configured,
                }),
            ),
            (
                metadata,
                ExecutableStatus::Found(Found {
                    path: PathBuf::from("/usr/bin/fake"),
                    origin: Origin::Path,
                }),
            ),
            (metadata, ExecutableStatus::NotOnPath),
            (metadata, ExecutableStatus::ConfiguredMissing(PathBuf::from("/gone"))),
            (metadata, ExecutableStatus::Unsupported(PathBuf::from("/shim.cmd"))),
        ];
        let value = parsed(&agents(&entries));
        assert_eq!(value["schema"], "agent-profile/agents/1");
        let list = value["agents"].as_array().unwrap();
        assert_eq!(list.len(), 5);
        let expected = [
            ("found", Some("/bin/fake"), Some("configured")),
            ("found", Some("/usr/bin/fake"), Some("path")),
            ("not_on_path", None, None),
            ("configured_missing", Some("/gone"), None),
            ("unsupported", Some("/shim.cmd"), None),
        ];
        for (element, (state, path, origin)) in list.iter().zip(expected) {
            let executable = &element["executable"];
            assert_eq!(executable["state"], state);
            assert_eq!(executable["path"], serde_json::json!(path));
            assert_eq!(executable["origin"], serde_json::json!(origin));
            assert_eq!(keys(executable), ["origin", "path", "state"]);
            let capabilities = element["capabilities"].as_array().unwrap();
            assert_eq!(capabilities.len(), 3);
            for (entry, capability) in capabilities.iter().zip(Capability::ALL) {
                assert_eq!(entry["capability"], capability_token(capability));
            }
            let order: Vec<&str> =
                capabilities.iter().map(|c| c["capability"].as_str().unwrap()).collect();
            assert_eq!(order, ["config", "credentials", "state"]);
            assert!(["proven", "experimental"].contains(&element["support"].as_str().unwrap()));
            assert_eq!(element["mechanism"]["id"], metadata.evidence.mechanism_id);
            assert_eq!(
                keys(&element["evidence"]),
                ["notes", "source_url", "upstream_version", "verified_at"]
            );
            assert_eq!(
                keys(element),
                ["agent", "capabilities", "evidence", "executable", "mechanism", "support"]
            );
        }
    }

    #[test]
    fn profiles_document_filters_by_agent_without_changing_rows() {
        use crate::lifecycle::Listing;
        let listing = Listing {
            rows: vec![
                (profile("personal"), vec!["claude".to_owned()]),
                (profile("work"), vec!["claude".to_owned(), "codex".to_owned()]),
            ],
            ignored: 2,
        };
        let all = parsed(&profiles(&listing, None));
        assert_eq!(all["schema"], "agent-profile/profiles/1");
        assert_eq!(all["agent"], Value::Null);
        assert_eq!(all["ignored_entries"], 2);
        assert_eq!(
            all["profiles"],
            serde_json::json!([
                {"name": "personal", "agents": ["claude"]},
                {"name": "work", "agents": ["claude", "codex"]}
            ])
        );
        let codex = AgentId::parse("codex").unwrap();
        let some = parsed(&profiles(&listing, Some(&codex)));
        assert_eq!(some["agent"], "codex");
        assert_eq!(
            some["profiles"],
            serde_json::json!([{"name": "work", "agents": ["claude", "codex"]}])
        );
    }
}
