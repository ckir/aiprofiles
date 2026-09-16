//! CLI grammar: launch syntax, wrapper options and reserved command words (spec §5; SP1 design §4) and the
//! repository commands (SP3 design §7).

use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::adapter::{self, PlanContext, ProfilePresence};
use crate::config::{self, AppRoot, Config};
use crate::error::{Error, Result};
use crate::launch::{self, LaunchOutcome};
use crate::name::{AgentId, Platform, ProfileName, RESERVED_WORDS, is_reserved_word};
use crate::output::{self, ReportMode};
use crate::repo::{self, Discovery};
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

const COMMANDS_HELP: &str = "\
Repository commands:
  agent-profile <agent> current [--repo <path>]
  agent-profile <agent> resolve [--repo <path>]
  agent-profile [<agent>] status [--repo <path>]
  agent-profile [<agent>] link <profile> [--repo <path>]
  agent-profile [<agent>] unlink [--repo <path>]";

/// Select and launch profiles for multiple coding agents.
#[derive(Parser)]
#[command(
    name = "agent-profile",
    version,
    disable_help_subcommand = true,
    arg_required_else_help = true,
    override_usage = "agent-profile <agent> <profile> [--dry-run] [--verbose] [-- <agent args>...]\n       \
                      agent-profile <command>",
    after_help = COMMANDS_HELP
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// The reserved words of spec §5.3 that are not implemented yet. `status`, `link`, `unlink`, `resolve` and
/// `current` are dispatched before Clap (SP3 design §7.2).
#[derive(Subcommand)]
enum Command {
    /// Not yet implemented.
    Agents(Rest),
    /// Not yet implemented.
    Profiles(Rest),
    /// Not yet implemented.
    List(Rest),
    /// Not yet implemented.
    Create(Rest),
    /// Not yet implemented.
    Delete(Rest),
    /// Not yet implemented.
    Doctor(Rest),
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
            Command::List(_) => "list",
            Command::Create(_) => "create",
            Command::Delete(_) => "delete",
            Command::Doctor(_) => "doctor",
            Command::Repositories(_) => "repositories",
            Command::Completions(_) => "completions",
            Command::Agent(_) => return None,
        })
    }
}

/// The repository commands (SP3 design §7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandWord {
    Current,
    Resolve,
    Status,
    Link,
    Unlink,
}

impl CommandWord {
    /// The exact lower-case spelling only.
    fn parse(word: &str) -> Option<CommandWord> {
        Some(match word {
            "current" => CommandWord::Current,
            "resolve" => CommandWord::Resolve,
            "status" => CommandWord::Status,
            "link" => CommandWord::Link,
            "unlink" => CommandWord::Unlink,
            _ => return None,
        })
    }

    fn as_str(self) -> &'static str {
        match self {
            CommandWord::Current => "current",
            CommandWord::Resolve => "resolve",
            CommandWord::Status => "status",
            CommandWord::Link => "link",
            CommandWord::Unlink => "unlink",
        }
    }

    /// The command's help text (SP3 design §7.3).
    fn usage(self) -> &'static str {
        match self {
            CommandWord::Current => {
                "Usage: agent-profile <agent> current [--repo <path>]\n\
                 Print the profile agent-profile would select for <agent> here.\n\n  \
                 --repo <path>  Use the repository at <path> instead of the current directory\n"
            }
            CommandWord::Resolve => {
                "Usage: agent-profile <agent> resolve [--repo <path>]\n\
                 Show the profile, where it comes from, and the repository.\n\n  \
                 --repo <path>  Use the repository at <path> instead of the current directory\n"
            }
            CommandWord::Status => {
                "Usage: agent-profile [<agent>] status [--repo <path>]\n\
                 Show the repository, its mappings, the default profile and what each agent resolves to.\n\n  \
                 --repo <path>  Use the repository at <path> instead of the current directory\n"
            }
            CommandWord::Link => {
                "Usage: agent-profile [<agent>] link <profile> [--repo <path>]\n\
                 Map this repository (or only <agent> in it) to <profile>.\n\n  \
                 --repo <path>  Use the repository at <path> instead of the current directory\n"
            }
            CommandWord::Unlink => {
                "Usage: agent-profile [<agent>] unlink [--repo <path>]\n\
                 Remove this repository's mapping (or only <agent>'s). With --repo, removes the mapping stored \
                 for that path.\n\n  \
                 --repo <path>  Use the repository at <path> instead of the current directory\n"
            }
        }
    }
}

/// Runs the CLI and returns the process exit code. This is the only place an exit code is decided.
pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let args: Vec<OsString> = args.into_iter().collect();
    // Clap drops a leading `--` from a subcommand's trailing arguments, so the repository commands are
    // dispatched on the raw first argument (SP3 design §7.2).
    if let Some(command) = args.get(1).and_then(|word| word.to_str()).and_then(CommandWord::parse) {
        let rest = args[2..].to_vec();
        return match split_top_level(command, rest).and_then(execute) {
            Ok(code) => code,
            Err(error) => report(error),
        };
    }
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
    let known = adapter::known_agents();
    match split(argv, &known)
        .map_err(|error| with_unknown_configured(error, None, &known))
        .and_then(execute)
    {
        Ok(code) => code,
        Err(error) => report(error),
    }
}

fn report(error: Error) -> i32 {
    let _ = writeln!(io::stderr(), "agent-profile: error: {error}");
    error.exit_code()
}

/// The outcome of the argument checks (SP1 design §4.2, SP3 design §7.2).
#[derive(Debug, PartialEq, Eq)]
enum Invocation {
    Help,
    CommandHelp(CommandWord),
    Version,
    Launch {
        agent: String,
        profile: Option<String>,
        dry_run: bool,
        verbose: bool,
        opaque: Vec<OsString>,
    },
    Command {
        agent: Option<String>,
        command: CommandWord,
        profile: Option<String>,
        repo: Option<OsString>,
    },
}

/// One token before the first `--`, after `--repo` binding (SP3 design §7.2 step 1).
#[derive(Debug)]
enum Token {
    Repo(OsString),
    Option(OsString),
    Bare(OsString),
}

fn usage(message: impl Into<String>) -> Error {
    Error::Usage { message: message.into() }
}

/// Splits at the first `--`: the tokens before it, and the opaque arguments when there is one.
fn cut(args: Vec<OsString>) -> (Vec<OsString>, Option<Vec<OsString>>) {
    match args.iter().position(|arg| arg == "--") {
        Some(index) => (args[..index].to_vec(), Some(args[index + 1..].to_vec())),
        None => (args, None),
    }
}

/// Step 1: `--repo=<v>` carries its value; a bare `--repo` consumes the next token whatever it is.
fn bind(pre: Vec<OsString>) -> Result<Vec<Token>> {
    let mut tokens = Vec::with_capacity(pre.len());
    let mut pre = pre.into_iter();
    while let Some(arg) = pre.next() {
        if arg == "--repo" {
            let value = pre.next().ok_or_else(|| usage("`--repo` needs a path"))?;
            tokens.push(Token::Repo(value));
        } else if let Some(value) = repo_value(&arg) {
            tokens.push(Token::Repo(value));
        } else if arg.as_encoded_bytes().first() == Some(&b'-') {
            tokens.push(Token::Option(arg));
        } else {
            tokens.push(Token::Bare(arg));
        }
    }
    Ok(tokens)
}

const REPO_EQUALS: &str = "--repo=";

/// The value of a `--repo=<v>` token, which may be non-UTF-8.
#[cfg(unix)]
fn repo_value(arg: &OsStr) -> Option<OsString> {
    use std::os::unix::ffi::OsStrExt;
    let bytes = arg.as_bytes();
    bytes
        .starts_with(REPO_EQUALS.as_bytes())
        .then(|| OsStr::from_bytes(&bytes[REPO_EQUALS.len()..]).to_owned())
}

/// The value of a `--repo=<v>` token, which may be non-UTF-8.
#[cfg(windows)]
fn repo_value(arg: &OsStr) -> Option<OsString> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    let units: Vec<u16> = arg.encode_wide().collect();
    let prefix: Vec<u16> = OsStr::new(REPO_EQUALS).encode_wide().collect();
    units.starts_with(&prefix).then(|| OsString::from_wide(&units[prefix.len()..]))
}

fn options(tokens: &[Token]) -> impl Iterator<Item = &OsString> {
    tokens.iter().filter_map(|token| match token {
        Token::Option(option) => Some(option),
        _ => None,
    })
}

fn bare_words(tokens: &[Token]) -> Vec<&OsString> {
    tokens
        .iter()
        .filter_map(|token| match token {
            Token::Bare(word) => Some(word),
            _ => None,
        })
        .collect()
}

fn has_help(tokens: &[Token]) -> bool {
    options(tokens).any(|option| option == "-h" || option == "--help")
}

fn has_version(tokens: &[Token]) -> bool {
    options(tokens).any(|option| option == "-V" || option == "--version")
}

/// Only the part before `=` is echoed: the value may be a secret meant for the agent (spec §36).
fn option_name(option: &OsStr) -> String {
    let shown = option.to_string_lossy();
    shown.split_once('=').map_or(&*shown, |(name, _)| name).to_owned()
}

/// `agent-profile status|link|unlink|resolve|current …` (SP3 design §7.2).
fn split_top_level(command: CommandWord, rest: Vec<OsString>) -> Result<Invocation> {
    let (pre, opaque) = cut(rest);
    let tokens = bind(pre)?;
    if has_help(&tokens) {
        return Ok(Invocation::CommandHelp(command));
    }
    if has_version(&tokens) {
        return Ok(Invocation::Version);
    }
    if matches!(command, CommandWord::Resolve | CommandWord::Current) {
        let name = command.as_str();
        return Err(usage(format!("`{name}` needs an agent: agent-profile <agent> {name}")));
    }
    let bare = bare_words(&tokens);
    validate_command(None, command, &tokens, &bare, opaque.is_some())
}

/// `agent-profile <agent> …`: SP1 design §4.2 rules 3-4 as replaced by SP3 design §7.2.
fn split(argv: Vec<OsString>, known: &[&str]) -> Result<Invocation> {
    let mut argv = argv.into_iter();
    let agent_word = argv.next().unwrap_or_default();
    let (pre, opaque) = cut(argv.collect());
    let tokens = bind(pre)?;
    let bare = bare_words(&tokens);
    let first = bare.first().and_then(|word| word.to_str());

    // 2. Help and version.
    if has_help(&tokens) {
        return Ok(match first.and_then(CommandWord::parse) {
            Some(command) => Invocation::CommandHelp(command),
            None => Invocation::Help,
        });
    }
    if has_version(&tokens) {
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
    // 4. A reserved first bare word.
    if let Some(word) = first
        && is_reserved_word(word)
    {
        if let Some(command) = CommandWord::parse(word) {
            return validate_command(Some(agent), command, &tokens, &bare[1..], opaque.is_some());
        }
        if RESERVED_WORDS.contains(&word) {
            return Err(Error::NotYetImplemented { command: format!("{agent} {word}") });
        }
        return Err(usage(format!(
            "command words are lower case: `{}`",
            word.to_ascii_lowercase()
        )));
    }
    // 6. A launch. A `--repo` binding is an unknown option here.
    let launch_options: Vec<OsString> = tokens
        .iter()
        .filter_map(|token| match token {
            Token::Option(option) => Some(option.clone()),
            Token::Repo(_) => Some(OsString::from("--repo")),
            Token::Bare(_) => None,
        })
        .collect();
    if let Some(bad) = launch_options
        .iter()
        .find(|arg| !matches!(arg.to_str(), Some("--dry-run" | "--verbose" | "--json")))
    {
        return Err(usage(format!(
            "unknown option {:?}; agent arguments must follow `--`",
            option_name(bad)
        )));
    }
    if bare.len() > 1 {
        return Err(usage("agent arguments must follow `--`"));
    }
    if launch_options.iter().any(|arg| arg == "--json") {
        return Err(Error::NotYetImplemented { command: "--json".to_owned() });
    }
    let profile = match bare.first() {
        Some(word) => Some(
            word.to_str().ok_or_else(|| usage("the profile name is not valid UTF-8"))?.to_owned(),
        ),
        None => None,
    };
    Ok(Invocation::Launch {
        agent,
        profile,
        dry_run: launch_options.iter().any(|arg| arg == "--dry-run"),
        verbose: launch_options.iter().any(|arg| arg == "--verbose"),
        opaque: opaque.unwrap_or_default(),
    })
}

/// Step 5: `bare` holds the bare words after the command word.
fn validate_command(
    agent: Option<String>,
    command: CommandWord,
    tokens: &[Token],
    bare: &[&OsString],
    has_cut: bool,
) -> Result<Invocation> {
    let name = command.as_str();
    if has_cut {
        return Err(usage(format!("`{name}` takes no agent arguments; remove `--`")));
    }
    let repos: Vec<&OsString> = tokens
        .iter()
        .filter_map(|token| match token {
            Token::Repo(repo) => Some(repo),
            _ => None,
        })
        .collect();
    if repos.len() > 1 {
        return Err(usage("`--repo` may be given only once"));
    }
    if repos.first().is_some_and(|repo| repo.is_empty()) {
        return Err(usage("`--repo` needs a non-empty path"));
    }
    if matches!(command, CommandWord::Status | CommandWord::Resolve)
        && options(tokens).any(|option| option == "--json")
    {
        return Err(Error::NotYetImplemented { command: "--json".to_owned() });
    }
    if let Some(option) = options(tokens).next() {
        return Err(usage(format!("unknown option {:?} for `{name}`", option_name(option))));
    }
    let allowed = usize::from(command == CommandWord::Link);
    if command == CommandWord::Link && bare.is_empty() {
        return Err(usage("`link` needs a profile: agent-profile [<agent>] link <profile>"));
    }
    if bare.len() > allowed {
        return Err(usage(match command {
            CommandWord::Link => "`link` takes one profile".to_owned(),
            _ => format!("`{name}` takes no arguments"),
        }));
    }
    let profile = match bare.first() {
        Some(word) => Some(
            word.to_str().ok_or_else(|| usage("the profile name is not valid UTF-8"))?.to_owned(),
        ),
        None => None,
    };
    Ok(Invocation::Command {
        agent,
        command,
        profile,
        repo: repos.first().map(|repo| (*repo).clone()),
    })
}

fn execute(invocation: Invocation) -> Result<i32> {
    match invocation {
        Invocation::Help => {
            write_out(LAUNCH_USAGE)?;
            Ok(0)
        }
        Invocation::CommandHelp(command) => {
            write_out(command.usage())?;
            Ok(0)
        }
        Invocation::Version => {
            write_out(&format!("agent-profile {}\n", env!("CARGO_PKG_VERSION")))?;
            Ok(0)
        }
        Invocation::Launch { agent, profile, dry_run, verbose, opaque } => {
            run_launch(agent, profile, dry_run, verbose, opaque)
        }
        Invocation::Command { agent, command, profile, repo } => {
            run_command(agent, command, profile, repo)
        }
    }
}

/// Whether a launch runs discovery (SP3 design §7.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaunchDiscovery {
    /// No profile word: discovery selects the profile, and its errors exit 4.
    Required,
    /// An explicit profile with a report: discovery only fills the report, and its errors are ignored.
    ReportOnly,
    /// An explicit profile without a report: nothing is discovered.
    Skipped,
}

fn launch_discovery(explicit: bool, dry_run: bool, verbose: bool) -> LaunchDiscovery {
    match (explicit, dry_run || verbose) {
        (false, _) => LaunchDiscovery::Required,
        (true, true) => LaunchDiscovery::ReportOnly,
        (true, false) => LaunchDiscovery::Skipped,
    }
}

fn parse_profile(name: String) -> Result<ProfileName> {
    ProfileName::parse(&name, Platform::host())
        .map_err(|reason| Error::InvalidProfileName { name, reason })
}

/// The directory a `--repo` value is joined to. An absolute `--repo` needs no current directory, so a command
/// given one still works from a working directory that no longer exists (design §6.2, §10 "`--repo` is the
/// explicit override").
fn base_dir(repo: Option<&OsStr>) -> Result<PathBuf> {
    match repo {
        Some(repo) if std::path::Path::new(repo).is_absolute() => Ok(PathBuf::new()),
        _ => current_dir(),
    }
}

fn current_dir() -> Result<PathBuf> {
    std::env::current_dir().map_err(|error| Error::Repository {
        path: PathBuf::from("."),
        reason: format!("cannot resolve the directory: {error}"),
    })
}

fn no_profile(agent: &AgentId, root: &AppRoot, discovery: &Discovery) -> Error {
    Error::NoProfile {
        agent: agent.to_string(),
        config_file: root.config_path(),
        in_repository: matches!(discovery, Discovery::Repository(_)),
    }
}

fn run_launch(
    agent: String,
    profile: Option<String>,
    dry_run: bool,
    verbose: bool,
    opaque: Vec<OsString>,
) -> Result<i32> {
    // SP1 design §5.1 steps 2-4, SP3 design §7.6.
    let known = adapter::known_agents();
    let profile = profile.map(parse_profile).transpose()?;
    let root = AppRoot::resolve()?;
    let config = Config::load(&root)?;
    let agent = AgentId::parse(&agent).expect("known agents are valid agent ids");
    let discovery = match launch_discovery(profile.is_some(), dry_run, verbose) {
        LaunchDiscovery::Required => repo::discover(&current_dir()?)?,
        LaunchDiscovery::ReportOnly => std::env::current_dir()
            .ok()
            .and_then(|cwd| repo::discover(&cwd).ok())
            .unwrap_or(Discovery::NotInRepository),
        LaunchDiscovery::Skipped => Discovery::NotInRepository,
    };
    let resolution = resolve::resolve(agent.clone(), profile, &config, &discovery);
    let Some(profile) = resolution.profile.as_ref() else {
        return Err(no_profile(&agent, &root, &discovery));
    };

    // SP2 design §4.3 step 1: parsing already refused unknown agents.
    let adapter = adapter::lookup(agent.as_str()).expect("known agents have an adapter");
    // Step 2: a pure conflict scan, before executable discovery.
    adapter::check_conflicts(adapter.metadata(), &opaque)?;
    // Step 3: discovery, the case-only-twin check, the plan.
    let path_var = std::env::var_os("PATH");
    let ctx = PlanContext {
        profile,
        root: &root,
        config: &config,
        args: &opaque,
        path_var: path_var.as_deref(),
    };
    let planned = adapter
        .plan(&ctx)
        .map_err(|error| with_unknown_configured(error, Some(&config), &known))?;

    // Step 4.
    if dry_run {
        let lines = output::report_lines(&planned, &resolution, ReportMode::DryRun);
        write_out(&lines.iter().map(|line| format!("{line}\n")).collect::<String>())?;
        return Ok(0);
    }

    // Step 5. The verbose report is rendered after initialization, so it never says "(would be created)".
    adapter.initialize(&planned)?;
    if verbose {
        let mut stderr = io::stderr();
        for line in &output::report_lines(&planned, &resolution, ReportMode::Verbose) {
            let _ = writeln!(stderr, "agent-profile: {line}");
        }
        let _ = stderr.flush();
    }
    match launch::launch(&planned.plan, verbose)? {
        LaunchOutcome::Exited(code) => Ok(code),
        LaunchOutcome::ReplacedProcess => Ok(0),
    }
}

/// SP3 design §7.4 steps 2-5.
fn run_command(
    agent: Option<String>,
    command: CommandWord,
    profile: Option<String>,
    repo: Option<OsString>,
) -> Result<i32> {
    let profile = profile.map(parse_profile).transpose()?;
    let root = AppRoot::resolve()?;
    let config = Config::load(&root)?;
    let agent =
        agent.map(|agent| AgentId::parse(&agent).expect("known agents are valid agent ids"));
    let cwd = base_dir(repo.as_deref())?;
    if command == CommandWord::Unlink {
        return run_unlink(&root, &config, agent.as_ref(), &cwd, repo.as_deref());
    }
    let start = match &repo {
        Some(repo) => cwd.join(repo),
        None => cwd,
    };
    let discovery = repo::discover(&start)?;
    let lines = match command {
        CommandWord::Current => {
            let agent = agent.expect("`current` has an agent");
            let resolution = resolve::resolve(agent.clone(), None, &config, &discovery);
            let Some(profile) = resolution.profile else {
                return Err(no_profile(&agent, &root, &discovery));
            };
            vec![profile.to_string()]
        }
        CommandWord::Resolve => {
            let agent = agent.expect("`resolve` has an agent");
            let resolution = resolve::resolve(agent.clone(), None, &config, &discovery);
            write_lines(&output::resolve_lines(&resolution))?;
            if resolution.profile.is_none() {
                return Err(no_profile(&agent, &root, &discovery));
            }
            return Ok(0);
        }
        CommandWord::Status => {
            let agents: Vec<AgentId> = match &agent {
                Some(agent) => vec![agent.clone()],
                None => adapter::known_agents()
                    .into_iter()
                    .map(|id| AgentId::parse(id).expect("known agents are valid agent ids"))
                    .collect(),
            };
            let rows: Vec<_> = agents
                .into_iter()
                .map(|id| {
                    let resolution = resolve::resolve(id, None, &config, &discovery);
                    let presence = match (&agent, &resolution.profile) {
                        (Some(agent), Some(profile)) => Some(presence(&root, agent, profile)),
                        _ => None,
                    };
                    (resolution, presence)
                })
                .collect();
            output::status_lines(&discovery, &config, &rows)
        }
        CommandWord::Link => {
            let Discovery::Repository(repository) = discovery else {
                return Err(Error::Repository {
                    path: repo::canonical(&start).unwrap_or(start),
                    reason: "not inside a Git repository".to_owned(),
                });
            };
            let profile = profile.expect("`link` has a profile");
            let outcome = config::link(&root, &repository, agent.as_ref(), &profile)?;
            let mut lines =
                vec![output::link_line(&outcome, agent.as_ref(), &repository, &profile)];
            if let Some(agent) = &agent {
                let adapter =
                    adapter::lookup(agent.as_str()).expect("known agents have an adapter");
                if adapter.presence(&root, &profile) == ProfilePresence::Absent {
                    lines.push(output::note(&format!(
                        "profile {profile} has not been launched with {agent} yet"
                    )));
                }
            }
            lines
        }
        CommandWord::Unlink => unreachable!("unlink returned above"),
    };
    write_lines(&lines)?;
    Ok(0)
}

/// `unlink` chooses its keys without discovery when `--repo` is given (SP3 design §6.2).
fn run_unlink(
    root: &AppRoot,
    config: &Config,
    agent: Option<&AgentId>,
    cwd: &std::path::Path,
    repo: Option<&OsStr>,
) -> Result<i32> {
    let keys = match repo {
        Some(repo) => repo::unlink_keys(cwd, repo),
        None => match repo::discover(cwd)? {
            Discovery::Repository(repository) => vec![repository],
            Discovery::NotInRepository => {
                return Err(Error::Repository {
                    path: repo::canonical(cwd).unwrap_or_else(|_| cwd.to_path_buf()),
                    reason: "not inside a Git repository".to_owned(),
                });
            }
        },
    };
    let outcome = config::unlink(root, &keys, agent)?;
    write_lines(&output::unlink_lines(&outcome, agent, config, &keys))?;
    Ok(0)
}

/// The `presence:` value of `<agent> status` (SP3 design §7.5).
fn presence(root: &AppRoot, agent: &AgentId, profile: &ProfileName) -> String {
    if let Err(Error::ProfileCaseConflict { existing, .. }) =
        config::check_case_twins(root, profile)
    {
        return format!("conflicts with {existing}");
    }
    let adapter = adapter::lookup(agent.as_str()).expect("known agents have an adapter");
    match adapter.presence(root, profile) {
        ProfilePresence::Materialized => "materialized",
        ProfilePresence::Known => "known",
        ProfilePresence::Absent => "absent",
    }
    .to_owned()
}

fn write_lines(lines: &[String]) -> Result<()> {
    write_out(&lines.iter().map(|line| format!("{line}\n")).collect::<String>())
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

    const KNOWN: &[&str] = &["fake", "claude"];

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

    fn command(
        agent: Option<&str>,
        command: CommandWord,
        profile: Option<&str>,
        repo: Option<&str>,
    ) -> Invocation {
        Invocation::Command {
            agent: agent.map(str::to_owned),
            command,
            profile: profile.map(str::to_owned),
            repo: repo.map(OsString::from),
        }
    }

    fn split_ok(items: &[&str]) -> Invocation {
        split(args(items), KNOWN).unwrap()
    }

    fn split_err(items: &[&str]) -> Error {
        split(args(items), KNOWN).unwrap_err()
    }

    /// Runs the top-level dispatch the way `run` does: `items[0]` is the command word.
    fn top(items: &[&str]) -> Result<Invocation> {
        let command = CommandWord::parse(items[0]).expect("a top-level command word");
        split_top_level(command, args(&items[1..]))
    }

    fn usage_message(result: Result<Invocation>) -> String {
        match result {
            Err(Error::Usage { message }) => message,
            other => panic!("{other:?}"),
        }
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
        for items in [&["zzz", "work"][..], &["Fake", "work"], &["zzz", "create"], &["zzz", "link"]]
        {
            assert!(matches!(split_err(items), Error::UnknownAgent { .. }), "{items:?}");
        }
    }

    #[test]
    fn reserved_first_bare_word_routes_by_exact_spelling() {
        match split_err(&["fake", "create", "work"]) {
            Error::NotYetImplemented { command } => assert_eq!(command, "fake create"),
            other => panic!("{other:?}"),
        }
        for (items, word) in [
            (&["fake", "CREATE", "--bogus"][..], "create"),
            (&["fake", "Create", "x"], "create"),
            (&["claude", "LINK"], "link"),
        ] {
            assert_eq!(
                usage_message(split(args(items), KNOWN)),
                format!("command words are lower case: `{word}`"),
                "{items:?}"
            );
        }
        assert_eq!(
            usage_message(split(args(&["fake", "--bogus", "link"]), KNOWN)),
            "unknown option \"--bogus\" for `link`"
        );
    }

    #[test]
    fn agent_scoped_commands_bind_their_words_and_repo() {
        assert_eq!(
            split_ok(&["claude", "current"]),
            command(Some("claude"), CommandWord::Current, None, None)
        );
        assert_eq!(
            split_ok(&["claude", "resolve", "--repo", "../x"]),
            command(Some("claude"), CommandWord::Resolve, None, Some("../x"))
        );
        assert_eq!(
            split_ok(&["claude", "--repo=a=b", "status"]),
            command(Some("claude"), CommandWord::Status, None, Some("a=b"))
        );
        assert_eq!(
            split_ok(&["claude", "link", "work", "--repo", "-dir"]),
            command(Some("claude"), CommandWord::Link, Some("work"), Some("-dir"))
        );
        assert_eq!(
            split_ok(&["claude", "unlink", "--repo", "link"]),
            command(Some("claude"), CommandWord::Unlink, None, Some("link"))
        );
    }

    #[test]
    fn top_level_commands_take_every_raw_token() {
        assert_eq!(top(&["status"]).unwrap(), command(None, CommandWord::Status, None, None));
        assert_eq!(
            top(&["link", "work", "--repo", "r"]).unwrap(),
            command(None, CommandWord::Link, Some("work"), Some("r"))
        );
        assert_eq!(
            top(&["link", "status"]).unwrap(),
            command(None, CommandWord::Link, Some("status"), None)
        );
        assert_eq!(
            top(&["link", "Create"]).unwrap(),
            command(None, CommandWord::Link, Some("Create"), None)
        );
        assert_eq!(
            top(&["unlink", "--repo=/gone"]).unwrap(),
            command(None, CommandWord::Unlink, None, Some("/gone"))
        );
    }

    #[test]
    fn command_help_names_the_command_word() {
        assert_eq!(top(&["link", "-h"]).unwrap(), Invocation::CommandHelp(CommandWord::Link));
        assert_eq!(
            top(&["link", "status", "-h"]).unwrap(),
            Invocation::CommandHelp(CommandWord::Link)
        );
        assert_eq!(top(&["resolve", "-h"]).unwrap(), Invocation::CommandHelp(CommandWord::Resolve));
        assert_eq!(top(&["status", "-V"]).unwrap(), Invocation::Version);
        assert_eq!(
            split_ok(&["claude", "status", "--help"]),
            Invocation::CommandHelp(CommandWord::Status)
        );
        assert_eq!(split_ok(&["claude", "Status", "--help"]), Invocation::Help);
    }

    #[test]
    fn a_consumed_repo_value_is_never_help_an_option_or_a_command_word() {
        assert_eq!(
            split_ok(&["claude", "link", "work", "--repo", "-h"]),
            command(Some("claude"), CommandWord::Link, Some("work"), Some("-h"))
        );
        assert_eq!(
            usage_message(split(args(&["claude", "--repo"]), KNOWN)),
            "`--repo` needs a path"
        );
        assert_eq!(
            usage_message(split(args(&["fake", "-h", "--repo"]), KNOWN)),
            "`--repo` needs a path"
        );
        assert_eq!(
            usage_message(split(args(&["fake", "work", "--repo", "--help"]), KNOWN)),
            "unknown option \"--repo\"; agent arguments must follow `--`"
        );
        assert_eq!(
            usage_message(split(args(&["fake", "--repo", "create"]), KNOWN)),
            "unknown option \"--repo\"; agent arguments must follow `--`"
        );
    }

    #[test]
    fn command_usage_errors_in_order() {
        for (items, message) in [
            (&["status", "--"][..], "`status` takes no agent arguments; remove `--`"),
            (&["unlink", "--", "--repo", "x"], "`unlink` takes no agent arguments; remove `--`"),
            (&["link", "work", "--repo", "a", "--repo=b"], "`--repo` may be given only once"),
            (&["link", "work", "--repo="], "`--repo` needs a non-empty path"),
            (&["link", "work", "--repo", ""], "`--repo` needs a non-empty path"),
            (&["link", "work", "--json"], "unknown option \"--json\" for `link`"),
            (&["unlink", "--dry-run"], "unknown option \"--dry-run\" for `unlink`"),
            (&["status", "--verbose=yes"], "unknown option \"--verbose\" for `status`"),
            (&["link"], "`link` needs a profile: agent-profile [<agent>] link <profile>"),
            (
                &["link", "--repo", "r"],
                "`link` needs a profile: agent-profile [<agent>] link <profile>",
            ),
            (&["link", "work", "extra"], "`link` takes one profile"),
            (&["status", "extra"], "`status` takes no arguments"),
            (&["unlink", "extra"], "`unlink` takes no arguments"),
            (&["resolve"], "`resolve` needs an agent: agent-profile <agent> resolve"),
            (
                &["current", "--repo", "x"],
                "`current` needs an agent: agent-profile <agent> current",
            ),
        ] {
            assert_eq!(usage_message(top(items)), message, "{items:?}");
        }
        for (items, message) in [
            (&["claude", "current", "extra"][..], "`current` takes no arguments"),
            (&["claude", "resolve", "--json", "--bogus"], "`--json` is not yet implemented"),
            (&["claude", "current", "--json"], "unknown option \"--json\" for `current`"),
            (&["claude", "link", "a", "--", "x"], "`link` takes no agent arguments; remove `--`"),
        ] {
            let result = split(args(items), KNOWN);
            let text = match result {
                Err(error @ (Error::Usage { .. } | Error::NotYetImplemented { .. })) => {
                    error.to_string()
                }
                other => panic!("{items:?}: {other:?}"),
            };
            assert_eq!(text, message, "{items:?}");
        }
        assert!(matches!(top(&["status", "--json"]), Err(Error::NotYetImplemented { .. })));
    }

    #[test]
    fn launch_discovery_runs_only_when_it_can_matter() {
        assert_eq!(launch_discovery(false, false, false), LaunchDiscovery::Required);
        assert_eq!(launch_discovery(false, true, true), LaunchDiscovery::Required);
        assert_eq!(launch_discovery(true, true, false), LaunchDiscovery::ReportOnly);
        assert_eq!(launch_discovery(true, false, true), LaunchDiscovery::ReportOnly);
        assert_eq!(launch_discovery(true, false, false), LaunchDiscovery::Skipped);
    }

    #[test]
    fn an_absolute_repo_is_joined_to_nothing_so_no_current_directory_is_needed() {
        let absolute = if cfg!(windows) { r"C:\src\acme" } else { "/src/acme" };
        assert_eq!(base_dir(Some(OsStr::new(absolute))).unwrap(), PathBuf::new());
        assert_eq!(PathBuf::new().join(absolute), PathBuf::from(absolute));
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(base_dir(Some(OsStr::new("relative"))).unwrap(), cwd);
        assert_eq!(base_dir(None).unwrap(), cwd);
    }

    /// Every `--repo` spelling must name the same target whether it is joined to the base `base_dir` chose or to
    /// the working directory; only then is skipping `current_dir()` invisible.
    #[test]
    fn every_repo_spelling_names_the_same_target_with_or_without_the_working_directory() {
        let cwd = std::env::current_dir().unwrap();
        let spellings: &[&str] = if cfg!(windows) {
            &[
                r"C:\src\acme",
                r"C:\",
                "C:/src/acme",
                r"\\server\share\x",
                r"\\?\C:\x",
                r"\\?\UNC\server\share\x",
                r"\\.\pipe\x",
                r"\foo",
                "C:rel",
                r"..\up",
                "relative",
            ]
        } else {
            &["/src/acme", "/", "../up", "relative", "./relative"]
        };
        for spelling in spellings {
            let base = base_dir(Some(OsStr::new(spelling))).unwrap();
            // `join` discards the base for an absolute value, so the equality below holds whatever `base_dir`
            // returned. This is the discriminating assertion: the working directory is skipped exactly for the
            // spellings that do not need it.
            assert_eq!(
                base.as_os_str().is_empty(),
                std::path::Path::new(spelling).is_absolute(),
                "{spelling:?} took the wrong branch of base_dir"
            );
            assert_eq!(
                base.join(spelling),
                cwd.join(spelling),
                "{spelling:?} resolves differently without the working directory"
            );
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
    fn unknown_option_error_never_echoes_its_value() {
        match split_err(&["fake", "work", "--openai-api-key=sk-secret=="]) {
            Error::Usage { message } => {
                assert!(message.starts_with("unknown option \"--openai-api-key\";"), "{message}");
                assert!(!message.contains("sk-secret"), "{message}");
            }
            other => panic!("{other:?}"),
        }
        let message = usage_message(top(&["status", "--api-key=sk-secret"]));
        assert_eq!(message, "unknown option \"--api-key\" for `status`");
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
        let link = vec![OsString::from("fake"), "link".into(), non_utf8()];
        assert_eq!(usage_message(split(link, KNOWN)), "the profile name is not valid UTF-8");
        let mut equals = OsString::from("--repo=");
        equals.push(non_utf8());
        for repo in [vec![OsString::from("--repo"), non_utf8()], vec![equals]] {
            let mut items = vec![OsString::from("fake"), "status".into()];
            items.extend(repo);
            assert_eq!(
                split(items, KNOWN).unwrap(),
                Invocation::Command {
                    agent: Some("fake".to_owned()),
                    command: CommandWord::Status,
                    profile: None,
                    repo: Some(non_utf8()),
                }
            );
        }
    }
}
