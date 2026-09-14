//! CLI grammar: launch syntax, wrapper options and reserved command words (spec §5; design §4).

use std::ffi::{OsStr, OsString};
use std::io::{self, Write};

use clap::{Args, Parser, Subcommand};

use crate::adapter::{self, PlannedLaunch};
use crate::config::{AppRoot, Config};
use crate::error::{Error, Result};
use crate::launch::{self, LaunchOutcome};
use crate::name::{AgentId, Platform, ProfileName, is_reserved_word};
use crate::output;
use crate::resolve;

const LAUNCH_USAGE: &str = "\
Usage: agent-profile <agent> <profile> [--dry-run] [--verbose] [-- <agent args>...]

Launch <agent> with <profile>. Everything after `--` is passed to the agent unchanged.

Options:
  --dry-run   Show what would be launched, without launching or creating anything
  --verbose   Print the launch report to stderr before launching
  -h, --help  Print this help
  -V, --version  Print the version
";

/// Select and launch profiles for multiple coding agents.
#[derive(Parser)]
#[command(
    name = "agent-profile",
    version,
    disable_help_subcommand = true,
    arg_required_else_help = true,
    override_usage = "agent-profile <agent> <profile> [--dry-run] [--verbose] [-- <agent args>...]\n       \
                      agent-profile <command>"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Every reserved word of spec §5.3 is a subcommand that is not implemented in SP1.
#[derive(Subcommand)]
enum Command {
    /// Not yet implemented.
    Agents(Rest),
    /// Not yet implemented.
    Profiles(Rest),
    /// Not yet implemented.
    Status(Rest),
    /// Not yet implemented.
    List(Rest),
    /// Not yet implemented.
    Create(Rest),
    /// Not yet implemented.
    Delete(Rest),
    /// Not yet implemented.
    Current(Rest),
    /// Not yet implemented.
    Resolve(Rest),
    /// Not yet implemented.
    Doctor(Rest),
    /// Not yet implemented.
    Link(Rest),
    /// Not yet implemented.
    Unlink(Rest),
    /// Not yet implemented.
    Repositories(Rest),
    /// Not yet implemented.
    Completions(Rest),
    #[command(external_subcommand)]
    Agent(Vec<OsString>),
}

#[derive(Args)]
#[command(disable_help_flag = true)]
struct Rest {
    #[arg(hide = true, trailing_var_arg = true, allow_hyphen_values = true)]
    _rest: Vec<OsString>,
}

impl Command {
    fn reserved_name(&self) -> Option<&'static str> {
        Some(match self {
            Command::Agents(_) => "agents",
            Command::Profiles(_) => "profiles",
            Command::Status(_) => "status",
            Command::List(_) => "list",
            Command::Create(_) => "create",
            Command::Delete(_) => "delete",
            Command::Current(_) => "current",
            Command::Resolve(_) => "resolve",
            Command::Doctor(_) => "doctor",
            Command::Link(_) => "link",
            Command::Unlink(_) => "unlink",
            Command::Repositories(_) => "repositories",
            Command::Completions(_) => "completions",
            Command::Agent(_) => return None,
        })
    }
}

/// Runs the CLI and returns the process exit code. This is the only place an exit code is decided.
pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return error.exit_code();
        }
    };
    if let Some(name) = cli.command.reserved_name() {
        return report(Error::NotYetImplemented { command: name.to_owned() });
    }
    let Command::Agent(argv) = cli.command else {
        unreachable!("reserved commands returned above")
    };
    match run_agent(argv) {
        Ok(code) => code,
        Err(error) => report(error),
    }
}

fn report(error: Error) -> i32 {
    let _ = writeln!(io::stderr(), "agent-profile: error: {error}");
    error.exit_code()
}

/// The outcome of the design §4.2 rule 4 checks.
#[derive(Debug, PartialEq, Eq)]
enum Invocation {
    Help,
    Version,
    Launch {
        agent: String,
        profile: Option<String>,
        dry_run: bool,
        verbose: bool,
        opaque: Vec<OsString>,
    },
}

/// Design §4.2 rules 3-4: the first `--` cut, then the ordered checks.
fn split(argv: Vec<OsString>, known: &[&str]) -> Result<Invocation> {
    let mut argv = argv.into_iter();
    let agent_word = argv.next().unwrap_or_default();
    let rest: Vec<OsString> = argv.collect();
    let cut = rest.iter().position(|arg| arg == "--");
    let (pre, opaque) = match cut {
        Some(index) => (rest[..index].to_vec(), rest[index + 1..].to_vec()),
        None => (rest, Vec::new()),
    };
    let is_option = |arg: &OsStr| arg.as_encoded_bytes().first() == Some(&b'-');
    let options: Vec<&OsString> = pre.iter().filter(|arg| is_option(arg)).collect();
    let bare: Vec<&OsString> = pre.iter().filter(|arg| !is_option(arg)).collect();

    // 1-2. Help and version.
    if options.iter().any(|arg| *arg == "-h" || *arg == "--help") {
        return Ok(Invocation::Help);
    }
    if options.iter().any(|arg| *arg == "-V" || *arg == "--version") {
        return Ok(Invocation::Version);
    }
    // 3. Unknown agent (exact, case-sensitive).
    let agent = match agent_word.to_str() {
        Some(agent) if known.contains(&agent) => agent.to_owned(),
        _ => {
            return Err(Error::UnknownAgent {
                agent: agent_word.to_string_lossy().into_owned(),
                known: known.iter().map(|agent| (*agent).to_owned()).collect(),
                unknown_configured: Vec::new(),
            });
        }
    };
    // 4. A reserved first bare word routes to a not-yet-implemented agent-scoped command.
    if let Some(word) = bare.first().and_then(|word| word.to_str())
        && is_reserved_word(word)
    {
        return Err(Error::NotYetImplemented { command: format!("{agent} {word}") });
    }
    // 5. Unknown options.
    if let Some(bad) = options
        .iter()
        .find(|arg| !matches!(arg.to_str(), Some("--dry-run" | "--verbose" | "--json")))
    {
        return Err(Error::Usage {
            message: format!(
                "unknown option {:?}; agent arguments must follow `--`",
                bad.to_string_lossy()
            ),
        });
    }
    // 6. At most one bare word.
    if bare.len() > 1 {
        return Err(Error::Usage { message: "agent arguments must follow `--`".to_owned() });
    }
    // 7. JSON output is not implemented yet.
    if options.iter().any(|arg| *arg == "--json") {
        return Err(Error::NotYetImplemented { command: "--json".to_owned() });
    }
    // 8. The profile word must be UTF-8.
    let profile = match bare.first() {
        Some(word) => Some(
            word.to_str()
                .ok_or_else(|| Error::Usage {
                    message: "the profile name is not valid UTF-8".to_owned(),
                })?
                .to_owned(),
        ),
        None => None,
    };
    // 9. A launch.
    Ok(Invocation::Launch {
        agent,
        profile,
        dry_run: options.iter().any(|arg| *arg == "--dry-run"),
        verbose: options.iter().any(|arg| *arg == "--verbose"),
        opaque,
    })
}

fn run_agent(argv: Vec<OsString>) -> Result<i32> {
    let known = adapter::known_agents();
    let invocation =
        split(argv, &known).map_err(|error| with_unknown_configured(error, None, &known))?;
    let (agent, profile, dry_run, verbose, opaque) = match invocation {
        Invocation::Help => {
            write_out(LAUNCH_USAGE)?;
            return Ok(0);
        }
        Invocation::Version => {
            write_out(&format!("agent-profile {}\n", env!("CARGO_PKG_VERSION")))?;
            return Ok(0);
        }
        Invocation::Launch { agent, profile, dry_run, verbose, opaque } => {
            (agent, profile, dry_run, verbose, opaque)
        }
    };

    // Design §5.1 steps 2-4.
    let profile = profile
        .map(|name| {
            ProfileName::parse(&name, Platform::host())
                .map_err(|reason| Error::InvalidProfileName { name, reason })
        })
        .transpose()?;
    let root = AppRoot::resolve()?;
    let config = Config::load(&root)?;
    let agent = AgentId::parse(&agent).expect("known agents are valid agent ids");
    let resolution = resolve::resolve(agent.clone(), profile);
    if resolution.profile.is_none() {
        return Err(Error::NoProfile { agent: agent.to_string() });
    }

    // Step 5.
    let path_var = std::env::var_os("PATH");
    let planned = adapter::plan(&agent, &resolution, &root, &config, opaque, path_var.as_deref())
        .map_err(|error| with_unknown_configured(error, Some(&config), &known))?;

    // Step 6.
    if dry_run {
        let lines = output::report_lines(&planned, &resolution);
        write_out(&lines.iter().map(|line| format!("{line}\n")).collect::<String>())?;
        return Ok(0);
    }

    // Step 7. The verbose report is built after lazy creation, so it never says "(would be created)".
    adapter::ensure_profile_dir(&planned)?;
    let planned = PlannedLaunch { profile_dir_exists: true, ..planned };
    if verbose {
        let mut stderr = io::stderr();
        for line in &output::report_lines(&planned, &resolution) {
            let _ = writeln!(stderr, "agent-profile: {line}");
        }
        let _ = stderr.flush();
    }
    match launch::launch(&planned.plan, verbose)? {
        LaunchOutcome::Exited(code) => Ok(code),
        LaunchOutcome::ReplacedProcess => Ok(0),
    }
}

fn write_out(text: &str) -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .write_all(text.as_bytes())
        .and_then(|()| stdout.flush())
        .map_err(|source| Error::Io { context: "could not write to stdout".to_owned(), source })
}

/// Fills `unknown_configured` on `UnknownAgent` and `AgentNotInstalled`. With no configuration at hand,
/// one best-effort load is attempted; its failure leaves the list empty (design §4.2 check 3).
fn with_unknown_configured(error: Error, config: Option<&Config>, known: &[&str]) -> Error {
    let unknown = |config: Option<&Config>| -> Vec<String> {
        let loaded;
        let config = match config {
            Some(config) => Some(config),
            None => {
                loaded = AppRoot::resolve().ok().and_then(|root| Config::load(&root).ok());
                loaded.as_ref()
            }
        };
        config
            .map(|config| {
                config
                    .configured_agents()
                    .filter(|id| !known.contains(id))
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    };
    match error {
        Error::UnknownAgent { agent, known: names, .. } => {
            Error::UnknownAgent { agent, known: names, unknown_configured: unknown(config) }
        }
        Error::AgentNotInstalled { agent, reason, .. } => {
            Error::AgentNotInstalled { agent, reason, unknown_configured: unknown(config) }
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWN: &[&str] = &["fake"];

    fn args(items: &[&str]) -> Vec<OsString> {
        items.iter().map(OsString::from).collect()
    }

    fn launch(profile: Option<&str>, dry_run: bool, verbose: bool, opaque: &[&str]) -> Invocation {
        Invocation::Launch {
            agent: "fake".to_owned(),
            profile: profile.map(str::to_owned),
            dry_run,
            verbose,
            opaque: args(opaque),
        }
    }

    fn split_ok(items: &[&str]) -> Invocation {
        split(args(items), KNOWN).unwrap()
    }

    fn split_err(items: &[&str]) -> Error {
        split(args(items), KNOWN).unwrap_err()
    }

    #[test]
    fn cut_is_at_the_first_double_dash_and_later_ones_are_opaque() {
        assert_eq!(
            split_ok(&["fake", "work", "--", "--dry-run", "--", "a b"]),
            launch(Some("work"), false, false, &["--dry-run", "--", "a b"])
        );
        assert_eq!(split_ok(&["fake", "--", "work"]), launch(None, false, false, &["work"]));
        assert_eq!(
            split_ok(&["fake", "work", "--", "--help"]),
            launch(Some("work"), false, false, &["--help"])
        );
    }

    #[test]
    fn options_may_appear_anywhere_before_the_cut() {
        assert_eq!(
            split_ok(&["fake", "--dry-run", "work", "--verbose"]),
            launch(Some("work"), true, true, &[])
        );
        assert_eq!(split_ok(&["fake"]), launch(None, false, false, &[]));
    }

    #[test]
    fn help_and_version_win_first() {
        assert_eq!(split_ok(&["zzz", "--help"]), Invocation::Help);
        assert_eq!(split_ok(&["fake", "create", "-h"]), Invocation::Help);
        assert_eq!(split_ok(&["fake", "-h", "--", "x"]), Invocation::Help);
        assert_eq!(split_ok(&["zzz", "--version"]), Invocation::Version);
        assert_eq!(split_ok(&["fake", "work", "-V"]), Invocation::Version);
    }

    #[test]
    fn unknown_agent_is_exact_and_precedes_reserved_words() {
        for items in [&["zzz", "work"][..], &["Fake", "work"], &["zzz", "create"]] {
            assert!(matches!(split_err(items), Error::UnknownAgent { .. }), "{items:?}");
        }
    }

    #[test]
    fn reserved_first_bare_word_ignores_everything_else() {
        for items in [
            &["fake", "create", "work"][..],
            &["fake", "CREATE", "--bogus"],
            &["fake", "--bogus", "link"],
        ] {
            assert!(matches!(split_err(items), Error::NotYetImplemented { .. }), "{items:?}");
        }
        match split_err(&["fake", "Create", "x"]) {
            Error::NotYetImplemented { command } => assert_eq!(command, "fake Create"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn usage_errors() {
        for items in [
            &["fake", "work", "--bogus"][..],
            &["fake", "-"],
            &["fake", "--dry-run=yes", "work"],
            &["fake", "work", "extra"],
            &["fake", "", "extra"],
            &["fake", "work", "--json", "extra"],
        ] {
            assert!(matches!(split_err(items), Error::Usage { .. }), "{items:?}");
        }
    }

    #[test]
    fn unknown_option_is_reported_before_extra_bare_words() {
        match split_err(&["fake", "work", "extra", "--bogus"]) {
            Error::Usage { message } => {
                assert!(message.starts_with("unknown option \"--bogus\""), "{message}")
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn json_is_not_yet_implemented() {
        for items in [&["fake", "work", "--json"][..], &["fake", "--json"]] {
            match split_err(items) {
                Error::NotYetImplemented { command } => assert_eq!(command, "--json"),
                other => panic!("{items:?}: {other:?}"),
            }
        }
    }

    #[test]
    fn empty_string_is_a_profile_word() {
        assert_eq!(split_ok(&["fake", ""]), launch(Some(""), false, false, &[]));
    }

    #[cfg(unix)]
    fn non_utf8() -> OsString {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(vec![0x66, 0xff])
    }

    #[cfg(windows)]
    fn non_utf8() -> OsString {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(&[0x66, 0xD800])
    }

    #[test]
    fn non_utf8_words() {
        let bare = vec![OsString::from("fake"), non_utf8()];
        assert!(matches!(split(bare, KNOWN), Err(Error::Usage { .. })));
        let agent = vec![non_utf8(), OsString::from("work")];
        assert!(matches!(split(agent, KNOWN), Err(Error::UnknownAgent { .. })));
        let opaque = vec![OsString::from("fake"), "work".into(), "--".into(), non_utf8()];
        match split(opaque, KNOWN).unwrap() {
            Invocation::Launch { opaque, .. } => assert_eq!(opaque, vec![non_utf8()]),
            other => panic!("{other:?}"),
        }
    }
}
