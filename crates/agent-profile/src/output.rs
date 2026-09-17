//! Human output: the dry-run and `--verbose` report (spec §26, SP1 design §7.4, SP2 design §7.3), the
//! repository command reports (SP3 design §7.5) and redaction (spec §22). JSON output (spec §32) arrives in
//! SP5.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::adapter::{
    AdapterMetadata, Capability, CapabilityState, PathKind, PlannedLaunch, SupportLevel,
};
use crate::config::{Config, LinkOutcome, UnlinkOutcome};
use crate::exe::Origin;
use crate::name::{AgentId, ProfileName};
use crate::repo::Discovery;
use crate::resolve::Resolution;

/// Substrings that mark an override variable as secret-bearing, whatever the adapter declared.
const SENSITIVE_NAME_PARTS: [&str; 6] =
    ["TOKEN", "SECRET", "KEY", "PASSWORD", "CREDENTIAL", "AUTH"];

const LABEL_WIDTH: usize = 14;

/// Which report is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportMode {
    /// Before anything is created: marks and lists what a launch would create.
    DryRun,
    /// After initialization: nothing is left to create, so there are no markers and no `creates:` line.
    Verbose,
}

/// How a support level is spelled in the report and in the launch hedge (SP4 design §7.1).
pub fn support_label(support: SupportLevel) -> &'static str {
    match support {
        SupportLevel::Proven => "proven",
        SupportLevel::Experimental => "experimental",
    }
}

/// How a capability is spelled. One vocabulary covers the report and the hedge (SP4 design §7.2).
pub fn capability_label(capability: Capability) -> &'static str {
    match capability {
        Capability::ConfigIsolation => "config",
        Capability::CredentialIsolation => "credentials",
        Capability::StateIsolation => "state",
    }
}

/// How a capability state is spelled: lower case, with spaces (SP4 design §7.1).
pub fn state_label(state: CapabilityState) -> &'static str {
    match state {
        CapabilityState::Supported => "supported",
        CapabilityState::NotSupported => "not supported",
        CapabilityState::NotGuaranteed => "not guaranteed",
        CapabilityState::Conditional => "conditional",
        CapabilityState::Unknown => "unknown",
    }
}

/// The one-line stderr hedge for an adapter that is not `Proven`, or `None` when it is (SP4 design §7.2).
///
/// It lists every capability whose state is not `Supported`, in `Capability::ALL` order, using the
/// report's own vocabulary. `NotGuaranteed` counts: leaving it out would have hidden the state the design
/// predicts for an adapter whose documented mechanism is ignored by part of the agent.
pub fn support_hedge(metadata: &AdapterMetadata) -> Option<String> {
    if metadata.support == SupportLevel::Proven {
        return None;
    }
    let support = support_label(metadata.support);
    let weak: Vec<String> = Capability::ALL
        .into_iter()
        .filter_map(|capability| {
            match metadata.capabilities.iter().find(|claim| claim.capability == capability) {
                None => Some(format!("{} not declared", capability_label(capability))),
                Some(claim) if claim.state != CapabilityState::Supported => {
                    Some(format!("{} {}", capability_label(capability), state_label(claim.state)))
                }
                Some(_) => None,
            }
        })
        .collect();
    let detail = if weak.is_empty() { String::new() } else { format!(": {}", weak.join(", ")) };
    Some(format!("{} is {support}{detail}. Run with --dry-run for detail.", metadata.id))
}

/// The report lines, without trailing newlines.
pub fn report_lines(
    planned: &PlannedLaunch,
    resolution: &Resolution,
    mode: ReportMode,
    metadata: &AdapterMetadata,
) -> Vec<String> {
    let mut lines = Vec::new();
    let line = |label: &str, value: String| format!("{:<LABEL_WIDTH$}{value}", format!("{label}:"));
    let continuation = |value: String| format!("{:LABEL_WIDTH$}{value}", "");
    lines.push(line("agent", resolution.agent.to_string()));
    lines.push(line("profile", format!("{} ({})", planned.profile, resolution.source.label())));
    let origin = match planned.executable_origin {
        Origin::Configured => "configured",
        Origin::Path => "PATH",
    };
    lines.push(line("executable", format!("{} ({origin})", planned.plan.executable.display())));
    let repository = match &resolution.repository {
        Some(path) => path.display().to_string(),
        None => "none".to_owned(),
    };
    lines.push(line("repository", repository));
    lines.push(line("mechanism", planned.mechanism.clone()));
    lines.push(line("support", support_label(metadata.support).to_owned()));
    // Every capability in `Capability::ALL`, in that fixed order, each followed by its basis. The matrix is
    // unconditional: SP4 design §6.1 rests on a support level never being shown without it.
    for (index, capability) in Capability::ALL.into_iter().enumerate() {
        let claim = metadata.capabilities.iter().find(|claim| claim.capability == capability);
        let label = capability_label(capability);
        let value = match claim {
            Some(claim) => format!("{label}: {}", state_label(claim.state)),
            // Not every caller is a registry adapter: `metadata_invariants` requires one claim per
            // capability, but `report_lines` is reachable from a fixture that declares none. "not declared"
            // is visibly different from "unknown", and neither panics.
            None => format!("{label}: not declared"),
        };
        lines.push(if index == 0 { line("isolation", value) } else { continuation(value) });
        if let Some(claim) = claim {
            lines.push(continuation(format!("  {}", claim.basis)));
        }
    }
    if planned.plan.env.is_empty() {
        lines.push(line("environment", "none".to_owned()));
    }
    let missing: Vec<_> = planned.paths.iter().filter(|entry| !entry.existed).collect();
    for (index, (key, value)) in planned.plan.env.iter().enumerate() {
        let shown = if is_sensitive(key, &planned.sensitive_env) {
            "<redacted>".to_owned()
        } else {
            let mut shown = value.to_string_lossy().into_owned();
            let will_create = missing.iter().any(|entry| {
                entry.kind == PathKind::Dir && entry.path.as_os_str() == value.as_os_str()
            });
            if mode == ReportMode::DryRun && will_create {
                shown.push_str(" (would be created)");
            }
            shown
        };
        let entry = format!("{}={shown}", key.to_string_lossy());
        lines.push(if index == 0 { line("environment", entry) } else { continuation(entry) });
    }
    if mode == ReportMode::DryRun {
        if missing.is_empty() {
            lines.push(line("creates", "none".to_owned()));
        }
        for (index, entry) in missing.iter().enumerate() {
            let value = format!("{} (would be created)", entry.path.display());
            lines.push(if index == 0 { line("creates", value) } else { continuation(value) });
        }
    }
    let args = render_args(&planned.plan.args);
    lines.push(line("arguments", format!("[{}]", args.join(", "))));
    for note in &planned.notes {
        lines.push(line("note", note.clone()));
    }
    lines
}

/// `label:` padded to the report column, then `value`.
fn labeled(label: &str, value: impl std::fmt::Display) -> String {
    format!("{:<LABEL_WIDTH$}{value}", format!("{label}:"))
}

/// A `note:` line.
pub fn note(text: &str) -> String {
    labeled("note", text)
}

fn or_none(value: Option<impl std::fmt::Display>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

/// `<agent> resolve` (SP3 design §7.5).
pub fn resolve_lines(resolution: &Resolution) -> Vec<String> {
    vec![
        labeled("agent", &resolution.agent),
        labeled("profile", or_none(resolution.profile.as_ref())),
        labeled("source", resolution.source.label()),
        labeled("repository", or_none(resolution.repository.as_deref().map(Path::display))),
    ]
}

/// `status` and `<agent> status`: `agents` holds one resolution per agent line and, for `<agent> status`,
/// its `presence:` value (SP3 design §7.5).
pub fn status_lines(
    discovery: &Discovery,
    config: &Config,
    agents: &[(Resolution, Option<String>)],
) -> Vec<String> {
    let (repository, mapping) = match discovery {
        Discovery::Repository(root) => (root.display().to_string(), config.mapping(root)),
        Discovery::NotInRepository => ("not in a repository".to_owned(), None),
    };
    let mut lines = vec![labeled("repository", repository)];
    lines.push(labeled("mapping", or_none(mapping.and_then(|mapping| mapping.profile.as_ref()))));
    let pairs = mapping
        .map(|mapping| {
            mapping.agents.iter().map(|(id, profile)| format!("{id}={profile}")).collect::<Vec<_>>()
        })
        .filter(|pairs| !pairs.is_empty())
        .map(|pairs| pairs.join(", "));
    lines.push(labeled("agents", or_none(pairs)));
    lines.push(labeled("default", or_none(config.default_profile())));
    for (resolution, presence) in agents {
        let value = match &resolution.profile {
            Some(profile) => format!("{profile} ({})", resolution.source.label()),
            None => "none".to_owned(),
        };
        lines.push(labeled(resolution.agent.as_str(), value));
        if let Some(presence) = presence {
            lines.push(labeled("presence", presence));
        }
    }
    if let Discovery::Repository(root) = discovery {
        for (key, _) in config.mappings().filter(|(key, _)| is_proper_ancestor(key, root)) {
            lines.push(note(&format!(
                "{} has a mapping that does not apply to this repository",
                key.display()
            )));
        }
    }
    lines
}

/// A path to paste into a command: in double quotes when it contains whitespace, which POSIX shells, PowerShell
/// and `cmd` all read as one argument.
fn shell_word(path: &Path) -> String {
    let shown = path.display().to_string();
    if shown.contains(char::is_whitespace) { format!("\"{shown}\"") } else { shown }
}

fn is_proper_ancestor(key: &Path, path: &Path) -> bool {
    path.starts_with(key) && key != path
}

/// `link` (SP3 design §7.5).
pub fn link_line(
    outcome: &LinkOutcome,
    agent: Option<&AgentId>,
    repository: &Path,
    profile: &ProfileName,
) -> String {
    let agent = agent.map(|agent| format!("{agent}: ")).unwrap_or_default();
    let repository = repository.display();
    match outcome {
        LinkOutcome::Linked => format!("linked {agent}{repository} -> {profile}"),
        LinkOutcome::Changed { old } => format!("changed {agent}{repository}: {old} -> {profile}"),
        LinkOutcome::AlreadyLinked => format!("already linked {agent}{repository} -> {profile}"),
    }
}

/// `unlink` (SP3 design §7.5). `config` is the configuration read before the command, `keys` the candidates.
pub fn unlink_lines(
    outcome: &UnlinkOutcome,
    agent: Option<&AgentId>,
    config: &Config,
    keys: &[PathBuf],
) -> Vec<String> {
    let prefix = agent.map(|agent| format!("{agent}: ")).unwrap_or_default();
    let shown = match outcome {
        UnlinkOutcome::Unlinked { root, old } => {
            return vec![format!("unlinked {prefix}{} (was {old})", root.display())];
        }
        UnlinkOutcome::NothingToRemove { shown } => shown,
    };
    let mut lines = vec![format!("no mapping to remove for {prefix}{}", shown.display())];
    for (key, _) in config.mappings().filter(|(key, _)| is_proper_ancestor(key, shown)) {
        lines.push(note(&format!(
            "{} has a mapping; remove it with agent-profile unlink --repo {}",
            key.display(),
            shell_word(key)
        )));
    }
    if agent.is_none()
        && let Some(mapping) = keys.iter().find_map(|key| config.mapping(key))
        && mapping.profile.is_none()
        && !mapping.agents.is_empty()
    {
        let pairs: Vec<String> =
            mapping.agents.iter().map(|(id, profile)| format!("{id}={profile}")).collect();
        lines.push(note(&format!(
            "agent mappings remain: {}; remove them with agent-profile <agent> unlink",
            pairs.join(", ")
        )));
    }
    lines
}

fn is_sensitive(key: &OsStr, declared: &[std::ffi::OsString]) -> bool {
    if declared.iter().any(|name| name == key) {
        return true;
    }
    has_sensitive_part(&key.to_string_lossy())
}

fn has_sensitive_part(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    SENSITIVE_NAME_PARTS.iter().any(|part| upper.contains(part))
}

/// How one opaque argument is shown (spec §26 "Sensitive values must be redacted", §36).
enum Shown {
    Verbatim,
    /// The argument's value is replaced: `prefix<redacted>`.
    Redacted(String),
    /// The argument is a secret-named option without `=`: the next argument is its value.
    HidesNext,
}

/// Renders the opaque arguments for the report. Only the report is redacted; the launched arguments never
/// change. Redaction is a shallow name rule, never a parser: a `--option` whose name holds a sensitive part
/// hides its value (`--api-key=<redacted>`, or the next argument); a `NAME=value` whose (possibly dotted) NAME
/// holds one hides the value (`mcp_servers.gh.env.GITHUB_TOKEN=<redacted>`); a `Name: value` header whose name
/// holds one hides the value (`Authorization: <redacted>`). Every argument is scanned, including after a `--`,
/// and boolean-looking names get no exemption, because hiding too much only costs readability.
fn render_args(args: &[std::ffi::OsString]) -> Vec<String> {
    let mut rendered = Vec::with_capacity(args.len());
    let mut hide_next = false;
    for arg in args {
        let shown = classify(&arg.to_string_lossy());
        if hide_next {
            // A hidden secret-named option still hides its own value, so a chain never leaks.
            hide_next = matches!(shown, Shown::HidesNext);
            rendered.push(format!("{:?}", "<redacted>"));
            continue;
        }
        rendered.push(match shown {
            Shown::Verbatim => render_arg(arg),
            Shown::Redacted(prefix) => format!("{:?}", format!("{prefix}<redacted>")),
            Shown::HidesNext => {
                hide_next = true;
                render_arg(arg)
            }
        });
    }
    rendered
}

fn classify(text: &str) -> Shown {
    if let Some(option) = text.strip_prefix("--").filter(|option| !option.is_empty()) {
        let (name, value) = match option.split_once('=') {
            Some((name, value)) => (name, Some(value)),
            None => (option, None),
        };
        if has_sensitive_part(name) {
            return match value {
                Some(_) => Shown::Redacted(format!("--{name}=")),
                None => Shown::HidesNext,
            };
        }
        return match value.and_then(sensitive_value_prefix) {
            Some(prefix) => Shown::Redacted(format!("--{name}={prefix}")),
            None => Shown::Verbatim,
        };
    }
    match sensitive_value_prefix(text) {
        Some(prefix) => Shown::Redacted(prefix),
        None => Shown::Verbatim,
    }
}

/// The shown prefix when `text` is a `NAME=value` assignment or a `Name: value` header whose name holds a
/// sensitive part: `NAME=` or `Name: `.
fn sensitive_value_prefix(text: &str) -> Option<String> {
    if let Some((name, _)) = text.split_once('=')
        && is_dotted_identifier(name)
        && has_sensitive_part(name)
    {
        return Some(format!("{name}="));
    }
    let (name, _) = text.split_once(':')?;
    let header = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    (header && has_sensitive_part(name)).then(|| format!("{name}: "))
}

/// `A_1`, `mcp_servers.gh.env.GITHUB_TOKEN` or `mcp_servers.chrome-devtools.http_headers.X-Api-Key`: TOML
/// bare keys (letters, digits, `_`, `-`) joined by dots, as Codex `-c` dotted paths are.
fn is_dotted_identifier(name: &str) -> bool {
    name.split('.').all(|segment| {
        !segment.is_empty()
            && segment.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    })
}

fn render_arg(arg: &OsStr) -> String {
    match arg.to_str() {
        Some(text) => format!("{text:?}"),
        None => format!("{:?} (non-UTF-8)", arg.to_string_lossy()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::ProfilePath;
    use crate::launch::LaunchPlan;
    use crate::name::{AgentId, Platform, ProfileName};
    use crate::resolve::resolve;
    use std::ffi::OsString;
    use std::path::PathBuf;

    use crate::adapter::{AdapterEvidence, CapabilityClaim};

    /// A registry-shaped fixture: one claim per capability, as `metadata_invariants` requires of a real
    /// adapter. `Fake` is not usable here — it is `#[cfg(debug_assertions)]` and this module is `cfg(test)`.
    static TEST_METADATA: AdapterMetadata = AdapterMetadata {
        id: "fake",
        executable: "fake-agent",
        mechanism_summary: "environment variable FAKE_AGENT_HOME",
        support: SupportLevel::Proven,
        evidence: AdapterEvidence {
            mechanism_id: "fake-home-v1",
            verified_at: "2026-09-16",
            upstream_version: "0.0.0",
            source_url: "measured",
            notes: "test fixture",
        },
        capabilities: &[
            CapabilityClaim {
                capability: Capability::ConfigIsolation,
                state: CapabilityState::Supported,
                basis: "measured: the fixture writes only under the profile directory",
            },
            CapabilityClaim {
                capability: Capability::CredentialIsolation,
                state: CapabilityState::Unknown,
                basis: "unmeasured: the fixture has no credentials",
            },
            CapabilityClaim {
                capability: Capability::StateIsolation,
                state: CapabilityState::NotGuaranteed,
                basis: "measured: the fixture keeps no state",
            },
        ],
        env: &[],
        conflicts: &[],
    };

    /// The report line beginning `<label>:`. Assertions that used a hard-coded index all broke when the
    /// isolation block shifted every position; looking the field up by name keeps them from breaking again.
    fn field<'a>(lines: &'a [String], label: &str) -> &'a str {
        let prefix = format!("{label}:");
        lines
            .iter()
            .find(|line| line.starts_with(&prefix))
            .unwrap_or_else(|| panic!("no {label}: line in {lines:?}"))
    }

    /// The seven lines every report now carries between `mechanism:` and `environment:`.
    fn isolation_block() -> Vec<String> {
        vec![
            "support:      proven".to_owned(),
            "isolation:    config: supported".to_owned(),
            "                measured: the fixture writes only under the profile directory"
                .to_owned(),
            "              credentials: unknown".to_owned(),
            "                unmeasured: the fixture has no credentials".to_owned(),
            "              state: not guaranteed".to_owned(),
            "                measured: the fixture keeps no state".to_owned(),
        ]
    }

    fn planned(
        env: Vec<(OsString, OsString)>,
        sensitive: Vec<OsString>,
        exists: bool,
    ) -> PlannedLaunch {
        let dir = PathBuf::from("/root/profiles/work/fake");
        PlannedLaunch {
            plan: LaunchPlan {
                executable: PathBuf::from("/bin/fake-agent"),
                args: vec!["--foo".into(), "a b".into()],
                env,
                cwd: None,
            },
            profile: ProfileName::parse("work", Platform::Unix).unwrap(),
            profile_dir: dir.clone(),
            paths: vec![ProfilePath { path: dir, kind: PathKind::Dir, existed: exists }],
            executable_origin: Origin::Configured,
            mechanism: "environment variable FAKE_AGENT_HOME".to_owned(),
            sensitive_env: sensitive,
            notes: Vec::new(),
        }
    }

    fn resolution() -> Resolution {
        resolve(
            AgentId::parse("fake").unwrap(),
            Some(ProfileName::parse("work", Platform::Unix).unwrap()),
            &crate::config::Config::default(),
            &Discovery::NotInRepository,
        )
    }

    fn home_env() -> Vec<(OsString, OsString)> {
        vec![("FAKE_AGENT_HOME".into(), "/root/profiles/work/fake".into())]
    }

    #[test]
    fn report_has_every_spec_26_field() {
        let lines = report_lines(
            &planned(home_env(), vec![], false),
            &resolution(),
            ReportMode::DryRun,
            &TEST_METADATA,
        );
        let dir = PathBuf::from("/root/profiles/work/fake");
        let mut expected = vec![
            "agent:        fake".to_owned(),
            "profile:      work (explicit)".to_owned(),
            format!("executable:   {} (configured)", PathBuf::from("/bin/fake-agent").display()),
            "repository:   none".to_owned(),
            "mechanism:    environment variable FAKE_AGENT_HOME".to_owned(),
        ];
        expected.extend(isolation_block());
        expected.extend([
            "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake (would be created)".to_owned(),
            format!("creates:      {} (would be created)", dir.display()),
            "arguments:    [\"--foo\", \"a b\"]".to_owned(),
        ]);
        assert_eq!(lines, expected);
        let existing = report_lines(
            &planned(home_env(), vec![], true),
            &resolution(),
            ReportMode::DryRun,
            &TEST_METADATA,
        );
        assert_eq!(
            field(&existing, "environment"),
            "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake"
        );
        assert_eq!(field(&existing, "creates"), "creates:      none");
    }

    #[test]
    fn verbose_has_no_markers_and_no_creates_line() {
        let lines = report_lines(
            &planned(home_env(), vec![], false),
            &resolution(),
            ReportMode::Verbose,
            &TEST_METADATA,
        );
        assert_eq!(lines.len(), 14, "{lines:?}");
        let text = lines.join("\n");
        assert!(!text.contains("(would be created)"), "{text}");
        assert!(!text.contains("creates:"), "{text}");
    }

    #[test]
    fn several_missing_paths_and_notes_are_listed_in_order() {
        let mut planned = planned(Vec::new(), vec![], false);
        let file = PathBuf::from("/root/profiles/work/aider/.aider.conf.yml");
        planned.paths.push(ProfilePath {
            path: file.clone(),
            kind: PathKind::File { contents: b"{}\n" },
            existed: false,
        });
        planned.notes = vec!["first note".to_owned(), "second note".to_owned()];
        let lines = report_lines(&planned, &resolution(), ReportMode::DryRun, &TEST_METADATA);
        assert_eq!(field(&lines, "environment"), "environment:  none");
        assert_eq!(
            field(&lines, "creates"),
            format!("creates:      {} (would be created)", planned.paths[0].path.display())
        );
        assert_eq!(lines[14], format!("              {} (would be created)", file.display()));
        assert_eq!(
            lines.iter().filter(|line| line.starts_with("note:")).collect::<Vec<_>>(),
            ["note:         first note", "note:         second note"]
        );
        let verbose = report_lines(&planned, &resolution(), ReportMode::Verbose, &TEST_METADATA);
        assert_eq!(verbose.last().unwrap(), "note:         second note");
    }

    #[test]
    fn sensitive_argument_values_are_redacted_in_the_report_only() {
        let args: Vec<std::ffi::OsString> = [
            "--api-key",
            "anthropic=sk-1",
            "--openai-api-key=sk-2",
            "--set-env",
            "ANTHROPIC_API_KEY=sk-3",
            "--set-env=OPENAI_API_KEY=sk-4",
            "--",
            "GITHUB_TOKEN=sk-5",
            "--no-op-key",
            "sk-6",
            "--model",
            "gpt",
            "path=a=b",
            "-c",
            "mcp_servers.gh.env.GITHUB_TOKEN=\"sk-7\"",
            "--config=model_providers.x.experimental_bearer_token=sk-8",
            "--header",
            "Authorization: Bearer sk-9",
            "--header=X-Api-Key:sk-10",
            "mcp_servers.chrome-devtools.env.GITHUB_TOKEN=sk-11",
            "mcp_servers.gh.http_headers.X-Api-Key=sk-12",
            "Proxy-Authorization: Basic sk-13:with-colon",
            "https://example.com/mcp",
            "a.b=c",
            "--Auth-Token",
            "--also-hidden",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        assert_eq!(
            render_args(&args),
            [
                r#""--api-key""#,
                r#""<redacted>""#,
                r#""--openai-api-key=<redacted>""#,
                r#""--set-env""#,
                r#""ANTHROPIC_API_KEY=<redacted>""#,
                r#""--set-env=OPENAI_API_KEY=<redacted>""#,
                r#""--""#,
                r#""GITHUB_TOKEN=<redacted>""#,
                r#""--no-op-key""#,
                r#""<redacted>""#,
                r#""--model""#,
                r#""gpt""#,
                r#""path=a=b""#,
                r#""-c""#,
                r#""mcp_servers.gh.env.GITHUB_TOKEN=<redacted>""#,
                r#""--config=model_providers.x.experimental_bearer_token=<redacted>""#,
                r#""--header""#,
                r#""Authorization: <redacted>""#,
                r#""--header=X-Api-Key: <redacted>""#,
                r#""mcp_servers.chrome-devtools.env.GITHUB_TOKEN=<redacted>""#,
                r#""mcp_servers.gh.http_headers.X-Api-Key=<redacted>""#,
                r#""Proxy-Authorization: <redacted>""#,
                r#""https://example.com/mcp""#,
                r#""a.b=c""#,
                r#""--Auth-Token""#,
                r#""<redacted>""#,
            ]
        );
        let mut planned = planned(Vec::new(), vec![], true);
        planned.plan.args = args.clone();
        let text =
            report_lines(&planned, &resolution(), ReportMode::Verbose, &TEST_METADATA).join("\n");
        for secret in ["sk-", "also-hidden"] {
            assert!(!text.contains(secret), "{text}");
        }
        assert_eq!(planned.plan.args, args, "the launched arguments never change");
    }

    #[test]
    fn every_sensitive_name_part_and_padded_headers_are_redacted() {
        let args: Vec<std::ffi::OsString> = [
            "--client-secret=a",
            "--db-password=b",
            "--credential-file=c",
            "--x-token=d",
            "--api-key=e",
            "--auth=f",
            "Authorization: Basic dXNlcjpwYXNz==",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        assert_eq!(
            render_args(&args),
            [
                r#""--client-secret=<redacted>""#,
                r#""--db-password=<redacted>""#,
                r#""--credential-file=<redacted>""#,
                r#""--x-token=<redacted>""#,
                r#""--api-key=<redacted>""#,
                r#""--auth=<redacted>""#,
                r#""Authorization: <redacted>""#,
            ]
        );
    }

    #[test]
    fn a_hidden_secret_option_still_hides_its_own_value() {
        let args: Vec<std::ffi::OsString> = [
            "--api-key",
            "--client-secret",
            "sk-live",
            "--model",
            "gpt",
            "--token",
            "GITHUB_TOKEN=x",
            "--shown",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        assert_eq!(
            render_args(&args),
            [
                r#""--api-key""#,
                r#""<redacted>""#,
                r#""<redacted>""#,
                r#""--model""#,
                r#""gpt""#,
                r#""--token""#,
                r#""<redacted>""#,
                r#""--shown""#,
            ]
        );
    }

    #[test]
    fn redaction_by_declaration_and_by_name() {
        let env: Vec<(OsString, OsString)> = vec![
            ("PLAIN".into(), "visible".into()),
            ("DECLARED".into(), "hidden-1".into()),
            ("MY_api_Token".into(), "hidden-2".into()),
            ("GITHUB_AUTH".into(), "hidden-3".into()),
        ];
        let lines = report_lines(
            &planned(env, vec!["DECLARED".into()], true),
            &resolution(),
            ReportMode::DryRun,
            &TEST_METADATA,
        );
        let text = lines.join("\n");
        assert!(text.contains("PLAIN=visible"), "{text}");
        for secret in ["hidden-1", "hidden-2", "hidden-3"] {
            assert!(!text.contains(secret), "{text}");
        }
        assert_eq!(text.matches("<redacted>").count(), 3, "{text}");
        assert!(lines[13].starts_with("              DECLARED="), "{text}");
    }

    /// The Unix spelling on Unix, the Windows spelling on Windows.
    fn host(unix: &str, windows: &str) -> PathBuf {
        PathBuf::from(if cfg!(windows) { windows } else { unix })
    }

    fn config(text: &str) -> Config {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.toml"), text).unwrap();
        Config::load(&crate::config::AppRoot::from_path(dir.path().to_path_buf())).unwrap()
    }

    fn key(path: &Path) -> String {
        toml_edit::Key::new(path.to_str().unwrap()).display_repr().into_owned()
    }

    fn name(text: &str) -> ProfileName {
        ProfileName::parse(text, Platform::host()).unwrap()
    }

    #[test]
    fn resolve_lines_show_the_source_and_none() {
        let acme = host("/src/acme", r"C:\src\acme");
        let text = format!("[repositories.{}]\nagents = {{ claude = \"personal\" }}\n", key(&acme));
        let config = config(&text);
        let claude = AgentId::parse("claude").unwrap();
        let inside = resolve(claude.clone(), None, &config, &Discovery::Repository(acme.clone()));
        assert_eq!(
            resolve_lines(&inside),
            [
                "agent:        claude".to_owned(),
                "profile:      personal".to_owned(),
                "source:       repository agent mapping".to_owned(),
                format!("repository:   {}", acme.display()),
            ]
        );
        let outside = resolve(claude, None, &config, &Discovery::NotInRepository);
        assert_eq!(
            resolve_lines(&outside),
            [
                "agent:        claude",
                "profile:      none",
                "source:       none",
                "repository:   none"
            ]
        );
    }

    #[test]
    fn status_lines_show_mappings_the_default_each_agent_and_ancestor_notes() {
        let src = host("/src", r"C:\src");
        let acme = src.join("acme");
        let text = format!(
            "default_profile = \"work\"\n[repositories.{}]\nprofile = \"work\"\nagents = {{ codex = \"b\", claude = \"personal\" }}\n[repositories.{}]\nprofile = \"outer\"\n[repositories.{}]\nprofile = \"root\"\n[repositories.{}]\nprofile = \"sibling\"\n",
            key(&acme),
            key(&src),
            key(&host("/", r"C:\")),
            key(&host("/src/acme-2", r"C:\src\acme-2")),
        );
        let config = config(&text);
        let discovery = Discovery::Repository(acme.clone());
        let rows: Vec<(Resolution, Option<String>)> = ["claude", "aider"]
            .into_iter()
            .map(|id| (resolve(AgentId::parse(id).unwrap(), None, &config, &discovery), None))
            .collect();
        assert_eq!(
            status_lines(&discovery, &config, &rows),
            [
                format!("repository:   {}", acme.display()),
                "mapping:      work".to_owned(),
                "agents:       claude=personal, codex=b".to_owned(),
                "default:      work".to_owned(),
                "claude:       personal (repository agent mapping)".to_owned(),
                "aider:        work (repository mapping)".to_owned(),
                format!(
                    "note:         {} has a mapping that does not apply to this repository",
                    host("/", r"C:\").display()
                ),
                format!(
                    "note:         {} has a mapping that does not apply to this repository",
                    src.display()
                ),
            ]
        );
    }

    #[test]
    fn agent_status_lines_add_presence_and_show_none_outside_a_repository() {
        let config = config("");
        let claude = AgentId::parse("claude").unwrap();
        let none = resolve(claude.clone(), None, &config, &Discovery::NotInRepository);
        assert_eq!(
            status_lines(&Discovery::NotInRepository, &config, &[(none, None)]),
            [
                "repository:   not in a repository",
                "mapping:      none",
                "agents:       none",
                "default:      none",
                "claude:       none",
            ]
        );
        let explicit = resolve(claude, Some(name("work")), &config, &Discovery::NotInRepository);
        let lines = status_lines(
            &Discovery::NotInRepository,
            &config,
            &[(explicit, Some("conflicts with Work".to_owned()))],
        );
        assert_eq!(
            &lines[4..],
            ["claude:       work (explicit)", "presence:     conflicts with Work"]
        );
    }

    #[test]
    fn link_lines_cover_every_outcome() {
        let acme = host("/src/acme", r"C:\src\acme");
        let claude = AgentId::parse("claude").unwrap();
        let shown = acme.display();
        for (outcome, agent, expected) in [
            (LinkOutcome::Linked, None, format!("linked {shown} -> work")),
            (LinkOutcome::Linked, Some(&claude), format!("linked claude: {shown} -> work")),
            (
                LinkOutcome::Changed { old: name("personal") },
                None,
                format!("changed {shown}: personal -> work"),
            ),
            (
                LinkOutcome::Changed { old: name("personal") },
                Some(&claude),
                format!("changed claude: {shown}: personal -> work"),
            ),
            (LinkOutcome::AlreadyLinked, None, format!("already linked {shown} -> work")),
            (
                LinkOutcome::AlreadyLinked,
                Some(&claude),
                format!("already linked claude: {shown} -> work"),
            ),
        ] {
            assert_eq!(link_line(&outcome, agent, &acme, &name("work")), expected);
        }
    }

    #[test]
    fn unlink_lines_cover_every_outcome_and_both_notes() {
        let src = host("/src", r"C:\src");
        let acme = src.join("acme");
        let claude = AgentId::parse("claude").unwrap();
        let text = format!(
            "[repositories.{}]\nprofile = \"outer\"\n[repositories.{}]\nagents = {{ codex = \"b\", claude = \"personal\" }}\n",
            key(&src),
            key(&acme)
        );
        let config = config(&text);
        let removed = UnlinkOutcome::Unlinked { root: acme.clone(), old: name("work") };
        assert_eq!(
            unlink_lines(&removed, None, &config, std::slice::from_ref(&acme)),
            [format!("unlinked {} (was work)", acme.display())]
        );
        assert_eq!(
            unlink_lines(&removed, Some(&claude), &config, std::slice::from_ref(&acme)),
            [format!("unlinked claude: {} (was work)", acme.display())]
        );
        let nothing = UnlinkOutcome::NothingToRemove { shown: acme.clone() };
        let ancestor = format!(
            "note:         {} has a mapping; remove it with agent-profile unlink --repo {}",
            src.display(),
            src.display()
        );
        assert_eq!(
            unlink_lines(&nothing, None, &config, std::slice::from_ref(&acme)),
            [
                format!("no mapping to remove for {}", acme.display()),
                ancestor.clone(),
                "note:         agent mappings remain: claude=personal, codex=b; remove them with \
                 agent-profile <agent> unlink"
                    .to_owned(),
            ]
        );
        assert_eq!(
            unlink_lines(&nothing, Some(&claude), &config, std::slice::from_ref(&acme)),
            [format!("no mapping to remove for claude: {}", acme.display()), ancestor]
        );
    }

    #[test]
    fn the_unlink_hint_quotes_a_path_with_whitespace() {
        let spaced = host("/my repos", r"C:\my repos");
        let inner = spaced.join("inner");
        let text = format!("[repositories.{}]\nprofile = \"outer\"\n", key(&spaced));
        let config = config(&text);
        let nothing = UnlinkOutcome::NothingToRemove { shown: inner.clone() };
        assert_eq!(
            unlink_lines(&nothing, None, &config, std::slice::from_ref(&inner))[1],
            format!(
                "note:         {} has a mapping; remove it with agent-profile unlink --repo \"{}\"",
                spaced.display(),
                spaced.display()
            )
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_argument_is_rendered_lossily_with_marker() {
        use std::os::unix::ffi::OsStringExt;
        assert_eq!(render_arg(&OsString::from_vec(vec![0x66, 0xff])), "\"f\u{fffd}\" (non-UTF-8)");
    }

    #[cfg(windows)]
    #[test]
    fn non_utf8_argument_is_rendered_lossily_with_marker() {
        use std::os::windows::ffi::OsStringExt;
        assert_eq!(render_arg(&OsString::from_wide(&[0x66, 0xD800])), "\"f\u{fffd}\" (non-UTF-8)");
    }

    /// `TEST_METADATA` with the support level and the state-isolation claim replaced.
    ///
    /// The hedge's own branches are unreachable through any shipped adapter: `fake` is the only
    /// non-`Proven` one and all three of its claims are `Unknown`, so nothing in the workspace ever sends
    /// a `NotGuaranteed` claim through `support_hedge`. That is the branch §10 asks for a fixture for.
    fn hedged(support: SupportLevel, state: CapabilityState) -> AdapterMetadata {
        const CLAIMS: [CapabilityClaim; 3] = [
            CapabilityClaim {
                capability: Capability::ConfigIsolation,
                state: CapabilityState::Supported,
                basis: "measured: the fixture writes only under the profile directory",
            },
            CapabilityClaim {
                capability: Capability::CredentialIsolation,
                state: CapabilityState::Unknown,
                basis: "unmeasured: the fixture has no credentials",
            },
            CapabilityClaim {
                capability: Capability::StateIsolation,
                state: CapabilityState::NotGuaranteed,
                basis: "measured: the fixture keeps no state",
            },
        ];
        let mut claims = CLAIMS;
        claims[2].state = state;
        // `capabilities` is `&'static [_]`, so the modified claims have to outlive this call.
        let leaked: &'static [CapabilityClaim] = Box::leak(Box::new(claims));
        AdapterMetadata { support, capabilities: leaked, ..TEST_METADATA }
    }

    #[test]
    fn a_proven_adapter_does_not_hedge() {
        assert_eq!(
            support_hedge(&hedged(SupportLevel::Proven, CapabilityState::NotGuaranteed)),
            None
        );
    }

    /// The regression the design names: an earlier draft listed only `Unknown` capabilities, which would
    /// have hedged about an adapter whose configuration demonstrably leaks without ever saying so.
    #[test]
    fn the_hedge_names_every_capability_that_is_not_supported() {
        let hedge =
            support_hedge(&hedged(SupportLevel::Experimental, CapabilityState::NotGuaranteed))
                .expect("a non-proven adapter hedges");
        assert_eq!(
            hedge,
            "fake is experimental: credentials unknown, state not guaranteed. \
             Run with --dry-run for detail.",
            "{hedge}"
        );
    }

    #[test]
    fn the_hedge_lists_only_the_capabilities_that_are_weak() {
        // `CredentialIsolation` stays `Unknown` in the fixture, so that is all this should name.
        assert_eq!(
            support_hedge(&hedged(SupportLevel::Experimental, CapabilityState::Supported))
                .as_deref(),
            Some("fake is experimental: credentials unknown. Run with --dry-run for detail.")
        );
    }

    /// A missing claim must render, not vanish: dropping it silently would let an adapter that declared
    /// nothing at all for a capability produce the same reassuring hedge as one that measured it and found
    /// it `Supported`.
    #[test]
    fn the_hedge_names_a_capability_that_was_never_declared() {
        let mut metadata = hedged(SupportLevel::Experimental, CapabilityState::Supported);
        let claims: Vec<CapabilityClaim> = metadata
            .capabilities
            .iter()
            .filter(|claim| claim.capability != Capability::CredentialIsolation)
            .copied()
            .collect();
        let leaked: &'static [CapabilityClaim] = Box::leak(claims.into_boxed_slice());
        metadata.capabilities = leaked;
        let hedge = support_hedge(&metadata).expect("a non-proven adapter hedges");
        assert!(
            hedge.contains(&format!(
                "{} not declared",
                capability_label(Capability::CredentialIsolation)
            )),
            "{hedge}"
        );
    }

    /// `report_lines` is reachable from a fixture that declares no capabilities at all
    /// (`adapter/mod.rs`'s `SECRETIVE`). A lookup-and-unwrap would panic there; a silent skip would make
    /// "not claimed" indistinguishable from "not rendered".
    #[test]
    fn a_capability_with_no_claim_renders_as_not_declared() {
        let metadata = AdapterMetadata { capabilities: &[], ..TEST_METADATA };
        let launch = planned(home_env(), Vec::new(), false);
        let lines = report_lines(&launch, &resolution(), ReportMode::DryRun, &metadata);
        assert!(field(&lines, "isolation").contains("config: not declared"), "{lines:?}");
        assert!(
            lines.iter().all(|line| !line.contains("measured:") && !line.contains("unmeasured:")),
            "a capability with no claim has no basis line: {lines:?}"
        );
    }

    /// Pins `state_label`'s five literal spellings. Nothing else in the suite exercises the strings
    /// `"not supported"` or `"conditional"` directly: collapsing either arm onto another word left all
    /// other tests green, because `support_hedge` (the only caller) never distinguishes the wording, only
    /// whether the string is present.
    #[test]
    fn every_capability_state_has_a_distinct_report_spelling() {
        let pairs = [
            (CapabilityState::Supported, "supported"),
            (CapabilityState::NotSupported, "not supported"),
            (CapabilityState::NotGuaranteed, "not guaranteed"),
            (CapabilityState::Conditional, "conditional"),
            (CapabilityState::Unknown, "unknown"),
        ];
        let mut spellings = Vec::with_capacity(pairs.len());
        for (state, expected) in pairs {
            let actual = state_label(state);
            assert_eq!(actual, expected, "{state:?}");
            spellings.push(actual);
        }
        let mut distinct = spellings.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(
            distinct.len(),
            spellings.len(),
            "spellings must be pairwise distinct: {spellings:?}"
        );
    }

    /// A `NotSupported` claim must hedge as "not supported", not as anything more reassuring. The expected
    /// capability word comes from `capability_label` (so a label rename doesn't break this), but "not
    /// supported" is hard-coded: routing it through `state_label` here would make the assertion mutate in
    /// lockstep with the function under test and never go red.
    #[test]
    fn the_hedge_spells_a_not_supported_claim_as_not_supported() {
        const CLAIMS: [CapabilityClaim; 3] = [
            CapabilityClaim {
                capability: Capability::ConfigIsolation,
                state: CapabilityState::Supported,
                basis: "measured: the fixture writes only under the profile directory",
            },
            CapabilityClaim {
                capability: Capability::CredentialIsolation,
                state: CapabilityState::NotSupported,
                basis: "measured: credentials stay in the default location",
            },
            CapabilityClaim {
                capability: Capability::StateIsolation,
                state: CapabilityState::Supported,
                basis: "measured: the fixture keeps no state",
            },
        ];
        let leaked: &'static [CapabilityClaim] = Box::leak(Box::new(CLAIMS));
        let metadata = AdapterMetadata {
            support: SupportLevel::Experimental,
            capabilities: leaked,
            ..TEST_METADATA
        };
        let hedge = support_hedge(&metadata).expect("a non-proven adapter hedges");
        assert!(
            hedge.contains(&format!(
                "{} not supported",
                capability_label(Capability::CredentialIsolation)
            )),
            "{hedge}"
        );
    }

    /// The `(would be created)` marker names only the env value that equals a missing `Dir` entry — not a
    /// value that merely matches no entry, and not one that equals a missing `File` entry (the predicate
    /// requires `Dir`). Every shipped fixture happened to have exactly one env entry whose value was the
    /// one missing path, so `true` in place of the predicate was indistinguishable from it.
    #[test]
    fn the_would_be_created_marker_names_only_the_path_that_will_be_created() {
        let dir = PathBuf::from("/root/profiles/work/fake");
        let file = PathBuf::from("/root/profiles/work/fake/.fake.conf");
        let elsewhere = PathBuf::from("/somewhere/else");
        let launch = PlannedLaunch {
            plan: LaunchPlan {
                executable: PathBuf::from("/bin/fake-agent"),
                args: vec!["--foo".into()],
                env: vec![
                    ("DIR_VAR".into(), dir.clone().into_os_string()),
                    ("ELSEWHERE_VAR".into(), elsewhere.into_os_string()),
                    ("FILE_VAR".into(), file.clone().into_os_string()),
                ],
                cwd: None,
            },
            profile: ProfileName::parse("work", Platform::Unix).unwrap(),
            profile_dir: dir.clone(),
            paths: vec![
                ProfilePath { path: dir, kind: PathKind::Dir, existed: false },
                ProfilePath {
                    path: file,
                    kind: PathKind::File { contents: b"{}\n" },
                    existed: false,
                },
            ],
            executable_origin: Origin::Configured,
            mechanism: "environment variable FAKE_AGENT_HOME".to_owned(),
            sensitive_env: Vec::new(),
            notes: Vec::new(),
        };
        let lines = report_lines(&launch, &resolution(), ReportMode::DryRun, &TEST_METADATA);
        let dir_line = lines.iter().find(|line| line.contains("DIR_VAR=")).expect("DIR_VAR line");
        assert!(dir_line.contains("(would be created)"), "{dir_line}");
        let elsewhere_line =
            lines.iter().find(|line| line.contains("ELSEWHERE_VAR=")).expect("ELSEWHERE_VAR line");
        assert!(!elsewhere_line.contains("(would be created)"), "{elsewhere_line}");
        let file_line =
            lines.iter().find(|line| line.contains("FILE_VAR=")).expect("FILE_VAR line");
        assert!(!file_line.contains("(would be created)"), "{file_line}");
    }
}
