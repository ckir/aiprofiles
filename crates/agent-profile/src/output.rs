//! Human output: the dry-run and `--verbose` report (spec §26, SP1 design §7.4, SP2 design §7.3) and
//! redaction (spec §22). JSON output (spec §32) arrives in SP5.

use std::ffi::OsStr;

use crate::adapter::{PathKind, PlannedLaunch};
use crate::exe::Origin;
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
        if hide_next {
            hide_next = false;
            rendered.push(format!("{:?}", "<redacted>"));
            continue;
        }
        rendered.push(match classify(&arg.to_string_lossy()) {
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
