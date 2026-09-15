//! Human output: the dry-run and `--verbose` report (spec §26, SP1 design §7.4, SP2 design §7.3), the
//! repository command reports (SP3 design §7.5) and redaction (spec §22). JSON output (spec §32) arrives in
//! SP5.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::adapter::{PathKind, PlannedLaunch};
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

/// The report lines, without trailing newlines.
pub fn report_lines(
    planned: &PlannedLaunch,
    resolution: &Resolution,
    mode: ReportMode,
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
            key.display()
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
        let lines =
            report_lines(&planned(home_env(), vec![], false), &resolution(), ReportMode::DryRun);
        let dir = PathBuf::from("/root/profiles/work/fake");
        assert_eq!(
            lines,
            [
                "agent:        fake".to_owned(),
                "profile:      work (explicit)".to_owned(),
                format!(
                    "executable:   {} (configured)",
                    PathBuf::from("/bin/fake-agent").display()
                ),
                "repository:   none".to_owned(),
                "mechanism:    environment variable FAKE_AGENT_HOME".to_owned(),
                "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake (would be created)"
                    .to_owned(),
                format!("creates:      {} (would be created)", dir.display()),
                "arguments:    [\"--foo\", \"a b\"]".to_owned(),
            ]
        );
        let existing =
            report_lines(&planned(home_env(), vec![], true), &resolution(), ReportMode::DryRun);
        assert_eq!(existing[5], "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake");
        assert_eq!(existing[6], "creates:      none");
    }

    #[test]
    fn verbose_has_no_markers_and_no_creates_line() {
        let lines =
            report_lines(&planned(home_env(), vec![], false), &resolution(), ReportMode::Verbose);
        assert_eq!(lines.len(), 7, "{lines:?}");
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
        let lines = report_lines(&planned, &resolution(), ReportMode::DryRun);
        assert_eq!(lines[5], "environment:  none");
        assert_eq!(
            lines[6],
            format!("creates:      {} (would be created)", planned.paths[0].path.display())
        );
        assert_eq!(lines[7], format!("              {} (would be created)", file.display()));
        assert_eq!(&lines[9..], ["note:         first note", "note:         second note"]);
        let verbose = report_lines(&planned, &resolution(), ReportMode::Verbose);
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
        let text = report_lines(&planned, &resolution(), ReportMode::Verbose).join("\n");
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
        );
        let text = lines.join("\n");
        assert!(text.contains("PLAIN=visible"), "{text}");
        for secret in ["hidden-1", "hidden-2", "hidden-3"] {
            assert!(!text.contains(secret), "{text}");
        }
        assert_eq!(text.matches("<redacted>").count(), 3, "{text}");
        assert!(lines[6].starts_with("              DECLARED="), "{text}");
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
}
