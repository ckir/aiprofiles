//! Human output: the dry-run and `--verbose` report (spec §26, design §7.4) and redaction (spec §22).
//! JSON output (spec §32) arrives in SP5.

use std::ffi::OsStr;

use crate::adapter::PlannedLaunch;
use crate::exe::Origin;
use crate::resolve::Resolution;

/// Substrings that mark an override variable as secret-bearing, whatever the adapter declared.
const SENSITIVE_NAME_PARTS: [&str; 6] =
    ["TOKEN", "SECRET", "KEY", "PASSWORD", "CREDENTIAL", "AUTH"];

const LABEL_WIDTH: usize = 14;

/// The report lines, without trailing newlines.
pub fn report_lines(planned: &PlannedLaunch, resolution: &Resolution) -> Vec<String> {
    let mut lines = Vec::new();
    let line = |label: &str, value: String| format!("{:<LABEL_WIDTH$}{value}", format!("{label}:"));
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
    for (index, (key, value)) in planned.plan.env.iter().enumerate() {
        let shown = if is_sensitive(key, &planned.sensitive_env) {
            "<redacted>".to_owned()
        } else {
            let mut shown = value.to_string_lossy().into_owned();
            if value.as_os_str() == planned.profile_dir.as_os_str() && !planned.profile_dir_exists {
                shown.push_str(" (would be created)");
            }
            shown
        };
        let entry = format!("{}={shown}", key.to_string_lossy());
        if index == 0 {
            lines.push(line("environment", entry));
        } else {
            lines.push(format!("{:LABEL_WIDTH$}{entry}", ""));
        }
    }
    let args: Vec<String> = planned.plan.args.iter().map(|arg| render_arg(arg)).collect();
    lines.push(line("arguments", format!("[{}]", args.join(", "))));
    lines
}

fn is_sensitive(key: &OsStr, declared: &[std::ffi::OsString]) -> bool {
    if declared.iter().any(|name| name == key) {
        return true;
    }
    let upper = key.to_string_lossy().to_ascii_uppercase();
    SENSITIVE_NAME_PARTS.iter().any(|part| upper.contains(part))
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
            profile_dir: dir,
            profile_dir_exists: exists,
            executable_origin: Origin::Configured,
            mechanism: "environment variable FAKE_AGENT_HOME".to_owned(),
            sensitive_env: sensitive,
        }
    }

    fn resolution() -> Resolution {
        resolve(
            AgentId::parse("fake").unwrap(),
            Some(ProfileName::parse("work", Platform::Unix).unwrap()),
        )
    }

    #[test]
    fn report_has_every_spec_26_field() {
        let env = vec![("FAKE_AGENT_HOME".into(), "/root/profiles/work/fake".into())];
        let lines = report_lines(&planned(env, vec![], false), &resolution());
        assert_eq!(
            lines,
            [
                "agent:        fake",
                "profile:      work (explicit)",
                &format!(
                    "executable:   {} (configured)",
                    PathBuf::from("/bin/fake-agent").display()
                ),
                "repository:   none",
                "mechanism:    environment variable FAKE_AGENT_HOME",
                "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake (would be created)",
                "arguments:    [\"--foo\", \"a b\"]",
            ]
        );
        let env = vec![("FAKE_AGENT_HOME".into(), "/root/profiles/work/fake".into())];
        let existing = report_lines(&planned(env, vec![], true), &resolution());
        assert_eq!(existing[5], "environment:  FAKE_AGENT_HOME=/root/profiles/work/fake");
    }

    #[test]
    fn redaction_by_declaration_and_by_name() {
        let env: Vec<(OsString, OsString)> = vec![
            ("PLAIN".into(), "visible".into()),
            ("DECLARED".into(), "hidden-1".into()),
            ("MY_api_Token".into(), "hidden-2".into()),
            ("GITHUB_AUTH".into(), "hidden-3".into()),
        ];
        let lines = report_lines(&planned(env, vec!["DECLARED".into()], true), &resolution());
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
